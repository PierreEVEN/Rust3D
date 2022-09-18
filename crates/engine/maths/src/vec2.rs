
use std::ops;
use macros::*;

#[derive(Debug, Copy, Clone, OpsAdd, OpsSub, OpsMul, OpsDiv, DefaultConstruct)]
pub struct Vec2<T: Default> {
    pub x: T,
    pub y: T,
}

pub type Vec2u32 = Vec2<u32>;
pub type Vec2u64 = Vec2<u64>;
pub type Vec2i32 = Vec2<i32>;
pub type Vec2F32 = Vec2<f32>;
pub type Vec2F64 = Vec2<f64>;