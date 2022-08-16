use std::ptr::null;
use std::sync::Arc;

use ash::vk::{AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, DependencyFlags, ImageLayout, PipelineBindPoint, PipelineStageFlags, RenderPassCreateInfo, SampleCountFlags, SUBPASS_EXTERNAL, SubpassDependency, SubpassDescription};

use gfx::GfxRef;
use gfx::render_pass::{RenderPass, RenderPassCreateInfos};
use gfx::types::{ClearValues, PixelFormat};

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check};
use crate::vk_types::VkPixelFormat;

pub struct VkRenderPass {
    pub render_pass: ash::vk::RenderPass,
}

impl RenderPass for VkRenderPass {
    
}

impl VkRenderPass {
    pub fn new(gfx: GfxRef, create_infos: RenderPassCreateInfos) -> Box<Self> {
        let mut attachment_descriptions = Vec::<AttachmentDescription>::new();
        let mut color_attachment_references = Vec::<AttachmentReference>::new();
        let mut depth_attachment_reference = None;
        let mut color_attachment_resolve_reference = None;

        // add color color_attachments
        for attachment in create_infos.color_attachments
        {
            match attachment.image_format {
                PixelFormat::UNDEFINED => { panic!("wrong pixel format") }
                _ => {}
            };

            let attachment_index: u32 = attachment_descriptions.len() as u32;

            attachment_descriptions.push(AttachmentDescription {
                format: *VkPixelFormat::from(&attachment.image_format),
                samples: SampleCountFlags::TYPE_1,
                load_op: match attachment.clear_value {
                    ClearValues::DontClear => { AttachmentLoadOp::DONT_CARE }
                    _ => { AttachmentLoadOp::CLEAR }
                },
                store_op: AttachmentStoreOp::STORE,
                stencil_load_op: AttachmentLoadOp::DONT_CARE,
                stencil_store_op: AttachmentStoreOp::DONT_CARE,
                initial_layout: ImageLayout::UNDEFINED,
                final_layout: if create_infos.is_present_pass { ImageLayout::PRESENT_SRC_KHR } else { ImageLayout::SHADER_READ_ONLY_OPTIMAL },
                ..AttachmentDescription::default()
            });

            color_attachment_references.push(AttachmentReference {
                attachment: attachment_index,
                layout: ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            });
        }

        // add depth attachment
        match create_infos.depth_attachment {
            None => {}
            Some(attachment) => {
                match attachment.image_format {
                    PixelFormat::UNDEFINED => { panic!("wrong depth pixel format") }
                    _ => {}
                };


                let attachment_index: u32 = attachment_descriptions.len() as u32;

                attachment_descriptions.push(AttachmentDescription {
                    format: *VkPixelFormat::from(&attachment.image_format),
                    samples: SampleCountFlags::TYPE_1,
                    load_op: match attachment.clear_value {
                        ClearValues::DontClear => { AttachmentLoadOp::DONT_CARE }
                        _ => { AttachmentLoadOp::CLEAR }
                    },
                    store_op: AttachmentStoreOp::STORE,
                    stencil_load_op: AttachmentLoadOp::DONT_CARE,
                    stencil_store_op: AttachmentStoreOp::DONT_CARE,
                    initial_layout: ImageLayout::UNDEFINED,
                    final_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    ..AttachmentDescription::default()
                });

                depth_attachment_reference = Some(AttachmentReference {
                    attachment: attachment_index,
                    layout: ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                });
            }
        }

        let subpass = SubpassDescription {
            pipeline_bind_point: PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,       // Input color_attachments can be used to sample from contents of a previous subpass
            p_input_attachments: null(), // (Input color_attachments not used here)
            color_attachment_count: color_attachment_references.len() as u32,
            p_color_attachments: color_attachment_references.as_ptr(),
            p_resolve_attachments: match color_attachment_resolve_reference {
                Some(attachment) => { &attachment }
                None => { null() }
            }, // resolve mean the target attachment for msaa
            p_depth_stencil_attachment: match depth_attachment_reference {
                Some(attachment) => { &attachment }
                None => { null() }
            }, // resolve mean the target attachment for msaa
            preserve_attachment_count: 0,       // Preserved color_attachments can be used to loop (and preserve) color_attachments through subpasses
            p_preserve_attachments: null(), // (Preserve color_attachments not used by this example)
            ..SubpassDescription::default()
        };

        let dependencies = vec![
            SubpassDependency {
                src_subpass: SUBPASS_EXTERNAL,                                                        // Producer of the dependency
                dst_subpass: 0,                                                                          // Consumer is our single subpass that will wait for the execution depdendency
                src_stage_mask: PipelineStageFlags::BOTTOM_OF_PIPE,                                       // Match our pWaitDstStageMask when we vkQueueSubmit
                dst_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,                              // is a loadOp stage for color color_attachments
                src_access_mask: AccessFlags::MEMORY_READ,                                  // semaphore wait already does memory dependency for us
                dst_access_mask: AccessFlags::COLOR_ATTACHMENT_READ | AccessFlags::COLOR_ATTACHMENT_WRITE, // is a loadOp CLEAR access mask for color color_attachments
                dependency_flags: DependencyFlags::BY_REGION,
                ..SubpassDependency::default()
            },
            SubpassDependency {
                src_subpass: 0,                                                                          // Producer of the dependency is our single subpass
                dst_subpass: SUBPASS_EXTERNAL,                                                        // Consumer are all commands outside of the renderpass
                src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,                             // is a storeOp stage for color color_attachments
                dst_stage_mask: PipelineStageFlags::BOTTOM_OF_PIPE,                                       // Do not block any subsequent work
                src_access_mask: AccessFlags::COLOR_ATTACHMENT_READ | AccessFlags::COLOR_ATTACHMENT_WRITE, // is a storeOp `STORE` access mask for color color_attachments
                dst_access_mask: AccessFlags::MEMORY_READ,
                dependency_flags: DependencyFlags::BY_REGION,
                ..SubpassDependency::default()
            },
        ];

        let render_pass_infos = RenderPassCreateInfo {
            attachment_count: attachment_descriptions.len() as u32,
            p_attachments: attachment_descriptions.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: dependencies.len() as u32,
            p_dependencies: dependencies.as_ptr(),
            ..RenderPassCreateInfo::default()
        };

        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        let render_pass = vk_check!(unsafe { gfx_object!(*device).device.create_render_pass(&render_pass_infos, None) });

        Box::new(Self {
            render_pass
        })
    }
}