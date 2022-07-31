﻿use std::ffi::CStr;
use ash::version::InstanceV1_0;
use ash::vk;
use ash::vk::{ExtensionProperties, QueueFlags};
use gfx::{PhysicalDevice, PhysicalDeviceType};
use crate::{gfx_object, GfxVulkan};

#[derive(Default, Clone)]
pub struct DeviceQueueProperties {
    pub index: u32,
    pub flags: QueueFlags,
}

#[derive(Default, Clone)]
pub struct VkPhysicalDevice {
    pub device: vk::PhysicalDevice,
    pub queues: Vec<DeviceQueueProperties>,
}


impl VkPhysicalDevice {
    pub fn new(instance: &ash::Instance, device: vk::PhysicalDevice) -> (PhysicalDevice, VkPhysicalDevice) {
        let mut device_properties = PhysicalDevice::default();
        let mut queues = Vec::new();
        let mut queue_index: u32 = 0;
        let mut score: u32 = 0;
        unsafe {
            let properties = instance.get_physical_device_properties(device);
            device_properties.api_version = properties.api_version;
            device_properties.driver_version = properties.driver_version;
            device_properties.vendor_id = properties.vendor_id;
            device_properties.device_id = properties.device_id;
            device_properties.device_name = CStr::from_ptr(properties.device_name.as_ptr()).to_str().expect("failed to read string").to_string();
            device_properties.device_type = match properties.device_type {
                vk::PhysicalDeviceType::OTHER => { PhysicalDeviceType::Undefined }
                vk::PhysicalDeviceType::INTEGRATED_GPU => { PhysicalDeviceType::IntegratedGPU }
                vk::PhysicalDeviceType::DISCRETE_GPU => { PhysicalDeviceType::DedicatedGPU }
                vk::PhysicalDeviceType::VIRTUAL_GPU => { PhysicalDeviceType::VirtualGPU }
                vk::PhysicalDeviceType::CPU => { PhysicalDeviceType::CPU }
                _ => PhysicalDeviceType::Undefined
            };

            score += properties.limits.max_image_dimension2_d;

            for property in instance.get_physical_device_queue_family_properties(device) {
                queues.push(DeviceQueueProperties {
                    index: queue_index,
                    flags: property.queue_flags,
                });

                if property.queue_flags.contains(QueueFlags::GRAPHICS) { score += 100; }
                if property.queue_flags.contains(QueueFlags::COMPUTE) { score += 200; }
                if property.queue_flags.contains(QueueFlags::TRANSFER) { score += 200; }
                
                queue_index += 1;
            }
        }
        
        match device_properties.device_type {
            PhysicalDeviceType::Undefined => { score += 0 }
            PhysicalDeviceType::IntegratedGPU => { score += 100 }
            PhysicalDeviceType::DedicatedGPU => { score += 1000 }
            PhysicalDeviceType::VirtualGPU => { score += 0 }
            PhysicalDeviceType::CPU => { score = 0 }
        }
        
        device_properties.score = score;

        (device_properties,
         Self {
             device,
             queues,
         }
        )
    }

    pub fn suitable_for_graphics(&self) -> bool {
        for queue in &self.queues {
            if queue.flags.contains(QueueFlags::GRAPHICS) {
                return true;
            }
        }
        false
    }
    
    pub fn enumerate_device_extensions(&self, gfx: &GfxVulkan) -> Vec<ExtensionProperties> {
        let mut result = Vec::new();
        unsafe {
            if let Some(extensions) = gfx_object!(gfx.instance).instance.enumerate_device_extension_properties(gfx_object!(gfx.physical_device_vk).device).ok() {
                for extension in extensions {
                    result.push(extension);
                }
            }
        }
        result
    }
}