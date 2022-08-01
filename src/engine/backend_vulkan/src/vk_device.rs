use std::cell::Cell;
use std::os::raw::{c_char, c_void};
use std::sync::{Arc, Mutex, RwLock};

use ash::vk;
use ash::vk::{Bool32, PhysicalDevice};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};

use crate::{gfx_object, GfxVulkan, vk_check};

pub fn get_required_device_extensions() -> Vec<*const c_char> {
    let mut result = Vec::new();
    result.push("VK_KHR_swapchain\0".as_ptr() as *const c_char);
    result
}


pub struct VkDevice {
    pub device: ash::Device,
    pub allocator: Mutex<Allocator>,
}

impl VkDevice {
    pub fn new(gfx: &GfxVulkan) -> VkDevice {
        let mut ci_queues = Vec::<vk::DeviceQueueCreateInfo>::new();

        let queue_priorities: f32 = 1.0;
        for queue in &gfx_object!(gfx.physical_device_vk).queues {
            ci_queues.push({
                vk::DeviceQueueCreateInfo {
                    queue_family_index: queue.index,
                    queue_count: 1,
                    p_queue_priorities: &queue_priorities,
                    ..Default::default()
                }
            });
        }

        let mut device_features = vk::PhysicalDeviceFeatures {
            geometry_shader: false as Bool32,
            sample_rate_shading: true as Bool32, // Sample Shading
            fill_mode_non_solid: true as Bool32, // Wireframe
            wide_lines: true as Bool32,
            sampler_anisotropy: true as Bool32,
            ..Default::default()
        };

        if gfx_object!(gfx.instance).enable_validation_layers() {
            device_features.robust_buffer_access = 0;
        }

        let index_features = [vk::PhysicalDeviceDescriptorIndexingFeatures::builder()
            .descriptor_binding_partially_bound(true)
            .runtime_descriptor_array(true)
            .build()];

        let index_features_2 = vk::PhysicalDeviceFeatures2 {
            p_next: &index_features as *const vk::PhysicalDeviceDescriptorIndexingFeatures as *mut c_void,
            features: device_features,
            ..Default::default()
        };

        let mut extensions = get_required_device_extensions().clone();

        if gfx_object!(gfx.instance).enable_validation_layers() {
            extensions.push("VK_EXT_debug_marker\0".as_ptr() as *const c_char);
        }

        let ci_device = vk::DeviceCreateInfo {
            queue_create_info_count: ci_queues.len() as u32,
            p_queue_create_infos: ci_queues.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            p_enabled_features: &index_features_2 as *const vk::PhysicalDeviceFeatures2 as *const vk::PhysicalDeviceFeatures,
            ..Default::default()
        };

        let mut ps: PhysicalDevice = Default::default();
        unsafe {
            if let Some(devices) = gfx_object!(gfx.instance).instance.enumerate_physical_devices().ok() {
                for device in devices {
                    ps = device;
                    break;
                }
            }
        }

        let device = vk_check!(unsafe { gfx_object!(gfx.instance).instance.create_device(
            ps,
            &ci_device,
            None
        ) });

        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: gfx_object!(gfx.instance).instance.clone(),
            device: device.clone(),
            physical_device: gfx_object!(gfx.physical_device_vk).device,
            debug_settings: gpu_allocator::AllocatorDebugSettings {
                log_leaks_on_shutdown: true,
                ..Default::default()
            },
            buffer_device_address: false,
        }).expect("failed to create AMD Vulkan memory allocator");

        Self {
            device,
            allocator: Mutex::new(allocator),
        }
    }
}