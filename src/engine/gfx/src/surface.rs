﻿use std::hash::{Hash, Hasher};
use std::sync::Arc;

use plateform::window::Window;

use crate::{GfxCast, GfxRef, RenderPass, RenderPassCreateInfos};
use crate::image::GfxImage;
use crate::render_pass::RenderPassInstance;
use crate::types::PixelFormat;

#[derive(Clone)]
pub struct GfxImageID {
    _gfx: GfxRef,
    pub image_index: u8,
    render_pass_index: u8,
}

impl GfxImageID {
    pub fn new(gfx: GfxRef, image_index: u8, render_pass_index: u8) -> Self {
        Self {
            _gfx: gfx,
            image_index,
            render_pass_index,
        }
    }
    pub fn gfx(&self) -> &GfxRef {
        &self._gfx
    }
}

impl PartialEq for GfxImageID {
    fn eq(&self, other: &Self) -> bool {
        self.image_index == other.image_index && self.render_pass_index == other.render_pass_index
    }
}

impl Eq for GfxImageID {}

impl Hash for GfxImageID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16((self.image_index as u16) << 8 | self.render_pass_index as u16)
    }
}

pub trait GfxSurface: GfxCast {
    fn create_or_recreate(&self);
    fn get_owning_window(&self) -> &Arc<dyn Window>;
    fn get_surface_pixel_format(&self) -> PixelFormat;
    fn get_image_count(&self) -> u8;
    fn get_current_ref(&self) -> GfxImageID;
    fn get_surface_texture(&self) -> Arc<dyn GfxImage>;

    fn create_render_pass(&self, create_infos: RenderPassCreateInfos) -> Arc<dyn RenderPass>;
    fn get_gfx(&self) -> &GfxRef;

    fn acquire(&self, render_pass: &Arc<dyn RenderPassInstance>) -> Result<(), String>;
    fn submit(&self);
}