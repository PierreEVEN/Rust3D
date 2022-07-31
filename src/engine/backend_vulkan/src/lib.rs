pub mod vk_instance;
pub mod vk_physical_device;
pub mod vk_device;
pub mod vk_swapchain;
pub mod vk_surface;

use std::sync::Arc;
use gfx::{GfxInterface, PhysicalDevice};
use ash::{Entry, vk};
use ash::version::InstanceV1_0;
use crate::vk_device::VkDevice;
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
macro_rules! gfx_object {
    ($object:expr) => {
        match &$object {
            None => {panic!("{} is not valid", stringify!($object))}
            Some(instance) => {instance}
        }
    }
}


#[macro_export]
macro_rules! vk_check {
    ($expression:expr) => {
        match $expression {
            Ok(object) => { object }
            Err(vk_err) => { panic!("vk error : {}\non '{}'", vk_err.to_string(), stringify!(expression)) }
        }
    }
}

#[derive(Default)]
pub struct GfxVulkan {
    pub instance: Option<VkInstance>,
    pub physical_device: Option<PhysicalDevice>,
    pub physical_device_vk: Option<VkPhysicalDevice>,
    pub device: Option<VkDevice>,
}

impl GfxInterface for GfxVulkan {
    fn set_physical_device(&mut self, selected_device: PhysicalDevice) {
        match self.physical_device {
            None => {
                self.physical_device = Some(selected_device.clone());
                self.physical_device_vk = Some(gfx_object!(self.instance).get_vk_device(&selected_device).expect("failed to get physical device information for vulkan").clone());
                
                self.device = Some(VkDevice::new(&self)) 
                
            }
            Some(_) => {
                panic!("physical device has already been selected");
            }
        }
    }


    fn enumerate_physical_devices(&self) -> Vec<PhysicalDevice> {
        gfx_object!(self.instance).enumerate_physical_devices()
    }

    fn find_best_suitable_physical_device(&self) -> Result<PhysicalDevice, String> {
        gfx_object!(self.instance).find_best_suitable_gpu_vk()
    }

    fn begin_frame(&self) {
    }

    fn end_frame(&self) {
    }
}

impl GfxVulkan {
    pub fn new() -> Self {
        unsafe { G_VULKAN = Some(Entry::new().expect("failed to load vulkan library")); } 
        
        let instance = VkInstance::new(InstanceCreateInfos {
            enable_validation_layers: true,
            ..Default::default()
        }).expect("failed to create instance");

        Self {
            instance: Some(instance),
            physical_device: None,
            physical_device_vk: None,
            device: None,
        }
    }
    
    pub fn do_test(&self) {
        match &self.instance {
            None => {}
            Some(inst) => { unsafe { inst.instance.enumerate_physical_devices(); } }
        }
    }
}

impl Drop for GfxVulkan {
    fn drop(&mut self) {        
        unsafe { G_VULKAN = None; }
    }
}