use maths::vec2::Vec2u32;

pub trait RenderPassInstance {
    
    
    fn resize(&self, new_res: Vec2u32);
    
    
    fn begin(&self);
    fn submit(&self);
    
}