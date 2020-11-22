use std::ops::*;

macro_rules! impl_operator {
  ({ impl $trait:ident for ($($ty:ty),+) $block:tt }) => {
    $(impl $trait for $ty $block)+
  };
}

pub trait Lerp<Rhs = Self, Param = Self> {
  type Output;
  fn lerp(self, rhs: Rhs, t: Param) -> Self::Output;
}

impl<T> Lerp for T
where
  T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Copy,
{
  type Output = Self;
  #[inline]
  fn lerp(self, rhs: Self, t: Self) -> Self::Output { (rhs - self) * t + self }
}

pub trait Clamp2<Range = Self> {
  type Output;
  fn clamp2(self, min: Range, max: Range) -> Self::Output;
}

impl_operator!({
  impl Clamp2 for (u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64) {
    type Output = Self;
    #[inline]
    fn clamp2(self, min: Self, max: Self) -> Self::Output { max.min(min.max(self)) }
  }
});

pub trait Clamp2Abs<Range = Self> {
  type Output;
  fn clamp2_abs(self, max: Range) -> Self::Output;
}

impl<T> Clamp2Abs for T
where
  T: Clamp2<Output = T> + Neg<Output = T> + Copy,
{
  type Output = Self;
  #[inline]
  fn clamp2_abs(self, max: Self) -> Self::Output { self.clamp2(-max, max) }
}

pub trait RangeMap<Range = Self> {
  type Output;
  fn range_map(self, from_range: (Range, Range), to_range: (Range, Range)) -> Self::Output;
}

impl<T> RangeMap for T
where
  T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Copy,
{
  type Output = Self;
  #[inline]
  fn range_map(
    self,
    (from_start, from_end): (Self, Self),
    (to_start, to_end): (Self, Self),
  ) -> Self::Output {
    (self - from_start) * (to_end - to_start) / (from_end - from_start) + to_start
  }
}

pub trait NumCastFrom<T>: Sized {
  fn cast_from(_: T) -> Self;
}

pub trait NumCastInto<T>: Sized {
  fn cast_into(self) -> T;
}

macro_rules! impl_num_cast_from {
  (all) => {
    impl_num_cast_from!(all: u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 f32 f64);
  };

  (all: $($into_ty:ty)+) => {
    $(impl_num_cast_from!($into_ty: u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 f32 f64);)+
  };

  ($into_ty:ty: $($from_ty:ty)+) => {
    $(impl NumCastFrom<$from_ty> for $into_ty {
      #[inline(always)]
      fn cast_from(n: $from_ty) -> Self { n as Self }
    })+
  };
}

impl_num_cast_from!(all);

impl<T, U> NumCastInto<U> for T
where
  U: NumCastFrom<T>,
{
  #[inline(always)]
  fn cast_into(self) -> U { U::cast_from(self) }
}
