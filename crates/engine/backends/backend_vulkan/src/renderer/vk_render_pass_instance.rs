use std::sync::{Arc, RwLock};

use ash::vk;

use gfx::command_buffer::GfxCommandBuffer;
use gfx::Gfx;
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::image::{GfxImage, ImageParams, ImageType, ImageUsage};
use gfx::renderer::render_pass::{RenderPass, RenderPassInstance};
use gfx::shader::PassID;
use gfx::surface::Frame;
use gfx::types::ClearValues;
use maths::vec2::Vec2u32;

use crate::{GfxVulkan, vk_check};
use crate::renderer::vk_render_pass::VkRenderPass;
use crate::vk_command_buffer::VkCommandBuffer;
use crate::vk_image::VkImage;

pub struct VkRenderPassInstance {
    pub render_finished_semaphore: GfxResource<vk::Semaphore>,
    pub wait_semaphores: RwLock<Option<vk::Semaphore>>,
    pub render_pass: Arc<VkRenderPass>,
    pub framebuffer: GfxResource<vk::Framebuffer>,
}

impl VkRenderPassInstance {
    pub fn new(vk_render_pass: Arc<VkRenderPass>, render_pass: &RenderPass, initial_res: Vec2u32) -> Self {
        Self {
            render_finished_semaphore: GfxResource::new(RbSemaphore { name: "todo_name".to_string() }),
            wait_semaphores: RwLock::new(None),
            render_pass: vk_render_pass.clone(),
            framebuffer: GfxResource::new(RbFramebuffer {
                render_pass: vk_render_pass.render_pass,
                res: initial_res,
                images: render_pass.images().clone(),
                name: "unnamed".to_string(),
            }),
        }
    }
}

impl RenderPassInstance for VkRenderPassInstance {
    fn bind(&self, frame: &Frame, context: &RenderPass, res: Vec2u32, pass_command_buffer: &dyn GfxCommandBuffer) {
        // Begin buffer
        let command_buffer = pass_command_buffer.cast::<VkCommandBuffer>().command_buffer.get(frame);

        let mut clear_values = Vec::new();
        for color in context.clear_values() {
            clear_values.push(match color {
                ClearValues::DontClear => vk::ClearValue::default(),
                ClearValues::Color(color) => vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [color.x, color.y, color.z, color.w],
                    },
                },
                ClearValues::DepthStencil(depth_stencil) => vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: depth_stencil.x,
                        stencil: depth_stencil.y as u32,
                    },
                },
            });
        }


        // begin pass
        let begin_infos = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass.render_pass)
            .framebuffer(self.framebuffer.get(frame))
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: res.x,
                    height: res.y,
                },
            })
            .clear_values(clear_values.as_slice())
            .build();

        unsafe {
            Gfx::get().cast::<GfxVulkan>().device().handle.cmd_begin_render_pass(
                command_buffer,
                &begin_infos,
                vk::SubpassContents::INLINE,
            )
        };

        unsafe {
            Gfx::get().cast::<GfxVulkan>().device().handle.cmd_set_viewport(
                command_buffer,
                0,
                &[vk::Viewport::builder()
                    .x(0.0)
                    .y(res.y as _)
                    .width(res.x as _)
                    .height(-(res.y as f32))
                    .min_depth(0.0)
                    .max_depth(1.0)
                    .build()],
            )
        };

        unsafe {
            Gfx::get().cast::<GfxVulkan>().device().handle.cmd_set_scissor(
                command_buffer,
                0,
                &[vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: vk::Extent2D {
                        width: res.x,
                        height: res.y,
                    },
                }],
            )
        };

    }

    fn submit(&self, frame: &Frame, context: &RenderPass, pass_command_buffer: &dyn GfxCommandBuffer) {


        pass_command_buffer.cast::<VkCommandBuffer>().init_for(
            PassID::new("null"),
            frame.clone(),
        );

        let command_buffer = pass_command_buffer.cast::<VkCommandBuffer>().command_buffer.get(frame);

        GfxVulkan::get().set_vk_object_name(
            command_buffer,
            format!(
                "command buffer\t\t: {} - {}",
                "null",
                frame.image_id()
            ).as_str(),
        );
        
        // End pass
        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .cmd_end_render_pass(command_buffer)
        };
        vk_check!(unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .handle
                .end_command_buffer(command_buffer)
        });

        // Submit buffer
        let mut wait_semaphores = Vec::new();
        if context.source().is_present_pass() {
            wait_semaphores.push(self.wait_semaphores.read().unwrap().unwrap());
        }

        for pass in context.inputs() {
            wait_semaphores.push(
                pass.instance().cast::<VkRenderPassInstance>().render_finished_semaphore.get(frame)
            )
        }

        // Which stages we wants to wait
        let wait_stages = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT; wait_semaphores.len()];

        // Submit
        unsafe {
            GfxVulkan::get()
                .device()
                .get_queue(vk::QueueFlags::GRAPHICS)
                .unwrap()
                .submit(
                    vk::SubmitInfo::builder()
                        .wait_semaphores(wait_semaphores.as_slice())
                        .wait_dst_stage_mask(wait_stages.as_slice())
                        .command_buffers(&[pass_command_buffer.cast::<VkCommandBuffer>().command_buffer.get(frame)])
                        .signal_semaphores(&[self.render_finished_semaphore.get(frame)])
                        .build(),
                );
        }
    }
}

pub struct RbSemaphore {
    pub name: String,
}

impl GfxImageBuilder<vk::Semaphore> for RbSemaphore {
    fn build(&self, swapchain_ref: &Frame) -> vk::Semaphore {
        let ci_semaphore = vk::SemaphoreCreateInfo::builder().build();

        GfxVulkan::get().set_vk_object_name(
            vk_check!(unsafe {
                GfxVulkan::get()
                    .device
                    .assume_init_ref()
                    .handle
                    .create_semaphore(&ci_semaphore, None)
            }),
            format!("semaphore\t\t: {}@{}", self.name, swapchain_ref).as_str(),
        )
    }
}

pub struct RbCommandBuffer {
    pub name: String,
}

impl GfxImageBuilder<vk::CommandBuffer> for RbCommandBuffer {
    fn build(&self, swapchain_ref: &Frame) -> vk::CommandBuffer {
        let ci_command_buffer = vk::CommandBufferAllocateInfo::builder()
            .command_pool(unsafe { GfxVulkan::get().command_pool.assume_init_ref() }.command_pool)
            .command_buffer_count(1)
            .build();

        let device = &GfxVulkan::get().device;
        let cmd_buffer = vk_check!(unsafe {
            device
                .assume_init_ref()
                .handle
                .allocate_command_buffers(&ci_command_buffer)
        })[0];
        GfxVulkan::get().set_vk_object_name(
            cmd_buffer,
            format!("command_buffer\t: {}@{}", self.name, swapchain_ref).as_str(),
        );
        cmd_buffer
    }
}

struct RbFramebuffer {
    render_pass: vk::RenderPass,
    res: Vec2u32,
    images: Vec<Arc<dyn GfxImage>>,
    name: String,
}

impl GfxImageBuilder<vk::Framebuffer> for RbFramebuffer {
    fn build(&self, swapchain_ref: &Frame) -> vk::Framebuffer {
        let mut attachments = Vec::new();

        for image in &self.images {
            attachments.push(image.cast::<VkImage>().view.get(swapchain_ref).0);
        }

        let create_infos = vk::FramebufferCreateInfo::builder()
            .render_pass(self.render_pass)
            .attachments(attachments.as_slice())
            .width(self.res.x)
            .height(self.res.y)
            .layers(1)
            .build();

        GfxVulkan::get().set_vk_object_name(
            vk_check!(unsafe {
                GfxVulkan::get()
                    .device
                    .assume_init_ref()
                    .handle
                    .create_framebuffer(&create_infos, None)
            }),
            format!("framebuffer\t\t: {}@{}", self.name, swapchain_ref).as_str(),
        )
    }
}