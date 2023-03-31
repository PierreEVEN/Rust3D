extern crate core;

use crate::vec2::*;
use crate::vec3::*;
use crate::vec4::*;

pub mod rect2d;
pub mod mat2;
pub mod mat3;
pub mod mat4;
pub mod vec2;
pub mod vec3;
pub mod vec4;

impl <T: Default>From<Vec4<T>> for Vec3<T> {
    fn from(v: Vec4<T>) -> Self {
        Vec3::new(v.x, v.y, v.z)
    }
}

impl <T: Default>From<Vec3<T>> for Vec2<T> {
    fn from(v: Vec3<T>) -> Self {
        Vec2::new(v.x, v.y)
    }
}

impl <T: Default>From<Vec4<T>> for Vec2<T> {
    fn from(v: Vec4<T>) -> Self {
        Vec2::new(v.x, v.y)
    }
}