use std::sync::{Arc, RwLock, Weak};

use ash::vk;
use gfx::{GfxRef};
use gfx::render_pass::{RenderPass, RenderPassCreateInfos, RenderPassInstance};
use gfx::shader::PassID;
use gfx::surface::GfxSurface;
use gfx::types::{ClearValues, PixelFormat};
use maths::vec2::Vec2u32;

use crate::{GfxVulkan, vk_check};
use crate::vk_render_pass_instance::VkRenderPassInstance;
use crate::vk_types::VkPixelFormat;

pub struct VkRenderPass {
    pub render_pass: vk::RenderPass,
    gfx: GfxRef,
    self_ref: RwLock<Weak<VkRenderPass>>,
    default_clear_values: Vec<ClearValues>,
    config: RenderPassCreateInfos,
    pass_id: PassID,
    name: String
}

impl RenderPass for VkRenderPass {
    fn instantiate(&self, surface: &Arc<dyn GfxSurface>, res: Vec2u32) -> Arc<dyn RenderPassInstance> {
        Arc::new(VkRenderPassInstance::new(&self.gfx, format!("{}_instance", self.name), surface, self.self_ref.read().unwrap().upgrade().unwrap(), res))
    }

    fn get_clear_values(&self) -> &Vec<ClearValues> {
        &self.default_clear_values
    }

    fn get_config(&self) -> &RenderPassCreateInfos {
        &self.config
    }

    fn get_pass_id(&self) -> PassID {
        self.pass_id.clone()
    }
}

impl VkRenderPass {
    pub fn new(gfx: &GfxRef, name: String, create_infos: RenderPassCreateInfos) -> Arc<Self> {
        let mut attachment_descriptions = Vec::<vk::AttachmentDescription>::new();
        let mut color_attachment_references = Vec::<vk::AttachmentReference>::new();
        let mut _depth_attachment_reference = vk::AttachmentReference::default();
        let mut clear_values = Vec::new();

        // add color color_attachments
        for attachment in &create_infos.color_attachments
        {
            match attachment.image_format {
                PixelFormat::UNDEFINED => { panic!("wrong pixel format") }
                _ => {}
            };

            let attachment_index: u32 = attachment_descriptions.len() as u32;

            attachment_descriptions.push(vk::AttachmentDescription::builder()
                .format(*VkPixelFormat::from(&attachment.image_format))
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(match attachment.clear_value {
                    ClearValues::DontClear => { vk::AttachmentLoadOp::DONT_CARE }
                    _ => { vk::AttachmentLoadOp::CLEAR }
                })
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(if create_infos.is_present_pass { vk::ImageLayout::PRESENT_SRC_KHR } else { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL })
                .build());

            color_attachment_references.push(vk::AttachmentReference {
                attachment: attachment_index,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            });

            clear_values.push(attachment.clear_value);
        }

        let mut subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachment_references.as_slice())
            .build();

        // add depth attachment
        match &create_infos.depth_attachment {
            None => {}
            Some(attachment) => {
                match attachment.image_format {
                    PixelFormat::UNDEFINED => { panic!("wrong depth pixel format") }
                    _ => {}
                };

                let attachment_index: u32 = attachment_descriptions.len() as u32;

                attachment_descriptions.push(vk::AttachmentDescription::builder()
                    .format(*VkPixelFormat::from(&attachment.image_format))
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(match attachment.clear_value {
                        ClearValues::DontClear => { vk::AttachmentLoadOp::DONT_CARE }
                        _ => { vk::AttachmentLoadOp::CLEAR }
                    })
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .build());

                _depth_attachment_reference = vk::AttachmentReference::builder()
                    .attachment(attachment_index)
                    .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .build();
                subpass.p_depth_stencil_attachment = &_depth_attachment_reference;
                clear_values.push(attachment.clear_value);
            }
        };
        
        let dependencies = vec![
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)                                                             // Producer of the dependency
                .dst_subpass(0)                                                                            // Consumer is our single subpass that will wait for the execution dependency
                .src_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)                                        // Match our pWaitDstStageMask when we vkQueueSubmit
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)                               // is a loadOp stage for color color_attachments
                .src_access_mask(vk::AccessFlags::MEMORY_READ)                                                 // semaphore wait already does memory dependency for us
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE) // is a loadOp CLEAR access mask for color color_attachments
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
            vk::SubpassDependency::builder()
                .src_subpass(0)                                                                            // Producer of the dependency is our single subpass
                .dst_subpass(vk::SUBPASS_EXTERNAL)                                                             // Consumer are all commands outside of the render pass
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)                               // is a storeOp stage for color color_attachments
                .dst_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)                                        // Do not block any subsequent work
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE) // is a storeOp `STORE` access mask for color color_attachments
                .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
        ];

        let render_pass_infos = vk::RenderPassCreateInfo::builder()
            .attachments(attachment_descriptions.as_slice())
            .subpasses(&[subpass])
            .dependencies(dependencies.as_slice())
            .build();

        let gfx_copy = gfx.clone();
        let render_pass = vk_check!(unsafe { gfx_copy.cast::<GfxVulkan>().device.handle.create_render_pass(&render_pass_infos, None) });

        gfx.cast::<GfxVulkan>().set_vk_object_name(render_pass, format!("render pass\t\t: {}", name).as_str());
            
        let vk_render_pass = Arc::new(Self {
            render_pass,
            gfx: gfx.clone(),
            self_ref: RwLock::new(Weak::new()),
            default_clear_values: clear_values,
            pass_id: create_infos.pass_id.clone(),
            config: create_infos.clone(),
            name
        });

        {
            let mut self_ref = vk_render_pass.self_ref.write().unwrap();
            *self_ref = Arc::downgrade(&vk_render_pass);
        }

        gfx.cast::<GfxVulkan>().render_passes.write().unwrap().insert(create_infos.pass_id, vk_render_pass.clone());

        vk_render_pass
    }
}