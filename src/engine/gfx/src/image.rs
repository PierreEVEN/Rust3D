
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
    GpuWriteDestination
}
pub type WindowFlags = BitFlags<ImageUsage>;

#[derive(Copy, Clone)]
pub struct ImageParams {
    pub pixel_format: PixelFormat,
    pub image_format: ImageType,
    pub read_only: bool,
    pub mip_levels: Option<u16>,
    pub usage: ImageUsage,
}

pub trait GfxImage: GfxCast {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_depth(&self) -> u32;
    fn get_format(&self) -> PixelFormat;
}