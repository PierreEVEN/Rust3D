use std::collections::HashMap;
use std::ptr::null;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use ash::vk;
use ash::vk::{DescriptorSet, DescriptorSetLayout, PipelineLayout, WriteDescriptorSet};

use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::GfxRef;
use gfx::image::GfxImage;
use gfx::image_sampler::ImageSampler;
use gfx::shader::{DescriptorBinding, DescriptorType};
use gfx::shader_instance::{BindPoint, ShaderInstance, ShaderInstanceCreateInfos};
use gfx::surface::GfxImageID;

use crate::{GfxVulkan, VkImage, VkImageSampler};
use crate::vk_descriptor_set::VkDescriptorSetLayout;

pub struct VkShaderInstance {
    _gfx: GfxRef,
    _write_descriptor_sets: RwLock<GfxResource<Vec<WriteDescriptorSet>>>,
    pub descriptor_sets: RwLock<GfxResource<DescriptorSet>>,
    pub pipeline_layout: Arc<PipelineLayout>,
    descriptors_dirty: GfxResource<Arc<AtomicBool>>,
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

struct RbDescriptorState {}

impl GfxImageBuilder<Arc<AtomicBool>> for RbDescriptorState {
    fn build(&self, _: &GfxRef, _: &GfxImageID) -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(true))
    }
}

struct RbDescriptorSet {
    layout: DescriptorSetLayout,
}

impl GfxImageBuilder<DescriptorSet> for RbDescriptorSet {
    fn build(&self, gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> DescriptorSet {
        gfx.cast::<GfxVulkan>().descriptor_pool.allocate(&self.layout)
    }
}

impl VkShaderInstance {
    pub fn new(gfx: &GfxRef, create_infos: ShaderInstanceCreateInfos, pipeline_layout: Arc<PipelineLayout>, desc_set_layout: Arc<VkDescriptorSetLayout>) -> Arc<Self> {
        Arc::new(VkShaderInstance {
            _gfx: gfx.clone(),
            _write_descriptor_sets: RwLock::default(),
            pipeline_layout,
            descriptor_sets: RwLock::new(GfxResource::new(gfx, RbDescriptorSet { layout: desc_set_layout.descriptor_set_layout })),
            descriptors_dirty: GfxResource::new(gfx, RbDescriptorState {}),
            binding: create_infos.bindings,
            textures: RwLock::default(),
            samplers: RwLock::default(),
        })
    }

    fn mark_descriptors_dirty(&self) {
        self.descriptors_dirty.invalidate(&self._gfx, RbDescriptorState {});
    }

    pub fn refresh_descriptors(&self, image_id: &GfxImageID) {
        if self.descriptors_dirty.get(image_id).compare_exchange(true, false, Ordering::Acquire, Ordering::Acquire).is_ok() {
            let mut desc_images = Vec::new();

            let mut write_desc_set = Vec::new();

            for binding in &self.binding {
                let (descriptor_type, image_info, buffer_info, texel_buffer_info) = match binding.descriptor_type {
                    DescriptorType::Sampler => {
                        let sampler = match self.samplers.read().unwrap().get(&binding.bind_point) {
                            None => { panic!("sampler {} is not specified", binding.bind_point.name) }
                            Some(sampler) => { sampler.cast::<VkImageSampler>().sampler_info }
                        };
                        desc_images.push(sampler);
                        (vk::DescriptorType::SAMPLER, desc_images.last(), None, None)
                    }
                    DescriptorType::CombinedImageSampler => {
                        todo!()
                    }
                    DescriptorType::SampledImage => {
                        let image = match &self.textures.read().unwrap().get(&binding.bind_point) {
                            None => { panic!("image {} is not specified", binding.bind_point.name) }
                            Some(image) => {
                                let image = image.cast::<VkImage>();
                                if image.image_params.read_only { image.view.get_static().1 } else { image.view.get(image_id).1 }
                            }
                        };
                        
                        desc_images.push(image);
                        (vk::DescriptorType::SAMPLED_IMAGE, desc_images.last(), None, None)
                    }
                    _ => { todo!() }
                    /*
                    DescriptorType::StorageImage => { (vk::DescriptorType::STORAGE_IMAGE, null(), null(), null()); todo!()  }
                    DescriptorType::UniformTexelBuffer => { (vk::DescriptorType::UNIFORM_TEXEL_BUFFER, null(), null(), null()); todo!() }
                    DescriptorType::StorageTexelBuffer => { (vk::DescriptorType::STORAGE_TEXEL_BUFFER, null(), null(), null()); todo!() }
                    DescriptorType::UniformBuffer => { (vk::DescriptorType::UNIFORM_BUFFER, null(), null(), null()); todo!() }
                    DescriptorType::StorageBuffer => { (vk::DescriptorType::STORAGE_BUFFER, null(), null(), null()); todo!() }
                    DescriptorType::UniformBufferDynamic => { (vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, null(), null(), null()); todo!() }
                    DescriptorType::StorageBufferDynamic => { (vk::DescriptorType::STORAGE_BUFFER_DYNAMIC, null(), null(), null()); todo!() }
                    DescriptorType::InputAttachment => { (vk::DescriptorType::INPUT_ATTACHMENT, null(), null(), null()); todo!() }
                     */
                };

                write_desc_set.push(WriteDescriptorSet {
                    dst_set: self.descriptor_sets.read().unwrap().get(image_id),
                    dst_binding: binding.binding,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type,
                    p_image_info: match image_info {
                        None => { null() }
                        Some(infos) => { infos }
                    },
                    p_buffer_info: match buffer_info {
                        None => { null() }
                        Some(infos) => { infos }
                    },
                    p_texel_buffer_view: match texel_buffer_info {
                        None => { null() }
                        Some(infos) => { infos }
                    },
                    ..WriteDescriptorSet::default()
                });
            };

            unsafe { self._gfx.cast::<GfxVulkan>().device.handle.update_descriptor_sets(write_desc_set.as_slice(), &[]); }
        }
    }
}