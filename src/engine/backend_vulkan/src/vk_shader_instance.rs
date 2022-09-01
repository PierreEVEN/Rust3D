use std::collections::HashMap;
use std::ptr::null;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use ash::vk;
use ash::vk::{DescriptorImageInfo, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, PipelineLayout, WriteDescriptorSet};

use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::GfxRef;
use gfx::image::GfxImage;
use gfx::image_sampler::ImageSampler;
use gfx::shader::{DescriptorBinding, DescriptorType};
use gfx::shader_instance::{BindPoint, ShaderInstance, ShaderInstanceCreateInfos};
use gfx::surface::GfxImageID;

use crate::{gfx_cast_vulkan,  GfxVulkan, vk_check, VkImage, VkImageSampler};

pub struct VkShaderInstance {
    _gfx: GfxRef,
    _write_descriptor_sets: RwLock<GfxResource<Vec<WriteDescriptorSet>>>,
    pub descriptor_sets: RwLock<GfxResource<DescriptorSet>>,
    pub pipeline_layout: Arc<PipelineLayout>,
    descriptors_dirty: AtomicBool,
    binding: Vec<DescriptorBinding>,
    
    textures: RwLock<HashMap<BindPoint, Arc<dyn GfxImage>>>,
    samplers: RwLock<HashMap<BindPoint, Arc<dyn ImageSampler>>>,
    
}

impl ShaderInstance for VkShaderInstance {
    fn bind_texture(&self, _bind_point: &BindPoint, _texture: &Arc<dyn GfxImage>) {
        let mut textures = self.textures.write().unwrap();
        textures.insert(_bind_point.clone(), _texture.clone());
        self.mark_descriptors_dirty();
    }

    fn bind_sampler(&self, _bind_point: &BindPoint, sampler: &Arc<dyn ImageSampler>) {
        let mut samplers = self.samplers.write().unwrap();
        samplers.insert(_bind_point.clone(), sampler.clone());
        self.mark_descriptors_dirty();
    }
}

struct RbDescriptorSet {
    layout: DescriptorSetLayout
}

impl GfxImageBuilder<DescriptorSet> for RbDescriptorSet {
    fn build(&self, _gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> DescriptorSet {
        let device = &gfx_cast_vulkan!(_gfx).device;
        unsafe {
            vk_check!((*device).device.allocate_descriptor_sets(&DescriptorSetAllocateInfo {
                descriptor_pool: Default::default(),
                descriptor_set_count: 1,
                p_set_layouts: &self.layout,
                ..DescriptorSetAllocateInfo::default()
            }))[0]
        }
    }
}

impl VkShaderInstance {
    pub fn new(gfx: &GfxRef, create_infos: ShaderInstanceCreateInfos, pipeline_layout: Arc<PipelineLayout>) -> Arc<Self> {
        Arc::new(VkShaderInstance {
            _gfx: gfx.clone(),
            _write_descriptor_sets: RwLock::default(),
            pipeline_layout,
            descriptor_sets: RwLock::new(GfxResource::new(Box::new(RbDescriptorSet { layout: DescriptorSetLayout::null() }))),
            descriptors_dirty: AtomicBool::new(true),
            binding: create_infos.bindings,
            textures: RwLock::default(),
            samplers: RwLock::default()
        })
    }

    fn mark_descriptors_dirty(&self) {
        self.descriptors_dirty.store(true, Ordering::Release);
    }

    pub fn refresh_descriptors(&self, image_id: &GfxImageID) {
        if self.descriptors_dirty.compare_exchange(true, false, Ordering::Acquire, Ordering::Acquire).is_ok() {
            let device = &gfx_cast_vulkan!(self._gfx).device;
            
            let mut write_desc_set = Vec::new();
            
            for binding in &self.binding {
                let (descriptor_type, image, buffer, texel_buffer) = match binding.descriptor_type {
                    DescriptorType::Sampler => { (vk::DescriptorType::SAMPLER, null(), null(), null()) }
                    DescriptorType::CombinedImageSampler => {
                        let sampler = match &self.samplers.read().unwrap().get(&binding.bind_point) {
                            None => { panic!("image sampler {} is not specified", binding.bind_point.name) }
                            Some(sampler) => { sampler.as_ref().as_any().downcast_ref::<VkImageSampler>().unwrap().image_infos }
                        };
                        (vk::DescriptorType::COMBINED_IMAGE_SAMPLER, &sampler as *const DescriptorImageInfo, null(), null())
                    }
                    DescriptorType::SampledImage => {
                        let image = match &self.textures.read().unwrap().get(&binding.bind_point) {
                            None => { panic!("image sampler {} is not specified", binding.bind_point.name) }
                            Some(image) => {
                                let (_, desc_info) = image.as_ref().as_any().downcast_ref::<VkImage>().unwrap().view.get(image_id);
                                desc_info
                            }
                        };
                        (vk::DescriptorType::SAMPLED_IMAGE, &image as *const DescriptorImageInfo, null(), null())
                    }
                    DescriptorType::StorageImage => { (vk::DescriptorType::STORAGE_IMAGE, null(), null(), null()) }
                    DescriptorType::UniformTexelBuffer => { (vk::DescriptorType::UNIFORM_TEXEL_BUFFER, null(), null(), null()) }
                    DescriptorType::StorageTexelBuffer => { (vk::DescriptorType::STORAGE_TEXEL_BUFFER, null(), null(), null()) }
                    DescriptorType::UniformBuffer => { (vk::DescriptorType::UNIFORM_BUFFER, null(), null(), null()) }
                    DescriptorType::StorageBuffer => { (vk::DescriptorType::STORAGE_BUFFER, null(), null(), null()) }
                    DescriptorType::UniformBufferDynamic => { (vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, null(), null(), null()) }
                    DescriptorType::StorageBufferDynamic => { (vk::DescriptorType::STORAGE_BUFFER_DYNAMIC, null(), null(), null()) }
                    DescriptorType::InputAttachment => { (vk::DescriptorType::INPUT_ATTACHMENT, null(), null(), null()) }
                };


                write_desc_set.push(WriteDescriptorSet {
                    dst_set: Default::default(),
                    dst_binding: binding.binding,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type,
                    p_image_info: image,
                    p_buffer_info: buffer,
                    p_texel_buffer_view: texel_buffer,
                    ..WriteDescriptorSet::default()
                })
            };

            unsafe { (*device).device.update_descriptor_sets(write_desc_set.as_slice(), &[]); }
        }
    }
}