extern crate core;

use std::any::{Any, TypeId};
use std::sync::{Arc, RwLock, Weak};

use ash::Entry;
use ash::vk::CommandPool;

use gfx::{GfxCast, GfxInterface, GfxRef, PhysicalDevice};
use gfx::buffer::{BufferCreateInfo, GfxBuffer};
use gfx::render_pass::{RenderPass, RenderPassCreateInfos};

use crate::vk_buffer::VkBuffer;
use crate::vk_command_buffer::VkCommandPool;
use crate::vk_device::VkDevice;
use crate::vk_instance::{InstanceCreateInfos, VkInstance};
use crate::vk_physical_device::VkPhysicalDevice;
use crate::vk_render_pass::VkRenderPass;

pub mod vk_instance;
pub mod vk_physical_device;
pub mod vk_device;
pub mod vk_swapchain;
pub mod vk_surface;
pub mod vk_types;
pub mod vk_render_pass;
pub mod vk_buffer;
pub mod vk_shader;
pub mod vk_descriptor_set;
pub mod vk_render_pass_instance;
pub mod vk_command_buffer;
pub mod vk_queue;

pub static mut G_VULKAN: Option<Entry> = None;

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
macro_rules! gfx_cast_vulkan {
    ($gfx:expr) => {        
        ($gfx.as_ref()).as_any().downcast_ref::<GfxVulkan>().expect("failed to cast to gfx vulkan")
    };
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

pub struct GfxVulkan {
    pub instance: Option<VkInstance>,
    pub physical_device: RwLock<Option<PhysicalDevice>>,
    pub physical_device_vk: RwLock<Option<VkPhysicalDevice>>,
    pub device: RwLock<Option<VkDevice>>,
    pub gfx_ref: RwLock<Weak<GfxVulkan>>,
    pub command_pool: RwLock<Option<VkCommandPool>>,
}

impl GfxInterface for GfxVulkan {
    fn set_physical_device(&self, selected_device: PhysicalDevice) {
        {
            let mut physical_device = self.physical_device.write().unwrap();
            let mut physical_device_vk = self.physical_device_vk.write().unwrap();

            match *physical_device {
                None => {
                    *physical_device = Some(selected_device.clone());
                    *physical_device_vk = Some(gfx_object!(self.instance).get_vk_device(&selected_device).expect("failed to get physical device information for vulkan").clone());
                }
                Some(_) => {
                    panic!("physical device has already been selected");
                }
            }
        }
        {
            let mut device = self.device.write().unwrap();
            *device = Some(VkDevice::new(&self.get_ref()));
        }
        {
            let mut command_pool = self.command_pool.write().unwrap();
            *command_pool = Some(VkCommandPool::new(self.get_ref()));
        }
    }


    fn enumerate_physical_devices(&self) -> Vec<PhysicalDevice> {
        gfx_object!(self.instance).enumerate_physical_devices()
    }

    fn find_best_suitable_physical_device(&self) -> Result<PhysicalDevice, String> {
        gfx_object!(self.instance).find_best_suitable_gpu_vk()
    }

    fn begin_frame(&self) {}

    fn end_frame(&self) {}

    fn create_buffer(&self, create_infos: &BufferCreateInfo) -> Box<dyn GfxBuffer> {
        Box::new(VkBuffer::new(&self.get_ref(), create_infos))
    }

    fn create_render_pass(&self, create_infos: RenderPassCreateInfos) -> Arc<dyn RenderPass> {
        VkRenderPass::new(self.get_ref(), create_infos)
    }

    fn get_ref(&self) -> GfxRef {
        self.gfx_ref.read().unwrap().upgrade().unwrap().clone()
    }
}

impl GfxVulkan {
    pub fn new() -> GfxRef {
        unsafe { G_VULKAN = Some(Entry::load().expect("failed to load vulkan library")); }

        let instance = VkInstance::new(InstanceCreateInfos {
            enable_validation_layers: true,
            ..Default::default()
        }).expect("failed to create instance");

        let mut gfx = Arc::new(Self {
            instance: Some(instance),
            physical_device: Default::default(),
            physical_device_vk: Default::default(),
            device: Default::default(),
            gfx_ref: RwLock::new(Weak::new()),
            command_pool: Default::default(),
        });

        {
            let mut gfx_ref = gfx.gfx_ref.write().unwrap();
            *gfx_ref = Arc::downgrade(&gfx);
        }

        gfx
    }
}

impl Drop for GfxVulkan {
    fn drop(&mut self) {
        unsafe {
            G_VULKAN = None;
        }
    }
}