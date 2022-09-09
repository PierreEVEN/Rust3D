use std::collections::HashMap;
use std::os::raw::{c_char, c_void};
use std::sync::{Arc, RwLock};

use ash::{Device, vk};
use ash::extensions::khr::Swapchain;
use ash::prelude::VkResult;
use ash::vk::{Bool32, DeviceCreateInfo, DeviceQueueCreateInfo, Fence, FenceCreateFlags, FenceCreateInfo, PhysicalDevice, PhysicalDeviceDescriptorIndexingFeatures, PhysicalDeviceFeatures, PhysicalDeviceFeatures2, PresentInfoKHR, Queue, QueueFlags, SubmitInfo};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};

use gfx::GfxRef;

use crate::{GfxVulkan, vk_check};

pub fn get_required_device_extensions() -> Vec<*const c_char> {
    let mut result = Vec::new();
    result.push("VK_KHR_swapchain\0".as_ptr() as *const c_char);
    result
}

pub struct VkQueue {
    queue: Queue,
    pub flags: QueueFlags,
    pub index: u32,
    gfx: GfxRef,
    fence: Fence,
}

impl VkQueue {
    pub fn new(device: &Device, queue: Queue, flags: QueueFlags, index: u32, gfx: &GfxRef) -> Arc<Self> {
        let ci_fence = FenceCreateInfo {
            flags: FenceCreateFlags::SIGNALED,
            ..FenceCreateInfo::default()
        };

        let fence = vk_check!(unsafe { device.create_fence(&ci_fence, None) });

        Arc::new(Self {
            queue,
            flags,
            index,
            gfx: gfx.clone(),
            fence,
        })
    }

    pub fn wait(&self) {
        vk_check!(unsafe { self.gfx.cast::<GfxVulkan>().device.handle.wait_for_fences(&[self.fence], true, u64::MAX) });
    }

    pub fn submit(&self, submit_infos: SubmitInfo) {
        self.wait();
        vk_check!(unsafe { self.gfx.cast::<GfxVulkan>().device.handle.reset_fences(&[self.fence]) });
        vk_check!(unsafe { self.gfx.cast::<GfxVulkan>().device.handle.queue_submit(self.queue, &[submit_infos], self.fence) });
    }
    pub fn present(&self, swapchain: &Swapchain, present_infos: PresentInfoKHR) -> VkResult<bool> {
        unsafe { swapchain.queue_present(self.queue, &present_infos) }
    }
}

pub struct VkDevice {
    pub handle: ash::Device,
    pub queues: HashMap<QueueFlags, Vec<Arc<VkQueue>>>,
    pub allocator: Arc<RwLock<Allocator>>,
}


impl VkDevice {
    pub fn new(gfx: &GfxRef) -> VkDevice {
        let mut ci_queues = Vec::<vk::DeviceQueueCreateInfo>::new();

        let queue_priorities: f32 = 1.0;
        for queue in &gfx.cast::<GfxVulkan>().physical_device_vk.queues {
            ci_queues.push({
                DeviceQueueCreateInfo {
                    queue_family_index: queue.index,
                    queue_count: 1,
                    p_queue_priorities: &queue_priorities,
                    ..Default::default()
                }
            });
        }

        let mut extensions = get_required_device_extensions().clone();
        if gfx.cast::<GfxVulkan>().instance.enable_validation_layers() {
            extensions.push("VK_EXT_debug_marker\0".as_ptr() as *const c_char);
        }

        let device_features = PhysicalDeviceFeatures {
            geometry_shader: false as Bool32,
            sample_rate_shading: true as Bool32, // Sample Shading
            fill_mode_non_solid: true as Bool32, // Wireframe
            wide_lines: true as Bool32,
            sampler_anisotropy: true as Bool32,
            robust_buffer_access: gfx.cast::<GfxVulkan>().instance.enable_validation_layers() as Bool32,
            ..Default::default()
        };

        let index_features = PhysicalDeviceDescriptorIndexingFeatures {
            descriptor_binding_partially_bound: false as Bool32,
            runtime_descriptor_array: false as Bool32,
            ..PhysicalDeviceDescriptorIndexingFeatures::default()
        };

        let index_features_2 = PhysicalDeviceFeatures2 {
            p_next: &index_features as *const PhysicalDeviceDescriptorIndexingFeatures as *mut c_void,
            features: device_features,
            ..PhysicalDeviceFeatures2::default()
        };

        let ci_device = DeviceCreateInfo {
            p_next: &index_features_2 as *const PhysicalDeviceFeatures2 as *mut c_void,
            queue_create_info_count: ci_queues.len() as u32,
            p_queue_create_infos: ci_queues.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            ..Default::default()
        };

        let mut ps: PhysicalDevice = Default::default();
        unsafe {
            if let Some(devices) = gfx.cast::<GfxVulkan>().instance.handle.enumerate_physical_devices().ok() {
                for device in devices {
                    ps = device;
                    break;
                }
            }
        }

        let device = vk_check!(unsafe { gfx.cast::<GfxVulkan>().instance.handle.create_device(
            ps,
            &ci_device,
            None
        ) });

        // Create allocator
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: gfx.cast::<GfxVulkan>().instance.handle.clone(),
            device: device.clone(),
            physical_device: gfx.cast::<GfxVulkan>().physical_device_vk.handle,
            debug_settings: gpu_allocator::AllocatorDebugSettings {
                log_leaks_on_shutdown: true,
                ..Default::default()
            },
            buffer_device_address: false,
        }).expect("failed to create GPU Vulkan memory allocator");

        let mut queue_map = HashMap::<QueueFlags, Vec<Arc<VkQueue>>>::new();
        for queue_details in &gfx.cast::<GfxVulkan>().physical_device_vk.queues
        {
            let queue = VkQueue::new(&device, unsafe { device.get_device_queue(queue_details.index, 0) }, queue_details.flags, queue_details.index, gfx);

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
            handle: device,
            queues : queue_map,
            allocator : Arc::new(RwLock::new(allocator)),
        }
    }

    pub fn get_queue(&self, flags: QueueFlags) -> Result<Arc<VkQueue>, ()> {
        match self.queues.get(&flags) {
            None => {}
            Some(queues) => {
                if !queues.is_empty() {
                    return Ok(queues[0].clone());
                }
            }
        }


        Err(())
    }
}