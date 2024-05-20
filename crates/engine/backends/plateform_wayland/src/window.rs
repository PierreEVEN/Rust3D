use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use smithay_client_toolkit::{
    activation::{ActivationHandler, ActivationState},
    compositor::CompositorHandler,
    delegate_activation, delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability,
        keyboard::{KeyboardHandler, KeyEvent, Keysym, Modifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler}, SeatHandler, SeatState,
    },
    shell::WaylandSurface,
    shm::{
        Shm,
        ShmHandler, slot::{Buffer, SlotPool},
    },
};
use smithay_client_toolkit::activation::RequestData;
use smithay_client_toolkit::reexports::calloop::LoopHandle;
use smithay_client_toolkit::shell::xdg::window::{Window, WindowConfigure, WindowDecorations, WindowHandler};
use wayland_client::{
    Connection,
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface}, QueueHandle,
};
use wayland_client::protocol::wl_display::WlDisplay;
use wayland_client::protocol::wl_keyboard::WlKeyboard;
use wayland_client::protocol::wl_surface::WlSurface;

use maths::rect2d::RectI32;
use plateform::window::{PlatformEvent, WindowCreateInfos, WindowEventDelegate, WindowFlags};
use plateform::window;

use crate::PlatformWayland;

pub struct WindowWayland {
    flags: WindowFlags,
    geometry: RwLock<RectI32>,
    background_alpha: RwLock<u8>,
    title: RwLock<String>,
    event_map: RwLock<HashMap<PlatformEvent, Vec<WindowEventDelegate>>>,



    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,

    exit: bool,
    first_configure: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    shift: Option<u32>,
    buffer: Option<Buffer>,
    window: Window,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    keyboard_focus: bool,
    pointer: Option<wl_pointer::WlPointer>,
    loop_handle: LoopHandle<'static, crate::window::WindowWayland>,
    xdg_activation: Arc<Option<ActivationState>>,
    surface: RwLock<WlSurface>,
    shm: Arc<RwLock<Shm>>,
    connection: Connection
}

impl WindowWayland {
    pub fn new(owning_platform: &PlatformWayland, create_infos: WindowCreateInfos) -> Arc<WindowWayland> {


        // A window is created from a surface.
        let surface = owning_platform.compositor.create_surface(&owning_platform.queue_handle);
        // And then we can create the window.
        let window = owning_platform.xdg_shell.create_window(surface.clone(), WindowDecorations::RequestServer, &owning_platform.queue_handle);
        // Configure the window, this may include hints to the compositor about the desired minimum size of the
        // window, app id for WM identification, the window title, etc.
        window.set_title(create_infos.name.clone());
        // GitHub does not let projects use the `org.github` domain but the `io.github` domain is fine.
        window.set_app_id("io.github.smithay.client-toolkit.SimpleWindow");
        window.set_min_size(Some((create_infos.geometry.width() as u32, create_infos.geometry.height() as u32)));

        // In order for the window to be mapped, we need to perform an initial commit with no attached buffer.
        // For more info, see WaylandSurface::commit
        //
        // The compositor will respond with an initial configure that we can then use to present to the window with
        // the correct options.
        window.commit();

        // To request focus, we first need to request a token
        if let Some(activation) = owning_platform.xdg_activation.as_ref() {
            activation.request_token(
                &owning_platform.queue_handle,
                RequestData {
                    seat_and_serial: None,
                    surface: Some(window.wl_surface().clone()),
                    app_id: Some(String::from("io.github.smithay.client-toolkit.SimpleWindow")),
                },
            )
        }

        // We don't know how large the window will be yet, so lets assume the minimum size we suggested for the
        // initial memory allocation.
        let pool = SlotPool::new(256 * 256 * 4, &*owning_platform.shm.read().unwrap()).expect("Failed to create pool");

        let mut simple_window = WindowWayland {
            // Seats and outputs may be hotplugged at runtime, therefore we need to setup a registry state to
            // listen for seats and outputs.
            flags: create_infos.window_flags.clone(),
            geometry: RwLock::new(create_infos.geometry.clone()),
            background_alpha: RwLock::new(create_infos.background_alpha),
            title: RwLock::new(create_infos.name.clone()),
            event_map: Default::default(),
            registry_state: RegistryState::new(&owning_platform.globals),
            seat_state: SeatState::new(&owning_platform.globals, &owning_platform.queue_handle),
            output_state: OutputState::new(&owning_platform.globals, &owning_platform.queue_handle),

            exit: false,
            first_configure: true,
            pool,
            width: 256,
            height: 256,
            shift: None,
            buffer: None,
            window,
            keyboard: None,
            keyboard_focus: false,
            pointer: None,
            loop_handle: owning_platform.event_loop.read().unwrap().handle(),
            xdg_activation: owning_platform.xdg_activation.clone(),
            surface: RwLock::new(surface),
            shm: owning_platform.shm.clone(),
            connection: owning_platform.connection.clone(),
        };

        // We don't draw immediately, the configure will notify us when to first draw.
        owning_platform.event_loop.write().unwrap().dispatch(Duration::from_millis(16), &mut simple_window).unwrap();

        Arc::new(simple_window)
    }

    pub fn get_surface_ptr(&self) -> *mut WlSurface {
        let wayland_surface = &mut* self.surface.write().unwrap();
        wayland_surface as *mut WlSurface
    }
    pub fn get_display_ptr(&self) -> *mut WlDisplay {
        &mut self.connection.display() as *mut WlDisplay
    }
}

impl window::Window for WindowWayland {
    fn set_geometry(&self, geometry: RectI32) {
        todo!()
    }

    fn get_geometry(&self) -> RectI32 {
        self.geometry.read().unwrap().clone()
    }

    fn set_title(&self, title: &str) {
        let title_data = &mut *self.title.write().unwrap();
        *title_data = title.to_string();
        self.window.set_title(title_data.as_str());
    }

    fn get_title(&self) -> String {
        self.title.read().unwrap().clone()
    }

    fn show(&self) {
       // todo!()
    }

    fn set_background_alpha(&self, alpha: u8) {
        todo!()
    }

    fn get_background_alpha(&self) -> u8 {
        todo!()
    }

    fn bind_event(&self, event_type: PlatformEvent, delegate: WindowEventDelegate) {
        todo!()
    }
}


impl CompositorHandler for WindowWayland {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Not needed for this example.
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Not needed for this example.
    }

    fn frame(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(conn, qh);
    }
}

impl OutputHandler for WindowWayland {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl WindowHandler for WindowWayland {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &smithay_client_toolkit::shell::xdg::window::Window) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _window: &smithay_client_toolkit::shell::xdg::window::Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        println!("Window configured to: {:?}", configure);

        self.buffer = None;
        self.width = configure.new_size.0.map(|v| v.get()).unwrap_or(256);
        self.height = configure.new_size.1.map(|v| v.get()).unwrap_or(256);

        // Initiate the first draw.
        if self.first_configure {
            self.first_configure = false;
            self.draw(conn, qh);
        }
    }
}

impl ActivationHandler for WindowWayland {
    type RequestData = RequestData;

    fn new_token(&mut self, token: String, _data: &Self::RequestData) {
        self.xdg_activation
            .as_ref()
            .as_ref()
            .unwrap()
            .activate::<WindowWayland>(self.window.wl_surface(), token);
    }
}

impl SeatHandler for WindowWayland {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            println!("Set keyboard capability");
            let keyboard = self
                .seat_state
                .get_keyboard_with_repeat(
                    qh,
                    &seat,
                    None,
                    self.loop_handle.clone(),
                    Box::new(|_state, _wl_kbd, event| {
                        println!("Repeat: {:?} ", event);
                    }),
                )
                .expect("Failed to create keyboard");

            self.keyboard = Some(keyboard);
        }

        if capability == Capability::Pointer && self.pointer.is_none() {
            println!("Set pointer capability");
            let pointer = self.seat_state.get_pointer(qh, &seat).expect("Failed to create pointer");
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            println!("Unset keyboard capability");
            self.keyboard.take().unwrap().release();
        }

        if capability == Capability::Pointer && self.pointer.is_some() {
            println!("Unset pointer capability");
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for WindowWayland {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        keysyms: &[Keysym],
    ) {
        if self.window.wl_surface() == surface {
            println!("Keyboard focus on window with pressed syms: {keysyms:?}");
            self.keyboard_focus = true;
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if self.window.wl_surface() == surface {
            println!("Release keyboard focus on window");
            self.keyboard_focus = false;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        println!("Key press: {event:?}");
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        println!("Key release: {event:?}");
    }

    fn update_modifiers(&mut self, conn: &Connection, qh: &QueueHandle<Self>, keyboard: &WlKeyboard, serial: u32, modifiers: Modifiers) {
        println!("Update modifiers: {modifiers:?}");
    }
}

impl PointerHandler for WindowWayland {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        use PointerEventKind::*;
        for event in events {
            // Ignore events for other surfaces
            if &event.surface != self.window.wl_surface() {
                continue;
            }

            match event.kind {
                Enter { .. } => {
                    println!("Pointer entered @{:?}", event.position);
                }
                Leave { .. } => {
                    println!("Pointer left");
                }
                Motion { .. } => {}
                Press { button, .. } => {
                    println!("Press {:x} @ {:?}", button, event.position);
                    self.shift = self.shift.xor(Some(0));
                }
                Release { button, .. } => {
                    println!("Release {:x} @ {:?}", button, event.position);
                }
                Axis { horizontal, vertical, .. } => {
                    println!("Scroll H:{horizontal:?}, V:{vertical:?}");
                }
            }
        }
    }
}

impl ShmHandler for WindowWayland {
    fn shm_state(&mut self) -> &mut Shm {
        match &mut self.shm.write() {
            Ok(shm) => {
                unsafe {

                    let mut data_ptr = (&mut **shm) as *mut Shm;
                    return &mut *data_ptr;
                }
            }
            Err(_) => {panic!("err")}
        }
    }
}

impl WindowWayland {
    pub fn draw(&mut self, _conn: &Connection, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = self.width as i32 * 4;

        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool
                .create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888)
                .expect("create buffer")
                .0
        });

        let canvas = match self.pool.canvas(buffer) {
            Some(canvas) => canvas,
            None => {
                // This should be rare, but if the compositor has not released the previous
                // buffer, we need double-buffering.
                let (second_buffer, canvas) = self
                    .pool
                    .create_buffer(
                        self.width as i32,
                        self.height as i32,
                        stride,
                        wl_shm::Format::Argb8888,
                    )
                    .expect("create buffer");
                *buffer = second_buffer;
                canvas
            }
        };

        // Draw to the window:
        {
            let shift = self.shift.unwrap_or(0);
            canvas.chunks_exact_mut(4).enumerate().for_each(|(index, chunk)| {
                let x = ((index + shift as usize) % width as usize) as u32;
                let y = (index / width as usize) as u32;

                let a = 0xFF;
                let r = u32::min(((width - x) * 0xFF) / width, ((height - y) * 0xFF) / height);
                let g = u32::min((x * 0xFF) / width, ((height - y) * 0xFF) / height);
                let b = u32::min(((width - x) * 0xFF) / width, (y * 0xFF) / height);
                let color = (a << 24) + (r << 16) + (g << 8) + b;

                let array: &mut [u8; 4] = chunk.try_into().unwrap();
                *array = color.to_le_bytes();
            });

            if let Some(shift) = &mut self.shift {
                *shift = (*shift + 1) % width;
            }
        }

        // Damage the entire window
        self.window.wl_surface().damage_buffer(0, 0, self.width as i32, self.height as i32);

        // Request our next frame
        self.window.wl_surface().frame(qh, self.window.wl_surface().clone());

        // Attach and commit to present.
        buffer.attach_to(self.window.wl_surface()).expect("buffer attach");
        self.window.commit();
    }
}

delegate_compositor!(WindowWayland);
delegate_output!(WindowWayland);
delegate_shm!(WindowWayland);

delegate_seat!(WindowWayland);
delegate_keyboard!(WindowWayland);
delegate_pointer!(WindowWayland);

delegate_xdg_shell!(WindowWayland);
delegate_xdg_window!(WindowWayland);
delegate_activation!(WindowWayland);

delegate_registry!(WindowWayland);

impl ProvidesRegistryState for WindowWayland {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState,];
}
