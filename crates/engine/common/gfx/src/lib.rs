use logger::fatal;
use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::sync::Arc;
use shader_base::pass_id::PassID;

use crate::buffer::{BufferCreateInfo, GfxBuffer};
use crate::command_buffer::GfxCommandBuffer;
use crate::image::{GfxImage, ImageCreateInfos};
use crate::image_sampler::{ImageSampler, SamplerCreateInfos};
use crate::mesh::{Mesh, MeshCreateInfos};
use crate::renderer::render_pass::{RenderPass, RenderPassInstance};
use crate::shader::{ShaderProgram, ShaderProgramInfos};
use crate::shader_instance::ShaderInstance;
use crate::types::GfxCast;

pub mod buffer;
pub mod command_buffer;
pub mod gfx_resource;
pub mod image;
pub mod image_sampler;
pub mod mesh;
pub mod shader;
pub mod shader_instance;
pub mod surface;
pub mod types;

pub mod renderer {
    pub mod frame_graph;
    pub mod render_node;
    pub mod renderer_resource;
    pub mod render_pass;
}

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
        create_infos: &ShaderProgramInfos,
    ) -> Arc<dyn ShaderProgram>;
    fn instantiate_render_pass(
        &self,
        render_pass: &RenderPass
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
}

static mut GFX_INSTANCE: MaybeUninit<Gfx> = MaybeUninit::<Gfx>::uninit();

impl Gfx {
    fn from(gfx: &dyn GfxInterface) -> Self {
        Self {
            instance: Some(gfx),
        }
    }
    pub fn get() -> &'static Self {
        unsafe { GFX_INSTANCE.assume_init_ref() }
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

    pub fn create_mesh(&self, name: String, create_infos: &MeshCreateInfos) -> Arc<Mesh> {
        Mesh::new(name, create_infos)
    }

    pub fn pre_init(&self) {
        unsafe { GFX_INSTANCE = MaybeUninit::new(Gfx::from(self)) }
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
