use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use ash::vk;
use gfx::render_node;
use gfx::types::{ClearValues, PixelFormat};
use maths::vec2::Vec2u32;

use crate::vk_render_pass_instance::VkRenderPassInstance;
use crate::vk_types::VkPixelFormat;
use crate::{vk_check, GfxVulkan};

#[derive(Default)]
pub struct RenderPassPool {
    render_passes: RwLock<HashMap<Arc<render_node::RenderNode>, Box<VkRenderPass>>>,
}

impl RenderPassPool {
    pub fn instantiate(&self, render_pass: &render_node::RenderPass, initial_res: Vec2u32) -> Box<dyn render_node::RenderPassInstance> {
        Box::new(self.find_or_create(render_pass).create_instance(render_pass, initial_res))
    }

    fn find_or_create(&self, render_pass: &render_node::RenderPass) -> &VkRenderPass {
        if !self.render_passes.read().unwrap().contains_key(render_pass.source()) {
            self.render_passes.write().unwrap().insert(render_pass.source().clone(), Box::new(VkRenderPass::new(render_pass.source())));
        }
        &self.render_passes.read().unwrap().get(render_pass.source()).unwrap()
    }
}

pub struct VkRenderPass {
    pub render_pass: vk::RenderPass,
    default_clear_values: Vec<ClearValues>,
}

impl VkRenderPass {
    pub fn new(render_pass: &Arc<render_node::RenderNode>) -> Self {
        let mut attachment_descriptions = Vec::<vk::AttachmentDescription>::new();
        let mut color_attachment_references = Vec::<vk::AttachmentReference>::new();
        let mut _depth_attachment_reference = vk::AttachmentReference::default();
        let mut clear_values = Vec::new();

        // add color color_attachments
        for resource in &render_pass.color_resources() {
            if let PixelFormat::UNDEFINED = resource.image_format {
                logger::fatal!("wrong pixel format")
            };

            let attachment_index: u32 = attachment_descriptions.len() as u32;

            attachment_descriptions.push(
                vk::AttachmentDescription::builder()
                    .format(*VkPixelFormat::from(&resource.image_format))
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(match resource.clear_value {
                        ClearValues::DontClear => vk::AttachmentLoadOp::DONT_CARE,
                        _ => vk::AttachmentLoadOp::CLEAR,
                    })
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(if render_pass.is_present_pass() {
                        vk::ImageLayout::PRESENT_SRC_KHR
                    } else {
                        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
                    })
                    .build(),
            );

            color_attachment_references.push(vk::AttachmentReference {
                attachment: attachment_index,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            });

            clear_values.push(resource.clear_value);
        }

        let mut sub_pass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachment_references.as_slice())
            .build();

        // add depth attachment
        for resource in &render_pass.depth_resource() {
            if let PixelFormat::UNDEFINED = resource.image_format {
                logger::fatal!("wrong depth pixel format")
            };

            let attachment_index: u32 = attachment_descriptions.len() as u32;

            attachment_descriptions.push(
                vk::AttachmentDescription::builder()
                    .format(*VkPixelFormat::from(&resource.image_format))
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(match resource.clear_value {
                        ClearValues::DontClear => vk::AttachmentLoadOp::DONT_CARE,
                        _ => vk::AttachmentLoadOp::CLEAR,
                    })
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .build(),
            );

            _depth_attachment_reference = vk::AttachmentReference::builder()
                .attachment(attachment_index)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build();
            sub_pass.p_depth_stencil_attachment = &_depth_attachment_reference;
            clear_values.push(resource.clear_value);
        };

        let dependencies = vec![
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL) // Producer of the dependency
                .dst_subpass(0) // Consumer is our single sub_pass that will wait for the execution dependency
                .src_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE) // Match our pWaitDstStageMask when we vkQueueSubmit
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT) // is a loadOp stage for color color_attachments
                .src_access_mask(vk::AccessFlags::MEMORY_READ) // semaphore wait already does memory dependency for us
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                ) // is a loadOp CLEAR access mask for color color_attachments
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
            vk::SubpassDependency::builder()
                .src_subpass(0) // Producer of the dependency is our single sub_pass
                .dst_subpass(vk::SUBPASS_EXTERNAL) // Consumer are all commands outside of the render pass
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT) // is a storeOp stage for color color_attachments
                .dst_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE) // Do not block any subsequent work
                .src_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                ) // is a storeOp `STORE` access mask for color color_attachments
                .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
        ];

        let render_pass_infos = vk::RenderPassCreateInfo::builder()
            .attachments(attachment_descriptions.as_slice())
            .subpasses(&[sub_pass])
            .dependencies(dependencies.as_slice())
            .build();

        let render_pass = vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .create_render_pass(&render_pass_infos, None)
        });

        GfxVulkan::get()
            .set_vk_object_name(render_pass, format!("render pass : undefined render pass name").as_str());

        let vk_render_pass = Self {
            render_pass,
            default_clear_values: clear_values,
        };

        vk_render_pass
    }

    pub fn create_instance(&self, render_pass: &render_node::RenderPass, initial_res: Vec2u32) -> VkRenderPassInstance {
        VkRenderPassInstance::new(self, render_pass, initial_res)        
    }
}
