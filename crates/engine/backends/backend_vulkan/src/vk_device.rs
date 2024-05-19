use std::collections::HashMap;
use std::fmt::Error;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use ash::extensions::khr::Swapchain;
use ash::prelude::VkResult;
use ash::vk;
use gpu_allocator::vulkan;

use crate::{vk_check, GfxVulkan};

pub fn get_required_device_extensions() -> Vec<*const c_char> {
    vec!["VK_KHR_swapchain\0".as_ptr() as *const c_char,
         "VK_KHR_timeline_semaphore\0".as_ptr() as *const c_char]
}

pub struct VkQueue {
    queue: Mutex<vk::Queue>,
    pub flags: vk::QueueFlags,
    pub index: u32,
    fence: vk::Fence,
}

impl VkQueue {
    pub fn new(
        device: &ash::Device,
        queue: vk::Queue,
        flags: vk::QueueFlags,
        index: u32,
    ) -> Arc<Self> {

        let ci_fence = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        let fence = vk_check!(unsafe { device.create_fence(&ci_fence, None) });

        Arc::new(Self {
            queue: Mutex::new(queue),
            flags,
            index,
            fence,
        })
    }
    
    pub fn wait(&self) {
        vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .wait_for_fences(&[self.fence], true, u64::MAX)
        });
    }

    pub fn submit(&self, submit_infos: vk::SubmitInfo) {
        let locked_queue = &*self.queue.lock().unwrap();
        self.wait();
        vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .reset_fences(&[self.fence])
        });
        vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .queue_submit(*locked_queue, &[submit_infos], self.fence)
        });
    }
    pub fn present(
        &self,
        swapchain: &Swapchain,
        present_infos: vk::PresentInfoKHR,
    ) -> VkResult<bool> {
        unsafe {
            let locked_queue = &*self.queue.lock().unwrap();
            swapchain.queue_present(*locked_queue, &present_infos) }
    }
}

pub struct VkDevice {
    pub handle: ash::Device,
    pub queues: HashMap<vk::QueueFlags, Vec<Arc<VkQueue>>>,
    pub allocator: Arc<RwLock<vulkan::Allocator>>,
}

impl Default for VkDevice {
    fn default() -> Self {
        let mut ci_queues = Vec::<vk::DeviceQueueCreateInfo>::new();

        let queue_priorities: f32 = 1.0;
        for queue in &GfxVulkan::get().physical_device_vk.queues {
            ci_queues.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(queue.index)
                    .queue_priorities(&[queue_priorities])
                    .build(),
            );
        }

        let mut extensions = get_required_device_extensions();
        unsafe {
            if GfxVulkan::get()
                .instance
                .assume_init_ref()
                .enable_validation_layers()
            {
                extensions.push("VK_EXT_debug_marker\0".as_ptr() as *const c_char);
            }
        }

        let device_features = vk::PhysicalDeviceFeatures::builder()
            .geometry_shader(false)
            .sample_rate_shading(true) // Sample Shading
            .fill_mode_non_solid(true) // Wireframe
            .wide_lines(true)
            .sampler_anisotropy(true)
            .robust_buffer_access(
                unsafe { GfxVulkan::get().instance.assume_init_ref() }.enable_validation_layers(),
            )
            .build();

        let mut index_features = vk::PhysicalDeviceDescriptorIndexingFeatures::builder()
            .descriptor_binding_partially_bound(false)
            .runtime_descriptor_array(false)
            .build();

        let mut index_features_2 = vk::PhysicalDeviceFeatures2::builder()
            .push_next(&mut index_features)
            .features(device_features)
            .build();

        logger::warning!("extensions : {:?}", extensions);
        let ci_device = vk::DeviceCreateInfo::builder()
            .push_next(&mut index_features_2)
            .queue_create_infos(ci_queues.as_slice())
            .enabled_extension_names(extensions.as_slice())
            .build();

        let mut ps: vk::PhysicalDevice = Default::default();
        unsafe {
            if let Ok(devices) = GfxVulkan::get()
                .instance
                .assume_init_ref()
                .handle
                .assume_init_ref()
                .enumerate_physical_devices()
            {
                if !devices.is_empty() {
                    ps = devices[0];
                }
            }
        }

        let device = vk_check!(unsafe {
            GfxVulkan::get()
                .instance
                .assume_init_ref()
                .handle
                .assume_init_ref()
                .create_device(ps, &ci_device, None)
        });

        // Create allocator
        let allocator = vulkan::Allocator::new(&vulkan::AllocatorCreateDesc {
            instance: unsafe {
                GfxVulkan::get()
                    .instance
                    .assume_init_ref()
                    .handle
                    .assume_init_ref()
            }
            .clone(),
            device: device.clone(),
            physical_device: GfxVulkan::get().physical_device_vk.handle,
            debug_settings: gpu_allocator::AllocatorDebugSettings {
                log_leaks_on_shutdown: true,
                ..Default::default()
            },
            buffer_device_address: false,
            allocation_sizes: Default::default(),
        })
        .expect("failed to create GPU Vulkan memory allocator");

        let mut queue_map = HashMap::<vk::QueueFlags, Vec<Arc<VkQueue>>>::new();
        for queue_details in &GfxVulkan::get().physical_device_vk.queues {
            let queue = VkQueue::new(
                &device,
                unsafe { device.get_device_queue(queue_details.index, 0) },
                queue_details.flags,
                queue_details.index,
            );

            if queue_details.flags.contains(vk::QueueFlags::GRAPHICS) {
                match queue_map.get_mut(&vk::QueueFlags::GRAPHICS) {
                    None => {
                        queue_map.insert(vk::QueueFlags::GRAPHICS, vec![queue.clone()]);
                    }
                    Some(map_item) => {
                        map_item.push(queue.clone());
                    }
                }
            }
            if queue_details.flags.contains(vk::QueueFlags::COMPUTE) {
                match queue_map.get_mut(&vk::QueueFlags::COMPUTE) {
                    None => {
                        queue_map.insert(vk::QueueFlags::COMPUTE, vec![queue.clone()]);
                    }
                    Some(map_item) => {
                        map_item.push(queue.clone());
                    }
                }
            }
            if queue_details.flags.contains(vk::QueueFlags::TRANSFER) {
                match queue_map.get_mut(&vk::QueueFlags::TRANSFER) {
                    None => {
                        queue_map.insert(vk::QueueFlags::TRANSFER, vec![queue.clone()]);
                    }
                    Some(map_item) => {
                        map_item.push(queue.clone());
                    }
                }
            }
            if queue_details.flags.contains(vk::QueueFlags::SPARSE_BINDING) {
                match queue_map.get_mut(&vk::QueueFlags::SPARSE_BINDING) {
                    None => {
                        queue_map.insert(vk::QueueFlags::SPARSE_BINDING, vec![queue.clone()]);
                    }
                    Some(map_item) => {
                        map_item.push(queue.clone());
                    }
                }
            }
        }

        Self {
            handle: device,
            queues: queue_map,
            allocator: Arc::new(RwLock::new(allocator)),
        }
    }
}

impl VkDevice {
    pub fn get_queue(&self, flags: vk::QueueFlags) -> Result<Arc<VkQueue>, Error> {
        match self.queues.get(&flags) {
            None => {}
            Some(queues) => {
                if !queues.is_empty() {
                    return Ok(queues[0].clone());
                }
            }
        }

        Err(Error::default())
    }
}
