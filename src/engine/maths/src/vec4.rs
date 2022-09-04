
use std::ops;
use macros::*;

#[derive(Debug, Copy, Clone, OpsAdd, OpsSub, OpsMul, OpsDiv, DefaultConstruct)]
pub struct Vec4<T: Default> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

pub type Vec4u32 = Vec4<u32>;
pub type Vec4u64 = Vec4<u64>;
pub type Vec4i32 = Vec4<i32>;
pub type Vec4F32 = Vec4<f32>;
pub type Vec4F64 = Vec4<f64>;