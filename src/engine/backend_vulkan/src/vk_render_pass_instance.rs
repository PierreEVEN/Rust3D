use std::any::{TypeId};
use std::sync::Arc;
use ash::vk::{ClearColorValue, ClearDepthStencilValue, ClearValue, RenderPassBeginInfo, Semaphore, SemaphoreCreateInfo};

use gfx::{GfxCast, GfxRef};
use gfx::render_pass::RenderPass;
use gfx::render_pass_instance::RenderPassInstance;
use gfx::types::ClearValues;
use maths::vec2::Vec2u32;

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check, VkRenderPass};
use crate::vk_swapchain_resource::VkSwapchainResource;

pub struct VkRenderPassInstance {
    pub render_finished_semaphore: VkSwapchainResource<Semaphore>,
    owner: Arc<dyn RenderPass>,
    gfx: GfxRef,
    pub clear_value: Vec<ClearValues>,
}

impl VkRenderPassInstance {
    pub fn new(gfx: &GfxRef, owner: Arc<dyn RenderPass>, res: Vec2u32) -> VkRenderPassInstance {
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();

        let create_infos = SemaphoreCreateInfo {
            ..SemaphoreCreateInfo::default()
        };

        let render_finished_semaphore = vk_check!(unsafe { gfx_object!(*device).device.create_semaphore(&create_infos, None) });

        let clear_values = (&owner).get_clear_values().clone();

        VkRenderPassInstance {
            render_finished_semaphore: VkSwapchainResource::new(vec![render_finished_semaphore], 3),
            owner,
            clear_value: clear_values.clone(),
            gfx: gfx.clone()
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
            render_pass: self.owner.as_any().downcast_ref::<VkRenderPass>().unwrap().render_pass,
            framebuffer: Default::default(),
            render_area: Default::default(),
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..RenderPassBeginInfo::default()
        };


        let device = gfx_cast_vulkan!(self.gfx).device.read().unwrap();
        //gfx_object!(*device).device.cmd_begin_render_pass()
    }

    fn submit(&self) {
        todo!()
    }
}