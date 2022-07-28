pub mod vk_instance;
pub mod vk_physical_device;

use gfx::{GfxInterface, PhysicalDevice};
use ash::{Entry, vk};
use crate::vk_instance::{InstanceCreateInfos, VkInstance};
use crate::vk_physical_device::VkPhysicalDevice;


static mut G_VULKAN: Option<Entry> = None;

#[macro_export]
macro_rules! g_vulkan {    
    () => {
        #[allow(unused_unsafe)]
        match unsafe { &G_VULKAN } {
            None => { panic!("vulkan has not been loaded yet"); }
            Some(entry) => { entry }
        }
    }
}

#[macro_export]
macro_rules! to_c_char {
    ($str:expr) => {        
        $str.as_ptr() as *const c_char
    }
}

#[macro_export]
macro_rules! instance_checked {
    ($self:ident) => {
        match &$self.instance {
            None => {panic!("instance is not valid")}
            Some(instance) => {instance}
        }
    }
}

#[derive(Default)]
pub struct GfxVulkan {
    instance: Option<VkInstance>,
    physical_device: Option<PhysicalDevice>,
    physical_device_vk: Option<VkPhysicalDevice>,
    device: vk::Device,
}



impl GfxInterface for GfxVulkan {
    fn set_physical_device(&mut self, selected_device: PhysicalDevice) {
        self.physical_device = Some(selected_device.clone());
        self.physical_device_vk = Some(instance_checked!(self).get_vk_device(&selected_device).expect("failed to get physical device information for vulkan").clone())}

    fn enumerate_physical_devices(&self) -> Vec<PhysicalDevice> {
        instance_checked!(self).enumerate_physical_devices()
    }

    fn find_best_suitable_physical_device(&self) -> Result<PhysicalDevice, String> {
        instance_checked!(self).find_best_suitable_gpu_vk()
    }
}

impl GfxVulkan {
    pub fn new() -> Self {
        unsafe { G_VULKAN = Some(Entry::load().expect("failed to load vulkan library")); } 
        
        let instance = VkInstance::new(InstanceCreateInfos {
            enable_validation_layers: true,
            ..Default::default()
        }).expect("failed to create instance");
        
        Self {
            instance: Some(instance),
            physical_device: None,
            physical_device_vk: None,
            ..Default::default()
        }
    }
}

impl Drop for GfxVulkan {
    fn drop(&mut self) {        
        self.instance = None;
        
        unsafe { G_VULKAN = None; }
    }
}