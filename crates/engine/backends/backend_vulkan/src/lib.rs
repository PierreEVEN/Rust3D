extern crate core;

use std::any::TypeId;
use std::collections::HashMap;
use std::default::Default;
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::sync::{Arc, RwLock, Weak};
use ash::vk;

use gfx::{GfxInterface, GfxRef, PhysicalDevice};
use gfx::buffer::{BufferCreateInfo, GfxBuffer};
use gfx::command_buffer::GfxCommandBuffer;
use gfx::image::{GfxImage, ImageCreateInfos};
use gfx::image_sampler::{ImageSampler, SamplerCreateInfos};
use gfx::render_pass::{RenderPass, RenderPassCreateInfos};
use gfx::shader::{PassID, ShaderProgram, ShaderProgramInfos};
use gfx::surface::GfxSurface;

use crate::vk_buffer::VkBuffer;
use crate::vk_command_buffer::{VkCommandBuffer, VkCommandPool};
use crate::vk_descriptor_pool::VkDescriptorPool;
use crate::vk_device::VkDevice;
use crate::vk_image::VkImage;
use crate::vk_image_sampler::VkImageSampler;
use crate::vk_instance::{InstanceCreateInfos, VkInstance};
use crate::vk_physical_device::VkPhysicalDevice;
use crate::vk_render_pass::VkRenderPass;
use crate::vk_shader::VkShaderProgram;
use crate::vk_shader_instance::VkShaderInstance;

pub mod vk_instance;
pub mod vk_physical_device;
pub mod vk_device;
pub mod vk_types;
pub mod vk_render_pass;
pub mod vk_buffer;
pub mod vk_shader;
pub mod vk_render_pass_instance;
pub mod vk_command_buffer;
pub mod vk_image;
pub mod vk_image_sampler;
pub mod vk_shader_instance;
pub mod vk_descriptor_pool;
pub mod vk_dst_set_layout;

pub static mut G_VULKAN: Option<ash::Entry> = None;

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
macro_rules! vk_check {
    ($expression:expr) => {
        match $expression {
            Ok(object) => { object }
            Err(vk_err) => { panic!("vk error : {}\non '{}'", vk_err.to_string(), stringify!(expression)) }
        }
    }
}

pub struct GfxVulkan {
    pub instance: VkInstance,
    pub physical_device: PhysicalDevice,
    pub physical_device_vk: VkPhysicalDevice,
    pub device: VkDevice,
    pub gfx_ref: Weak<GfxVulkan>,
    pub command_pool: VkCommandPool,
    pub descriptor_pool: VkDescriptorPool,
    render_passes: RwLock<HashMap<PassID, Arc<dyn RenderPass>>>,
}

impl GfxInterface for GfxVulkan {
    fn set_physical_device(&self, selected_device: PhysicalDevice) {
        unsafe { (&self.physical_device as *const PhysicalDevice as *mut PhysicalDevice).write(selected_device.clone()) };
        unsafe { (&self.physical_device_vk as *const VkPhysicalDevice as *mut VkPhysicalDevice).write(self.instance.get_vk_device(&selected_device).expect("failed to get physical device information for vulkan").clone()) };
        unsafe { (&self.device as *const VkDevice as *mut VkDevice).write(VkDevice::new(&self.get_ref())) };
        unsafe { (&self.command_pool as *const VkCommandPool as *mut VkCommandPool).write(VkCommandPool::new(&self.get_ref(), format!("global"))) };
        unsafe { (&self.descriptor_pool as *const VkDescriptorPool as *mut VkDescriptorPool).write(VkDescriptorPool::new(&self.get_ref(), 64, 64)) };

        self.set_vk_object_name(self.device.handle.handle(), format!("device\t\t: global ").as_str());
        self.set_vk_object_name(self.physical_device_vk.handle, format!("physical device\t\t: {}", self.physical_device.device_name).as_str());
    }


    fn enumerate_physical_devices(&self) -> Vec<PhysicalDevice> {
        self.instance.enumerate_physical_devices()
    }

    fn find_best_suitable_physical_device(&self) -> Result<PhysicalDevice, String> {
        self.instance.find_best_suitable_gpu_vk()
    }

    fn create_buffer(&self, name: String, create_infos: &BufferCreateInfo) -> Arc<dyn GfxBuffer> {
        Arc::new(VkBuffer::new(&self.get_ref(), name, create_infos))
    }

    fn create_shader_program(&self, name: String, render_pass: &Arc<dyn RenderPass>, create_infos: &ShaderProgramInfos) -> Arc<dyn ShaderProgram> {
        VkShaderProgram::new(&self.get_ref(), name, render_pass, create_infos)
    }

    fn create_render_pass(&self, name: String, create_infos: RenderPassCreateInfos) -> Arc<dyn RenderPass> {
        VkRenderPass::new(&self.get_ref(), name, create_infos)
    }

    fn create_image(&self, name: String, create_infos: ImageCreateInfos) -> Arc<dyn GfxImage> {
        VkImage::new(&self.get_ref(), name, create_infos)
    }

    fn create_image_sampler(&self, name: String, create_infos: SamplerCreateInfos) -> Arc<dyn ImageSampler> {
        VkImageSampler::new(&self.get_ref(), name, create_infos)
    }

    fn find_render_pass(&self, pass_id: &PassID) -> Option<Arc<dyn RenderPass>> {
        match self.render_passes.read().unwrap().get(pass_id) {
            None => { None }
            Some(pass) => { Some(pass.clone()) }
        }
    }

    fn create_command_buffer(&self, name: String, surface: &Arc<dyn GfxSurface>) -> Arc<dyn GfxCommandBuffer> {
        VkCommandBuffer::new(&self.get_ref(), name, surface)
    }

    fn get_ref(&self) -> GfxRef {
        self.gfx_ref.upgrade().unwrap().clone()
    }
}

impl GfxVulkan {
    pub fn new() -> GfxRef {
        unsafe { G_VULKAN = Some(ash::Entry::load().expect("failed to load vulkan library")); }

        let instance = VkInstance::new(InstanceCreateInfos {
            enable_validation_layers: true,
            ..InstanceCreateInfos::default()
        }).expect("failed to create instance");

        let physical_device = MaybeUninit::zeroed();
        let physical_device_vk = MaybeUninit::zeroed();
        let device = MaybeUninit::zeroed();
        let command_pool = MaybeUninit::zeroed();
        let descriptor_pool = MaybeUninit::zeroed();
        
        let gfx = Arc::new(Self {
            instance,
            physical_device: unsafe { physical_device.assume_init() },
            physical_device_vk: unsafe { physical_device_vk.assume_init() },
            device: unsafe { device.assume_init() },
            gfx_ref: Weak::new(),
            command_pool: unsafe { command_pool.assume_init() },
            descriptor_pool: unsafe { descriptor_pool.assume_init() },
            render_passes: RwLock::default(),
        });
        unsafe { (&gfx.gfx_ref as *const Weak<GfxVulkan> as *mut Weak<GfxVulkan>).write(Arc::downgrade(&gfx)) };
        gfx
    }

    pub fn set_vk_object_name<T: vk::Handle + 'static + Copy>(&self, object: T, name: &str) -> T {
        let object_type =
            if TypeId::of::<vk::Instance>() == TypeId::of::<T>() {
                vk::ObjectType::INSTANCE
            } else if TypeId::of::<vk::PhysicalDevice>() == TypeId::of::<T>() {
                vk::ObjectType::PHYSICAL_DEVICE
            } else if TypeId::of::<vk::Device>() == TypeId::of::<T>() {
                vk::ObjectType::DEVICE
            } else if TypeId::of::<vk::Queue>() == TypeId::of::<T>() {
                vk::ObjectType::QUEUE
            } else if TypeId::of::<vk::Semaphore>() == TypeId::of::<T>() {
                vk::ObjectType::SEMAPHORE
            } else if TypeId::of::<vk::CommandBuffer>() == TypeId::of::<T>() {
                vk::ObjectType::COMMAND_BUFFER
            } else if TypeId::of::<vk::Fence>() == TypeId::of::<T>() {
                vk::ObjectType::FENCE
            } else if TypeId::of::<vk::DeviceMemory>() == TypeId::of::<T>() {
                vk::ObjectType::DEVICE_MEMORY
            } else if TypeId::of::<vk::Buffer>() == TypeId::of::<T>() {
                vk::ObjectType::BUFFER
            } else if TypeId::of::<vk::Image>() == TypeId::of::<T>() {
                vk::ObjectType::IMAGE
            } else if TypeId::of::<vk::Event>() == TypeId::of::<T>() {
                vk::ObjectType::EVENT
            } else if TypeId::of::<vk::QueryPool>() == TypeId::of::<T>() {
                vk::ObjectType::QUERY_POOL
            } else if TypeId::of::<vk::BufferView>() == TypeId::of::<T>() {
                vk::ObjectType::BUFFER_VIEW
            } else if TypeId::of::<vk::ImageView>() == TypeId::of::<T>() {
                vk::ObjectType::IMAGE_VIEW
            } else if TypeId::of::<vk::ShaderModule>() == TypeId::of::<T>() {
                vk::ObjectType::SHADER_MODULE
            } else if TypeId::of::<vk::PipelineCache>() == TypeId::of::<T>() {
                vk::ObjectType::PIPELINE_CACHE
            } else if TypeId::of::<vk::PipelineLayout>() == TypeId::of::<T>() {
                vk::ObjectType::PIPELINE_LAYOUT
            } else if TypeId::of::<vk::RenderPass>() == TypeId::of::<T>() {
                vk::ObjectType::RENDER_PASS
            } else if TypeId::of::<vk::Pipeline>() == TypeId::of::<T>() {
                vk::ObjectType::PIPELINE
            } else if TypeId::of::<vk::DescriptorSetLayout>() == TypeId::of::<T>() {
                vk::ObjectType::DESCRIPTOR_SET_LAYOUT
            } else if TypeId::of::<vk::Sampler>() == TypeId::of::<T>() {
                vk::ObjectType::SAMPLER
            } else if TypeId::of::<vk::DescriptorPool>() == TypeId::of::<T>() {
                vk::ObjectType::DESCRIPTOR_POOL
            } else if TypeId::of::<vk::DescriptorSet>() == TypeId::of::<T>() {
                vk::ObjectType::DESCRIPTOR_SET
            } else if TypeId::of::<vk::Framebuffer>() == TypeId::of::<T>() {
                vk::ObjectType::FRAMEBUFFER
            } else if TypeId::of::<vk::CommandPool>() == TypeId::of::<T>() {
                vk::ObjectType::COMMAND_POOL
            } else if TypeId::of::<vk::SurfaceKHR>() == TypeId::of::<T>() {
                vk::ObjectType::SURFACE_KHR
            } else if TypeId::of::<vk::SwapchainKHR>() == TypeId::of::<T>() {
                vk::ObjectType::SWAPCHAIN_KHR
            } else {
                panic!("unhandled object type id")
            };

        let string_name = format!("{}\0", name);

        unsafe {
            vk_check!(self.instance.debug_util_loader.debug_utils_set_object_name(self.device.handle.handle(), &vk::DebugUtilsObjectNameInfoEXT::builder()
                .object_type(object_type)
                .object_handle(object.as_raw())
                .object_name(CStr::from_ptr(string_name.as_ptr() as *const c_char))
                .build()))
        }

        object
    }
}

impl Drop for GfxVulkan {
    fn drop(&mut self) {
        unsafe {
            G_VULKAN = None;
        }
    }
}

pub trait GfxVkObject {
    fn construct(&mut self, gfx: &GfxRef);
    fn is_valid(&self) -> bool;
}
