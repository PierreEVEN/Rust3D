use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

use smithay_client_toolkit::{
    activation::{ActivationHandler, ActivationState},
    compositor::{CompositorHandler, CompositorState}

    ,
    output::OutputHandler,
    registry::ProvidesRegistryState
    ,
    seat::{
        keyboard::KeyboardHandler,
        pointer::PointerHandler
        , SeatHandler,
    },
    shell::{
        WaylandSurface,
        xdg::XdgShell,
    },
    shm::{
        Shm, ShmHandler,
    },
};
use smithay_client_toolkit::reexports::calloop::EventLoop;
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::shell::xdg::window::WindowHandler;
use wayland_client::{
    Connection
    ,
    globals::registry_queue_init, QueueHandle,
};
use wayland_client::globals::GlobalList;

use plateform::{Monitor, Platform};
use plateform::input_system::InputManager;
use plateform::window::WindowCreateInfos;

use crate::window::WindowWayland;

pub mod window;
mod wayland_inputs;

const WIN_CLASS_NAME: &str = "r3d_window";

pub struct PlatformWayland {
    monitors: Vec<Monitor>,
    input_manager: InputManager,
    pub compositor: CompositorState,
    pub xdg_shell: XdgShell,
    pub shm: Arc<RwLock<Shm>>,
    pub xdg_activation: Arc<Option<ActivationState>>,
    pub queue_handle: QueueHandle<WindowWayland>,
    pub globals: GlobalList,
    pub event_loop: Arc<RwLock<EventLoop<'static, WindowWayland>>>,
    pub connection: Connection,
}

impl PlatformWayland {
    pub fn new() -> Arc<PlatformWayland> {

        // All Wayland apps start by connecting the compositor (server).
        let conn = Connection::connect_to_env().unwrap();
        // Enumerate the list of globals to get the protocols the server implements.
        let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        let qh = event_queue.handle();
        let mut event_loop: EventLoop<WindowWayland> =
            EventLoop::try_new().expect("Failed to initialize the event loop!");
        let loop_handle = event_loop.handle();
        WaylandSource::new(conn.clone(), event_queue).insert(loop_handle).unwrap();

        // The compositor (not to be confused with the server which is commonly called the compositor) allows
        // configuring surfaces to be presented.
        let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
        // For desktop platforms, the XDG shell is the standard protocol for creating desktop windows.
        let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg shell is not available");
        // Since we are not using the GPU in this example, we use wl_shm to allow software rendering to a buffer
        // we share with the compositor process.
        let shm = Shm::bind(&globals, &qh).expect("wl shm is not available.");
        // If the compositor supports xdg-activation it probably wants us to use it to get focus
        let xdg_activation = ActivationState::bind(&globals, &qh).ok();

        Arc::new(Self {
            monitors: vec![],
            input_manager: InputManager::new(),
            compositor,
            xdg_shell,
            shm: Arc::new(RwLock::new(shm)),
            xdg_activation: Arc::new(xdg_activation),
            queue_handle: qh,
            globals,
            event_loop : Arc::new(RwLock::new(event_loop)),
            connection: conn,
        })
    }
}

impl Drop for PlatformWayland {
    fn drop(&mut self) {
        unsafe {
        }
    }
}


impl Platform for PlatformWayland {
    fn create_window(&self, create_infos: WindowCreateInfos) -> Result<Arc<dyn plateform::window::Window>, ()> {
        return Ok(WindowWayland::new(self, create_infos));
    }

    fn monitor_count(&self) -> usize {
        self.monitors.len()
    }

    fn get_monitor(&self, index: usize) -> Monitor {
        self.monitors[index]
    }

    fn collect_monitors(&self) {
    }

    fn poll_events(&self) {
    }

    fn input_manager(&self) -> &InputManager {
        &self.input_manager
    }
}
