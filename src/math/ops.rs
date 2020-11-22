use prelude_plus::*;
use std::ops::{Add, Div, Mul, Sub};

pub trait Lerp<Rhs = Self, Param = Self> {
  type Output;
  fn lerp(self, rhs: Rhs, t: Param) -> Self::Output;
}

macro_rules! impl_lerp_for_floats {
  ($($ty:ident),+ $(,)?) => { $(
    impl Lerp for $ty {
      type Output = Self;
      #[inline(always)]
      fn lerp(self, rhs: Self, t: Self) -> Self::Output { (rhs - self) * t + self }
    }
  )+ };
}

impl_lerp_for_floats!(f32, f64);

pub trait Clamp2<Range = Self> {
  type Output;
  fn clamp2(self, min: Range, max: Range) -> Self::Output;
  fn clamp2_abs(self, max: Range) -> Self::Output;
}

macro_rules! impl_clamp2_for_floats {
  ($($ty:ident),+ $(,)?) => { $(
    impl Clamp2 for $ty {
      type Output = Self;
      #[inline(always)]
      fn clamp2(self, min: Self, max: Self) -> Self::Output { max.min(min.max(self)) }
      fn clamp2_abs(self, max: Self) -> Self::Output { self.clamp2(-max, max) }
    }

    impl Clamp2 for super::Vec2<$ty> {
      type Output = Self;
      #[inline(always)]
      fn clamp2(self, min: Self, max: Self) -> Self::Output {
        Self { x: self.x.clamp2(min.x, max.x), y: self.y.clamp2(min.y, max.y) }
      }
      fn clamp2_abs(self, max: Self) -> Self::Output {
        Self { x: self.x.clamp2_abs(max.x), y: self.y.clamp2_abs(max.y) }
      }
    }

    impl Clamp2<$ty> for super::Vec2<$ty> {
      type Output = Self;
      #[inline(always)]
      fn clamp2(self, min: $ty, max: $ty) -> Self::Output {
        Self { x: self.x.clamp2(min, max), y: self.y.clamp2(min, max) }
      }
      fn clamp2_abs(self, max: $ty) -> Self::Output {
        Self { x: self.x.clamp2_abs(max), y: self.y.clamp2_abs(max) }
      }
    }
  )+ };
}

impl_clamp2_for_floats!(f32, f64);

pub trait RangeMap<Range = Self> {
  type Output;
  fn range_map(self, from_range: (Range, Range), to_range: (Range, Range)) -> Self::Output;
}

impl<T> RangeMap for T
where
  T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Copy,
{
  type Output = Self;
  #[inline(always)]
  fn range_map(
    self,
    (from_start, from_end): (Self, Self),
    (to_start, to_end): (Self, Self),
  ) -> Self::Output {
    (self - from_start) * (to_end - to_start) / (from_end - from_start) + to_start
  }
}
