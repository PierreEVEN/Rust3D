
use std::ops;
use macros::*;


#[derive(Debug, Copy, Clone, OpsAdd, OpsSub, DefaultConstruct)]
pub struct Mat2<T: Default> {
    pub x1: T,
    pub x2: T,
    pub y1: T,
    pub y2: T,
}

pub type Mat2F32 = Mat2<f32>;
pub type Mat2F64 = Mat2<f64>;