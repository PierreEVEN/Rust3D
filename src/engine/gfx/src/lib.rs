use std::hash::{Hash, Hasher};

pub trait GfxResource {
    fn load() -> Result<String, String>;
    fn load_now() -> Result<String, String>;
    fn unload() -> Result<String, String>;
    fn unload_now() -> Result<String, String>;
}

pub trait GfxInterface {
    fn set_physical_device(&mut self, selected_device: PhysicalDevice);
    fn enumerate_physical_devices(&self) -> Vec<PhysicalDevice>;
    fn find_best_suitable_physical_device(&self) -> Result<PhysicalDevice, String>;
    fn begin_frame(&self);
    fn end_frame(&self);
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