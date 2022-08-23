use std::sync::Arc;

use ash::vk;
use ash::vk::{ClearColorValue, ClearDepthStencilValue, ClearValue, CommandBuffer, CommandBufferAllocateInfo, Extent2D, Framebuffer, FramebufferCreateInfo, Image, ImageView, Offset2D, RenderPassBeginInfo, Semaphore, SemaphoreCreateInfo, SubpassContents};

use gfx::GfxRef;
use gfx::image::GfxImage;
use gfx::render_pass::{RenderPass, RenderPassInstance};
use gfx::surface::{GfxImageID, GfxSurface};
use gfx::types::{ClearValues, GfxCast};
use maths::vec2::Vec2u32;

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check, VkCommandPool, VkRenderPass};
use crate::vk_command_buffer::VkCommandBuffer;
use crate::vk_image::VkImage;
use crate::vk_swapchain_resource::{GfxImageBuilder, VkSwapchainResource};

pub struct VkRenderPassInstance {
    pub render_finished_semaphore: VkSwapchainResource<Semaphore>,
    pub pass_command_buffers: VkSwapchainResource<CommandBuffer>,
    owner: Arc<dyn RenderPass>,
    gfx: GfxRef,
    surface: Arc<dyn GfxSurface>,
    _framebuffers: VkSwapchainResource<Framebuffer>,
    pub clear_value: Vec<ClearValues>,
    pub resolution: Vec2u32,
}

pub struct RbSemaphore {}

impl GfxImageBuilder<Semaphore> for RbSemaphore {
    fn build(&self, gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> Semaphore {
        let ci_semaphore = SemaphoreCreateInfo {
            ..SemaphoreCreateInfo::default()
        };

        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        vk_check!(unsafe {gfx_object!(*device).device.create_semaphore(&ci_semaphore, None)})
    }
}

pub struct RbCommandBuffer {}

impl GfxImageBuilder<CommandBuffer> for RbCommandBuffer {
    fn build(&self, gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> CommandBuffer {
        let ci_command_buffer = CommandBufferAllocateInfo {
            command_pool: gfx_object!(*gfx_cast_vulkan!(gfx).command_pool.read().unwrap()).command_pool,
            command_buffer_count: 1,
            ..CommandBufferAllocateInfo::default()
        };

        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        vk_check!(unsafe {gfx_object!(*device).device.allocate_command_buffers(&ci_command_buffer)})[0]
    }
}

struct RbFramebuffer {
    render_pass: vk::RenderPass,
    res: Vec2u32,
    images: Vec<Arc<dyn GfxImage>>,
}

impl GfxImageBuilder<Framebuffer> for RbFramebuffer {
    fn build(&self, gfx: &GfxRef, _swapchain_ref: &GfxImageID) -> Framebuffer {
        let mut attachments = Vec::new();

        for image in &self.images {
            attachments.push(image.as_ref().as_any().downcast_ref::<VkImage>().unwrap().view.get(_swapchain_ref));
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
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();
        vk_check!(unsafe { gfx_object!(*device).device.create_framebuffer(&create_infos, None) })
    }
}

impl VkRenderPassInstance {
    pub fn new(gfx: &GfxRef, surface: &Arc<dyn GfxSurface>, owner: Arc<dyn RenderPass>, res: Vec2u32) -> VkRenderPassInstance {
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();

        let create_infos = SemaphoreCreateInfo {
            ..SemaphoreCreateInfo::default()
        };

        let render_finished_semaphore = vk_check!(unsafe { gfx_object!(*device).device.create_semaphore(&create_infos, None) });

        let clear_values = (&owner).get_clear_values().clone();

        let mut command_buffers = Vec::new();
        for _ in 0..surface.get_image_count() {
            command_buffers.push(VkCommandBuffer::new(gfx));
        }

        let render_pass = owner.as_ref().as_any().downcast_ref::<VkRenderPass>().unwrap().render_pass;

        let mut images = Vec::new();
        if owner.get_config().is_present_pass {
            images.push(surface.get_surface_texture())
        }
        else {
            for att_color in &owner.get_config().color_attachments {}
            for att_depth in &owner.get_config().depth_attachment {}            
        }


        VkRenderPassInstance {
            render_finished_semaphore: VkSwapchainResource::new(Box::new(RbSemaphore {})),
            pass_command_buffers: VkSwapchainResource::new(Box::new(RbCommandBuffer {})),
            _framebuffers: VkSwapchainResource::new(Box::new(RbFramebuffer { render_pass, res, images })),
            owner,
            clear_value: clear_values.clone(),
            gfx: gfx.clone(),
            surface: surface.clone(),
            resolution: res,
        }
    }
}

impl RenderPassInstance for VkRenderPassInstance {
    fn resize(&self, _new_res: Vec2u32) {
        todo!()
    }

    fn begin(&self) {
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

        let begin_infos = RenderPassBeginInfo {
            render_pass: self.owner.as_ref().as_any().downcast_ref::<VkRenderPass>().expect("invalid render pass").render_pass,
            framebuffer: self._framebuffers.get(&self.surface.get_current_ref()),
            render_area: vk::Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: Extent2D { width: self.resolution.x, height: self.resolution.y },
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..RenderPassBeginInfo::default()
        };

        let command_buffer = self.pass_command_buffers.get(&self.surface.get_current_ref());

        let device = gfx_cast_vulkan!(self.gfx).device.read().unwrap();

        vk_check!(unsafe { gfx_object!(*device).device.begin_command_buffer(command_buffer, &vk::CommandBufferBeginInfo::default()) });


        unsafe { gfx_object!(*device).device.cmd_begin_render_pass(command_buffer, &begin_infos, SubpassContents::INLINE) }
    }

    fn end(&self) {
        let device = gfx_cast_vulkan!(self.gfx).device.read().unwrap();


        let command_buffer = self.pass_command_buffers.get(&self.surface.get_current_ref());

        unsafe { gfx_object!(*device).device.cmd_end_render_pass(command_buffer) }

        vk_check!(unsafe { gfx_object!(*device).device.end_command_buffer(command_buffer) });
    }
}