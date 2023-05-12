use macros::*;
use std::ops;

#[derive(Debug, Copy, Clone, PartialEq, OpsAdd, OpsSub, OpsMul, OpsDiv, DefaultConstruct)]
pub struct Vec3<T: Default> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type Vec3u32 = Vec3<u32>;
pub type Vec3u64 = Vec3<u64>;
pub type Vec3i32 = Vec3<i32>;
pub type Vec3F32 = Vec3<f32>;
pub type Vec3F64 = Vec3<f64>;
