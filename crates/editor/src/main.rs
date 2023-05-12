use core::engine::{Builder, Engine};
use core::world::World;
use ecs::entity::GameObject;
use plateform::input_system::{InputAction, InputMapping, KeyboardKey};
use plateform::window::PlatformEvent;
use std::sync::Arc;

mod gfx_demo;

struct Camera {}

#[derive(Default)]
pub struct TestApp {
    world: Arc<World>,
    primary_camera: GameObject,
    //world_view: WorldView,
}

impl core::engine::App for TestApp {
    fn pre_initialize(&mut self, _: &mut Builder) {}
    fn initialized(&mut self) {
        let input_manager = Engine::get().platform().input_manager();
        input_manager.new_action(
            "MoveForward",
            InputAction::new().map(InputMapping::Keyboard(KeyboardKey::KeyZ)),
        );
        input_manager.new_action(
            "MoveRight",
            InputAction::new().map(InputMapping::Keyboard(KeyboardKey::KeyD)),
        );
        input_manager.new_action(
            "MoveLeft",
            InputAction::new().map(InputMapping::Keyboard(KeyboardKey::KeyQ)),
        );
        input_manager.new_action(
            "MoveBackward",
            InputAction::new().map(InputMapping::Keyboard(KeyboardKey::KeyS)),
        );

        // Create world
        self.world = Engine::get().new_world();
        self.primary_camera = self.world.add_object::<Camera>(Camera {});

        // Create primary window
        let main_window = Engine::get()
            .platform()
            .create_window(plateform::window::WindowCreateInfos {
                name: "Rust3D Editor".to_string(),
                geometry: maths::rect2d::Rect2D::rect(300, 400, 800, 600),
                window_flags: plateform::window::WindowFlags::from_flag(
                    plateform::window::WindowFlagBits::Resizable,
                ),
                background_alpha: 255,
            })
            .unwrap();
        main_window.show();

        main_window.bind_event(
            PlatformEvent::WindowClosed,
            Box::new(|_| {
                Engine::get().shutdown();
            }),
        );

        // Create world view
        //self.world_view = Engine::get().create_view(main_window, Renderer::default_pbr());
        //self.world_view.attach_to(self.primary_camera);
    }

    fn new_frame(&mut self, _delta_seconds: f64) {
        //self.primary_camera.movement(_delta_seconds);
    }

    fn request_shutdown(&self) {}
    fn stopped(&self) {}
}

fn main() {
    let mut engine = Engine::new::<TestApp>();
    engine.start();
}

/*
// Create ImGui context
let imgui_context = ImGUiContext::new(&engine.gfx);

// Create render pass and pass instances
let g_buffer_pass = engine.gfx.create_render_pass("gbuffer".to_string(), RenderPassCreateInfos {
    pass_id: PassID::new("deferred_combine"),
    color_attachments: vec![RenderPassAttachment {
        name: "color".to_string(),
        clear_value: ClearValues::Color(Vec4F32::new(0.0, 0.0, 0.0, 1.0)),
        image_format: PixelFormat::R8G8B8A8_UNORM,
    }],
    depth_attachment: None,
    is_present_pass: false,
});
let def_combine = g_buffer_pass.instantiate(&main_window_surface, main_window_surface.get_extent());
let imgui_pass = imgui_context.instantiate_for_surface(&main_window_surface);

// Create framegraph
let main_framegraph = FrameGraph::from_surface(&engine.gfx, &main_window_surface, Vec4F32::new(1.0, 0.0, 0.0, 1.0));
main_framegraph.main_pass().attach(def_combine.clone());
main_framegraph.main_pass().attach(imgui_pass.clone());

// Create material
let demo_material = MaterialAsset::new(&engine.asset_manager);
demo_material.meta_data().set_save_path(Path::new("data/demo_shader"));
demo_material.meta_data().set_name("demo shader".to_string());
demo_material.set_shader_code(Path::new("data/shaders/resolve.shb"), fs::read_to_string("data/shaders/resolve.shb").expect("failed to read shader_file"));

// Create images
let background_image = read_image_from_file(&engine.gfx, Path::new("data/textures/cat_stretching.png")).expect("failed to create image");

// Create sampler
let generic_image_sampler = engine.gfx.create_image_sampler("bg_image".to_string(), SamplerCreateInfos {});

// Create material instance
let surface_combine_shader = demo_material.get_program(&PassID::new("surface_pass")).unwrap().instantiate();
surface_combine_shader.bind_texture(&BindPoint::new("ui_result"), &imgui_pass.get_images()[0]);
surface_combine_shader.bind_texture(&BindPoint::new("scene_result"), &def_combine.get_images()[0]);
surface_combine_shader.bind_sampler(&BindPoint::new("global_sampler"), &generic_image_sampler);

let background_shader = demo_material.get_program(&PassID::new("deferred_combine")).unwrap().instantiate();
background_shader.bind_texture(&BindPoint::new("bg_texture"), &background_image);
background_shader.bind_sampler(&BindPoint::new("global_sampler"), &generic_image_sampler);

{
    let start = Instant::now();
    let mut time_pc_data = TestPc { time: 0.0 };
    let surface_shader_instance = surface_combine_shader.clone();
    let demo_material = demo_material.clone();
    main_framegraph.main_pass().on_render(Box::new(move |command_buffer| {
        match demo_material.get_program(&command_buffer.get_pass_id()) {
            None => { logger::fatal!("failed to find compatible permutation [{}]", command_buffer.get_pass_id()); }
            Some(program) => {
                time_pc_data.time = start.elapsed().as_millis() as f32 / 1000.0;
                command_buffer.bind_program(&program);
                command_buffer.bind_shader_instance(&surface_shader_instance);
                command_buffer.push_constant(&program, BufferMemory::from_struct(&time_pc_data), ShaderStage::Fragment);
                command_buffer.draw_procedural(4, 0, 1, 0);
            }
        };
    }));
}

#[repr(C, align(4))]
struct TestPc {
    time: f32,
}

{
    let start = Instant::now();
    let mut time_pc_data = TestPc { time: 0.0 };
    let shader_2_instance = background_shader.clone();
    def_combine.on_render(Box::new(move |command_buffer| {
        match demo_material.get_program(&command_buffer.get_pass_id()) {
            None => { logger::fatal!("failed to find compatible permutation [{}]", command_buffer.get_pass_id()); }
            Some(program) => {
                time_pc_data.time = start.elapsed().as_millis() as f32 / 1000.0;
                command_buffer.bind_program(&program);
                command_buffer.bind_shader_instance(&shader_2_instance);
                command_buffer.push_constant(&program, BufferMemory::from_struct(&time_pc_data), ShaderStage::Fragment);
                command_buffer.draw_procedural(4, 0, 1, 0);
            }
        };
    }));
}

main_window.bind_event(PlatformEvent::WindowClosed, Box::new(|_| {
    Engine::get().shutdown();
}));

main_window.bind_event(PlatformEvent::WindowResized(0, 0), Box::new(move |event| {
    match event {
        PlatformEvent::WindowResized(width, height) => {
            imgui_pass.resize(Vec2u32::new(*width, *height));
            surface_combine_shader.bind_texture(&BindPoint::new("ui_result"), &imgui_pass.get_images()[0]);
            surface_combine_shader.bind_texture(&BindPoint::new("scene_result"), &def_combine.get_images()[0]);
        }
        PlatformEvent::WindowClosed => {}
    }
}));

// Game loop
while engine.run() {
    engine.platform.poll_events();
    if main_framegraph.begin().is_ok() { main_framegraph.submit(); };
}
 */
