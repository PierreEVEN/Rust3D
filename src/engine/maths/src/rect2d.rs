
use std::{fmt, ops};
use maths_operators::*;

#[derive(Debug, Copy, Clone, OpsAdd, OpsSub, DefaultConstruct)]
pub struct Rect2D<T: Default> {
    _min_x: T,
    _min_y: T,
    _max_x: T,
    _max_y: T,
}

impl<T: Default + ops::Sub + Copy + PartialOrd> Rect2D<T> where <T as ops::Sub>::Output: Into<T> {
    pub fn width(&self) -> T {
        return (self.max_x() - self.min_x()).into()
    }
    pub fn height(&self) -> T {
        return (self.max_y() - self.min_y()).into()
    }

    pub fn min_x(&self) -> T {
        if self._min_x > self._max_x {
            return self._max_x
        }
        return self._min_x
    }
    
    pub fn min_y(&self) -> T {
        if self._min_y > self._max_y {
            return self._max_y
        }
        return self._min_y
    }

    pub fn max_x(&self) -> T {
        if self._max_x < self._min_x {
            return self._min_x
        }
        return self._max_x
    }

    pub fn max_y(&self) -> T {
        if self._max_y < self._min_y {
            return self._min_y
        }
        return self._max_y
    }
}

impl<T: Default + fmt::Display + ops::Sub + Copy + PartialOrd> ToString for Rect2D<T> where <T as ops::Sub>::Output: Into<T> {
    fn to_string(&self) -> String {
        format!("x={}, y={}, res={}x{}", self.min_x(), self.min_y(), self.width(), self.height())
    }
}

pub type RectF32 = Rect2D<f32>;
pub type RectF64 = Rect2D<f64>;

pub type RectI32 = Rect2D<i32>;
pub type RectI64 = Rect2D<i64>;

pub type RectU32 = Rect2D<u32>;
pub type RectU64 = Rect2D<u64>;