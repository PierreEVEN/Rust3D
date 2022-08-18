use std::any::{TypeId};
use std::sync::Arc;
use ash::vk::{ClearColorValue, ClearDepthStencilValue, ClearValue, CommandBuffer, Framebuffer, FramebufferCreateInfo, RenderPassBeginInfo, Semaphore, SemaphoreCreateInfo, SubpassContents};

use gfx::{GfxRef};
use gfx::render_pass::{RenderPass, RenderPassInstance};
use gfx::surface::GfxSurface;
use gfx::types::{ClearValues, GfxCast};
use maths::vec2::Vec2u32;

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check, VkCommandPool, VkRenderPass};
use crate::vk_command_buffer::VkCommandBuffer;
use crate::vk_image::VkImage;
use crate::vk_swapchain_resource::VkSwapchainResource;

pub struct VkRenderPassInstance {
    pub render_finished_semaphore: VkSwapchainResource<Semaphore>,
    pub pass_command_buffers: VkSwapchainResource<VkCommandBuffer>,
    owner: Arc<dyn RenderPass>,
    gfx: GfxRef,
    surface: Arc<dyn GfxSurface>,
    framebuffers: VkSwapchainResource<Framebuffer>,
    pub clear_value: Vec<ClearValues>,
}

impl VkRenderPassInstance {
    pub fn new(gfx: &GfxRef, surface: &Arc<dyn GfxSurface>, owner: Arc<dyn RenderPass>, res: Vec2u32, use_surface_images: bool) -> VkRenderPassInstance {
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
        
        
        let mut framebuffers = Vec::new();
        for i in 0..surface.get_image_count() {

            let mut images = Vec::new();
            if use_surface_images {
                images.push(surface.get_images()[i as usize].as_any().downcast_ref::<VkImage>().unwrap().view);
            }
            
            
            let create_infos = FramebufferCreateInfo {
                render_pass: owner.as_ref().as_any().downcast_ref::<VkRenderPass>().unwrap().render_pass,
                attachment_count: images.len() as u32,
                p_attachments: images.as_ptr(),
                width: res.x,
                height: res.y,
                layers: 0,
                ..FramebufferCreateInfo::default()
            };
            unsafe { framebuffers.push(vk_check!(gfx_object!(*device).device.create_framebuffer(&create_infos, None))); }
        }

        VkRenderPassInstance {
            render_finished_semaphore: VkSwapchainResource::new(vec![render_finished_semaphore], surface.get_image_count()),
            pass_command_buffers: VkSwapchainResource::new(command_buffers, surface.get_image_count()),
            framebuffers: VkSwapchainResource::new(framebuffers, surface.get_image_count()),
            owner,
            clear_value: clear_values.clone(),
            gfx: gfx.clone(),
            surface: surface.clone(),
        }
    }
}

impl RenderPassInstance for VkRenderPassInstance {
    fn resize(&self, new_res: Vec2u32) {
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
            framebuffer: Default::default(),
            render_area: Default::default(),
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..RenderPassBeginInfo::default()
        };

        let command_buffer = self.pass_command_buffers.get_image(self.surface.get_current_image());
        command_buffer.start();

        let device = gfx_cast_vulkan!(self.gfx).device.read().unwrap();
        unsafe { gfx_object!(*device).device.cmd_begin_render_pass(command_buffer.command_buffer, &begin_infos, SubpassContents::INLINE) }
    }

    fn end(&self) {
        let device = gfx_cast_vulkan!(self.gfx).device.read().unwrap();


        let command_buffer = self.pass_command_buffers.get_image(self.surface.get_current_image());

        unsafe { gfx_object!(*device).device.cmd_end_render_pass(command_buffer.command_buffer) }

        command_buffer.submit();        
    }
}