use std::{fs, slice};
use std::io::Error;
use std::os::raw::{c_int};
use std::path::Path;
use std::sync::Arc;

use stb_image_rust::{stbi_image_free, stbi_load_from_memory};
use gfx::GfxRef;

use gfx::image::{GfxImage, GfxImageUsageFlags, ImageCreateInfos, ImageParams, ImageType, ImageUsage};
use gfx::types::PixelFormat;

#[derive(Copy, Clone)]
pub enum ImageFormat {
    BMP,
    JPG,
    PNG,
    PSD,
    TGA
}

pub fn read_image_from_file(gfx: &GfxRef, file: &Path) -> Result<Arc<dyn GfxImage>, Error> {
    let mut width: i32 = 0;
    let mut height: i32 = 0;
    let mut components: i32 = 0;
    let required_components: i32 = 4;
    
    unsafe {
        match fs::read(file) {
            Ok(data) => {
                let raw_data = stbi_load_from_memory(data.as_ptr(), data.len() as i32, &mut width as *mut c_int, &mut height as *mut c_int, &mut components as *mut c_int, required_components);

                components = 4;
                let image = gfx.create_image(ImageCreateInfos {
                    params: ImageParams {
                        pixel_format: PixelFormat::R8G8B8A8_UNORM,
                        image_format: ImageType::Texture2d(width as u32, height as u32),
                        read_only: true,
                        mip_levels: Some((width as f32).max( height as f32).max( 1.0).log2().floor() as u16 + 1),
                        usage: GfxImageUsageFlags::from_flag(ImageUsage::Sampling)
                    },
                    pixels: Some(slice::from_raw_parts(raw_data as *const u8, width as usize * height as usize * components as usize).to_vec())
                });
                
                stbi_image_free(raw_data);

                Ok(image)
            }

            Err(error) => {
                Err(error)
            }
        }
    }
}