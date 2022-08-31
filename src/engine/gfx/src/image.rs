use enumflags2::{bitflags, BitFlags};

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

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ImageUsage {
    Any,
    CopySource,
    CopyDestination,
    Sampling,
    GpuWriteDestination,
}

pub type GfxImageUsageFlags = BitFlags<ImageUsage>;

#[derive(Copy, Clone)]
pub struct ImageParams {
    pub pixel_format: PixelFormat,
    pub image_format: ImageType,
    pub read_only: bool,
    pub mip_levels: Option<u16>,
    pub usage: ImageUsage,
}

#[derive(Clone)]
pub struct ImageCreateInfos {
    pub params: ImageParams,
    pub pixels: Option<Vec<u8>>,
}

pub trait GfxImage: GfxCast {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_depth(&self) -> u32;
    fn get_format(&self) -> PixelFormat;
    fn get_data(&self) -> Vec<u8>;
    fn set_data(&self, data: Vec<u8>);
    fn get_data_size(&self) -> u32;
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
            ImageType::Texture2d(x, y) => {(*x, *y, 1) }
            ImageType::Texture3d(x, y, z) => { (*x, *y, *z) }
            ImageType::Texture1dArray(x) => { (*x, 1, 1) }
            ImageType::Texture2dArray(x, y) => { (*x, *y, 1) }
            ImageType::TextureCube(x, y) => { (*x, *y, 6) }
        }}
}