use std::default;
use std::ops::Deref;
use std::os::raw::{c_char, c_void};
use std::ptr::null;
use ash::prelude::VkResult;
use ash::vk;
use ash::vk::{Bool32, PhysicalDevice, PhysicalDeviceFeatures};
use crate::{gfx_object, GfxVulkan, vk_check, VkInstance};

pub fn get_required_device_extensions() -> Vec<*const c_char>{
    let mut result = Vec::new();
    result.push("VK_KHR_swapchain".as_ptr() as *const c_char);    
    result
}


pub struct VkDevice {
    device: ash::Device,
}

impl VkDevice {
    pub fn new(gfx: &GfxVulkan) -> VkDevice {
        let mut ci_queues = Vec::<vk::DeviceQueueCreateInfo>::new();

        ci_queues.push({
            vk::DeviceQueueCreateInfo {
                ..Default::default()
            }
        });

        let mut device_features = vk::PhysicalDeviceFeatures {
            geometry_shader: false as Bool32,
            sample_rate_shading: true as Bool32, // Sample Shading
            fill_mode_non_solid: true as Bool32, // Wireframe
            wide_lines: true as Bool32,
            sampler_anisotropy: true as Bool32,
            ..Default::default()
        };
        if gfx_object!(gfx.instance).enable_validation_layers() {
            device_features.robust_buffer_access = true as Bool32;
        }

        let index_features = vk::PhysicalDeviceDescriptorIndexingFeatures {
            descriptor_binding_partially_bound: true as Bool32,
            runtime_descriptor_array: true as Bool32,
            ..Default::default()
        };
        let index_features_2 = vk::PhysicalDeviceFeatures2 {
            p_next: &index_features as *const vk::PhysicalDeviceDescriptorIndexingFeatures as *mut c_void,
            features: device_features,
            ..Default::default()
        };

        let index_features= vk::PhysicalDeviceFeatures {
            ..Default::default()
        };
        
        let mut extensions = get_required_device_extensions().clone();
        
        if gfx_object!(gfx.instance).enable_validation_layers() {
            extensions.push("VK_EXT_debug_marker".as_ptr() as *const c_char);
        }

        let ci_device = vk::DeviceCreateInfo {
            queue_create_info_count: 0,//ci_queues.len() as u32,
            p_queue_create_infos: null(),// ci_queues.as_ptr(),
            enabled_extension_count: 0,  //extensions.len() as u32,
            pp_enabled_extension_names: null(), //extensions.as_ptr(),
            p_enabled_features: &index_features as *const vk::PhysicalDeviceFeatures,
            ..Default::default()
        };
        
        println!("get device");
        
        let mut ps: PhysicalDevice = Default::default();
        unsafe {
            if let Some(devices) = gfx_object!(gfx.instance).instance.enumerate_physical_devices().ok() {
                for device in devices {
                    ps = device;
                    break;
                }
            }
        }

        println!("get instance");
        
        let device = vk_check!(unsafe { gfx_object!(gfx.instance).instance.create_device(
            ps,
            &ci_device,
            None
        ) });
        
        Self {
            device,
        }
    }
}