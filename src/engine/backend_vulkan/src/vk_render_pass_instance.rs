use std::ptr::null;
use std::sync::Arc;
use ash::vk::{RenderPassBeginInfo, Semaphore, SemaphoreCreateInfo};

use gfx::{GfxCast, GfxRef};
use gfx::render_pass::RenderPass;
use gfx::render_pass_instance::RenderPassInstance;
use maths::vec2::Vec2u32;

use crate::{gfx_cast_vulkan, gfx_object, GfxVulkan, vk_check, VkRenderPass};

pub struct VkRenderPassInstance {
    pub render_finished_semaphore: Semaphore,
    owner: Arc<dyn RenderPass>,
}

impl VkRenderPassInstance {
    pub fn new(gfx: GfxRef, owner: Arc<dyn RenderPass>, res: Vec2u32) -> VkRenderPassInstance {
        let device = gfx_cast_vulkan!(gfx).device.read().unwrap();

        let create_infos = SemaphoreCreateInfo {
            ..SemaphoreCreateInfo::default()
        };

        let render_finished_semaphore = vk_check!(unsafe { gfx_object!(*device).device.create_semaphore(&create_infos, None) });


        VkRenderPassInstance {
            render_finished_semaphore,
            owner
        }
    }
}

impl RenderPassInstance for VkRenderPassInstance {
    fn resize(&self, new_res: Vec2u32) {
        todo!()
    }

    fn begin(&self) {
        todo!();
        
        //@TODO : clear values
        let begin_infos = RenderPassBeginInfo {
            render_pass: self.owner.as_any().downcast_ref::<VkRenderPass>().unwrap().render_pass,
            framebuffer: Default::default(),
            render_area: Default::default(),
            clear_value_count: 0,
            p_clear_values: null(),
            ..RenderPassBeginInfo::default()
        };
    }

    fn submit(&self) {
        todo!()
    }
}