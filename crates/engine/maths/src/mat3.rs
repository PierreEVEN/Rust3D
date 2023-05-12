use macros::*;
use std::ops;

#[derive(Debug, Copy, Clone, OpsAdd, OpsSub, DefaultConstruct)]
pub struct Mat3<T: Default> {
    pub x1: T,
    pub x2: T,
    pub x3: T,
    pub y1: T,
    pub y2: T,
    pub y3: T,
    pub z1: T,
    pub z2: T,
    pub z3: T,
}

pub type Mat3F32 = Mat3<f32>;
pub type Mat3F64 = Mat3<f64>;
