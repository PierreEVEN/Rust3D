use std::ops;

use macros::*;

#[derive(Debug, Copy, Clone, PartialEq, OpsAdd, OpsSub, OpsMul, OpsDiv, DefaultConstruct)]
pub struct Vec2<T: Default> {
    pub x: T,
    pub y: T,
}

pub type Vec2u32 = Vec2<u32>;
pub type Vec2u64 = Vec2<u64>;
pub type Vec2i32 = Vec2<i32>;
pub type Vec2i64 = Vec2<i64>;
pub type Vec2f32 = Vec2<f32>;
pub type Vec2f64 = Vec2<f64>;

impl From<Vec2u64> for Vec2u32 { fn from(v: Vec2u64) -> Self { Vec2u32::new(v.x as u32, v.y as u32) } }
impl From<Vec2i32> for Vec2u32 { fn from(v: Vec2i32) -> Self { Vec2u32::new(v.x as u32, v.y as u32) } }
impl From<Vec2i64> for Vec2u32 { fn from(v: Vec2i64) -> Self { Vec2u32::new(v.x as u32, v.y as u32) } }
impl From<Vec2f32> for Vec2u32 { fn from(v: Vec2f32) -> Self { Vec2u32::new(v.x as u32, v.y as u32) } }
impl From<Vec2f64> for Vec2u32 { fn from(v: Vec2f64) -> Self { Vec2u32::new(v.x as u32, v.y as u32) } }

impl From<Vec2u32> for Vec2u64 { fn from(v: Vec2u32) -> Self { Vec2u64::new(v.x as u64, v.y as u64) } }
impl From<Vec2i32> for Vec2u64 { fn from(v: Vec2i32) -> Self { Vec2u64::new(v.x as u64, v.y as u64) } }
impl From<Vec2i64> for Vec2u64 { fn from(v: Vec2i64) -> Self { Vec2u64::new(v.x as u64, v.y as u64) } }
impl From<Vec2f32> for Vec2u64 { fn from(v: Vec2f32) -> Self { Vec2u64::new(v.x as u64, v.y as u64) } }
impl From<Vec2f64> for Vec2u64 { fn from(v: Vec2f64) -> Self { Vec2u64::new(v.x as u64, v.y as u64) } }

#[test]
fn vec_test() {
    assert_eq!(Vec2f32::new(1.0, 2.0), Vec2f32::new(1.0, 2.0));
    assert_ne!(Vec2f32::new(1.0, 2.0), Vec2f32::new(2.0, 1.0));
    assert_ne!(Vec2f32::new(2.0, 1.0), Vec2f32::new(1.0, 2.0));
    assert_ne!(Vec2f32::new(1.0, 2.0), Vec2f32::new(5.0, 8.0));

    assert_eq!((Vec2f32::new(1.0, 2.0) - Vec2f32::new(5.0, 8.0)).x, -4.0);
    assert_eq!((Vec2f32::new(1.0, 2.0) - Vec2f32::new(5.0, 8.0)).y, -6.0);

    assert_eq!(Vec2f32::new(1.0, 2.0) + Vec2f32::new(5.0, 8.0), Vec2f32::new(6.0, 10.0));
    assert_eq!(Vec2f32::new(10.0, 4.0) - Vec2f32::new(6.0, 3.0), Vec2f32::new(4.0, 1.0));

    assert_eq!(Vec2f32::new(10.0, 4.0) * Vec2f32::new(6.0, 3.0), Vec2f32::new(60.0, 12.0));
    assert_eq!(Vec2f32::new(10.0, 4.0) / Vec2f32::new(2.0, 4.0), Vec2f32::new(5.0, 1.0));
}



