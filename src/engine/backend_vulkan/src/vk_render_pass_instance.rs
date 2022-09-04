use std::sync::{Arc, RwLock};

use ash::vk;
use ash::vk::{ClearColorValue, ClearDepthStencilValue, ClearValue, CommandBuffer, CommandBufferAllocateInfo, Extent2D, Framebuffer, FramebufferCreateInfo, Offset2D, PipelineStageFlags, QueueFlags, Rect2D, RenderPassBeginInfo, Semaphore, SemaphoreCreateInfo, SubmitInfo, SubpassContents, Viewport};

use gfx::command_buffer::GfxCommandBuffer;
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::GfxRef;
use gfx::image::{GfxImage, ImageCreateInfos, ImageParams, ImageType, ImageUsage};
use gfx::render_pass::{GraphRenderCallback, RenderPass, RenderPassInstance};
use gfx::surface::{GfxImageID, GfxSurface};
use gfx::types::{ClearValues};
use maths::vec2::Vec2u32;

use crate::{GfxVulkan, vk_check, VkRenderPass};
use crate::vk_command_buffer::VkCommandBuffer;
use crate::vk_image::VkImage;

pub struct VkRenderPassInstance {
    pub render_finished_semaphore: GfxResource<Semaphore>,
    pub pass_command_buffers: Arc<VkCommandBuffer>,
    owner: Arc<dyn RenderPass>,
    gfx: GfxRef,
    surface: Arc<dyn GfxSurface>,
    framebuffers: GfxResource<Framebuffer>,
    pub clear_value: Vec<ClearValues>,
    pub resolution: RwLock<Vec2u32>,
    pub wait_semaphores: RwLock<Option<Semaphore>>,
    pub render_callback: RwLock<Option<Box<dyn GraphRenderCallback>>>,
    pub children: RwLock<Vec<Arc<dyn RenderPassInstance>>>,
}

pub struct RbSemaphore {}

impl GfxImageBuilder<Semaphore> for RbSemaphore {
    fn build(&self, gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> Semaphore {
        let ci_semaphore = SemaphoreCreateInfo {
            ..SemaphoreCreateInfo::default()
        };

        let device = &gfx.cast::<GfxVulkan>().device;
        vk_check!(unsafe {(*device).handle.create_semaphore(&ci_semaphore, None)})
    }
}

pub struct RbCommandBuffer {}

impl GfxImageBuilder<CommandBuffer> for RbCommandBuffer {
    fn build(&self, gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> CommandBuffer {
        let ci_command_buffer = CommandBufferAllocateInfo {
            command_pool: gfx.cast::<GfxVulkan>().command_pool.command_pool,
            command_buffer_count: 1,
            ..CommandBufferAllocateInfo::default()
        };

        let device = &gfx.cast::<GfxVulkan>().device;
        vk_check!(unsafe {device.handle.allocate_command_buffers(&ci_command_buffer)})[0]
    }
}

struct RbFramebuffer {
    render_pass: vk::RenderPass,
    res: Vec2u32,
    images: Vec<Arc<dyn GfxImage>>,
}

impl GfxImageBuilder<Framebuffer> for RbFramebuffer {
    fn build(&self, gfx: &GfxRef, swapchain_ref: &GfxImageID) -> Framebuffer {
        let mut attachments = Vec::new();

        for image in &self.images {
            attachments.push(image.as_ref().cast::<VkImage>().view.get(swapchain_ref).0);
        }

        let create_infos = FramebufferCreateInfo {
            render_pass: self.render_pass,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: self.res.x,
            height: self.res.y,
            layers: 1,
            ..FramebufferCreateInfo::default()
        };
        let device = &gfx.cast::<GfxVulkan>().device;
        vk_check!(unsafe { (*device).handle.create_framebuffer(&create_infos, None) })
    }
}

impl VkRenderPassInstance {
    pub fn new(gfx: &GfxRef, surface: &Arc<dyn GfxSurface>, owner: Arc<dyn RenderPass>, res: Vec2u32) -> VkRenderPassInstance {
        let clear_values = (&owner).get_clear_values().clone();

        let mut command_buffers = Vec::new();
        for _ in 0..surface.get_image_count() {
            command_buffers.push(VkCommandBuffer::new(gfx));
        }

        let render_pass = owner.cast::<VkRenderPass>().render_pass;

        let mut images = Vec::new();
        if owner.get_config().is_present_pass {
            images.push(surface.get_surface_texture())
        } else {
            for _att_color in &owner.get_config().color_attachments {
                images.push(gfx.create_image(ImageCreateInfos {
                    params: ImageParams {
                        pixel_format: _att_color.image_format,
                        image_type: ImageType::Texture2d(res.x, res.y),
                        read_only: false,
                        mip_levels: None,
                        usage: ImageUsage::GpuWriteDestination | ImageUsage::Sampling,
                    },
                    pixels: None,
                }));
            }
            for _att_depth in &owner.get_config().depth_attachment {}
        }

        VkRenderPassInstance {
            render_finished_semaphore: GfxResource::new(gfx, RbSemaphore {}),
            pass_command_buffers: VkCommandBuffer::new(gfx),
            framebuffers: GfxResource::new(gfx, RbFramebuffer { render_pass, res, images }),
            owner,
            clear_value: clear_values.clone(),
            gfx: gfx.clone(),
            surface: surface.clone(),
            resolution: RwLock::new(res),
            wait_semaphores: RwLock::new(None),
            render_callback: RwLock::new(None),
            children: RwLock::default(),
        }
    }
}

impl RenderPassInstance for VkRenderPassInstance {
    fn resize(&self, new_res: Vec2u32) {
        let render_pass = self.owner.cast::<VkRenderPass>().render_pass;

        let mut images = Vec::new();
        if self.owner.get_config().is_present_pass {
            images.push(self.surface.get_surface_texture())
        } else {
            for att_color in &self.owner.get_config().color_attachments {
                images.push(self.gfx.create_image(ImageCreateInfos {
                    params: ImageParams {
                        pixel_format: att_color.image_format,
                        image_type: ImageType::Texture2d(new_res.x, new_res.y),
                        read_only: false,
                        mip_levels: None,
                        usage: ImageUsage::GpuWriteDestination | ImageUsage::Sampling,
                    },
                    pixels: None,
                }))
            }
            for _att_depth in &self.owner.get_config().depth_attachment {}
        }
        self.framebuffers.invalidate(&self.gfx, Box::new(RbFramebuffer { render_pass, res: new_res, images }));
        let mut res = self.resolution.write().unwrap();
        *res = new_res;
    }

    fn draw(&self) {
        for child in &*self.children.read().unwrap() {
            child.draw();
        }

        let device = &self.gfx.cast::<GfxVulkan>().device;

        // Begin buffer
        vk_check!(unsafe { (*device).handle.begin_command_buffer(self.pass_command_buffers.command_buffer.get(&self.surface.get_current_ref()), &vk::CommandBufferBeginInfo::default()) });

        let mut clear_values = Vec::new();
        for clear_value in &self.clear_value {
            clear_values.push(match clear_value {
                ClearValues::DontClear => { ClearValue::default() }
                ClearValues::Color(color) => {
                    ClearValue {
                        color: ClearColorValue {
                            float32: [color.x, color.y, color.z, color.w]
                        }
                    }
                }
                ClearValues::DepthStencil(depth_stencil) => {
                    ClearValue {
                        depth_stencil: ClearDepthStencilValue {
                            depth: depth_stencil.x,
                            stencil: depth_stencil.y as u32,
                        }
                    }
                }
            });
        }

        // begin pass
        let command_buffer = self.pass_command_buffers.command_buffer.get(&self.surface.get_current_ref());
        let res = self.resolution.read().unwrap();
        let begin_infos = RenderPassBeginInfo {
            render_pass: self.owner.cast::<VkRenderPass>().render_pass,
            framebuffer: self.framebuffers.get(&self.surface.get_current_ref()),
            render_area: Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: Extent2D { width: res.x, height: res.y },
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..RenderPassBeginInfo::default()
        };
        unsafe { (*device).handle.cmd_begin_render_pass(command_buffer, &begin_infos, SubpassContents::INLINE) };

        unsafe {
            (*device).handle.cmd_set_viewport(command_buffer, 0, &[Viewport {
                x: 0.0,
                y: res.y as f32,
                width: res.x as f32,
                height: -(res.y as f32),
                min_depth: 0.0,
                max_depth: 1.0,
            }])
        };

        unsafe {
            (*device).handle.cmd_set_scissor(command_buffer, 0, &[Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: Extent2D { width: res.x, height: res.y },
            }])
        };

        self.pass_command_buffers.init_for(self.owner.get_pass_id(), self.surface.get_current_ref().clone());

        // Draw content
        match &*self.render_callback.read().unwrap() {
            None => {}
            Some(callback) => { callback.draw(&(self.pass_command_buffers.clone() as Arc<dyn GfxCommandBuffer>)) }
        }

        let command_buffer = self.pass_command_buffers.command_buffer.get(&self.surface.get_current_ref());
        let device = &self.gfx.cast::<GfxVulkan>().device;

        // End pass
        unsafe { (*device).handle.cmd_end_render_pass(command_buffer) };
        vk_check!(unsafe { (*device).handle.end_command_buffer(command_buffer) });

        // Submit buffer
        let mut wait_semaphores = Vec::new();
        if self.owner.cast::<VkRenderPass>().get_config().is_present_pass {
            wait_semaphores.push(self.wait_semaphores.read().unwrap().unwrap());
        }
        for child in &*self.children.read().unwrap() {
            wait_semaphores.push(child.cast::<VkRenderPassInstance>().render_finished_semaphore.get(&self.surface.get_current_ref()));
        }

        let wait_stages = vec![PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT; wait_semaphores.len()];

        let submit_infos = SubmitInfo {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            signal_semaphore_count: 1,
            p_signal_semaphores: &self.render_finished_semaphore.get(&self.surface.get_current_ref()),
            ..SubmitInfo::default()
        };

        (*device).get_queue(QueueFlags::GRAPHICS).unwrap().submit(submit_infos);
    }

    fn on_render(&self, callback: Box<dyn GraphRenderCallback>) {
        *self.render_callback.write().unwrap() = Some(callback);
    }

    fn attach(&self, child: Arc<dyn RenderPassInstance>) {
        self.children.write().unwrap().push(child);
    }
}