use maths::vec2::Vec2u32;

pub struct SurfaceCreateInfos {
    pub image_count: u32,
    pub extent: Vec2u32,
}


pub trait GfxSurface {
    fn create_or_recreate(&mut self, create_infos: SurfaceCreateInfos);
}