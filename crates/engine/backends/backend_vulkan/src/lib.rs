extern crate core;

use std::any::TypeId;
use std::default::Default;
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use ash::vk;

use gfx::{Gfx, GfxInterface, PhysicalDevice};
use gfx::buffer::{BufferCreateInfo, GfxBuffer};
use gfx::command_buffer::GfxCommandBuffer;
use gfx::image::{GfxImage, ImageCreateInfos};
use gfx::image_sampler::{ImageSampler, SamplerCreateInfos};
use gfx::renderer::render_pass::{RenderPass, RenderPassInstance};
use gfx::shader::{ShaderProgram};
use logger::fatal;
use shader_base::pass_id::PassID;
use shader_base::{CompilationError, ShaderInterface};

use crate::renderer::render_pass_pool::RenderPassPool;
use crate::vk_buffer::VkBuffer;
use crate::vk_command_buffer::{VkCommandBuffer, VkCommandPool};
use crate::vk_descriptor_pool::VkDescriptorPool;
use crate::vk_device::VkDevice;
use crate::vk_image::VkImage;
use crate::vk_image_sampler::VkImageSampler;
use crate::vk_instance::{InstanceCreateInfos, VkInstance};
use crate::vk_physical_device::VkPhysicalDevice;
use crate::vk_shader::VkShaderProgram;
use crate::vk_shader_instance::VkShaderInstance;

pub mod vk_buffer;
pub mod vk_command_buffer;
pub mod vk_descriptor_pool;
pub mod vk_device;
pub mod vk_dst_set_layout;
pub mod vk_image;
pub mod vk_image_sampler;
pub mod vk_instance;
pub mod vk_physical_device;
pub mod vk_shader;
pub mod vk_shader_instance;
pub mod vk_types;
pub mod dynamic_resource_ctx;

pub mod renderer {
    pub mod vk_render_pass_instance;
    pub mod render_pass_pool;
    pub mod vk_render_pass;
}

#[macro_export]
macro_rules! to_c_char {
    ($str:expr) => {
        $str.as_ptr() as *const c_char
    };
}

#[macro_export]
macro_rules! vk_check {
    ($expression:expr) => {
        match $expression {
            Ok(object) => object,
            Err(vk_err) => {
                logger::fatal!(
                    "vk error : {}\non '{}'",
                    vk_err.to_string(),
                    stringify!(expression)
                )
            }
        }
    };
}

pub struct GfxVulkan {
    ash_entry: Option<ash::Entry>,
    initialized: AtomicBool,

    pub instance: MaybeUninit<VkInstance>,
    pub physical_device: PhysicalDevice,
    pub physical_device_vk: VkPhysicalDevice,
    pub device: MaybeUninit<VkDevice>,
    pub gfx_ref: Weak<GfxVulkan>,
    pub command_pool: MaybeUninit<VkCommandPool>,
    pub descriptor_pool: MaybeUninit<VkDescriptorPool>,
    pub render_pass_pool: RenderPassPool,
}

impl Default for GfxVulkan {
    fn default() -> Self {
        let ash_entry = unsafe { ash::Entry::load() }.expect("failed to load vulkan library");

        let gfx = Self {
            ash_entry: Some(ash_entry),
            initialized: AtomicBool::new(false),
            instance: MaybeUninit::uninit(),
            physical_device: Default::default(),
            physical_device_vk: Default::default(),
            device: MaybeUninit::uninit(),
            gfx_ref: Default::default(),
            command_pool: MaybeUninit::uninit(),
            descriptor_pool: MaybeUninit::uninit(),
            render_pass_pool: Default::default(),
        };
        logger::info!("Created vulkan gfx backend. Waiting for initialization...");
        gfx
    }
}

impl GfxInterface for GfxVulkan {
    fn init(&mut self) {
        Gfx::get();
        match VkInstance::new(
            self,
            InstanceCreateInfos {
                enable_validation_layers: true,
                required_extensions: vec![],
                ..InstanceCreateInfos::default()
            },
        ) {
            Ok(instance) => {
                self.instance.write(instance);
            }
            Err(error) => {
                fatal!("failed to create vulkan instance : {error}");
            }
        }

        self.initialized.store(true, Ordering::SeqCst);
    }

    fn is_ready(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    fn set_physical_device(&self, selected_device: PhysicalDevice) {
        unsafe {
            (&self.physical_device as *const PhysicalDevice as *mut PhysicalDevice)
                .write(selected_device.clone())
        };
        unsafe {
            (&self.physical_device_vk as *const VkPhysicalDevice as *mut VkPhysicalDevice).write(
                self.instance
                    .assume_init_ref()
                    .get_vk_device(&selected_device)
                    .expect("failed to get physical device information for vulkan")
                    .clone(),
            )
        };
        unsafe {
            (&self.device as *const MaybeUninit<VkDevice> as *mut MaybeUninit<VkDevice>)
                .write(MaybeUninit::new(VkDevice::default()))
        };
        unsafe {
            (&self.command_pool as *const MaybeUninit<VkCommandPool>
                as *mut MaybeUninit<VkCommandPool>)
                .write(MaybeUninit::new(VkCommandPool::new("global".to_string())))
        };
        unsafe {
            (&self.descriptor_pool as *const MaybeUninit<VkDescriptorPool>
                as *mut MaybeUninit<VkDescriptorPool>)
                .write(MaybeUninit::new(VkDescriptorPool::new(64, 64)))
        };

        unsafe {
            self.set_vk_object_name(
                self.device.assume_init_ref().handle.handle(),
                "device\t\t: global ",
            );
        }
        self.set_vk_object_name(
            self.physical_device_vk.handle,
            format!("physical device\t\t: {}", self.physical_device.device_name).as_str(),
        );
    }

    fn enumerate_physical_devices(&self) -> Vec<PhysicalDevice> {
        unsafe { self.instance.assume_init_ref().enumerate_physical_devices() }
    }

    fn find_best_suitable_physical_device(&self) -> Result<PhysicalDevice, String> {
        unsafe { self.instance.assume_init_ref().find_best_suitable_gpu_vk() }
    }

    fn create_buffer(&self, name: String, create_infos: &BufferCreateInfo) -> Arc<dyn GfxBuffer> {
        Arc::new(VkBuffer::new(name, create_infos))
    }

    fn create_shader_program(
        &self,
        name: String,
        pass_id: PassID,
        create_infos: &dyn ShaderInterface,
    ) -> Result<Arc<dyn ShaderProgram>, CompilationError> {
        match VkShaderProgram::new(name, pass_id, create_infos) {
            Ok(sp) => { Ok(sp) }
            Err(err) => { Err(err) }
        }
    }

    fn instantiate_render_pass(
        &self,
        render_pass: &RenderPass,
    ) -> Box<dyn RenderPassInstance> {
        Box::new(self.render_pass_pool.instantiate(render_pass))
    }

    fn create_image(&self, name: String, create_infos: ImageCreateInfos) -> Arc<dyn GfxImage> {
        VkImage::new_ptr(name, create_infos)
    }

    fn create_image_sampler(
        &self,
        name: String,
        create_infos: SamplerCreateInfos,
    ) -> Arc<dyn ImageSampler> {
        VkImageSampler::new(name, create_infos)
    }

    fn create_command_buffer(
        &self,
        name: String,
    ) -> Arc<dyn GfxCommandBuffer> {
        VkCommandBuffer::new(name)
    }
}

impl GfxVulkan {
    pub fn render_pass_pool(&self) -> &RenderPassPool {
        &self.render_pass_pool
    }

    pub fn device(&self) -> &VkDevice {
        unsafe { self.device.assume_init_ref() }
    }

    pub fn set_vk_object_name<T: vk::Handle + 'static + Copy>(&self, object: T, name: &str) -> T {
        let object_type = if TypeId::of::<vk::Instance>() == TypeId::of::<T>() {
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
            fatal!("unhandled object type id")
        };

        let string_name = format!("{}\0", name);

        unsafe {
            vk_check!(self
                .instance
                .assume_init_ref()
                .debug_util_loader
                .assume_init_ref()
                .set_debug_utils_object_name(
                    self.device.assume_init_ref().handle.handle(),
                    &vk::DebugUtilsObjectNameInfoEXT::builder()
                        .object_type(object_type)
                        .object_handle(object.as_raw())
                        .object_name(CStr::from_ptr(string_name.as_ptr() as *const c_char))
                        .build()
                ))
        }

        object
    }

    pub fn entry(&self) -> &ash::Entry {
        match &self.ash_entry {
            None => {
                fatal!("ash entry is not valid")
            }
            Some(entry) => entry,
        }
    }

    pub fn is_layer_available(&self, layer: &str) -> bool {
        if let Ok(layer_properties) = self.entry().enumerate_instance_layer_properties() {
            unsafe {
                for layer_details in layer_properties {
                    if CStr::from_ptr(layer_details.layer_name.as_ptr())
                        .to_str()
                        .expect("failed to read layer name")
                        == layer
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_extension_available(&self, layer: &str) -> bool {
        if let Ok(extensions_properties) =
            self.entry().enumerate_instance_extension_properties(None)
        {
            unsafe {
                for extension in extensions_properties {
                    if CStr::from_ptr(extension.extension_name.as_ptr())
                        .to_str()
                        .expect("failed to read extension name")
                        == layer
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn get() -> &'static Self {
        Gfx::get().cast::<GfxVulkan>()
    }
}

impl Drop for GfxVulkan {
    fn drop(&mut self) {
        logger::info!("Destroyed vulkan gfx backend");
    }
}

pub trait GfxVkObject {
    fn construct(&mut self);
    fn is_valid(&self) -> bool;
}
