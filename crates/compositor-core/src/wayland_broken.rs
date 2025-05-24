use compositor_utils::prelude::*;
use vulkan_renderer::VulkanRenderer;
use ash;

use smithay::{
    backend::allocator::Buffer,
    desktop::{Space, Window},
    input::{Seat, SeatHandler, SeatState},
    output::{Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{EventLoop, LoopSignal},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::{wl_seat::WlSeat, wl_surface::WlSurface, wl_shm},
            Display, Resource,
        },
    },
    utils::{Clock, Monotonic, Serial},
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState, SurfaceAttributes, with_states},
        dmabuf,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        },
        shm::{ShmHandler, ShmState, self},
        socket::ListeningSocketSource,
    },
};

use std::sync::{Arc, Mutex};

/// Client state data
#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

/// Main Wayland server state
pub struct WaylandServerState {
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,
    pub space: Space<Window>,
    pub clock: Clock<Monotonic>,
    pub socket_name: Option<String>,
    /// Vulkan renderer for surface compositing
    pub renderer: Option<Arc<Mutex<VulkanRenderer>>>,
}

/// Wayland server implementation using smithay and calloop
pub struct WaylandServer {
    pub event_loop: EventLoop<'static, WaylandServerState>,
    pub state: WaylandServerState,
    pub display: Display<WaylandServerState>,
    pub loop_signal: LoopSignal,
}

impl WaylandServer {
    /// Create a new Wayland server with event loop
    pub fn new() -> Result<Self> {
        info!("Initializing Wayland server with smithay and calloop");
        
        // Create event loop first
        let event_loop = EventLoop::try_new()
            .map_err(|e| CompositorError::wayland(format!("Failed to create event loop: {}", e)))?;
        
        let _loop_handle = event_loop.handle();
        let loop_signal = event_loop.get_signal();
        
        // Create display with the loop handle
        let display = Display::new()
            .map_err(|e| CompositorError::wayland(format!("Failed to create display: {}", e)))?;
        
        let dh = display.handle();
        
        // Initialize compositor state
        let compositor_state = CompositorState::new::<WaylandServerState>(&dh);
        let xdg_shell_state = XdgShellState::new::<WaylandServerState>(&dh);
        let shm_state = ShmState::new::<WaylandServerState>(&dh, vec![]);
        let seat_state = SeatState::new();
        
        // Create default output (4K setup)
        let output = Output::new(
            "custom-compositor-output".to_string(),
            PhysicalProperties {
                size: (3840, 2160).into(), // 4K default
                subpixel: Subpixel::Unknown,
                make: "Custom Compositor".into(),
                model: "Virtual Output".into(),
            },
        );
        
        // Add modes to output
        output.add_mode(smithay::output::Mode {
            size: (3840, 2160).into(),
            refresh: 60_000, // 60Hz in mHz
        });
        output.set_preferred(smithay::output::Mode {
            size: (3840, 2160).into(),
            refresh: 60_000,
        });
        
        // Create space and map output
        let mut space = Space::default();
        space.map_output(&output, (0, 0));
        
        let clock = Clock::new();
        
        let state = WaylandServerState {
            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
            space,
            clock,
            socket_name: None,
            renderer: None, // Initialize with no renderer
        };
        
        info!("Wayland server state initialized with calloop");
        
        Ok(Self {
            event_loop,
            state,
            display,
            loop_signal,
        })
    }
    
    /// Start listening on a Wayland socket and integrate with event loop
    pub fn start_listening(&mut self) -> Result<()> {
        info!("Starting Wayland socket and integrating with event loop");
        
        // Create listening socket
        let socket_source = ListeningSocketSource::new_auto()
            .map_err(|e| CompositorError::wayland(format!("Failed to create socket: {}", e)))?;
        
        let socket_name = socket_source.socket_name().to_string_lossy().into_owned();
        self.state.socket_name = Some(socket_name.clone());
        
        // Insert socket into event loop
        let mut display_handle = self.display.handle();
        self.event_loop
            .handle()
            .insert_source(socket_source, move |client_stream, _, _state| {
                // Handle new client connections
                if let Err(err) = display_handle.insert_client(client_stream, Arc::new(ClientState::default())) {
                    error!("Failed to insert client: {}", err);
                }
            })
            .map_err(|e| CompositorError::wayland(format!("Failed to insert socket source: {}", e)))?;
        
        info!("Wayland server listening on socket: {}", socket_name);
        info!("Set WAYLAND_DISPLAY={} to connect clients", socket_name);
        
        // Set environment variable for clients
        std::env::set_var("WAYLAND_DISPLAY", &socket_name);
        
        Ok(())
    }
    
    /// Run the event loop (blocking)
    pub fn run(mut self) -> Result<()> {
        info!("Starting Wayland server event loop");
        
        // Main event loop using smithay's standard pattern
        loop {
            // Dispatch wayland events
            if let Err(e) = self.display.dispatch_clients(&mut self.state) {
                error!("Error dispatching clients: {}", e);
                break;
            }
            
            // Flush pending events  
            if let Err(e) = self.display.flush_clients() {
                error!("Error flushing clients: {}", e);
                break;
            }
            
            // Run event loop iteration
            if let Err(e) = self.event_loop.dispatch(Some(std::time::Duration::from_millis(16)), &mut self.state) {
                error!("Event loop error: {}", e);
                break;
            }
        }
        
        info!("Wayland server event loop terminated");
        Ok(())
    }
    
    /// Run the event loop asynchronously (non-blocking)
    pub async fn run_async(mut self) -> Result<()> {
        info!("Starting Wayland server async event loop");
        
        // Async event loop using smithay's standard pattern
        loop {
            // Dispatch wayland events
            if let Err(e) = self.display.dispatch_clients(&mut self.state) {
                error!("Error dispatching clients: {}", e);
                break;
            }
            
            // Flush pending events  
            if let Err(e) = self.display.flush_clients() {
                error!("Error flushing clients: {}", e);
                break;
            }
            
            // Run event loop iteration with async yield
            if let Err(e) = self.event_loop.dispatch(Some(std::time::Duration::from_millis(16)), &mut self.state) {
                error!("Event loop error: {}", e);
                break;
            }
            
            // Yield to other async tasks
            tokio::task::yield_now().await;
        }
        
        info!("Wayland server async event loop terminated");
        Ok(())
    }
    
    /// Set the Vulkan renderer for surface rendering
    pub fn set_renderer(&mut self, renderer: Arc<Mutex<VulkanRenderer>>) {
        info!("Setting Vulkan renderer for Wayland server");
        self.state.renderer = Some(renderer);
    }
    
    /// Get the loop signal for shutdown
    pub fn loop_signal(&self) -> LoopSignal {
        self.loop_signal.clone()
    }
    
    /// Get socket name if listening
    pub fn socket_name(&self) -> Option<&str> {
        self.state.socket_name.as_deref()
    }
    
    /// Shutdown the Wayland server
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Wayland server");
        
        // Signal the event loop to stop
        self.loop_signal.stop();
        
        // The event loop should stop processing after receiving the signal
        info!("Wayland server shutdown complete");
        Ok(())
    }
}

// Implement required smithay handlers
impl CompositorHandler for WaylandServerState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
    
    fn client_compositor_state<'a>(&self, client: &'a wayland_server::Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }
    
    fn new_surface(&mut self, surface: &WlSurface) {
        debug!("New surface created: {:?}", surface.id());
    }
    
    fn commit(&mut self, surface: &WlSurface) {
        debug!("Surface committed: {:?}", surface.id());
        
        // Handle surface commits for rendering
        with_states(surface, |surface_data| {
            let surface_attributes = surface_data.cached_state.get::<SurfaceAttributes>();
            
            // Check if there's a buffer
            if let Some(buffer) = surface_attributes.buffer.as_ref() {
                debug!("Surface buffer attached: {:?}", buffer);
                
                // Extract buffer data and send to renderer
                let surface_id = surface.id().protocol_id() as u32;
                
                if let Ok(dmabuf) = dmabuf::get_dmabuf(&buffer) {
                    debug!("DMA-BUF buffer: {}x{}, format: {:?}", 
                           dmabuf.width(), dmabuf.height(), dmabuf.format());
                    
                    // TODO: Handle DMA-BUF conversion to renderer format
                    warn!("DMA-BUF rendering not yet implemented");
                    
                } else if let Ok(shm_buffer) = shm::with_buffer_contents(&buffer, |data, len, spec| {
                    debug!("SHM buffer: {}x{}, stride: {}, format: {:?}", 
                           spec.width, spec.height, spec.stride, spec.format);
                    
                    // Create slice from raw pointer and length
                    let buffer_data = unsafe { std::slice::from_raw_parts(data, len) };
                    
                    // Convert SHM buffer to vulkan format and update renderer
                    if let Some(ref renderer) = self.renderer {
                        if let Ok(mut renderer_guard) = renderer.try_lock() {
                            // Convert SHM format to Vulkan format
                            let vulkan_format = match spec.format {
                                wl_shm::Format::Argb8888 => ash::vk::Format::B8G8R8A8_UNORM,
                                wl_shm::Format::Xrgb8888 => ash::vk::Format::B8G8R8A8_UNORM,
                                wl_shm::Format::Rgba8888 => ash::vk::Format::R8G8B8A8_UNORM,
                                wl_shm::Format::Rgbx8888 => ash::vk::Format::R8G8B8A8_UNORM,
                                _ => {
                                    warn!("Unsupported SHM format: {:?}", spec.format);
                                    ash::vk::Format::R8G8B8A8_UNORM
                                }
                            };
                            
                            // Update surface texture in renderer
                            if let Err(e) = renderer_guard.update_surface_buffer(
                                surface_id,
                                buffer_data,
                                spec.width.try_into().unwrap(),
                                spec.height.try_into().unwrap(),
                                vulkan_format,
                            ) {
                                error!("Failed to update surface buffer in renderer: {}", e);
                            } else {
                                debug!("Surface {} texture updated in renderer", surface_id);
                            }
                        } else {
                            warn!("Could not lock renderer for surface update");
                        }
                    } else {
                        debug!("No renderer available for surface buffer");
                    }
                    
                    Ok(())
                }) {
                    debug!("SHM buffer processed successfully");
                } else {
                    warn!("Unknown buffer type attached to surface");
                }
            }
            
            // Handle frame callbacks
            let frame_callbacks = surface_attributes.frame_callbacks.drain(..).collect::<Vec<_>>();
            drop(surface_attributes); // Release the lock before using callbacks
            
            for callback in frame_callbacks {
                callback.done(self.clock.now().as_millis() as u32);
            }
        });
        
        // Schedule a repaint for this surface
        self.space.refresh();
        
        // TODO: Trigger actual frame rendering in compositor
        debug!("Surface commit processed, space refreshed");
    }
}

impl XdgShellHandler for WaylandServerState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }
    
    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        info!("New toplevel window created");
        
        // Create window and add to space using the new API
        let window = Window::new_wayland_window(surface);
        self.space.map_element(window, (100, 100), false);
    }
    
    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        debug!("New popup created");
        // TODO: Handle popups
    }
    
    fn toplevel_destroyed(&mut self, _surface: ToplevelSurface) {
        info!("Toplevel window destroyed");
        // TODO: Remove window from space
    }
    
    fn popup_destroyed(&mut self, _surface: PopupSurface) {
        debug!("Popup destroyed");
        // TODO: Handle popup destruction
    }
    
    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        debug!("Popup grab requested");
        // TODO: Handle popup grabs
    }
    
    fn reposition_request(&mut self, _surface: PopupSurface, _positioner: PositionerState, _token: u32) {
        debug!("Popup reposition requested");
        // TODO: Handle popup repositioning
    }
}

impl ShmHandler for WaylandServerState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl BufferHandler for WaylandServerState {
    fn buffer_destroyed(&mut self, _buffer: &wayland_server::protocol::wl_buffer::WlBuffer) {
        debug!("Buffer destroyed");
        // TODO: Handle buffer cleanup
    }
}

impl SeatHandler for WaylandServerState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;
    
    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
    
    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&Self::KeyboardFocus>) {
        debug!("Focus changed for seat");
    }
    
    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: smithay::input::pointer::CursorImageStatus) {
        // TODO: Handle cursor image updates
    }
}

// Delegate handlers to implementations
smithay::delegate_compositor!(WaylandServerState);
smithay::delegate_xdg_shell!(WaylandServerState);
smithay::delegate_shm!(WaylandServerState);
smithay::delegate_seat!(WaylandServerState);
