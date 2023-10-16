use std::collections::HashMap;
use std::slice;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use ash::vk;

use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::material::MaterialResources;
use gfx::shader_instance::{ShaderInstance, ShaderInstanceCreateInfos};
use gfx::surface::Frame;

use crate::{GfxVulkan, VkImage, VkImageSampler};
use crate::vk_dst_set_layout::VkDescriptorSetLayout;

pub struct VkShaderInstance {
    _write_descriptor_sets: RwLock<GfxResource<Vec<vk::WriteDescriptorSet>>>,
    pub descriptor_sets: RwLock<GfxResource<vk::DescriptorSet>>,
    pub pipeline_layout: Arc<vk::PipelineLayout>,
    descriptors_dirty: GfxResource<Arc<AtomicBool>>,
    resource_bindings: Arc<MaterialResources>
}

impl ShaderInstance for VkShaderInstance {}

struct RbDescriptorState {}

impl GfxImageBuilder<Arc<AtomicBool>> for RbDescriptorState {
    fn build(&self, _: &Frame) -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(true))
    }
}

struct RbDescriptorSet {
    layout: vk::DescriptorSetLayout,
    name: String,
}

impl GfxImageBuilder<vk::DescriptorSet> for RbDescriptorSet {
    fn build(&self, _swapchain_ref: &Frame) -> vk::DescriptorSet {
        unsafe {
            GfxVulkan::get()
                .descriptor_pool
                .assume_init_ref()
                .allocate(self.name.clone(), self.layout)
        }
    }
}

impl VkShaderInstance {
    pub fn new(
        name: String,
        create_infos: ShaderInstanceCreateInfos,
        pipeline_layout: Arc<vk::PipelineLayout>,
        desc_set_layout: Arc<VkDescriptorSetLayout>,
    ) -> Arc<Self> {
        Arc::new(VkShaderInstance {
            _write_descriptor_sets: RwLock::default(),
            pipeline_layout,
            descriptor_sets: RwLock::new(GfxResource::new(RbDescriptorSet {
                layout: desc_set_layout.descriptor_set_layout,
                name,
            })),
            descriptors_dirty: GfxResource::new(RbDescriptorState {}),
            resource_bindings: Arc::new(todo!("Add resource bindings")),
        })
    }

    fn mark_descriptors_dirty(&self) {
        self.descriptors_dirty.invalidate(RbDescriptorState {});
    }

    pub fn refresh_descriptors(&self, image_id: &Frame) {
        if self
            .descriptors_dirty
            .get(image_id)
            .compare_exchange(true, false, Ordering::Acquire, Ordering::Acquire)
            .is_ok()
        {
            let mut desc_images = Vec::new();

            let mut write_desc_set = Vec::new();

            for (bp, binding) in &self.base_bindings {
                write_desc_set.push(
                    match self.bindings.read().unwrap().get(bp) {
                        None => {
                            logger::error!("binding {:?} is not specified", bp);
                            return;
                        }
                        Some(bindings) => match bindings {
                            MaterialResource::Sampler(sampler) => {
                                desc_images.push(sampler.cast::<VkImageSampler>().sampler_info);
                                vk::WriteDescriptorSet::builder()
                                    .descriptor_type(vk::DescriptorType::SAMPLER)
                                    .image_info(slice::from_ref(
                                        &desc_images[desc_images.len() - 1],
                                    ))
                            }
                            MaterialResource::SampledImage(sampled_image) => {
                                let vk_image = sampled_image.cast::<VkImage>();
                                desc_images.push(if vk_image.image_params.read_only {
                                    vk_image.view.get_static().1
                                } else {
                                    vk_image.view.get(image_id).1
                                });
                                vk::WriteDescriptorSet::builder()
                                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                                    .image_info(slice::from_ref(
                                        &desc_images[desc_images.len() - 1],
                                    ))
                            }
                        },
                    }
                        .dst_set(self.descriptor_sets.read().unwrap().get(image_id))
                        .dst_binding(binding.binding)
                        .build(),
                );
            }

            unsafe {
                GfxVulkan::get()
                    .device
                    .assume_init_ref()
                    .handle
                    .update_descriptor_sets(write_desc_set.as_slice(), &[]);
            }
        }
    }
}
