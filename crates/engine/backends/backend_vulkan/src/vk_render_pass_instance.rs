use std::sync::{Arc, RwLock};

use ash::vk;

use gfx::command_buffer::GfxCommandBuffer;
use gfx::gfx_resource::{GfxImageBuilder, GfxResource};
use gfx::image::{GfxImage, ImageCreateInfos, ImageParams, ImageType, ImageUsage};
use gfx::render_pass::{GraphRenderCallback, RenderPass, RenderPassInstance};
use gfx::surface::{GfxImageID, GfxSurface};
use gfx::types::ClearValues;
use gfx::{Gfx, render_node};
use maths::vec2::Vec2u32;

use crate::vk_command_buffer::VkCommandBuffer;
use crate::vk_image::VkImage;
use crate::{vk_check, GfxVulkan, VkRenderPass};

pub struct VkRenderPassInstance {
    pub render_finished_semaphore: GfxResource<vk::Semaphore>,
    pub pass_command_buffers: Arc<VkCommandBuffer>,
    owner: Arc<dyn RenderPass>,
    surface: Arc<dyn GfxSurface>,
    images: Vec<Arc<dyn GfxImage>>,
    framebuffers: GfxResource<vk::Framebuffer>,
    pub clear_value: Vec<ClearValues>,
    pub resolution: RwLock<Vec2u32>,
    pub wait_semaphores: RwLock<Option<vk::Semaphore>>,
    pub render_callback: RwLock<Option<GraphRenderCallback>>,
    pub children: RwLock<Vec<Arc<dyn RenderPassInstance>>>,
    name: String,
}

impl render_node::RenderPassInstance  for VkRenderPassInstance {
    fn init(&self, context: &render_node::RenderPass) {
        todo!()
    }

    fn bind(&self, context: &render_node::RenderPass, res: Vec2u32) {
        todo!()
    }

    fn submit(&self, context: &render_node::RenderPass, res: Vec2u32) {
        todo!()
    }
}

pub struct RbSemaphore {
    pub name: String,
}

impl GfxImageBuilder<vk::Semaphore> for RbSemaphore {
    fn build(&self, swapchain_ref: &GfxImageID) -> vk::Semaphore {
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
    fn build(&self, swapchain_ref: &GfxImageID) -> vk::CommandBuffer {
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
    fn build(&self, swapchain_ref: &GfxImageID) -> vk::Framebuffer {
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

impl VkRenderPassInstance {
    pub fn new(owner: &VkRenderPass, render_pass: &render_node::RenderPass  , res: Vec2u32) -> VkRenderPassInstance {

        let mut images = Vec::new();
        if owner.get_config().is_present_pass {
            images.push(surface.get_surface_texture())
        } else {
            for att_color in &owner.get_config().color_attachments {
                images.push(Gfx::get().create_image(
                    format!("render_pass[{}]::attachment[{}]", "undefined name", att_color.name),
                    ImageCreateInfos {
                        params: ImageParams {
                            pixel_format: att_color.image_format,
                            image_type: ImageType::Texture2d(res.x, res.y),
                            read_only: false,
                            mip_levels: None,
                            usage: ImageUsage::GpuWriteDestination | ImageUsage::Sampling,
                        },
                        pixels: None,
                    },
                ));
            }
            match &owner.get_config().depth_attachment {
                None => {}
                Some(depth_attachment) => {
                    images.push(Gfx::get().create_image(
                        format!("render_pass[{}]::depth_attachment", name),
                        ImageCreateInfos {
                            params: ImageParams {
                                pixel_format: depth_attachment.image_format,
                                image_type: ImageType::Texture2d(res.x, res.y),
                                read_only: false,
                                mip_levels: None,
                                usage: ImageUsage::GpuWriteDestination | ImageUsage::Sampling,
                            },
                            pixels: None,
                        },
                    ));
                }
            }
        }

        VkRenderPassInstance {
            render_finished_semaphore: GfxResource::new(RbSemaphore { name: name.clone() }),
            pass_command_buffers: VkCommandBuffer::new(name.clone(), surface),
            framebuffers: GfxResource::new(RbFramebuffer {
                render_pass,
                res,
                images: images.clone(),
                name: name.clone(),
            }),
            owner,
            clear_value: clear_values,
            surface: surface.clone(),
            resolution: RwLock::new(res),
            wait_semaphores: RwLock::new(None),
            render_callback: RwLock::new(None),
            children: RwLock::default(),
            images,
            name,
        }
    }
}

impl RenderPassInstance for VkRenderPassInstance {
    fn resize(&self, new_res: Vec2u32) {
        if self.owner.get_config().is_present_pass {
            let surf_text = self
                .surface
                .get_surface_texture()
                .cast::<VkImage>()
                .image
                .read()
                .unwrap()
                .clone();
            self.images[0]
                .cast::<VkImage>()
                .resize_from_existing_images(ImageType::Texture2d(new_res.x, new_res.y), surf_text);
        } else {
            for image in &self.images {
                image.resize(ImageType::Texture2d(new_res.x, new_res.y));
            }
        }

        self.framebuffers.invalidate(RbFramebuffer {
            render_pass: self.owner.cast::<VkRenderPass>().render_pass,
            res: new_res,
            images: self.images.clone(),
            name: self.name.clone(),
        });
        let mut res = self.resolution.write().unwrap();
        *res = new_res;
    }

    fn draw(&self) {
        for child in &*self.children.read().unwrap() {
            child.draw();
        }

        let device = &GfxVulkan::get().device;

        // Begin buffer
        let command_buffer = self
            .pass_command_buffers
            .command_buffer
            .get(self.surface.get_current_ref());

        vk_check!(unsafe {
            device
                .assume_init_ref()
                .handle
                .begin_command_buffer(command_buffer, &vk::CommandBufferBeginInfo::default())
        });

        let mut clear_values = Vec::new();
        for clear_value in &self.clear_value {
            clear_values.push(match clear_value {
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
        let res = self.resolution.read().unwrap();
        let begin_infos = vk::RenderPassBeginInfo::builder()
            .render_pass(self.owner.cast::<VkRenderPass>().render_pass)
            .framebuffer(self.framebuffers.get(self.surface.get_current_ref()))
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
            device.assume_init_ref().handle.cmd_begin_render_pass(
                command_buffer,
                &begin_infos,
                vk::SubpassContents::INLINE,
            )
        };

        unsafe {
            device.assume_init_ref().handle.cmd_set_viewport(
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
            device.assume_init_ref().handle.cmd_set_scissor(
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

        self.pass_command_buffers.init_for(
            self.owner.get_pass_id(),
            self.surface.get_current_ref().clone(),
        );

        // Draw content
        match self.render_callback.write() {
            Ok(mut render_callback) => match render_callback.as_mut() {
                None => {}
                Some(callback) => {
                    callback(&(self.pass_command_buffers.clone() as Arc<dyn GfxCommandBuffer>))
                }
            },
            Err(_) => {
                logger::fatal!("failed to access render callback")
            }
        }

        let command_buffer = self
            .pass_command_buffers
            .command_buffer
            .get(self.surface.get_current_ref());
        GfxVulkan::get().set_vk_object_name(
            command_buffer,
            format!(
                "command buffer\t\t: {} - {}",
                self.owner.get_config().pass_id,
                self.surface.get_current_ref()
            )
            .as_str(),
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
        if self
            .owner
            .cast::<VkRenderPass>()
            .get_config()
            .is_present_pass
        {
            wait_semaphores.push(self.wait_semaphores.read().unwrap().unwrap());
        }
        for child in &*self.children.read().unwrap() {
            wait_semaphores.push(
                child
                    .cast::<VkRenderPassInstance>()
                    .render_finished_semaphore
                    .get(self.surface.get_current_ref()),
            );
        }

        let wait_stages =
            vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT; wait_semaphores.len()];

        unsafe {
            GfxVulkan::get()
                .device
                .assume_init_ref()
                .get_queue(vk::QueueFlags::GRAPHICS)
                .unwrap()
                .submit(
                    vk::SubmitInfo::builder()
                        .wait_semaphores(wait_semaphores.as_slice())
                        .wait_dst_stage_mask(wait_stages.as_slice())
                        .command_buffers(&[command_buffer])
                        .signal_semaphores(&[self
                            .render_finished_semaphore
                            .get(self.surface.get_current_ref())])
                        .build(),
                );
        }
    }

    fn on_render(&self, callback: GraphRenderCallback) {
        *self.render_callback.write().unwrap() = Some(callback);
    }

    fn attach(&self, child: Arc<dyn RenderPassInstance>) {
        self.children.write().unwrap().push(child);
    }

    fn get_images(&self) -> &Vec<Arc<dyn GfxImage>> {
        &self.images
    }

    fn get_surface(&self) -> Arc<dyn GfxSurface> {
        self.surface.clone()
    }
}
