use std::cell::RefCell;
use std::collections::HashMap;
use std::os::raw::{c_char, c_void};
use std::sync::Arc;

use ash::vk;
use ash::vk::{Bool32, DeviceQueueCreateInfo, PhysicalDevice, Queue, QueueFlags};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};

use gfx::GfxRef;

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check};

pub fn get_required_device_extensions() -> Vec<*const c_char> {
    let mut result = Vec::new();
    result.push("VK_KHR_swapchain\0".as_ptr() as *const c_char);
    result
}

pub struct VkQueue {
    pub queue: Queue,
    pub flags: QueueFlags,
    pub index: u32,
}

impl VkQueue {}

pub struct VkDevice {
    pub device: ash::Device,
    pub queues: HashMap<QueueFlags, Vec<Arc<VkQueue>>>,
    pub allocator: Arc<RefCell<Allocator>>,
}

impl VkDevice {
    pub fn new(gfx: &GfxRef) -> VkDevice {
        let mut ci_queues = Vec::<vk::DeviceQueueCreateInfo>::new();
        let physical_device = gfx_cast_vulkan!(gfx).physical_device_vk.read().unwrap();
        let instance = gfx_object!(gfx_cast_vulkan!(gfx).instance);

        let queue_priorities: f32 = 1.0;
        for queue in &gfx_object!(*physical_device).queues {
            ci_queues.push({
                DeviceQueueCreateInfo {
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

        if gfx_object!(gfx_cast_vulkan!(gfx).instance).enable_validation_layers() {
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

        if instance.enable_validation_layers() {
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
            if let Some(devices) = instance.instance.enumerate_physical_devices().ok() {
                for device in devices {
                    ps = device;
                    break;
                }
            }
        }

        let device = vk_check!(unsafe { instance.instance.create_device(
            ps,
            &ci_device,
            None
        ) });

        // Create allocator
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.instance.clone(),
            device: device.clone(),
            physical_device: gfx_object!(*physical_device).device,
            debug_settings: gpu_allocator::AllocatorDebugSettings {
                log_leaks_on_shutdown: true,
                ..Default::default()
            },
            buffer_device_address: false,
        }).expect("failed to create AMD Vulkan memory allocator");

        let mut queue_map = HashMap::<QueueFlags, Vec<Arc<VkQueue>>>::new();
        for queue_details in &gfx_object!(*physical_device).queues
        {
            let queue = Arc::new(VkQueue {
                queue: unsafe { device.get_device_queue(queue_details.index, 0) },
                flags: queue_details.flags,
                index: queue_details.index,
            });

            if queue_details.flags.contains(QueueFlags::GRAPHICS) {
                match queue_map.get_mut(&QueueFlags::GRAPHICS) {
                    None => { queue_map.insert(QueueFlags::GRAPHICS, vec![queue.clone()]); }
                    Some(map_item) => { map_item.push(queue.clone()); }
                }
            }
            if queue_details.flags.contains(QueueFlags::COMPUTE) {
                match queue_map.get_mut(&QueueFlags::COMPUTE) {
                    None => { queue_map.insert(QueueFlags::COMPUTE, vec![queue.clone()]); }
                    Some(map_item) => { map_item.push(queue.clone()); }
                }
            }
            if queue_details.flags.contains(QueueFlags::TRANSFER) {
                match queue_map.get_mut(&QueueFlags::TRANSFER) {
                    None => { queue_map.insert(QueueFlags::TRANSFER, vec![queue.clone()]); }
                    Some(map_item) => { map_item.push(queue.clone()); }
                }
            }
            if queue_details.flags.contains(QueueFlags::SPARSE_BINDING) {
                match queue_map.get_mut(&QueueFlags::SPARSE_BINDING) {
                    None => { queue_map.insert(QueueFlags::SPARSE_BINDING, vec![queue.clone()]); }
                    Some(map_item) => { map_item.push(queue.clone()); }
                }
            }
        }


        Self {
            device,
            queues: queue_map,
            allocator: Arc::new(RefCell::new(allocator)),
        }
    }
}