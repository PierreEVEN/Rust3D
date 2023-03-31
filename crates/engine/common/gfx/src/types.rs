use std::any::Any;
use maths::vec2::Vec2f32;
use maths::vec4::Vec4F32;

#[allow(non_camel_case_types)]
pub enum ColorSpace {
    SRGB_NONLINEAR,
    DISPLAY_P3_NONLINEAR,
    EXTENDED_SRGB_LINEAR,
    DISPLAY_P3_LINEAR,
    DCI_P3_NONLINEAR,
    BT709_LINEAR,
    BT709_NONLINEAR,
    BT2020_LINEAR,
    HDR10_ST2084,
    DOLBYVISION,
    HDR10_HLG,
    ADOBERGB_LINEAR,
    ADOBERGB_NONLINEAR,
    PASS_THROUGH,
    EXTENDED_SRGB_NONLINEAR,
    DISPLAY_NATIVE,
    DCI_P3_LINEAR,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum PixelFormat {
    UNDEFINED,
    R4G4_UNORM_PACK8,
    R4G4B4A4_UNORM_PACK16,
    B4G4R4A4_UNORM_PACK16,
    R5G6B5_UNORM_PACK16,
    B5G6R5_UNORM_PACK16,
    R5G5B5A1_UNORM_PACK16,
    B5G5R5A1_UNORM_PACK16,
    A1R5G5B5_UNORM_PACK16,
    R8_UNORM,
    R8_SNORM,
    R8_USCALED,
    R8_SSCALED,
    R8_UINT,
    R8_SINT,
    R8_SRGB,
    R8G8_UNORM,
    R8G8_SNORM,
    R8G8_USCALED,
    R8G8_SSCALED,
    R8G8_UINT,
    R8G8_SINT,
    R8G8_SRGB,
    R8G8B8_UNORM,
    R8G8B8_SNORM,
    R8G8B8_USCALED,
    R8G8B8_SSCALED,
    R8G8B8_UINT,
    R8G8B8_SINT,
    R8G8B8_SRGB,
    B8G8R8_UNORM,
    B8G8R8_SNORM,
    B8G8R8_USCALED,
    B8G8R8_SSCALED,
    B8G8R8_UINT,
    B8G8R8_SINT,
    B8G8R8_SRGB,
    R8G8B8A8_UNORM,
    R8G8B8A8_SNORM,
    R8G8B8A8_USCALED,
    R8G8B8A8_SSCALED,
    R8G8B8A8_UINT,
    R8G8B8A8_SINT,
    R8G8B8A8_SRGB,
    B8G8R8A8_UNORM,
    B8G8R8A8_SNORM,
    B8G8R8A8_USCALED,
    B8G8R8A8_SSCALED,
    B8G8R8A8_UINT,
    B8G8R8A8_SINT,
    B8G8R8A8_SRGB,
    A8B8G8R8_UNORM_PACK32,
    A8B8G8R8_SNORM_PACK32,
    A8B8G8R8_USCALED_PACK32,
    A8B8G8R8_SSCALED_PACK32,
    A8B8G8R8_UINT_PACK32,
    A8B8G8R8_SINT_PACK32,
    A8B8G8R8_SRGB_PACK32,
    A2R10G10B10_UNORM_PACK32,
    A2R10G10B10_SNORM_PACK32,
    A2R10G10B10_USCALED_PACK32,
    A2R10G10B10_SSCALED_PACK32,
    A2R10G10B10_UINT_PACK32,
    A2R10G10B10_SINT_PACK32,
    A2B10G10R10_UNORM_PACK32,
    A2B10G10R10_SNORM_PACK32,
    A2B10G10R10_USCALED_PACK32,
    A2B10G10R10_SSCALED_PACK32,
    A2B10G10R10_UINT_PACK32,
    A2B10G10R10_SINT_PACK32,
    R16_UNORM,
    R16_SNORM,
    R16_USCALED,
    R16_SSCALED,
    R16_UINT,
    R16_SINT,
    R16_SFLOAT,
    R16G16_UNORM,
    R16G16_SNORM,
    R16G16_USCALED,
    R16G16_SSCALED,
    R16G16_UINT,
    R16G16_SINT,
    R16G16_SFLOAT,
    R16G16B16_UNORM,
    R16G16B16_SNORM,
    R16G16B16_USCALED,
    R16G16B16_SSCALED,
    R16G16B16_UINT,
    R16G16B16_SINT,
    R16G16B16_SFLOAT,
    R16G16B16A16_UNORM,
    R16G16B16A16_SNORM,
    R16G16B16A16_USCALED,
    R16G16B16A16_SSCALED,
    R16G16B16A16_UINT,
    R16G16B16A16_SINT,
    R16G16B16A16_SFLOAT,
    R32_UINT,
    R32_SINT,
    R32_SFLOAT,
    R32G32_UINT,
    R32G32_SINT,
    R32G32_SFLOAT,
    R32G32B32_UINT,
    R32G32B32_SINT,
    R32G32B32_SFLOAT,
    R32G32B32A32_UINT,
    R32G32B32A32_SINT,
    R32G32B32A32_SFLOAT,
    R64_UINT,
    R64_SINT,
    R64_SFLOAT,
    R64G64_UINT,
    R64G64_SINT,
    R64G64_SFLOAT,
    R64G64B64_UINT,
    R64G64B64_SINT,
    R64G64B64_SFLOAT,
    R64G64B64A64_UINT,
    R64G64B64A64_SINT,
    R64G64B64A64_SFLOAT,
    B10G11R11_UFLOAT_PACK32,
    E5B9G9R9_UFLOAT_PACK32,
    D16_UNORM,
    X8_D24_UNORM_PACK32,
    D32_SFLOAT,
    S8_UINT,
    D16_UNORM_S8_UINT,
    D24_UNORM_S8_UINT,
    D32_SFLOAT_S8_UINT,
    BC1_RGB_UNORM_BLOCK,
    BC1_RGB_SRGB_BLOCK,
    BC1_RGBA_UNORM_BLOCK,
    BC1_RGBA_SRGB_BLOCK,
    BC2_UNORM_BLOCK,
    BC2_SRGB_BLOCK,
    BC3_UNORM_BLOCK,
    BC3_SRGB_BLOCK,
    BC4_UNORM_BLOCK,
    BC4_SNORM_BLOCK,
    BC5_UNORM_BLOCK,
    BC5_SNORM_BLOCK,
    BC6H_UFLOAT_BLOCK,
    BC6H_SFLOAT_BLOCK,
    BC7_UNORM_BLOCK,
    BC7_SRGB_BLOCK,
    ETC2_R8G8B8_UNORM_BLOCK,
    ETC2_R8G8B8_SRGB_BLOCK,
    ETC2_R8G8B8A1_UNORM_BLOCK,
    ETC2_R8G8B8A1_SRGB_BLOCK,
    ETC2_R8G8B8A8_UNORM_BLOCK,
    ETC2_R8G8B8A8_SRGB_BLOCK,
    EAC_R11_UNORM_BLOCK,
    EAC_R11_SNORM_BLOCK,
    EAC_R11G11_UNORM_BLOCK,
    EAC_R11G11_SNORM_BLOCK,
    ASTC_4X4_UNORM_BLOCK,
    ASTC_4X4_SRGB_BLOCK,
    ASTC_5X4_UNORM_BLOCK,
    ASTC_5X4_SRGB_BLOCK,
    ASTC_5X5_UNORM_BLOCK,
    ASTC_5X5_SRGB_BLOCK,
    ASTC_6X5_UNORM_BLOCK,
    ASTC_6X5_SRGB_BLOCK,
    ASTC_6X6_UNORM_BLOCK,
    ASTC_6X6_SRGB_BLOCK,
    ASTC_8X5_UNORM_BLOCK,
    ASTC_8X5_SRGB_BLOCK,
    ASTC_8X6_UNORM_BLOCK,
    ASTC_8X6_SRGB_BLOCK,
    ASTC_8X8_UNORM_BLOCK,
    ASTC_8X8_SRGB_BLOCK,
    ASTC_10X5_UNORM_BLOCK,
    ASTC_10X5_SRGB_BLOCK,
    ASTC_10X6_UNORM_BLOCK,
    ASTC_10X6_SRGB_BLOCK,
    ASTC_10X8_UNORM_BLOCK,
    ASTC_10X8_SRGB_BLOCK,
    ASTC_10X10_UNORM_BLOCK,
    ASTC_10X10_SRGB_BLOCK,
    ASTC_12X10_UNORM_BLOCK,
    ASTC_12X10_SRGB_BLOCK,
    ASTC_12X12_UNORM_BLOCK,
    ASTC_12X12_SRGB_BLOCK,
}


impl PixelFormat {
    pub fn type_size(&self) -> u32 {
        match &self {
            PixelFormat::UNDEFINED => {
                panic!("not available")
            }
            PixelFormat::R4G4_UNORM_PACK8 => { 1 }
            PixelFormat::R4G4B4A4_UNORM_PACK16 => { 2 }
            PixelFormat::B4G4R4A4_UNORM_PACK16 => { 2 }
            PixelFormat::R5G6B5_UNORM_PACK16 => { 2 }
            PixelFormat::B5G6R5_UNORM_PACK16 => { 2 }
            PixelFormat::R5G5B5A1_UNORM_PACK16 => { 2 }
            PixelFormat::B5G5R5A1_UNORM_PACK16 => { 2 }
            PixelFormat::A1R5G5B5_UNORM_PACK16 => { 2 }
            PixelFormat::R8_UNORM => { 1 }
            PixelFormat::R8_SNORM => { 1 }
            PixelFormat::R8_USCALED => { 1 }
            PixelFormat::R8_SSCALED => { 1 }
            PixelFormat::R8_UINT => { 1 }
            PixelFormat::R8_SINT => { 1 }
            PixelFormat::R8_SRGB => { 1 }
            PixelFormat::R8G8_UNORM => { 2 }
            PixelFormat::R8G8_SNORM => { 2 }
            PixelFormat::R8G8_USCALED => { 2 }
            PixelFormat::R8G8_SSCALED => { 2 }
            PixelFormat::R8G8_UINT => { 2 }
            PixelFormat::R8G8_SINT => { 2 }
            PixelFormat::R8G8_SRGB => { 2 }
            PixelFormat::R8G8B8_UNORM => { 3 }
            PixelFormat::R8G8B8_SNORM => { 3 }
            PixelFormat::R8G8B8_USCALED => { 3 }
            PixelFormat::R8G8B8_SSCALED => { 3 }
            PixelFormat::R8G8B8_UINT => { 3 }
            PixelFormat::R8G8B8_SINT => { 3 }
            PixelFormat::R8G8B8_SRGB => { 3 }
            PixelFormat::B8G8R8_UNORM => { 3 }
            PixelFormat::B8G8R8_SNORM => { 3 }
            PixelFormat::B8G8R8_USCALED => { 3 }
            PixelFormat::B8G8R8_SSCALED => { 3 }
            PixelFormat::B8G8R8_UINT => { 3 }
            PixelFormat::B8G8R8_SINT => { 3 }
            PixelFormat::B8G8R8_SRGB => { 3 }
            PixelFormat::R8G8B8A8_UNORM => { 4 }
            PixelFormat::R8G8B8A8_SNORM => { 4 }
            PixelFormat::R8G8B8A8_USCALED => { 4 }
            PixelFormat::R8G8B8A8_SSCALED => { 4 }
            PixelFormat::R8G8B8A8_UINT => { 4 }
            PixelFormat::R8G8B8A8_SINT => { 4 }
            PixelFormat::R8G8B8A8_SRGB => { 4 }
            PixelFormat::B8G8R8A8_UNORM => { 4 }
            PixelFormat::B8G8R8A8_SNORM => { 4 }
            PixelFormat::B8G8R8A8_USCALED => { 4 }
            PixelFormat::B8G8R8A8_SSCALED => { 4 }
            PixelFormat::B8G8R8A8_UINT => { 4 }
            PixelFormat::B8G8R8A8_SINT => { 4 }
            PixelFormat::B8G8R8A8_SRGB => { 4 }
            PixelFormat::A8B8G8R8_UNORM_PACK32 => { 4 }
            PixelFormat::A8B8G8R8_SNORM_PACK32 => { 4 }
            PixelFormat::A8B8G8R8_USCALED_PACK32 => { 4 }
            PixelFormat::A8B8G8R8_SSCALED_PACK32 => { 4 }
            PixelFormat::A8B8G8R8_UINT_PACK32 => { 4 }
            PixelFormat::A8B8G8R8_SINT_PACK32 => { 4 }
            PixelFormat::A8B8G8R8_SRGB_PACK32 => { 4 }
            PixelFormat::A2R10G10B10_UNORM_PACK32 => { 4 }
            PixelFormat::A2R10G10B10_SNORM_PACK32 => { 4 }
            PixelFormat::A2R10G10B10_USCALED_PACK32 => { 4 }
            PixelFormat::A2R10G10B10_SSCALED_PACK32 => { 4 }
            PixelFormat::A2R10G10B10_UINT_PACK32 => { 4 }
            PixelFormat::A2R10G10B10_SINT_PACK32 => { 4 }
            PixelFormat::A2B10G10R10_UNORM_PACK32 => { 4 }
            PixelFormat::A2B10G10R10_SNORM_PACK32 => { 4 }
            PixelFormat::A2B10G10R10_USCALED_PACK32 => { 4 }
            PixelFormat::A2B10G10R10_SSCALED_PACK32 => { 4 }
            PixelFormat::A2B10G10R10_UINT_PACK32 => { 4 }
            PixelFormat::A2B10G10R10_SINT_PACK32 => { 4 }
            PixelFormat::R16_UNORM => { 2 }
            PixelFormat::R16_SNORM => { 2 }
            PixelFormat::R16_USCALED => { 2 }
            PixelFormat::R16_SSCALED => { 2 }
            PixelFormat::R16_UINT => { 2 }
            PixelFormat::R16_SINT => { 2 }
            PixelFormat::R16_SFLOAT => { 2 }
            PixelFormat::R16G16_UNORM => { 4 }
            PixelFormat::R16G16_SNORM => { 4 }
            PixelFormat::R16G16_USCALED => { 4 }
            PixelFormat::R16G16_SSCALED => { 4 }
            PixelFormat::R16G16_UINT => { 4 }
            PixelFormat::R16G16_SINT => { 4 }
            PixelFormat::R16G16_SFLOAT => { 4 }
            PixelFormat::R16G16B16_UNORM => { 6 }
            PixelFormat::R16G16B16_SNORM =>  { 6 }
            PixelFormat::R16G16B16_USCALED =>  { 6 }
            PixelFormat::R16G16B16_SSCALED =>  { 6 }
            PixelFormat::R16G16B16_UINT =>  { 6 }
            PixelFormat::R16G16B16_SINT =>  { 6 }
            PixelFormat::R16G16B16_SFLOAT =>  { 6 }
            PixelFormat::R16G16B16A16_UNORM => { 8 }
            PixelFormat::R16G16B16A16_SNORM => { 8 }
            PixelFormat::R16G16B16A16_USCALED => { 8 }
            PixelFormat::R16G16B16A16_SSCALED => { 8 }
            PixelFormat::R16G16B16A16_UINT => { 8 }
            PixelFormat::R16G16B16A16_SINT => { 8 }
            PixelFormat::R16G16B16A16_SFLOAT => { 8 }
            PixelFormat::R32_UINT => { 4 }
            PixelFormat::R32_SINT => { 4 }
            PixelFormat::R32_SFLOAT => { 4 }
            PixelFormat::R32G32_UINT => { 8 }
            PixelFormat::R32G32_SINT => { 8 }
            PixelFormat::R32G32_SFLOAT => { 8 }
            PixelFormat::R32G32B32_UINT => { 12 }
            PixelFormat::R32G32B32_SINT => { 12 }
            PixelFormat::R32G32B32_SFLOAT => { 12 }
            PixelFormat::R32G32B32A32_UINT => { 16 }
            PixelFormat::R32G32B32A32_SINT => { 16 }
            PixelFormat::R32G32B32A32_SFLOAT => { 16 }
            PixelFormat::R64_UINT => { 8 }
            PixelFormat::R64_SINT => { 8 }
            PixelFormat::R64_SFLOAT => { 8 }
            PixelFormat::R64G64_UINT => { 16 }
            PixelFormat::R64G64_SINT => { 16 }
            PixelFormat::R64G64_SFLOAT => { 16 }
            PixelFormat::R64G64B64_UINT => { 24 }
            PixelFormat::R64G64B64_SINT => { 24 }
            PixelFormat::R64G64B64_SFLOAT => { 24 }
            PixelFormat::R64G64B64A64_UINT => { 32 }
            PixelFormat::R64G64B64A64_SINT => { 32 }
            PixelFormat::R64G64B64A64_SFLOAT => { 32 }
            PixelFormat::B10G11R11_UFLOAT_PACK32 => { 4 }
            PixelFormat::E5B9G9R9_UFLOAT_PACK32 => { 4 }
            PixelFormat::D16_UNORM => { 2 }
            PixelFormat::X8_D24_UNORM_PACK32 => { 4 }
            PixelFormat::D32_SFLOAT => { 4 }
            PixelFormat::S8_UINT => { 1 }
            PixelFormat::D16_UNORM_S8_UINT => { 3 }
            PixelFormat::D24_UNORM_S8_UINT => { 4 }
            PixelFormat::D32_SFLOAT_S8_UINT => { 5 }
            PixelFormat::BC1_RGB_UNORM_BLOCK => { todo!() }
            PixelFormat::BC1_RGB_SRGB_BLOCK => { todo!() }
            PixelFormat::BC1_RGBA_UNORM_BLOCK => { todo!() }
            PixelFormat::BC1_RGBA_SRGB_BLOCK => { todo!() }
            PixelFormat::BC2_UNORM_BLOCK => { todo!() }
            PixelFormat::BC2_SRGB_BLOCK => { todo!() }
            PixelFormat::BC3_UNORM_BLOCK => { todo!() }
            PixelFormat::BC3_SRGB_BLOCK => { todo!() }
            PixelFormat::BC4_UNORM_BLOCK => { todo!() }
            PixelFormat::BC4_SNORM_BLOCK => { todo!() }
            PixelFormat::BC5_UNORM_BLOCK => { todo!() }
            PixelFormat::BC5_SNORM_BLOCK => { todo!() }
            PixelFormat::BC6H_UFLOAT_BLOCK => { todo!() }
            PixelFormat::BC6H_SFLOAT_BLOCK => { todo!() }
            PixelFormat::BC7_UNORM_BLOCK => { todo!() }
            PixelFormat::BC7_SRGB_BLOCK => { todo!() }
            PixelFormat::ETC2_R8G8B8_UNORM_BLOCK => { todo!() }
            PixelFormat::ETC2_R8G8B8_SRGB_BLOCK => { todo!() }
            PixelFormat::ETC2_R8G8B8A1_UNORM_BLOCK => { todo!() }
            PixelFormat::ETC2_R8G8B8A1_SRGB_BLOCK => { todo!() }
            PixelFormat::ETC2_R8G8B8A8_UNORM_BLOCK => { todo!() }
            PixelFormat::ETC2_R8G8B8A8_SRGB_BLOCK => { todo!() }
            PixelFormat::EAC_R11_UNORM_BLOCK => { todo!() }
            PixelFormat::EAC_R11_SNORM_BLOCK => { todo!() }
            PixelFormat::EAC_R11G11_UNORM_BLOCK => { todo!() }
            PixelFormat::EAC_R11G11_SNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_4X4_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_4X4_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_5X4_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_5X4_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_5X5_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_5X5_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_6X5_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_6X5_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_6X6_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_6X6_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_8X5_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_8X5_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_8X6_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_8X6_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_8X8_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_8X8_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_10X5_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_10X5_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_10X6_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_10X6_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_10X8_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_10X8_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_10X10_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_10X10_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_12X10_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_12X10_SRGB_BLOCK => { todo!() }
            PixelFormat::ASTC_12X12_UNORM_BLOCK => { todo!() }
            PixelFormat::ASTC_12X12_SRGB_BLOCK => { todo!() }
        }
    }
}

impl PixelFormat {
    pub fn is_depth_format(&self) -> bool {
        return match self {
            PixelFormat::D32_SFLOAT => { true }
            PixelFormat::D24_UNORM_S8_UINT => { true }
            PixelFormat::D16_UNORM => { true }
            PixelFormat::D16_UNORM_S8_UINT => { true }
            PixelFormat::D32_SFLOAT_S8_UINT => { true }
            PixelFormat::X8_D24_UNORM_PACK32 => { true }
            _ => { false }
        };
    }
}


#[derive(Copy, Clone)]
pub enum ClearValues {
    DontClear,
    Color(Vec4F32),
    DepthStencil(Vec2f32),
}

pub trait GfxCast: 'static {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> GfxCast for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Copy)]
pub struct Scissors {
    pub min_x: i32,
    pub min_y: i32,
    pub width: u32,
    pub height: u32,
}