use crate::prelude::*;
use std::ops::{Add, Div, Mul, Sub};

pub trait Lerp<Rhs = Self, Param = Self> {
  type Output;
  fn lerp(self, rhs: Rhs, t: Param) -> Self::Output;
}

macro_rules! impl_lerp_for_floats {
  ($($ty:ident),+ $(,)?) => { $(
    impl Lerp for $ty {
      type Output = Self;
      fn lerp(self, rhs: Self, t: Self) -> Self::Output { (rhs - self) * t + self }
    }
  )+ };
}

impl_lerp_for_floats!(f32, f64);

pub trait Clamp<Range = Self> {
  type Output;
  fn clamp(self, min: Range, max: Range) -> Self::Output;
}

macro_rules! impl_clamp_for_floats {
  ($($ty:ident),+ $(,)?) => { $(
    impl Clamp for $ty {
      type Output = Self;
      fn clamp(self, min: Self, max: Self) -> Self::Output { max.min(min.max(self)) }
    }
  )+ };
}

impl_clamp_for_floats!(f32, f64);

pub trait RangeMap<Range = Self> {
  type Output;
  fn range_map(self, from_range: (Range, Range), to_range: (Range, Range)) -> Self::Output;
}

impl<T> RangeMap for T
where
  T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Copy,
{
  type Output = Self;
  fn range_map(
    self,
    (from_start, from_end): (Self, Self),
    (to_start, to_end): (Self, Self),
  ) -> Self::Output {
    (self - from_start) * (to_end - to_start) / (from_end - from_start) + to_start
  }
}
