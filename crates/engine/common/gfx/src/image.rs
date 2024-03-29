﻿use enumflags2::{bitflags, BitFlags};

use crate::GfxCast;
use crate::types::PixelFormat;

#[derive(Copy, Clone)]
pub enum ImageType {
    Texture1d(u32),
    Texture2d(u32, u32),
    Texture3d(u32, u32, u32),
    Texture1dArray(u32),
    Texture2dArray(u32, u32),
    TextureCube(u32, u32),
}

impl PartialEq for ImageType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ImageType::Texture1d(x1) => {
                match other {
                    ImageType::Texture1d(x2) => { x1 == x2 }
                    _ => { false }
                }
            }
            ImageType::Texture2d(x1, y1) => {
                match other {
                    ImageType::Texture2d(x2, y2) => { x1 == x2 && y1 == y2 }
                    _ => { false }
                }
            }
            ImageType::Texture3d(x1, y1, z1) => {
                match other {
                    ImageType::Texture3d(x2, y2, z2) => { x1 == x2 && y1 == y2 && z1 == z2 }
                    _ => { false }
                }
            }
            ImageType::Texture1dArray(x1) => {
                match other {
                    ImageType::Texture1dArray(x2) => { x1 == x2 }
                    _ => { false }
                }
            }
            ImageType::Texture2dArray(x1, y1) => {
                match other {
                    ImageType::Texture2dArray(x2, y2) => { x1 == x2 && y1 == y2 }
                    _ => { false }
                }
            }
            ImageType::TextureCube(x1, y1) => {
                match other {
                    ImageType::TextureCube(x2, y2) => { x1 == x2 && y1 == y2 }
                    _ => { false }
                }
            }
        }
    }
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ImageUsage {
    CopySource,
    CopyDestination,
    Sampling,
    GpuWriteDestination,
}

pub type GfxImageUsageFlags = BitFlags<ImageUsage>;

#[derive(Copy, Clone)]
pub struct ImageParams {
    pub pixel_format: PixelFormat,
    pub image_type: ImageType,
    pub read_only: bool,
    pub mip_levels: Option<u16>,
    pub usage: GfxImageUsageFlags,
}

impl ImageParams {
    pub fn get_mip_levels(&self) -> u16 {
        match self.mip_levels {
            None => { 1 }
            Some(levels) => { levels }
        }
    }

    pub fn array_layers(&self) -> u32 {
        match self.image_type {
            ImageType::TextureCube(_, _) => { 6 }
            _ => { 1 }
        }
    }
}

#[derive(Clone)]
pub struct ImageCreateInfos {
    pub params: ImageParams,
    pub pixels: Option<Vec<u8>>,
}

pub trait GfxImage: GfxCast {
    fn get_type(&self) -> ImageType;
    fn get_format(&self) -> PixelFormat;
    fn get_data(&self) -> &[u8];
    fn set_data(&self, data: &[u8]);
    fn get_data_size(&self) -> u32;
    fn resize(&self, new_type: ImageType);
    fn __static_view_handle(&self) -> u64;
}

impl dyn GfxImage {
    pub fn cast<U: GfxImage + 'static>(&self) -> &U {
        self.as_any().downcast_ref::<U>().unwrap()
    }
}

impl ImageType {
    pub fn pixel_count(&self) -> u32 {
        match self {
            ImageType::Texture1d(x) => { *x }
            ImageType::Texture2d(x, y) => { x * y }
            ImageType::Texture3d(x, y, z) => { x * y * z }
            ImageType::Texture1dArray(x) => { *x }
            ImageType::Texture2dArray(x, y) => { x * y }
            ImageType::TextureCube(x, y) => { x * y * 6 }
        }
    }

    pub fn dimensions(&self) -> (u32, u32, u32) {
        match self {
            ImageType::Texture1d(x) => { (*x, 1, 1) }
            ImageType::Texture2d(x, y) => { (*x, *y, 1) }
            ImageType::Texture3d(x, y, z) => { (*x, *y, *z) }
            ImageType::Texture1dArray(x) => { (*x, 1, 1) }
            ImageType::Texture2dArray(x, y) => { (*x, *y, 1) }
            ImageType::TextureCube(x, y) => { (*x, *y, 6) }
        }
    }
}