
use std::ops;
use maths_operators::*;

#[derive(Debug, Copy, Clone, OpsAdd, OpsSub, DefaultConstruct)]
pub struct Mat4<T: Default> {
    pub x1: T,
    pub x2: T,
    pub x3: T,
    pub x4: T,
    pub y1: T,
    pub y2: T,
    pub y3: T,
    pub y4: T,
    pub z1: T,
    pub z2: T,
    pub z3: T,
    pub z4: T,
    pub w1: T,
    pub w2: T,
    pub w3: T,
    pub w4: T,
}

pub type Mat4F32 = Mat4<f32>;
pub type Mat4F64 = Mat4<f64>;