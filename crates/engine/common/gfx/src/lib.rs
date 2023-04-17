use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::buffer::{BufferCreateInfo, GfxBuffer};
use crate::command_buffer::GfxCommandBuffer;
use crate::image::{GfxImage, ImageCreateInfos};
use crate::image_sampler::{ImageSampler, SamplerCreateInfos};
use crate::mesh::{Mesh, MeshCreateInfos};
use crate::render_pass::{RenderPass, RenderPassCreateInfos};
use crate::shader::{PassID, ShaderProgram, ShaderProgramInfos};
use crate::shader_instance::{ShaderInstance};
use crate::surface::GfxSurface;
use crate::types::GfxCast;

pub mod surface;
pub mod types;
pub mod buffer;
pub mod shader;
pub mod render_pass;
pub mod image;
pub mod gfx_resource;
pub mod command_buffer;
pub mod image_sampler;
pub mod shader_instance;
pub mod mesh;

pub type GfxRef = Arc<dyn GfxInterface>;

pub trait GfxInterface: GfxCast {

    fn init(&mut self);
    fn is_ready(&self) -> bool;
    
    fn set_physical_device(&self, selected_device: PhysicalDevice);
    fn enumerate_physical_devices(&self) -> Vec<PhysicalDevice>;
    fn find_best_suitable_physical_device(&self) -> Result<PhysicalDevice, String>;
    fn create_buffer(&self, name: String, create_infos: &BufferCreateInfo) -> Arc<dyn GfxBuffer>;
    fn create_shader_program(&self, name: String, render_pass: &Arc<dyn RenderPass>, create_infos: &ShaderProgramInfos) -> Arc<dyn ShaderProgram>;
    fn create_render_pass(&self, name: String, create_infos: RenderPassCreateInfos) -> Arc<dyn RenderPass>;
    fn create_image(&self, name: String, create_infos: ImageCreateInfos) -> Arc<dyn GfxImage>;
    fn create_image_sampler(&self, name: String, create_infos: SamplerCreateInfos) -> Arc<dyn ImageSampler>;
    fn find_render_pass(&self, pass_id: &PassID) -> Option<Arc<dyn RenderPass>>;
    fn create_command_buffer(&self, name: String, surface: &Arc<dyn GfxSurface>) -> Arc<dyn GfxCommandBuffer>;
    fn get_ref(&self) -> GfxRef;
}

impl dyn GfxInterface {
    pub fn cast<U: GfxInterface + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }

    pub fn create_mesh(&self, name: String, create_infos: &MeshCreateInfos) -> Arc<Mesh> {
        Mesh::new(&self.get_ref(), name, create_infos)
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