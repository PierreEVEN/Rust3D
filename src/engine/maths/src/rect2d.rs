
use std::ops;
use maths_operators::*;

#[derive(Debug, Copy, Clone, OpsAdd, OpsSub, DefaultConstruct)]
pub struct Rect2D<T: Default> {
    pub min_x: T,
    pub min_y: T,
    pub max_x: T,
    pub max_y: T,
}

pub type RectF32 = Rect2D<f32>;
pub type RectF64 = Rect2D<f64>;