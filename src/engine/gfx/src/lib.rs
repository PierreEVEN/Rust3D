use std::any::Any;
use std::hash::{Hash, Hasher};
use std::sync::{Arc};

use crate::buffer::{BufferCreateInfo, GfxBuffer};
use crate::render_pass::{RenderPass, RenderPassCreateInfos};
use crate::surface::GfxSurface;
use crate::types::GfxCast;

pub mod surface;
pub mod types;
pub mod buffer;
pub mod shader;
pub mod render_pass;
pub mod image;

pub trait GfxResource {
    fn load() -> Result<String, String>;
    fn load_now() -> Result<String, String>;
    fn unload() -> Result<String, String>;
    fn unload_now() -> Result<String, String>;
}

pub type GfxRef = Arc<dyn GfxInterface>;

pub trait GfxInterface: GfxCast {
    fn set_physical_device(&self, selected_device: PhysicalDevice);
    fn enumerate_physical_devices(&self) -> Vec<PhysicalDevice>;
    fn find_best_suitable_physical_device(&self) -> Result<PhysicalDevice, String>;
    fn create_buffer(&self, create_infos: &BufferCreateInfo) -> Box<dyn GfxBuffer>;
    fn create_render_pass(&self, create_infos: RenderPassCreateInfos) -> Arc<dyn RenderPass>;
    fn get_ref(&self) -> GfxRef;
}

#[derive(Copy, Clone)]
pub enum PhysicalDeviceType {
    Undefined,
    IntegratedGPU,
    DedicatedGPU,
    VirtualGPU,
    CPU,
}

impl Default for PhysicalDeviceType {
    fn default() -> Self { PhysicalDeviceType::Undefined }
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