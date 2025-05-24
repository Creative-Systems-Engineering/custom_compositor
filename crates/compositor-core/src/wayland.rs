use compositor_utils::prelude::*;

use smithay::{
    desktop::{Space, Window},
    input::{Seat, SeatHandler, SeatState},
    output::{Output, PhysicalProperties, Subpixel},
    reexports::{
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
            Display, DisplayHandle, Resource,
        },
    },
    utils::{Clock, Monotonic, Serial},
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        },
        shm::{ShmHandler, ShmState},
        socket::ListeningSocketSource,
    },
};

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
}

/// Wayland server implementation using smithay
pub struct WaylandServer {
    display: Display<WaylandServerState>,
    state: WaylandServerState,
    socket_source: Option<ListeningSocketSource>,
}

impl WaylandServer {
    /// Create a new Wayland server
    pub fn new() -> Result<Self> {
        info!("Initializing Wayland server with smithay");
        
        let display = Display::new()
            .map_err(|e| CompositorError::wayland(format!("Failed to create display: {}", e)))?;
        
        let dh = display.handle();
        
        // Initialize compositor state
        let compositor_state = CompositorState::new::<WaylandServerState>(&dh);
        let xdg_shell_state = XdgShellState::new::<WaylandServerState>(&dh);
        let shm_state = ShmState::new::<WaylandServerState>(&dh, vec![]);
        let seat_state = SeatState::new();
        
        // Create default output (we'll make this more sophisticated later)
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
        
        // Create space 
        let mut space = Space::default();
        space.map_output(&output, (0, 0));
        
        // Seat will be created when needed
        
        let clock = Clock::new();
        
        let state = WaylandServerState {
            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
            space,
            clock,
            socket_name: None,
        };
        
        info!("Wayland server state initialized");
        
        Ok(Self {
            display,
            state,
            socket_source: None,
        })
    }
    
    /// Start listening on a Wayland socket
    pub fn start_listening(&mut self) -> Result<()> {
        let socket_source = ListeningSocketSource::new_auto()
            .map_err(|e| CompositorError::wayland(format!("Failed to create socket: {}", e)))?;
        
        let socket_name = socket_source.socket_name().to_string_lossy().into_owned();
        self.state.socket_name = Some(socket_name.clone());
        
        // Store socket source for later integration with calloop event loop
        self.socket_source = Some(socket_source);
        
        info!("Wayland server listening on socket: {}", socket_name);
        info!("Set WAYLAND_DISPLAY={} to connect clients", socket_name);
        Ok(())
    }
    
    /// Get the socket source for event loop integration
    pub fn take_socket_source(&mut self) -> Option<ListeningSocketSource> {
        self.socket_source.take()
    }
    
    /// Process pending Wayland events
    pub async fn process_events(&mut self) -> Result<()> {
        // Dispatch pending events
        self.display.dispatch_clients(&mut self.state)
            .map_err(|e| CompositorError::wayland(format!("Failed to dispatch clients: {}", e)))?;
        
        // Flush pending events
        self.display.flush_clients()
            .map_err(|e| CompositorError::wayland(format!("Failed to flush clients: {}", e)))?;
        
        tokio::task::yield_now().await;
        Ok(())
    }
    
    /// Shutdown the Wayland server
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Wayland server");
        
        if let Some(socket_name) = &self.state.socket_name {
            info!("Closed socket: {}", socket_name);
        }
        
        Ok(())
    }
    
    /// Get the display handle
    pub fn display_handle(&self) -> DisplayHandle {
        self.display.handle()
    }
    
    /// Get socket name if listening
    pub fn socket_name(&self) -> Option<&str> {
        self.state.socket_name.as_deref()
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
        // TODO: Handle surface commits for rendering
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
