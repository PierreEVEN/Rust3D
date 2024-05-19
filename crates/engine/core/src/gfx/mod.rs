use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::sync::{Arc, RwLock, Weak};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::thread;
use std::thread::JoinHandle;

use logger::fatal;
use plateform::window::Window;
use shader_base::{CompilationError, ShaderInterface};
use shader_base::pass_id::PassID;
use shader_base::types::GfxCast;

use crate::gfx::buffer::{BufferCreateInfo, GfxBuffer};
use crate::gfx::command_buffer::{CommandCtx, GfxCommandBuffer};
use crate::gfx::image::{GfxImage, ImageCreateInfos};
use crate::gfx::image_sampler::{ImageSampler, SamplerCreateInfos};
use crate::gfx::material::MaterialResourcePool;
use crate::gfx::mesh::{Mesh, MeshCreateInfos};
use crate::gfx::program_pool::ProgramPool;
use crate::gfx::renderer::render_pass::{RenderPass, RenderPassInstance};
use crate::gfx::renderer::renderer::Renderer;
use crate::gfx::shader::ShaderProgram;
use crate::gfx::surface::{Frame, GfxSurface};

pub mod buffer;
pub mod command_buffer;
pub mod gfx_resource;
pub mod image;
pub mod image_sampler;
pub mod mesh;
pub mod shader;
pub mod shader_instance;
pub mod surface;
pub mod material;
pub mod program_pool;
pub mod renderer;

static mut GFX_INSTANCE: MaybeUninit<Gfx> = MaybeUninit::<Gfx>::uninit();

pub type SurfaceBuilderFunc = dyn FnMut(&Weak<dyn Window>) -> Box<dyn GfxSurface>;

pub trait GfxInterface: GfxCast {
    fn init(&mut self);
    fn is_ready(&self) -> bool;

    fn set_physical_device(&self, selected_device: PhysicalDevice);
    fn enumerate_physical_devices(&self) -> Vec<PhysicalDevice>;
    fn find_best_suitable_physical_device(&self) -> Result<PhysicalDevice, String>;
    fn create_buffer(&self, name: String, create_infos: &BufferCreateInfo) -> Arc<dyn GfxBuffer>;
    fn create_shader_program(
        &self,
        name: String,
        pass_id: PassID,
        create_infos: &dyn ShaderInterface,
        resources: Arc<MaterialResourcePool>,
    ) -> Result<Arc<dyn ShaderProgram>, CompilationError>;
    fn instantiate_render_pass(
        &self,
        render_pass: &RenderPass,
    ) -> Box<dyn RenderPassInstance>;
    fn create_image(&self, name: String, create_infos: ImageCreateInfos) -> Arc<dyn GfxImage>;
    fn create_image_sampler(
        &self,
        name: String,
        create_infos: SamplerCreateInfos,
    ) -> Arc<dyn ImageSampler>;
    fn create_command_buffer(
        &self,
        name: String,
    ) -> Arc<dyn GfxCommandBuffer>;
}

#[derive(Default)]
pub struct Gfx {
    instance: Option<*const dyn GfxInterface>,
    program_pool: ProgramPool,
    image_count: AtomicU8,
    render_threads: RwLock<Vec<JoinHandle<()>>>,
    stop_rendering: AtomicBool,
    // Should we interrupt render threads
    views: RwLock<Vec<Renderer>>,
    surface_builder: Option<Box<SurfaceBuilderFunc>>,
}

pub enum GfxEventBinding {
    BeforeFrameStart,
    AfterFrameEnd,
}

impl Gfx {
    fn from(gfx: &dyn GfxInterface, image_count: u8, builder: Box<SurfaceBuilderFunc>) -> Self {
        Self {
            instance: Some(gfx),
            program_pool: ProgramPool::default(),
            image_count: AtomicU8::new(image_count),
            render_threads: RwLock::default(),
            stop_rendering: AtomicBool::new(false),
            views: Default::default(),
            surface_builder: Some(builder),
        }
    }
    pub fn get() -> &'static Self {
        unsafe { GFX_INSTANCE.assume_init_ref() }
    }

    pub fn get_mut() -> &'static mut Self {
        unsafe { GFX_INSTANCE.assume_init_mut() }
    }

    pub fn shutdown(&self) {
        self.views.write().unwrap().clear();
    }

    // This is where shader program are stored and tracked.
    pub fn get_program_pool(&self) -> &ProgramPool { &self.program_pool }

    // The current number of swapchain images
    pub fn get_image_count(&self) -> u8 {
        Gfx::get().image_count.load(Ordering::SeqCst)
    }

    pub fn bind_event<T: FnMut()>(&self, binding: GfxEventBinding, frame: Frame, callback: T) {}

    pub fn create_mesh(&self, name: String, create_infos: &MeshCreateInfos) -> Arc<Mesh> {
        Mesh::new(name, create_infos)
    }

    pub fn add_renderer(&self, renderer: Renderer) {
        self.views.write().unwrap().push(renderer);
    }

    pub fn new_surface(&mut self, window: &Weak<dyn Window>) -> Box<dyn GfxSurface> {
        match &mut self.surface_builder {
            None => { panic!("Invalid surface builder") }
            Some(builder) => { (*builder)(window) }
        }
    }

    // Stop all rendering tasks
    pub fn stop_rendering_tasks(&self) {
        self.stop_rendering.store(true, Ordering::SeqCst);
        let render_threads = &mut *self.render_threads.write().unwrap();
        while !&render_threads.is_empty() {
            let render_thread = render_threads.pop().unwrap();
            render_thread.join().unwrap();
        }
    }

    pub fn set_frame_count(&self, new_frame_count: u8) {
        self.stop_rendering_tasks();

        self.image_count.store(new_frame_count, Ordering::SeqCst);

        // Restart rendering
        self.launch_render_threads();
    }

    pub fn launch_render_threads(&self) {
        self.stop_rendering.store(false, Ordering::SeqCst);
        let render_threads = &mut *self.render_threads.write().unwrap();
        for image_index in 0..self.image_count.load(Ordering::SeqCst) {
            let global_frame = Frame::new(image_index);
            render_threads.push(
                thread::Builder::new()
                    .name(format!("RenderThread_{image_index}")).spawn(move || {
                    logger::set_thread_label(thread::current().id(), format!("RenderThread_{image_index}").as_str());
                    while !&Gfx::get().stop_rendering.load(Ordering::SeqCst) {
                        Gfx::get().render_frame(&global_frame)
                    }                
                }).ok().unwrap())
        }
    }

    pub fn render_frame(&self, global_frame: &Frame) {
        if let Ok(renderers) = self.views.read() {
            for renderer in &*renderers {
                renderer.new_frame(global_frame);
            }
        }
    }
}

impl Deref for Gfx {
    type Target = dyn GfxInterface;

    fn deref(&self) -> &Self::Target {
        match self.instance {
            None => {
                fatal!("This gfx reference have not been initialized. Please ensure GfxInterface.init() have been called before")
            }
            Some(gfx) => unsafe { gfx.as_ref().unwrap() },
        }
    }
}

impl dyn GfxInterface {
    pub fn cast<U: GfxInterface + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }

    pub fn pre_init(&self, image_count: u8, builder: Box<SurfaceBuilderFunc>) {
        unsafe { GFX_INSTANCE = MaybeUninit::new(Gfx::from(self, image_count, builder)) }
    }
}

#[derive(Copy, Clone, Default)]
pub enum PhysicalDeviceType {
    #[default]
    Undefined,
    IntegratedGPU,
    DedicatedGPU,
    VirtualGPU,
    CPU,
}

#[derive(Default, Clone)]
pub struct PhysicalDevice {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: PhysicalDeviceType,
    pub device_name: String,
    pub score: u32,
}

impl Hash for PhysicalDevice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.device_id);
    }
}

impl PartialEq<Self> for PhysicalDevice {
    fn eq(&self, other: &Self) -> bool {
        self.device_id == other.device_id
    }
}

impl Eq for PhysicalDevice {}
