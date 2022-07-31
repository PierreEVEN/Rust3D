use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ops::Add;
use std::os::raw::c_char;
use std::sync::Arc;

use ash::{Instance, vk};
use ash::extensions::ext::DebugUtils;
use ash::vk::DebugUtilsMessengerEXT;

use gfx::PhysicalDevice;

use crate::{G_VULKAN, g_vulkan, to_c_char};
use crate::vk_physical_device::VkPhysicalDevice;

#[derive(Default, Clone)]
pub struct InstanceCreateInfos {
    pub required_layers: Vec<(String, bool)>,
    pub required_extensions: Vec<(String, bool)>,
    pub enable_validation_layers: bool,
}

pub struct VkInstance {
    pub instance: Instance,
    debug_util_loader: DebugUtils,
    debug_messenger: DebugUtilsMessengerEXT,
    enable_validation_layers: bool,
    device_map: HashMap<PhysicalDevice, VkPhysicalDevice>,
}

impl VkInstance {
    pub fn new(create_infos: InstanceCreateInfos) -> Result<VkInstance, std::io::Error> {
        // Build extensions and layer
        let mut required_layers = Vec::new();
        let mut required_extensions = Vec::new();        
        let mut layers_names_raw = Vec::<*const c_char>::new();
        let mut extension_names_raw = Vec::<*const c_char>::new();
        
        for (mut layer_name, required) in create_infos.required_layers {
            let is_available = VkInstance::is_layer_available(layer_name.as_str());
            if !is_available {
                if required { panic!("required layer [{}] is not available", layer_name); }
                else { println!("optional layer [{}] is not available", layer_name); }
                continue;
            }
            layer_name += "\0";
            required_layers.push(layer_name);
        }        
        for (mut extension_name, required) in create_infos.required_extensions {
            let is_available = VkInstance::is_extension_available(extension_name.as_str());
            if !is_available {
                if required { panic!("required layer [{}] is not available", extension_name); } else { println!("optional layer [{}] is not available", extension_name); }
                continue;
            }
            extension_name += "\0";
            required_extensions.push(extension_name);
        }
        
        // Add validation layers
        let enable_validation_layers = create_infos.enable_validation_layers && VkInstance::is_layer_available("VK_LAYER_KHRONOS_validation");
        if enable_validation_layers {
            required_layers.push("VK_LAYER_KHRONOS_validation\0".to_string());
            required_extensions.push(DebugUtils::name().to_str().unwrap().to_string() + "\0");
            required_extensions.push("VK_EXT_debug_report\0".to_string());
        }
        required_extensions.push("VK_KHR_surface\0".to_string());
        
        for layer in &required_layers {
            layers_names_raw.push(layer.as_str().as_ptr() as *const c_char);
        }
        for extension in &required_extensions {
            extension_names_raw.push(extension.as_str().as_ptr() as *const c_char);
        }

        // Create instance
        let ci_instance = vk::InstanceCreateInfo {
            p_application_info: &vk::ApplicationInfo {
                p_application_name: to_c_char!(""),
                application_version: vk::make_api_version(0, 0, 1, 0),
                p_engine_name: to_c_char!(""),
                engine_version: vk::make_api_version(0, 0, 1, 0),
                api_version: vk::make_api_version(0, 1, 2, 0),
                ..Default::default()
            },
            pp_enabled_layer_names: layers_names_raw.as_ptr(),
            enabled_layer_count: layers_names_raw.len() as u32,
            pp_enabled_extension_names: extension_names_raw.as_ptr(),
            enabled_extension_count: extension_names_raw.len() as u32,
            ..Default::default()
        };
        let instance = unsafe { g_vulkan!().create_instance(&ci_instance, None) }.expect("failed to create instance");
        
        // Create debug messenger
        let _debug_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            pfn_user_callback: Some(vulkan_debug_callback),
            ..Default::default()
        };

        let debug_util_loader = DebugUtils::new(g_vulkan!(), &instance);

        let debug_messenger =
            if enable_validation_layers {
                unsafe { debug_util_loader.create_debug_utils_messenger(&_debug_info, None) }.unwrap()
            } else {
                Default::default()
            };

        let mut device_map = HashMap::new();
        unsafe {
            if let Some(devices) = instance.enumerate_physical_devices().ok() {
                for device in devices {
                    let (device, vk_device) = VkPhysicalDevice::new(&instance, device);
                    device_map.insert(device, vk_device);
                }
            }
        }

        Ok(Self {
            instance,
            debug_util_loader,
            debug_messenger,
            enable_validation_layers,
            device_map,
        })
    }

    pub fn is_layer_available(layer: &str) -> bool {
        if let Some(layer_properties) = g_vulkan!().enumerate_instance_layer_properties().ok() {
            unsafe {
                for layer_details in layer_properties {
                    if CStr::from_ptr(layer_details.layer_name.as_ptr()).to_str().expect("failed to read layer name") == layer {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_extension_available(layer: &str) -> bool {
        if let Some(extensions_properties) = g_vulkan!().enumerate_instance_extension_properties(None).ok() {
            unsafe {
                for extension in extensions_properties {
                    if CStr::from_ptr(extension.extension_name.as_ptr()).to_str().expect("failed to read extension name") == layer {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn enable_validation_layers(&self) -> bool {
        return self.enable_validation_layers;
    }

    pub fn enumerate_physical_devices(&self) -> Vec<PhysicalDevice> {
        let mut result = Vec::new();
        for (device, _) in &self.device_map {
            result.push(device.clone());
        }
        result
    }

    pub fn enumerate_graphic_devices_vk(&self) -> Vec<PhysicalDevice> {
        let mut result = Vec::new();
        for device in self.enumerate_physical_devices() {
            match self.get_vk_device(&device) {
                Ok(vk_device) => {
                    if vk_device.suitable_for_graphics() {
                        result.push(device);
                    }
                }
                Err(_) => {}
            }
        }
        result
    }

    pub fn get_vk_device(&self, device: &PhysicalDevice) -> Result<&VkPhysicalDevice, ()> {
        match self.device_map.get(device) {
            None => { Err(()) }
            Some(elem) => { Ok(elem) }
        }
    }

    pub fn find_best_suitable_gpu_vk(&self) -> Result<PhysicalDevice, String> {
        let mut max_found: PhysicalDevice = Default::default();
        let mut max_score: u32 = 0;

        for device in self.enumerate_graphic_devices_vk() {
            if device.score > max_score {
                max_score = device.score;
                max_found = device;
            }
        }

        if max_score > 0 {
            return Ok(max_found);
        }

        Err("failed to find suitable GPU".to_string())
    }
}

impl Drop for VkInstance {
    fn drop(&mut self) {
        if self.enable_validation_layers {
            // unsafe { self.debug_util_loader.destroy_debug_utils_messenger(self.debug_messenger, None); }
        }
    }
}

unsafe extern "system" fn vulkan_debug_callback(message_severity: vk::DebugUtilsMessageSeverityFlagsEXT, message_type: vk::DebugUtilsMessageTypeFlagsEXT, p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT, _user_data: *mut std::os::raw::c_void) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}