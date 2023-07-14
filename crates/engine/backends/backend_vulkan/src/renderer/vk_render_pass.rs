use ash::vk;
use gfx::renderer::render_pass::RenderPass;
use gfx::types::{BackgroundColor, PixelFormat};
use crate::vk_types::VkPixelFormat;
use crate::{vk_check, GfxVulkan};

pub struct VkRenderPass {
    pub render_pass: vk::RenderPass,
}

impl VkRenderPass {
    pub fn new(render_pass: &RenderPass) -> Self {
        let mut attachment_descriptions = Vec::<vk::AttachmentDescription>::new();
        let mut color_attachment_references = Vec::<vk::AttachmentReference>::new();
        let mut depth_attachment_reference = vk::AttachmentReference::default();
        let mut clear_values = Vec::new();


        for image in render_pass.images() {
            if let PixelFormat::UNDEFINED = image.get_format() {
                logger::fatal!("wrong pixel format")
            };

            let attachment_index: u32 = attachment_descriptions.len() as u32;

            attachment_descriptions.push(
                vk::AttachmentDescription::builder()
                    .format(*VkPixelFormat::from(&image.get_format()))
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(match image.background_color() {
                        BackgroundColor::None => vk::AttachmentLoadOp::DONT_CARE,
                        _ => vk::AttachmentLoadOp::CLEAR,
                    })
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(if render_pass.source().is_present_pass() { vk::ImageLayout::PRESENT_SRC_KHR } else { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL })
                    .build(),
            );

            if image.get_format().is_depth_format() {
                depth_attachment_reference = vk::AttachmentReference::builder()
                    .attachment(attachment_index)
                    .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .build();
            } else {
                color_attachment_references.push(vk::AttachmentReference {
                    attachment: attachment_index,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                });
            }

            clear_values.push(image.background_color());
        }

        let mut sub_pass = ash::vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachment_references.as_slice())
            .build();
        
        if depth_attachment_reference.layout != vk::ImageLayout::UNDEFINED {
            sub_pass.p_depth_stencil_attachment = &depth_attachment_reference
        }

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

        let vk_render_pass = vk_check!(unsafe {
            GfxVulkan::get()
                .device()
                .handle
                .create_render_pass(&render_pass_infos, None)
        });

        GfxVulkan::get()
            .set_vk_object_name(vk_render_pass, render_pass.source().get_name());

        Self {
            render_pass: vk_render_pass,
        }
    }
}
