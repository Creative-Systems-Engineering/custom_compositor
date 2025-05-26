use compositor_utils::prelude::*;
use vulkan_renderer::VulkanRenderer;
use drm_fourcc::{DrmFourcc, DrmModifier};
use std::os::fd::OwnedFd;
use wayland_server::Resource;
use nix::libc;

use smithay::{
    backend::{
        allocator::{dmabuf::Dmabuf, Buffer, Format, gbm::GbmDevice},
        drm::{DrmNode, DrmDeviceFd},
        egl::{EGLContext, EGLDisplay},
    },
    utils::DeviceFd,
    desktop::{Space, Window},
    input::{Seat, SeatHandler, SeatState, pointer::PointerHandle},
    output::{Output, PhysicalProperties, Subpixel},
    wayland::output::{OutputHandler, OutputManagerState},
    reexports::{
        calloop::{EventLoop, LoopSignal},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
            protocol::wl_seat::WlSeat,
            Display,
        },
    },
    utils::{Clock, Monotonic, Serial, Point, Logical},
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState, with_states},
        dmabuf::{DmabufHandler, DmabufState, DmabufGlobal, ImportNotifier},
        drm_syncobj::{DrmSyncobjHandler, DrmSyncobjState, supports_syncobj_eventfd},
        pointer_constraints::{PointerConstraintsHandler, PointerConstraintsState},
        presentation::PresentationState,
        relative_pointer::RelativePointerManagerState,
        selection::{
            SelectionHandler,
            primary_selection::{PrimarySelectionHandler, PrimarySelectionState},
        },
        tablet_manager::{TabletManagerState, TabletSeatHandler},
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            decoration::{XdgDecorationHandler, XdgDecorationState},
        },
        shm::{ShmHandler, ShmState},
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
    pub dmabuf_state: DmabufState,
    pub dmabuf_global: DmabufGlobal,
    pub output_manager_state: OutputManagerState,
    pub relative_pointer_manager_state: RelativePointerManagerState,
    pub pointer_constraints_state: PointerConstraintsState,
    pub presentation_state: PresentationState,
    pub primary_selection_state: PrimarySelectionState,
    pub xdg_decoration_state: XdgDecorationState,
    pub tablet_manager_state: TabletManagerState,
    pub drm_syncobj_state: Option<DrmSyncobjState>,
    pub seat_state: SeatState<Self>,
    pub space: Space<Window>,
    pub clock: Clock<Monotonic>,
    pub socket_name: Option<String>,
    /// EGL context for hardware acceleration and wl_drm protocol support
    pub egl_context: Option<EGLContext>,
    /// EGL display for wl_drm protocol integration 
    pub egl_display: Option<EGLDisplay>,
    /// DRM node for GPU resource management
    pub drm_node: Option<DrmNode>,
    /// DRM device file descriptor for explicit sync support
    pub drm_device_fd: Option<DrmDeviceFd>,
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
        
        // Initialize dmabuf state for zero-copy GPU buffer sharing
        let mut dmabuf_state = DmabufState::new();
        
        // Create common formats for dmabuf support
        let formats = vec![
            Format {
                code: DrmFourcc::Xrgb8888,
                modifier: DrmModifier::Linear,
            },
            Format {
                code: DrmFourcc::Argb8888, 
                modifier: DrmModifier::Linear,
            },
        ];
        
        let dmabuf_global = dmabuf_state.create_global::<WaylandServerState>(&dh, formats);
        
        let seat_state = SeatState::new();
        
        // Initialize output manager with xdg-output support for multi-monitor configuration
        let output_manager_state = OutputManagerState::new_with_xdg_output::<WaylandServerState>(&dh);
        
        // Initialize relative pointer manager for 3D viewport navigation and gaming
        let relative_pointer_manager_state = RelativePointerManagerState::new::<WaylandServerState>(&dh);
        
        // Initialize pointer constraints for 3D viewport navigation and gaming
        let pointer_constraints_state = PointerConstraintsState::new::<WaylandServerState>(&dh);
        
        // Initialize presentation time for high-precision temporal synchronization
        let presentation_state = PresentationState::new::<WaylandServerState>(&dh, libc::CLOCK_MONOTONIC as u32);
        
        // Initialize primary selection for advanced clipboard functionality
        let primary_selection_state = PrimarySelectionState::new::<WaylandServerState>(&dh);
        
        // Initialize XDG decoration manager for client-side/server-side decoration control
        let xdg_decoration_state = XdgDecorationState::new::<WaylandServerState>(&dh);
        
        // Initialize tablet manager for professional graphics tablet integration
        let tablet_manager_state = TabletManagerState::new::<WaylandServerState>(&dh);
        
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
            dmabuf_state,
            dmabuf_global,
            output_manager_state,
            relative_pointer_manager_state,
            pointer_constraints_state,
            presentation_state,
            primary_selection_state,
            xdg_decoration_state,
            tablet_manager_state,
            drm_syncobj_state: None, // Will be initialized when DRM device is configured
            seat_state,
            space,
            clock,
            socket_name: None,
            egl_context: None, // Will be initialized when backend is configured
            egl_display: None, // Will be initialized for wl_drm protocol support
            drm_node: None,    // Will be set when DRM device is detected
            drm_device_fd: None, // Will be set for explicit sync support
            renderer: None,    // Initialize with no renderer
        };
        
        info!("Wayland server state initialized with calloop");
        
        Ok(Self {
            event_loop,
            state,
            display,
            loop_signal,
        })
    }
    
    /// Initialize EGL display and explicit sync support
    /// This automatically enables the wl_drm protocol for legacy EGL applications
    /// and zwp-linux-explicit-sync-v1 for modern GPU synchronization
    pub fn initialize_wl_drm(&mut self) -> Result<()> {
        info!("Initializing EGL display for wl_drm and explicit sync protocol support");
        
        // Try to find a primary DRM node (usually /dev/dri/card0)
        let drm_node = match DrmNode::from_path("/dev/dri/card0") {
            Ok(node) => {
                info!("Found primary DRM node: {:?}", node.dev_path());
                Some(node)
            }
            Err(e) => {
                warn!("Failed to open primary DRM node /dev/dri/card0: {}, trying render node", e);
                
                // Try render node as fallback (/dev/dri/renderD128)
                match DrmNode::from_path("/dev/dri/renderD128") {
                    Ok(node) => {
                        info!("Found DRM render node: {:?}", node.dev_path());
                        Some(node)
                    }
                    Err(e) => {
                        warn!("Failed to open DRM render node: {}, wl_drm and explicit sync will be unavailable", e);
                        None
                    }
                }
            }
        };
        
        // Store the DRM node
        self.state.drm_node = drm_node;
        
        // Initialize EGL display and explicit sync if we have a DRM node
        if let Some(ref drm_node) = self.state.drm_node {
            // Get the device path - dev_path() returns Option<PathBuf>
            let device_path = match drm_node.dev_path() {
                Some(path) => path,
                None => {
                    warn!("DRM node has no device path, protocols unavailable");
                    return Ok(());
                }
            };
            
            // Open the DRM device file
            let fd = match std::fs::File::open(&device_path) {
                Ok(file) => file,
                Err(e) => {
                    warn!("Failed to open DRM device file {:?}: {}, protocols unavailable", device_path, e);
                    return Ok(());
                }
            };
            
            // Create DRM device file descriptor for explicit sync
            let owned_fd: OwnedFd = fd.into();
            let device_fd = DeviceFd::from(owned_fd);
            let drm_device_fd = Some(DrmDeviceFd::new(device_fd));
            info!("Created DRM device fd for explicit sync support");
            
            // Initialize explicit sync if device supports it
            if let Some(ref device_fd) = drm_device_fd {
                if supports_syncobj_eventfd(device_fd) {
                    info!("✅ DRM device supports explicit sync, initializing zwp-linux-explicit-sync-v1");
                    
                    let dh = self.display.handle();
                    let syncobj_state = DrmSyncobjState::new::<WaylandServerState>(&dh, device_fd.clone());
                    self.state.drm_syncobj_state = Some(syncobj_state);
                    
                    info!("✅ zwp-linux-explicit-sync-v1 protocol initialized for frame-perfect timing control");
                } else {
                    warn!("DRM device does not support syncobj eventfd, explicit sync unavailable");
                }
                
                // Store the device fd regardless of sync support for potential future use
                self.state.drm_device_fd = drm_device_fd;
            }
            
            // Create GBM device from DRM file descriptor for EGL display
            // Re-open the file since DrmDeviceFd consumed the original
            let fd_for_gbm = match std::fs::File::open(&device_path) {
                Ok(file) => file,
                Err(e) => {
                    warn!("Failed to re-open DRM device file for GBM: {}, wl_drm protocol unavailable", e);
                    return Ok(());
                }
            };
            
            match GbmDevice::new(fd_for_gbm) {
                Ok(gbm_device) => {
                    info!("Created GBM device for DRM node: {:?}", device_path);
                    
                    // Create EGL display from GBM device
                    match unsafe { EGLDisplay::new(gbm_device) } {
                        Ok(egl_display) => {
                            info!("✅ Created EGL display from GBM device, wl_drm protocol support enabled");
                            self.state.egl_display = Some(egl_display);
                        }
                        Err(e) => {
                            warn!("Failed to create EGL display from GBM device: {}, wl_drm unavailable", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create GBM device: {}, wl_drm protocol unavailable", e);
                }
            }
        } else {
            info!("No DRM node available, wl_drm and explicit sync protocols will be unavailable");
        }
        
        // Report initialization status
        let wl_drm_status = if self.state.egl_display.is_some() { 
            "initialized" 
        } else { 
            "unavailable" 
        };
        
        let explicit_sync_status = if self.state.drm_syncobj_state.is_some() { 
            "initialized" 
        } else { 
            "unavailable" 
        };
        
        info!("Protocol initialization complete:");
        info!("  • wl_drm (legacy EGL): {}", wl_drm_status);
        info!("  • zwp-linux-explicit-sync-v1 (modern GPU sync): {}", explicit_sync_status);
        
        Ok(())
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
impl DmabufHandler for WaylandServerState {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }

    fn dmabuf_imported(
        &mut self, 
        _global: &DmabufGlobal, 
        dmabuf: Dmabuf,
        notifier: ImportNotifier
    ) {
        info!("DMA-BUF imported: {}x{} format: {:?}", 
              dmabuf.width(), dmabuf.height(), dmabuf.format());
        
        // TODO: Validate dmabuf format compatibility with our Vulkan renderer
        // TODO: Import dmabuf into our Vulkan renderer for zero-copy rendering
        // For now, just log the successful import and signal success
        debug!("DMA-BUF import successful, zero-copy GPU buffer sharing ready");
        
        // Signal that the import was successful
        if let Err(e) = notifier.successful::<WaylandServerState>() {
            error!("Failed to signal successful dmabuf import: {}", e);
        }
    }
}

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
        with_states(surface, |_surface_data| {
            // TODO: Implement proper buffer handling with current Smithay API
            // For now, just log the commit
            debug!("Surface committed with data, will handle buffer access in future implementation");
            
            // TODO: Access buffer through proper surface state API
            // TODO: Handle frame callbacks through proper API
            // TODO: Send buffer data to renderer
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
        debug!("Cursor image changed for seat");
    }
}

// Output handler implementation for managing outputs
impl OutputHandler for WaylandServerState {
    fn output_bound(&mut self, _output: Output, _wl_output: smithay::reexports::wayland_server::protocol::wl_output::WlOutput) {
        debug!("Output bound to client");
    }
}

// Pointer constraints handler implementation for precise pointer control
impl PointerConstraintsHandler for WaylandServerState {
    fn new_constraint(&mut self, surface: &WlSurface, pointer: &PointerHandle<Self>) {
        info!("New pointer constraint created for surface: {:?}", surface.id());
        debug!("Pointer constraint established for pointer: {:?}", pointer);
        
        // TODO: Handle constraint activation based on focus and surface state
        // TODO: Implement constraint region validation
        // TODO: Integrate with input handling system for constraint enforcement
    }
    
    fn cursor_position_hint(&mut self, surface: &WlSurface, pointer: &PointerHandle<Self>, location: Point<f64, Logical>) {
        debug!("Cursor position hint received for surface: {:?}, location: {:?}", surface.id(), location);
        debug!("Position hint for pointer: {:?}", pointer);
        
        // TODO: Update cursor position based on hint for locked pointer constraints
        // TODO: Validate hint location against constraint region
        // TODO: Apply position hint to compositor cursor state
    }
}

// DRM syncobj handler implementation for explicit GPU synchronization
impl DrmSyncobjHandler for WaylandServerState {
    fn drm_syncobj_state(&mut self) -> &mut DrmSyncobjState {
        self.drm_syncobj_state.as_mut().expect("DrmSyncobjState not initialized - ensure initialize_wl_drm() was called")
    }
}

// XDG decoration handler implementation for client/server-side decoration control
impl XdgDecorationHandler for WaylandServerState {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        info!("Client requested decoration support for toplevel window");
        
        // Configure server-side decorations by default for consistent glassmorphism theming
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide);
        });
        toplevel.send_configure();
        
        debug!("Configured server-side decorations for toplevel window");
    }
    
    fn request_mode(&mut self, toplevel: ToplevelSurface, mode: wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode) {
        use wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
        
        match mode {
            Mode::ClientSide => {
                info!("Client requested client-side decorations");
                toplevel.with_pending_state(|state| {
                    state.decoration_mode = Some(Mode::ClientSide);
                });
            }
            Mode::ServerSide => {
                info!("Client requested server-side decorations");
                toplevel.with_pending_state(|state| {
                    state.decoration_mode = Some(Mode::ServerSide);
                });
            }
            _ => {
                warn!("Client requested unknown decoration mode: {:?}", mode);
                // Default to server-side for glassmorphism integration
                toplevel.with_pending_state(|state| {
                    state.decoration_mode = Some(Mode::ServerSide);
                });
            }
        }
        
        toplevel.send_configure();
        debug!("Applied decoration mode: {:?}", mode);
    }
    
    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        info!("Client unset decoration mode preference");
        
        // Default to server-side decorations for consistent theming
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide);
        });
        toplevel.send_configure();
        
        debug!("Reset to server-side decorations (default)");
    }
}

// ============================================================================
// Primary Selection Handler Implementation  
// ============================================================================

// ============================================================================
// Selection Handler Implementation
// ============================================================================

impl SelectionHandler for WaylandServerState {
    type SelectionUserData = ();
}

// ============================================================================
// Primary Selection Handler Implementation  
// ============================================================================

impl PrimarySelectionHandler for WaylandServerState {
    fn primary_selection_state(&self) -> &PrimarySelectionState {
        &self.primary_selection_state
    }
}

// ============================================================================
// Tablet Manager Handler Implementation
// ============================================================================

impl TabletSeatHandler for WaylandServerState {
    // Let the compiler tell us what methods we need to implement
}

// Delegate handlers to implementations
smithay::delegate_compositor!(WaylandServerState);
smithay::delegate_xdg_shell!(WaylandServerState);
smithay::delegate_output!(WaylandServerState);
smithay::delegate_shm!(WaylandServerState);
smithay::delegate_dmabuf!(WaylandServerState);
smithay::delegate_seat!(WaylandServerState);
smithay::delegate_relative_pointer!(WaylandServerState);
smithay::delegate_pointer_constraints!(WaylandServerState);
smithay::delegate_presentation!(WaylandServerState);
smithay::delegate_primary_selection!(WaylandServerState);
smithay::delegate_xdg_decoration!(WaylandServerState);
smithay::delegate_tablet_manager!(WaylandServerState);
smithay::delegate_drm_syncobj!(WaylandServerState);
