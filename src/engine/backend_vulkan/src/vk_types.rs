use std::ops::Deref;

use ash::vk;
use ash::vk::{ColorSpaceKHR, Extent2D, Format};

use gfx::types::{ColorSpace, PixelFormat};
use maths::vec2::Vec2u32;

pub struct VkExtent2D(Extent2D);

impl Deref for VkExtent2D {
    type Target = Extent2D;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec2u32> for VkExtent2D {
    fn from(size: Vec2u32) -> Self {
        VkExtent2D { 0: Extent2D { width: size.x, height: size.y } }
    }
}

pub struct VkPixelFormat(Format);

impl Deref for VkPixelFormat {
    type Target = Format;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct GfxPixelFormat(PixelFormat);

impl Deref for GfxPixelFormat {
    type Target = PixelFormat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&PixelFormat> for VkPixelFormat {
    fn from(format: &PixelFormat) -> Self {
        match format {
            PixelFormat::UNDEFINED => { VkPixelFormat(Format::UNDEFINED) }
            PixelFormat::R4G4_UNORM_PACK8 => { VkPixelFormat(Format::R4G4_UNORM_PACK8) }
            PixelFormat::R4G4B4A4_UNORM_PACK16 => { VkPixelFormat(Format::R4G4B4A4_UNORM_PACK16) }
            PixelFormat::B4G4R4A4_UNORM_PACK16 => { VkPixelFormat(Format::B4G4R4A4_UNORM_PACK16) }
            PixelFormat::R5G6B5_UNORM_PACK16 => { VkPixelFormat(Format::R5G6B5_UNORM_PACK16) }
            PixelFormat::B5G6R5_UNORM_PACK16 => { VkPixelFormat(Format::B5G6R5_UNORM_PACK16) }
            PixelFormat::R5G5B5A1_UNORM_PACK16 => { VkPixelFormat(Format::R5G5B5A1_UNORM_PACK16) }
            PixelFormat::B5G5R5A1_UNORM_PACK16 => { VkPixelFormat(Format::B5G5R5A1_UNORM_PACK16) }
            PixelFormat::A1R5G5B5_UNORM_PACK16 => { VkPixelFormat(Format::A1R5G5B5_UNORM_PACK16) }
            PixelFormat::R8_UNORM => { VkPixelFormat(Format::R8_UNORM) }
            PixelFormat::R8_SNORM => { VkPixelFormat(Format::R8_SNORM) }
            PixelFormat::R8_USCALED => { VkPixelFormat(Format::R8_USCALED) }
            PixelFormat::R8_SSCALED => { VkPixelFormat(Format::R8_SSCALED) }
            PixelFormat::R8_UINT => { VkPixelFormat(Format::R8_UINT) }
            PixelFormat::R8_SINT => { VkPixelFormat(Format::R8_SINT) }
            PixelFormat::R8_SRGB => { VkPixelFormat(Format::R8_SRGB) }
            PixelFormat::R8G8_UNORM => { VkPixelFormat(Format::R8G8_UNORM) }
            PixelFormat::R8G8_SNORM => { VkPixelFormat(Format::R8G8_SNORM) }
            PixelFormat::R8G8_USCALED => { VkPixelFormat(Format::R8G8_USCALED) }
            PixelFormat::R8G8_SSCALED => { VkPixelFormat(Format::R8G8_SSCALED) }
            PixelFormat::R8G8_UINT => { VkPixelFormat(Format::R8G8_UINT) }
            PixelFormat::R8G8_SINT => { VkPixelFormat(Format::R8G8_SINT) }
            PixelFormat::R8G8_SRGB => { VkPixelFormat(Format::R8G8_SRGB) }
            PixelFormat::R8G8B8_UNORM => { VkPixelFormat(Format::R8G8B8_UNORM) }
            PixelFormat::R8G8B8_SNORM => { VkPixelFormat(Format::R8G8B8_SNORM) }
            PixelFormat::R8G8B8_USCALED => { VkPixelFormat(Format::R8G8B8_USCALED) }
            PixelFormat::R8G8B8_SSCALED => { VkPixelFormat(Format::R8G8B8_SSCALED) }
            PixelFormat::R8G8B8_UINT => { VkPixelFormat(Format::R8G8B8_UINT) }
            PixelFormat::R8G8B8_SINT => { VkPixelFormat(Format::R8G8B8_SINT) }
            PixelFormat::R8G8B8_SRGB => { VkPixelFormat(Format::R8G8B8_SRGB) }
            PixelFormat::B8G8R8_UNORM => { VkPixelFormat(Format::B8G8R8_UNORM) }
            PixelFormat::B8G8R8_SNORM => { VkPixelFormat(Format::B8G8R8_SNORM) }
            PixelFormat::B8G8R8_USCALED => { VkPixelFormat(Format::B8G8R8_USCALED) }
            PixelFormat::B8G8R8_SSCALED => { VkPixelFormat(Format::B8G8R8_SSCALED) }
            PixelFormat::B8G8R8_UINT => { VkPixelFormat(Format::B8G8R8_UINT) }
            PixelFormat::B8G8R8_SINT => { VkPixelFormat(Format::B8G8R8_SINT) }
            PixelFormat::B8G8R8_SRGB => { VkPixelFormat(Format::B8G8R8_SRGB) }
            PixelFormat::R8G8B8A8_UNORM => { VkPixelFormat(Format::R8G8B8A8_UNORM) }
            PixelFormat::R8G8B8A8_SNORM => { VkPixelFormat(Format::R8G8B8A8_SNORM) }
            PixelFormat::R8G8B8A8_USCALED => { VkPixelFormat(Format::R8G8B8A8_USCALED) }
            PixelFormat::R8G8B8A8_SSCALED => { VkPixelFormat(Format::R8G8B8A8_SSCALED) }
            PixelFormat::R8G8B8A8_UINT => { VkPixelFormat(Format::R8G8B8A8_UINT) }
            PixelFormat::R8G8B8A8_SINT => { VkPixelFormat(Format::R8G8B8A8_SINT) }
            PixelFormat::R8G8B8A8_SRGB => { VkPixelFormat(Format::R8G8B8A8_SRGB) }
            PixelFormat::B8G8R8A8_UNORM => { VkPixelFormat(Format::B8G8R8A8_UNORM) }
            PixelFormat::B8G8R8A8_SNORM => { VkPixelFormat(Format::B8G8R8A8_SNORM) }
            PixelFormat::B8G8R8A8_USCALED => { VkPixelFormat(Format::B8G8R8A8_USCALED) }
            PixelFormat::B8G8R8A8_SSCALED => { VkPixelFormat(Format::B8G8R8A8_SSCALED) }
            PixelFormat::B8G8R8A8_UINT => { VkPixelFormat(Format::B8G8R8A8_UINT) }
            PixelFormat::B8G8R8A8_SINT => { VkPixelFormat(Format::B8G8R8A8_SINT) }
            PixelFormat::B8G8R8A8_SRGB => { VkPixelFormat(Format::B8G8R8A8_SRGB) }
            PixelFormat::A8B8G8R8_UNORM_PACK32 => { VkPixelFormat(Format::A8B8G8R8_UNORM_PACK32) }
            PixelFormat::A8B8G8R8_SNORM_PACK32 => { VkPixelFormat(Format::A8B8G8R8_SNORM_PACK32) }
            PixelFormat::A8B8G8R8_USCALED_PACK32 => { VkPixelFormat(Format::A8B8G8R8_USCALED_PACK32) }
            PixelFormat::A8B8G8R8_SSCALED_PACK32 => { VkPixelFormat(Format::A8B8G8R8_SSCALED_PACK32) }
            PixelFormat::A8B8G8R8_UINT_PACK32 => { VkPixelFormat(Format::A8B8G8R8_UINT_PACK32) }
            PixelFormat::A8B8G8R8_SINT_PACK32 => { VkPixelFormat(Format::A8B8G8R8_SINT_PACK32) }
            PixelFormat::A8B8G8R8_SRGB_PACK32 => { VkPixelFormat(Format::A8B8G8R8_SRGB_PACK32) }
            PixelFormat::A2R10G10B10_UNORM_PACK32 => { VkPixelFormat(Format::A2R10G10B10_UNORM_PACK32) }
            PixelFormat::A2R10G10B10_SNORM_PACK32 => { VkPixelFormat(Format::A2R10G10B10_SNORM_PACK32) }
            PixelFormat::A2R10G10B10_USCALED_PACK32 => { VkPixelFormat(Format::A2R10G10B10_USCALED_PACK32) }
            PixelFormat::A2R10G10B10_SSCALED_PACK32 => { VkPixelFormat(Format::A2R10G10B10_SSCALED_PACK32) }
            PixelFormat::A2R10G10B10_UINT_PACK32 => { VkPixelFormat(Format::A2R10G10B10_UINT_PACK32) }
            PixelFormat::A2R10G10B10_SINT_PACK32 => { VkPixelFormat(Format::A2R10G10B10_SINT_PACK32) }
            PixelFormat::A2B10G10R10_UNORM_PACK32 => { VkPixelFormat(Format::A2B10G10R10_UNORM_PACK32) }
            PixelFormat::A2B10G10R10_SNORM_PACK32 => { VkPixelFormat(Format::A2B10G10R10_SNORM_PACK32) }
            PixelFormat::A2B10G10R10_USCALED_PACK32 => { VkPixelFormat(Format::A2B10G10R10_USCALED_PACK32) }
            PixelFormat::A2B10G10R10_SSCALED_PACK32 => { VkPixelFormat(Format::A2B10G10R10_SSCALED_PACK32) }
            PixelFormat::A2B10G10R10_UINT_PACK32 => { VkPixelFormat(Format::A2B10G10R10_UINT_PACK32) }
            PixelFormat::A2B10G10R10_SINT_PACK32 => { VkPixelFormat(Format::A2B10G10R10_SINT_PACK32) }
            PixelFormat::R16_UNORM => { VkPixelFormat(Format::R16_UNORM) }
            PixelFormat::R16_SNORM => { VkPixelFormat(Format::R16_SNORM) }
            PixelFormat::R16_USCALED => { VkPixelFormat(Format::R16_USCALED) }
            PixelFormat::R16_SSCALED => { VkPixelFormat(Format::R16_SSCALED) }
            PixelFormat::R16_UINT => { VkPixelFormat(Format::R16_UINT) }
            PixelFormat::R16_SINT => { VkPixelFormat(Format::R16_SINT) }
            PixelFormat::R16_SFLOAT => { VkPixelFormat(Format::R16_SFLOAT) }
            PixelFormat::R16G16_UNORM => { VkPixelFormat(Format::R16G16_UNORM) }
            PixelFormat::R16G16_SNORM => { VkPixelFormat(Format::R16G16_SNORM) }
            PixelFormat::R16G16_USCALED => { VkPixelFormat(Format::R16G16_USCALED) }
            PixelFormat::R16G16_SSCALED => { VkPixelFormat(Format::R16G16_SSCALED) }
            PixelFormat::R16G16_UINT => { VkPixelFormat(Format::R16G16_UINT) }
            PixelFormat::R16G16_SINT => { VkPixelFormat(Format::R16G16_SINT) }
            PixelFormat::R16G16_SFLOAT => { VkPixelFormat(Format::R16G16_SFLOAT) }
            PixelFormat::R16G16B16_UNORM => { VkPixelFormat(Format::R16G16B16_UNORM) }
            PixelFormat::R16G16B16_SNORM => { VkPixelFormat(Format::R16G16B16_SNORM) }
            PixelFormat::R16G16B16_USCALED => { VkPixelFormat(Format::R16G16B16_USCALED) }
            PixelFormat::R16G16B16_SSCALED => { VkPixelFormat(Format::R16G16B16_SSCALED) }
            PixelFormat::R16G16B16_UINT => { VkPixelFormat(Format::R16G16B16_UINT) }
            PixelFormat::R16G16B16_SINT => { VkPixelFormat(Format::R16G16B16_SINT) }
            PixelFormat::R16G16B16_SFLOAT => { VkPixelFormat(Format::R16G16B16_SFLOAT) }
            PixelFormat::R16G16B16A16_UNORM => { VkPixelFormat(Format::R16G16B16A16_UNORM) }
            PixelFormat::R16G16B16A16_SNORM => { VkPixelFormat(Format::R16G16B16A16_SNORM) }
            PixelFormat::R16G16B16A16_USCALED => { VkPixelFormat(Format::R16G16B16A16_USCALED) }
            PixelFormat::R16G16B16A16_SSCALED => { VkPixelFormat(Format::R16G16B16A16_SSCALED) }
            PixelFormat::R16G16B16A16_UINT => { VkPixelFormat(Format::R16G16B16A16_UINT) }
            PixelFormat::R16G16B16A16_SINT => { VkPixelFormat(Format::R16G16B16A16_SINT) }
            PixelFormat::R16G16B16A16_SFLOAT => { VkPixelFormat(Format::R16G16B16A16_SFLOAT) }
            PixelFormat::R32_UINT => { VkPixelFormat(Format::R32_UINT) }
            PixelFormat::R32_SINT => { VkPixelFormat(Format::R32_SINT) }
            PixelFormat::R32_SFLOAT => { VkPixelFormat(Format::R32_SFLOAT) }
            PixelFormat::R32G32_UINT => { VkPixelFormat(Format::R32G32_UINT) }
            PixelFormat::R32G32_SINT => { VkPixelFormat(Format::R32G32_SINT) }
            PixelFormat::R32G32_SFLOAT => { VkPixelFormat(Format::R32G32_SFLOAT) }
            PixelFormat::R32G32B32_UINT => { VkPixelFormat(Format::R32G32B32_UINT) }
            PixelFormat::R32G32B32_SINT => { VkPixelFormat(Format::R32G32B32_SINT) }
            PixelFormat::R32G32B32_SFLOAT => { VkPixelFormat(Format::R32G32B32_SFLOAT) }
            PixelFormat::R32G32B32A32_UINT => { VkPixelFormat(Format::R32G32B32A32_UINT) }
            PixelFormat::R32G32B32A32_SINT => { VkPixelFormat(Format::R32G32B32A32_SINT) }
            PixelFormat::R32G32B32A32_SFLOAT => { VkPixelFormat(Format::R32G32B32A32_SFLOAT) }
            PixelFormat::R64_UINT => { VkPixelFormat(Format::R64_UINT) }
            PixelFormat::R64_SINT => { VkPixelFormat(Format::R64_SINT) }
            PixelFormat::R64_SFLOAT => { VkPixelFormat(Format::R64_SFLOAT) }
            PixelFormat::R64G64_UINT => { VkPixelFormat(Format::R64G64_UINT) }
            PixelFormat::R64G64_SINT => { VkPixelFormat(Format::R64G64_SINT) }
            PixelFormat::R64G64_SFLOAT => { VkPixelFormat(Format::R64G64_SFLOAT) }
            PixelFormat::R64G64B64_UINT => { VkPixelFormat(Format::R64G64B64_UINT) }
            PixelFormat::R64G64B64_SINT => { VkPixelFormat(Format::R64G64B64_SINT) }
            PixelFormat::R64G64B64_SFLOAT => { VkPixelFormat(Format::R64G64B64_SFLOAT) }
            PixelFormat::R64G64B64A64_UINT => { VkPixelFormat(Format::R64G64B64A64_UINT) }
            PixelFormat::R64G64B64A64_SINT => { VkPixelFormat(Format::R64G64B64A64_SINT) }
            PixelFormat::R64G64B64A64_SFLOAT => { VkPixelFormat(Format::R64G64B64A64_SFLOAT) }
            PixelFormat::B10G11R11_UFLOAT_PACK32 => { VkPixelFormat(Format::B10G11R11_UFLOAT_PACK32) }
            PixelFormat::E5B9G9R9_UFLOAT_PACK32 => { VkPixelFormat(Format::E5B9G9R9_UFLOAT_PACK32) }
            PixelFormat::D16_UNORM => { VkPixelFormat(Format::D16_UNORM) }
            PixelFormat::X8_D24_UNORM_PACK32 => { VkPixelFormat(Format::X8_D24_UNORM_PACK32) }
            PixelFormat::D32_SFLOAT => { VkPixelFormat(Format::D32_SFLOAT) }
            PixelFormat::S8_UINT => { VkPixelFormat(Format::S8_UINT) }
            PixelFormat::D16_UNORM_S8_UINT => { VkPixelFormat(Format::D16_UNORM_S8_UINT) }
            PixelFormat::D24_UNORM_S8_UINT => { VkPixelFormat(Format::D24_UNORM_S8_UINT) }
            PixelFormat::D32_SFLOAT_S8_UINT => { VkPixelFormat(Format::D32_SFLOAT_S8_UINT) }
            PixelFormat::BC1_RGB_UNORM_BLOCK => { VkPixelFormat(Format::BC1_RGB_UNORM_BLOCK) }
            PixelFormat::BC1_RGB_SRGB_BLOCK => { VkPixelFormat(Format::BC1_RGB_SRGB_BLOCK) }
            PixelFormat::BC1_RGBA_UNORM_BLOCK => { VkPixelFormat(Format::BC1_RGBA_UNORM_BLOCK) }
            PixelFormat::BC1_RGBA_SRGB_BLOCK => { VkPixelFormat(Format::BC1_RGBA_SRGB_BLOCK) }
            PixelFormat::BC2_UNORM_BLOCK => { VkPixelFormat(Format::BC2_UNORM_BLOCK) }
            PixelFormat::BC2_SRGB_BLOCK => { VkPixelFormat(Format::BC2_SRGB_BLOCK) }
            PixelFormat::BC3_UNORM_BLOCK => { VkPixelFormat(Format::BC3_UNORM_BLOCK) }
            PixelFormat::BC3_SRGB_BLOCK => { VkPixelFormat(Format::BC3_SRGB_BLOCK) }
            PixelFormat::BC4_UNORM_BLOCK => { VkPixelFormat(Format::BC4_UNORM_BLOCK) }
            PixelFormat::BC4_SNORM_BLOCK => { VkPixelFormat(Format::BC4_SNORM_BLOCK) }
            PixelFormat::BC5_UNORM_BLOCK => { VkPixelFormat(Format::BC5_UNORM_BLOCK) }
            PixelFormat::BC5_SNORM_BLOCK => { VkPixelFormat(Format::BC5_SNORM_BLOCK) }
            PixelFormat::BC6H_UFLOAT_BLOCK => { VkPixelFormat(Format::BC6H_UFLOAT_BLOCK) }
            PixelFormat::BC6H_SFLOAT_BLOCK => { VkPixelFormat(Format::BC6H_SFLOAT_BLOCK) }
            PixelFormat::BC7_UNORM_BLOCK => { VkPixelFormat(Format::BC7_UNORM_BLOCK) }
            PixelFormat::BC7_SRGB_BLOCK => { VkPixelFormat(Format::BC7_SRGB_BLOCK) }
            PixelFormat::ETC2_R8G8B8_UNORM_BLOCK => { VkPixelFormat(Format::ETC2_R8G8B8_UNORM_BLOCK) }
            PixelFormat::ETC2_R8G8B8_SRGB_BLOCK => { VkPixelFormat(Format::ETC2_R8G8B8_SRGB_BLOCK) }
            PixelFormat::ETC2_R8G8B8A1_UNORM_BLOCK => { VkPixelFormat(Format::ETC2_R8G8B8A1_UNORM_BLOCK) }
            PixelFormat::ETC2_R8G8B8A1_SRGB_BLOCK => { VkPixelFormat(Format::ETC2_R8G8B8A1_SRGB_BLOCK) }
            PixelFormat::ETC2_R8G8B8A8_UNORM_BLOCK => { VkPixelFormat(Format::ETC2_R8G8B8A8_UNORM_BLOCK) }
            PixelFormat::ETC2_R8G8B8A8_SRGB_BLOCK => { VkPixelFormat(Format::ETC2_R8G8B8A8_SRGB_BLOCK) }
            PixelFormat::EAC_R11_UNORM_BLOCK => { VkPixelFormat(Format::EAC_R11_UNORM_BLOCK) }
            PixelFormat::EAC_R11_SNORM_BLOCK => { VkPixelFormat(Format::EAC_R11_SNORM_BLOCK) }
            PixelFormat::EAC_R11G11_UNORM_BLOCK => { VkPixelFormat(Format::EAC_R11G11_UNORM_BLOCK) }
            PixelFormat::EAC_R11G11_SNORM_BLOCK => { VkPixelFormat(Format::EAC_R11G11_SNORM_BLOCK) }
            PixelFormat::ASTC_4X4_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_4X4_UNORM_BLOCK) }
            PixelFormat::ASTC_4X4_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_4X4_SRGB_BLOCK) }
            PixelFormat::ASTC_5X4_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_5X4_UNORM_BLOCK) }
            PixelFormat::ASTC_5X4_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_5X4_SRGB_BLOCK) }
            PixelFormat::ASTC_5X5_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_5X5_UNORM_BLOCK) }
            PixelFormat::ASTC_5X5_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_5X5_SRGB_BLOCK) }
            PixelFormat::ASTC_6X5_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_6X5_UNORM_BLOCK) }
            PixelFormat::ASTC_6X5_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_6X5_SRGB_BLOCK) }
            PixelFormat::ASTC_6X6_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_6X6_UNORM_BLOCK) }
            PixelFormat::ASTC_6X6_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_6X6_SRGB_BLOCK) }
            PixelFormat::ASTC_8X5_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_8X5_UNORM_BLOCK) }
            PixelFormat::ASTC_8X5_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_8X5_SRGB_BLOCK) }
            PixelFormat::ASTC_8X6_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_8X6_UNORM_BLOCK) }
            PixelFormat::ASTC_8X6_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_8X6_SRGB_BLOCK) }
            PixelFormat::ASTC_8X8_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_8X8_UNORM_BLOCK) }
            PixelFormat::ASTC_8X8_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_8X8_SRGB_BLOCK) }
            PixelFormat::ASTC_10X5_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_10X5_UNORM_BLOCK) }
            PixelFormat::ASTC_10X5_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_10X5_SRGB_BLOCK) }
            PixelFormat::ASTC_10X6_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_10X6_UNORM_BLOCK) }
            PixelFormat::ASTC_10X6_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_10X6_SRGB_BLOCK) }
            PixelFormat::ASTC_10X8_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_10X8_UNORM_BLOCK) }
            PixelFormat::ASTC_10X8_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_10X8_SRGB_BLOCK) }
            PixelFormat::ASTC_10X10_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_10X10_SRGB_BLOCK) }
            PixelFormat::ASTC_10X10_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_10X10_SRGB_BLOCK) }
            PixelFormat::ASTC_12X10_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_12X10_UNORM_BLOCK) }
            PixelFormat::ASTC_12X10_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_12X10_SRGB_BLOCK) }
            PixelFormat::ASTC_12X12_UNORM_BLOCK => { VkPixelFormat(Format::ASTC_12X12_UNORM_BLOCK) }
            PixelFormat::ASTC_12X12_SRGB_BLOCK => { VkPixelFormat(Format::ASTC_12X12_SRGB_BLOCK) }
        }
    }
}

impl From<vk::Format> for GfxPixelFormat {
    fn from(format: Format) -> Self {
        GfxPixelFormat {
            0: match format {
                Format::UNDEFINED => { PixelFormat::UNDEFINED }
                Format::R4G4_UNORM_PACK8 => { PixelFormat::R4G4_UNORM_PACK8 }
                Format::R4G4B4A4_UNORM_PACK16 => { PixelFormat::R4G4B4A4_UNORM_PACK16 }
                Format::B4G4R4A4_UNORM_PACK16 => { PixelFormat::B4G4R4A4_UNORM_PACK16 }
                Format::R5G6B5_UNORM_PACK16 => { PixelFormat::R5G6B5_UNORM_PACK16 }
                Format::B5G6R5_UNORM_PACK16 => { PixelFormat::B5G6R5_UNORM_PACK16 }
                Format::R5G5B5A1_UNORM_PACK16 => { PixelFormat::R5G5B5A1_UNORM_PACK16 }
                Format::B5G5R5A1_UNORM_PACK16 => { PixelFormat::B5G5R5A1_UNORM_PACK16 }
                Format::A1R5G5B5_UNORM_PACK16 => { PixelFormat::A1R5G5B5_UNORM_PACK16 }
                Format::R8_UNORM => { PixelFormat::R8_UNORM }
                Format::R8_SNORM => { PixelFormat::R8_SNORM }
                Format::R8_USCALED => { PixelFormat::R8_USCALED }
                Format::R8_SSCALED => { PixelFormat::R8_SSCALED }
                Format::R8_UINT => { PixelFormat::R8_UINT }
                Format::R8_SINT => { PixelFormat::R8_SINT }
                Format::R8_SRGB => { PixelFormat::R8_SRGB }
                Format::R8G8_UNORM => { PixelFormat::R8G8_UNORM }
                Format::R8G8_SNORM => { PixelFormat::R8G8_SNORM }
                Format::R8G8_USCALED => { PixelFormat::R8G8_USCALED }
                Format::R8G8_SSCALED => { PixelFormat::R8G8_SSCALED }
                Format::R8G8_UINT => { PixelFormat::R8G8_UINT }
                Format::R8G8_SINT => { PixelFormat::R8G8_SINT }
                Format::R8G8_SRGB => { PixelFormat::R8G8_SRGB }
                Format::R8G8B8_UNORM => { PixelFormat::R8G8B8_UNORM }
                Format::R8G8B8_SNORM => { PixelFormat::R8G8B8_SNORM }
                Format::R8G8B8_USCALED => { PixelFormat::R8G8B8_USCALED }
                Format::R8G8B8_SSCALED => { PixelFormat::R8G8B8_SSCALED }
                Format::R8G8B8_UINT => { PixelFormat::R8G8B8_UINT }
                Format::R8G8B8_SINT => { PixelFormat::R8G8B8_SINT }
                Format::R8G8B8_SRGB => { PixelFormat::R8G8B8_SRGB }
                Format::B8G8R8_UNORM => { PixelFormat::B8G8R8_UNORM }
                Format::B8G8R8_SNORM => { PixelFormat::B8G8R8_SNORM }
                Format::B8G8R8_USCALED => { PixelFormat::B8G8R8_USCALED }
                Format::B8G8R8_SSCALED => { PixelFormat::B8G8R8_SSCALED }
                Format::B8G8R8_UINT => { PixelFormat::B8G8R8_UINT }
                Format::B8G8R8_SINT => { PixelFormat::B8G8R8_SINT }
                Format::B8G8R8_SRGB => { PixelFormat::B8G8R8_SRGB }
                Format::R8G8B8A8_UNORM => { PixelFormat::R8G8B8A8_UNORM }
                Format::R8G8B8A8_SNORM => { PixelFormat::R8G8B8A8_SNORM }
                Format::R8G8B8A8_USCALED => { PixelFormat::R8G8B8A8_USCALED }
                Format::R8G8B8A8_SSCALED => { PixelFormat::R8G8B8A8_SSCALED }
                Format::R8G8B8A8_UINT => { PixelFormat::R8G8B8A8_UINT }
                Format::R8G8B8A8_SINT => { PixelFormat::R8G8B8A8_SINT }
                Format::R8G8B8A8_SRGB => { PixelFormat::R8G8B8A8_SRGB }
                Format::B8G8R8A8_UNORM => { PixelFormat::B8G8R8A8_UNORM }
                Format::B8G8R8A8_SNORM => { PixelFormat::B8G8R8A8_SNORM }
                Format::B8G8R8A8_USCALED => { PixelFormat::B8G8R8A8_USCALED }
                Format::B8G8R8A8_SSCALED => { PixelFormat::B8G8R8A8_SSCALED }
                Format::B8G8R8A8_UINT => { PixelFormat::B8G8R8A8_UINT }
                Format::B8G8R8A8_SINT => { PixelFormat::B8G8R8A8_SINT }
                Format::B8G8R8A8_SRGB => { PixelFormat::B8G8R8A8_SRGB }
                Format::A8B8G8R8_UNORM_PACK32 => { PixelFormat::A8B8G8R8_UNORM_PACK32 }
                Format::A8B8G8R8_SNORM_PACK32 => { PixelFormat::A8B8G8R8_SNORM_PACK32 }
                Format::A8B8G8R8_USCALED_PACK32 => { PixelFormat::A8B8G8R8_USCALED_PACK32 }
                Format::A8B8G8R8_SSCALED_PACK32 => { PixelFormat::A8B8G8R8_SSCALED_PACK32 }
                Format::A8B8G8R8_UINT_PACK32 => { PixelFormat::A8B8G8R8_UINT_PACK32 }
                Format::A8B8G8R8_SINT_PACK32 => { PixelFormat::A8B8G8R8_SINT_PACK32 }
                Format::A8B8G8R8_SRGB_PACK32 => { PixelFormat::A8B8G8R8_SRGB_PACK32 }
                Format::A2R10G10B10_UNORM_PACK32 => { PixelFormat::A2R10G10B10_UNORM_PACK32 }
                Format::A2R10G10B10_SNORM_PACK32 => { PixelFormat::A2R10G10B10_SNORM_PACK32 }
                Format::A2R10G10B10_USCALED_PACK32 => { PixelFormat::A2R10G10B10_USCALED_PACK32 }
                Format::A2R10G10B10_SSCALED_PACK32 => { PixelFormat::A2R10G10B10_SSCALED_PACK32 }
                Format::A2R10G10B10_UINT_PACK32 => { PixelFormat::A2R10G10B10_UINT_PACK32 }
                Format::A2R10G10B10_SINT_PACK32 => { PixelFormat::A2R10G10B10_SINT_PACK32 }
                Format::A2B10G10R10_UNORM_PACK32 => { PixelFormat::A2B10G10R10_UNORM_PACK32 }
                Format::A2B10G10R10_SNORM_PACK32 => { PixelFormat::A2B10G10R10_SNORM_PACK32 }
                Format::A2B10G10R10_USCALED_PACK32 => { PixelFormat::A2B10G10R10_USCALED_PACK32 }
                Format::A2B10G10R10_SSCALED_PACK32 => { PixelFormat::A2B10G10R10_SSCALED_PACK32 }
                Format::A2B10G10R10_UINT_PACK32 => { PixelFormat::A2B10G10R10_UINT_PACK32 }
                Format::A2B10G10R10_SINT_PACK32 => { PixelFormat::A2B10G10R10_SINT_PACK32 }
                Format::R16_UNORM => { PixelFormat::R16_UNORM }
                Format::R16_SNORM => { PixelFormat::R16_SNORM }
                Format::R16_USCALED => { PixelFormat::R16_USCALED }
                Format::R16_SSCALED => { PixelFormat::R16_SSCALED }
                Format::R16_UINT => { PixelFormat::R16_UINT }
                Format::R16_SINT => { PixelFormat::R16_SINT }
                Format::R16_SFLOAT => { PixelFormat::R16_SFLOAT }
                Format::R16G16_UNORM => { PixelFormat::R16G16_UNORM }
                Format::R16G16_SNORM => { PixelFormat::R16G16_SNORM }
                Format::R16G16_USCALED => { PixelFormat::R16G16_USCALED }
                Format::R16G16_SSCALED => { PixelFormat::R16G16_SSCALED }
                Format::R16G16_UINT => { PixelFormat::R16G16_UINT }
                Format::R16G16_SINT => { PixelFormat::R16G16_SINT }
                Format::R16G16_SFLOAT => { PixelFormat::R16G16_SFLOAT }
                Format::R16G16B16_UNORM => { PixelFormat::R16G16B16_UNORM }
                Format::R16G16B16_SNORM => { PixelFormat::R16G16B16_SNORM }
                Format::R16G16B16_USCALED => { PixelFormat::R16G16B16_USCALED }
                Format::R16G16B16_SSCALED => { PixelFormat::R16G16B16_SSCALED }
                Format::R16G16B16_UINT => { PixelFormat::R16G16B16_UINT }
                Format::R16G16B16_SINT => { PixelFormat::R16G16B16_SINT }
                Format::R16G16B16_SFLOAT => { PixelFormat::R16G16B16_SFLOAT }
                Format::R16G16B16A16_UNORM => { PixelFormat::R16G16B16A16_UNORM }
                Format::R16G16B16A16_SNORM => { PixelFormat::R16G16B16A16_SNORM }
                Format::R16G16B16A16_USCALED => { PixelFormat::R16G16B16A16_USCALED }
                Format::R16G16B16A16_SSCALED => { PixelFormat::R16G16B16A16_SSCALED }
                Format::R16G16B16A16_UINT => { PixelFormat::R16G16B16A16_UINT }
                Format::R16G16B16A16_SINT => { PixelFormat::R16G16B16A16_SINT }
                Format::R16G16B16A16_SFLOAT => { PixelFormat::R16G16B16A16_SFLOAT }
                Format::R32_UINT => { PixelFormat::R32_UINT }
                Format::R32_SINT => { PixelFormat::R32_SINT }
                Format::R32_SFLOAT => { PixelFormat::R32_SFLOAT }
                Format::R32G32_UINT => { PixelFormat::R32G32_UINT }
                Format::R32G32_SINT => { PixelFormat::R32G32_SINT }
                Format::R32G32_SFLOAT => { PixelFormat::R32G32_SFLOAT }
                Format::R32G32B32_UINT => { PixelFormat::R32G32B32_UINT }
                Format::R32G32B32_SINT => { PixelFormat::R32G32B32_SINT }
                Format::R32G32B32_SFLOAT => { PixelFormat::R32G32B32_SFLOAT }
                Format::R32G32B32A32_UINT => { PixelFormat::R32G32B32A32_UINT }
                Format::R32G32B32A32_SINT => { PixelFormat::R32G32B32A32_SINT }
                Format::R32G32B32A32_SFLOAT => { PixelFormat::R32G32B32A32_SFLOAT }
                Format::R64_UINT => { PixelFormat::R64_UINT }
                Format::R64_SINT => { PixelFormat::R64_SINT }
                Format::R64_SFLOAT => { PixelFormat::R64_SFLOAT }
                Format::R64G64_UINT => { PixelFormat::R64G64_UINT }
                Format::R64G64_SINT => { PixelFormat::R64G64_SINT }
                Format::R64G64_SFLOAT => { PixelFormat::R64G64_SFLOAT }
                Format::R64G64B64_UINT => { PixelFormat::R64G64B64_UINT }
                Format::R64G64B64_SINT => { PixelFormat::R64G64B64_SINT }
                Format::R64G64B64_SFLOAT => { PixelFormat::R64G64B64_SFLOAT }
                Format::R64G64B64A64_UINT => { PixelFormat::R64G64B64A64_UINT }
                Format::R64G64B64A64_SINT => { PixelFormat::R64G64B64A64_SINT }
                Format::R64G64B64A64_SFLOAT => { PixelFormat::R64G64B64A64_SFLOAT }
                Format::B10G11R11_UFLOAT_PACK32 => { PixelFormat::B10G11R11_UFLOAT_PACK32 }
                Format::E5B9G9R9_UFLOAT_PACK32 => { PixelFormat::E5B9G9R9_UFLOAT_PACK32 }
                Format::D16_UNORM => { PixelFormat::D16_UNORM }
                Format::X8_D24_UNORM_PACK32 => { PixelFormat::X8_D24_UNORM_PACK32 }
                Format::D32_SFLOAT => { PixelFormat::D32_SFLOAT }
                Format::S8_UINT => { PixelFormat::S8_UINT }
                Format::D16_UNORM_S8_UINT => { PixelFormat::D16_UNORM_S8_UINT }
                Format::D24_UNORM_S8_UINT => { PixelFormat::D24_UNORM_S8_UINT }
                Format::D32_SFLOAT_S8_UINT => { PixelFormat::D32_SFLOAT_S8_UINT }
                Format::BC1_RGB_UNORM_BLOCK => { PixelFormat::BC1_RGB_UNORM_BLOCK }
                Format::BC1_RGB_SRGB_BLOCK => { PixelFormat::BC1_RGB_SRGB_BLOCK }
                Format::BC1_RGBA_UNORM_BLOCK => { PixelFormat::BC1_RGBA_UNORM_BLOCK }
                Format::BC1_RGBA_SRGB_BLOCK => { PixelFormat::BC1_RGBA_SRGB_BLOCK }
                Format::BC2_UNORM_BLOCK => { PixelFormat::BC2_UNORM_BLOCK }
                Format::BC2_SRGB_BLOCK => { PixelFormat::BC2_SRGB_BLOCK }
                Format::BC3_UNORM_BLOCK => { PixelFormat::BC3_UNORM_BLOCK }
                Format::BC3_SRGB_BLOCK => { PixelFormat::BC3_SRGB_BLOCK }
                Format::BC4_UNORM_BLOCK => { PixelFormat::BC4_UNORM_BLOCK }
                Format::BC4_SNORM_BLOCK => { PixelFormat::BC4_SNORM_BLOCK }
                Format::BC5_UNORM_BLOCK => { PixelFormat::BC5_UNORM_BLOCK }
                Format::BC5_SNORM_BLOCK => { PixelFormat::BC5_SNORM_BLOCK }
                Format::BC6H_UFLOAT_BLOCK => { PixelFormat::BC6H_UFLOAT_BLOCK }
                Format::BC6H_SFLOAT_BLOCK => { PixelFormat::BC6H_SFLOAT_BLOCK }
                Format::BC7_UNORM_BLOCK => { PixelFormat::BC7_UNORM_BLOCK }
                Format::BC7_SRGB_BLOCK => { PixelFormat::BC7_SRGB_BLOCK }
                Format::ETC2_R8G8B8_UNORM_BLOCK => { PixelFormat::ETC2_R8G8B8_UNORM_BLOCK }
                Format::ETC2_R8G8B8_SRGB_BLOCK => { PixelFormat::ETC2_R8G8B8_SRGB_BLOCK }
                Format::ETC2_R8G8B8A1_UNORM_BLOCK => { PixelFormat::ETC2_R8G8B8A1_UNORM_BLOCK }
                Format::ETC2_R8G8B8A1_SRGB_BLOCK => { PixelFormat::ETC2_R8G8B8A1_SRGB_BLOCK }
                Format::ETC2_R8G8B8A8_UNORM_BLOCK => { PixelFormat::ETC2_R8G8B8A8_UNORM_BLOCK }
                Format::ETC2_R8G8B8A8_SRGB_BLOCK => { PixelFormat::ETC2_R8G8B8A8_SRGB_BLOCK }
                Format::EAC_R11_UNORM_BLOCK => { PixelFormat::EAC_R11_UNORM_BLOCK }
                Format::EAC_R11_SNORM_BLOCK => { PixelFormat::EAC_R11_SNORM_BLOCK }
                Format::EAC_R11G11_UNORM_BLOCK => { PixelFormat::EAC_R11G11_UNORM_BLOCK }
                Format::EAC_R11G11_SNORM_BLOCK => { PixelFormat::EAC_R11G11_SNORM_BLOCK }
                Format::ASTC_4X4_UNORM_BLOCK => { PixelFormat::ASTC_4X4_UNORM_BLOCK }
                Format::ASTC_4X4_SRGB_BLOCK => { PixelFormat::ASTC_4X4_SRGB_BLOCK }
                Format::ASTC_5X4_UNORM_BLOCK => { PixelFormat::ASTC_5X4_UNORM_BLOCK }
                Format::ASTC_5X4_SRGB_BLOCK => { PixelFormat::ASTC_5X4_SRGB_BLOCK }
                Format::ASTC_5X5_UNORM_BLOCK => { PixelFormat::ASTC_5X5_UNORM_BLOCK }
                Format::ASTC_5X5_SRGB_BLOCK => { PixelFormat::ASTC_5X5_SRGB_BLOCK }
                Format::ASTC_6X5_UNORM_BLOCK => { PixelFormat::ASTC_6X5_UNORM_BLOCK }
                Format::ASTC_6X5_SRGB_BLOCK => { PixelFormat::ASTC_6X5_SRGB_BLOCK }
                Format::ASTC_6X6_UNORM_BLOCK => { PixelFormat::ASTC_6X6_UNORM_BLOCK }
                Format::ASTC_6X6_SRGB_BLOCK => { PixelFormat::ASTC_6X6_SRGB_BLOCK }
                Format::ASTC_8X5_UNORM_BLOCK => { PixelFormat::ASTC_8X5_UNORM_BLOCK }
                Format::ASTC_8X5_SRGB_BLOCK => { PixelFormat::ASTC_8X5_SRGB_BLOCK }
                Format::ASTC_8X6_UNORM_BLOCK => { PixelFormat::ASTC_8X6_UNORM_BLOCK }
                Format::ASTC_8X6_SRGB_BLOCK => { PixelFormat::ASTC_8X6_SRGB_BLOCK }
                Format::ASTC_8X8_UNORM_BLOCK => { PixelFormat::ASTC_8X8_UNORM_BLOCK }
                Format::ASTC_8X8_SRGB_BLOCK => { PixelFormat::ASTC_8X8_SRGB_BLOCK }
                Format::ASTC_10X5_UNORM_BLOCK => { PixelFormat::ASTC_10X5_UNORM_BLOCK }
                Format::ASTC_10X5_SRGB_BLOCK => { PixelFormat::ASTC_10X5_SRGB_BLOCK }
                Format::ASTC_10X6_UNORM_BLOCK => { PixelFormat::ASTC_10X6_UNORM_BLOCK }
                Format::ASTC_10X6_SRGB_BLOCK => { PixelFormat::ASTC_10X6_SRGB_BLOCK }
                Format::ASTC_10X8_UNORM_BLOCK => { PixelFormat::ASTC_10X8_UNORM_BLOCK }
                Format::ASTC_10X8_SRGB_BLOCK => { PixelFormat::ASTC_10X8_SRGB_BLOCK }
                Format::ASTC_10X10_SRGB_BLOCK => { PixelFormat::ASTC_10X10_UNORM_BLOCK }
                Format::ASTC_12X10_UNORM_BLOCK => { PixelFormat::ASTC_12X10_UNORM_BLOCK }
                Format::ASTC_12X10_SRGB_BLOCK => { PixelFormat::ASTC_12X10_SRGB_BLOCK }
                Format::ASTC_12X12_UNORM_BLOCK => { PixelFormat::ASTC_12X12_UNORM_BLOCK }
                Format::ASTC_12X12_SRGB_BLOCK => { PixelFormat::ASTC_12X12_SRGB_BLOCK }
                _ => { PixelFormat::UNDEFINED }
            }
        }
    }
}

pub struct VkColorSpace(ColorSpaceKHR);

impl Deref for VkColorSpace {
    type Target = ColorSpaceKHR;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ColorSpace> for VkColorSpace {
    fn from(color_space: ColorSpace) -> Self {
        VkColorSpace {
            0: match color_space {
                ColorSpace::SRGB_NONLINEAR => { ColorSpaceKHR::SRGB_NONLINEAR }
                ColorSpace::DISPLAY_P3_NONLINEAR => { ColorSpaceKHR::DISPLAY_P3_NONLINEAR_EXT }
                ColorSpace::EXTENDED_SRGB_LINEAR => { ColorSpaceKHR::EXTENDED_SRGB_LINEAR_EXT }
                ColorSpace::DISPLAY_P3_LINEAR => { ColorSpaceKHR::DISPLAY_P3_LINEAR_EXT }
                ColorSpace::DCI_P3_NONLINEAR => { ColorSpaceKHR::DCI_P3_NONLINEAR_EXT }
                ColorSpace::BT709_LINEAR => { ColorSpaceKHR::BT709_LINEAR_EXT }
                ColorSpace::BT709_NONLINEAR => { ColorSpaceKHR::BT709_NONLINEAR_EXT }
                ColorSpace::BT2020_LINEAR => { ColorSpaceKHR::BT2020_LINEAR_EXT }
                ColorSpace::HDR10_ST2084 => { ColorSpaceKHR::HDR10_ST2084_EXT }
                ColorSpace::DOLBYVISION => { ColorSpaceKHR::DOLBYVISION_EXT }
                ColorSpace::HDR10_HLG => { ColorSpaceKHR::HDR10_HLG_EXT }
                ColorSpace::ADOBERGB_LINEAR => { ColorSpaceKHR::ADOBERGB_LINEAR_EXT }
                ColorSpace::ADOBERGB_NONLINEAR => { ColorSpaceKHR::ADOBERGB_NONLINEAR_EXT }
                ColorSpace::PASS_THROUGH => { ColorSpaceKHR::PASS_THROUGH_EXT }
                ColorSpace::EXTENDED_SRGB_NONLINEAR => { ColorSpaceKHR::EXTENDED_SRGB_NONLINEAR_EXT }
                ColorSpace::DISPLAY_NATIVE => { ColorSpaceKHR::DISPLAY_NATIVE_AMD }
                ColorSpace::DCI_P3_LINEAR => { ColorSpaceKHR::DCI_P3_NONLINEAR_EXT }
            }
        }
    }
}