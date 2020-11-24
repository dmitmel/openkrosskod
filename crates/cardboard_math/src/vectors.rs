// See also:
// <https://github.com/rustgd/cgmath/blob/a691de871493f652836281e71e2c86c1eb5b50ca/src/vector.rs>
// <https://github.com/rustgd/cgmath/blob/a691de871493f652836281e71e2c86c1eb5b50ca/src/macros.rs>
// <https://github.com/rustgd/cgmath/blob/a691de871493f652836281e71e2c86c1eb5b50ca/src/structure.rs>

use crate::ops::*;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::*;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Deserialize, Serialize)]
#[repr(C)] // TODO: do we lose efficiency with repr(C) in this case?
pub struct Vec2<T> {
  pub x: T,
  pub y: T,
}

impl<T: fmt::Debug> fmt::Debug for Vec2<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("Vec2").field(&self.x).field(&self.y).finish()
  }
}

pub type Vec2d = Vec2<f64>;
pub type Vec2f = Vec2<f32>;

#[inline(always)]
pub const fn vec2<T>(x: T, y: T) -> Vec2<T> { Vec2 { x, y } }

#[cfg(not(feature = "const_fn"))]
#[inline]
pub fn vec2n<T: Copy>(n: T) -> Vec2<T> { Vec2 { x: n, y: n } }
#[cfg(feature = "const_fn")]
#[inline]
pub const fn vec2n<T: Copy>(n: T) -> Vec2<T> { Vec2 { x: n, y: n } }

impl<T> Vec2<T> {
  #[inline]
  pub fn from2<U>(v: Vec2<U>) -> Self
  where
    T: From<U>,
  {
    Self { x: T::from(v.x), y: T::from(v.y) }
  }

  #[inline]
  pub fn into2<U>(self) -> Vec2<U>
  where
    T: Into<U>,
  {
    Vec2 { x: T::into(self.x), y: T::into(self.y) }
  }

  #[inline]
  pub fn try_from2<U, E>(v: Vec2<U>) -> Result<Self, E>
  where
    T: TryFrom<U, Error = E>,
  {
    Ok(Self { x: T::try_from(v.x)?, y: T::try_from(v.y)? })
  }

  #[inline]
  pub fn try_into2<U, E>(self) -> Result<Vec2<U>, E>
  where
    T: TryInto<U, Error = E>,
  {
    Ok(Vec2 { x: T::try_into(self.x)?, y: T::try_into(self.y)? })
  }

  #[inline(always)]
  pub const fn new(x: T, y: T) -> Self { Self { x, y } }

  #[inline]
  pub fn map<U, F: FnMut(T) -> U>(self, mut op: F) -> Vec2<U> {
    Vec2 { x: op(self.x), y: op(self.y) }
  }

  #[inline]
  pub fn zip<U, V, F: FnMut(T, U) -> V>(self, other: Vec2<U>, mut op: F) -> Vec2<V> {
    Vec2 { x: op(self.x, other.x), y: op(self.y, other.y) }
  }
}

macro_rules! impl_vec2_operator {
  (unary, $ty:ty, $op:ident, fn $op_fn:ident($this:ident) $op_fn_body:block) => {
    impl $op for Vec2<$ty> {
      type Output = Self;
      #[inline]
      fn $op_fn(self) -> Self::Output {
        let $this = self;
        $op_fn_body
      }
    }
  };

  (binary, $ty:ty, $op:ident, fn $op_fn:ident($lhs:ident, $rhs:ident) $op_fn_body:block) => {
    impl $op for Vec2<$ty> {
      type Output = Self;
      #[inline]
      fn $op_fn(self, other: Self) -> Self::Output {
        let ($lhs, $rhs) = (self, other);
        $op_fn_body
      }
    }
  };

  (binary_scalar, $ty:ty, $op:ident, fn $op_fn:ident($lhs:ident, $rhs:ident) $op_fn_body:block) => {
    impl $op<$ty> for Vec2<$ty> {
      type Output = Self;
      #[inline]
      fn $op_fn(self, other: $ty) -> Self::Output {
        let ($lhs, $rhs) = (self, other);
        $op_fn_body
      }
    }
    impl $op<Vec2<$ty>> for $ty {
      type Output = Vec2<$ty>;
      #[inline]
      fn $op_fn(self, other: Vec2<$ty>) -> Self::Output {
        let ($lhs, $rhs) = (other, self);
        $op_fn_body
      }
    }
  };

  (binary_assign, $ty:ty, $op:ident, fn $op_fn:ident($lhs:ident, $rhs:ident) $op_fn_body:block) => {
    impl $op for Vec2<$ty> {
      #[inline]
      fn $op_fn(&mut self, other: Self) {
        let (mut $lhs, $rhs) = (self, other);
        $op_fn_body
      }
    }
  };

  (binary_scalar_assign, $ty:ty, $op:ident, fn $op_fn:ident($lhs:ident, $rhs:ident) $op_fn_body:block) => {
    impl $op<$ty> for Vec2<$ty> {
      #[inline]
      fn $op_fn(&mut self, other: $ty) {
        let (mut $lhs, $rhs) = (self, other);
        $op_fn_body
      }
    }
  };
}

macro_rules! impl_vec2 {
  ($ty:ty) => {
    impl Vec2<$ty> {
      pub const ONE: Self = vec2(1 as _, 1 as _);
      pub const ZERO: Self = vec2(0 as _, 0 as _);

      #[inline]
      pub fn is_zero(self) -> bool { self == Self::ZERO }

      #[inline]
      pub fn sqr_magnitude(self) -> $ty { self.x * self.x + self.y * self.y }
      #[inline]
      pub fn sqr_distance(self, rhs: Self) -> $ty { (rhs - self).sqr_magnitude() }
      #[inline]
      pub fn dot(self, rhs: Self) -> $ty { self.x * rhs.x + self.y * rhs.y }

      #[inline]
      pub fn min_components(self, rhs: Self) -> Self {
        Self { x: self.x.min(rhs.x), y: self.y.min(rhs.y) }
      }
      #[inline]
      pub fn max_components(self, rhs: Self) -> Self {
        Self { x: self.x.max(rhs.x), y: self.y.max(rhs.y) }
      }

      #[inline]
      pub fn mul_components(self) -> $ty {
        self.x * self.y
      }

      #[inline]
      pub fn reflected_normal(self, normal: Self) -> Self {
        self - (2 as $ty) * self.dot(normal) * normal
      }
    }

    impl_vec2_operator!(binary, $ty, Add, fn add(a, b) { Self { x: a.x + b.x, y: a.y + b.y } });
    impl_vec2_operator!(binary, $ty, Sub, fn sub(a, b) { Self { x: a.x - b.x, y: a.y - b.y } });
    impl_vec2_operator!(binary, $ty, Mul, fn mul(a, b) { Self { x: a.x * b.x, y: a.y * b.y } });
    impl_vec2_operator!(binary, $ty, Div, fn div(a, b) { Self { x: a.x / b.x, y: a.y / b.y } });
    impl_vec2_operator!(binary, $ty, Rem, fn rem(a, b) { Self { x: a.x % b.x, y: a.y % b.y } });

    impl_vec2_operator!(binary_assign, $ty, AddAssign, fn add_assign(a, b) { a.x += b.x; a.y += b.y; });
    impl_vec2_operator!(binary_assign, $ty, SubAssign, fn sub_assign(a, b) { a.x -= b.x; a.y -= b.y; });
    impl_vec2_operator!(binary_assign, $ty, MulAssign, fn mul_assign(a, b) { a.x *= b.x; a.y *= b.y; });
    impl_vec2_operator!(binary_assign, $ty, DivAssign, fn div_assign(a, b) { a.x /= b.x; a.y /= b.y; });
    impl_vec2_operator!(binary_assign, $ty, RemAssign, fn rem_assign(a, b) { a.x %= b.x; a.y %= b.y; });

    impl_vec2_operator!(binary_scalar, $ty, Mul, fn mul(v, s) { Vec2 { x: v.x * s, y: v.y * s } });
    impl_vec2_operator!(binary_scalar, $ty, Div, fn div(v, s) { Vec2 { x: v.x / s, y: v.y / s } });
    impl_vec2_operator!(binary_scalar, $ty, Rem, fn rem(v, s) { Vec2 { x: v.x % s, y: v.y % s } });

    impl_vec2_operator!(binary_scalar_assign, $ty, MulAssign, fn mul_assign(v, s) { v.x *= s; v.y *= s; });
    impl_vec2_operator!(binary_scalar_assign, $ty, DivAssign, fn div_assign(v, s) { v.x /= s; v.y /= s; });
    impl_vec2_operator!(binary_scalar_assign, $ty, RemAssign, fn rem_assign(v, s) { v.x %= s; v.y %= s; });
  };

  ($ty:ty, signed) => {
    impl_vec2!($ty);
    impl_vec2_operator!(unary, $ty, Neg, fn neg(a) { Self { x: -a.x, y: -a.y } });

    impl Vec2<$ty> {
      pub const UP: Self = vec2(0 as _, 1 as _);
      pub const RIGHT: Self = vec2(1 as _, 0 as _);
      pub const DOWN: Self = vec2(0 as _, -1 as _);
      pub const LEFT: Self = vec2(-1 as _, 0 as _);

      #[inline]
      pub fn abs(self) -> Self { Self { x: self.x.abs(), y: self.y.abs() } }
      #[inline]
      pub fn signum(self) -> Self { Self { x: self.x.signum(), y: self.y.signum() } }

      #[inline]
      pub fn perpendicular_cw(self) -> Self { Self { x: self.y, y: -self.x } }
      #[inline]
      pub fn perpendicular_ccw(self) -> Self { Self { x: -self.y, y: self.x } }

      #[inline]
      pub fn angle_sign(self, rhs: Self) -> $ty { (self.x * rhs.y - self.y * rhs.x).signum() }
    }

    impl Clamp2Abs<$ty> for Vec2<$ty> {
      type Output = Self;
      #[inline]
      fn clamp2_abs(self, max: $ty) -> Self::Output {
        Self { x: self.x.clamp2_abs(max), y: self.y.clamp2_abs(max) }
      }
    }
  };

  ($ty:ident, float) => {
    impl_vec2!($ty, signed);

    impl Vec2<$ty> {
      #[inline]
      pub fn magnitude(self) -> $ty { self.sqr_magnitude().sqrt() }
      #[inline]
      pub fn distance(self, rhs: Self) -> $ty { self.sqr_distance(rhs).sqrt() }

      #[inline]
      pub fn normalized(self) -> Self {
        let mag = self.magnitude();
        if mag != 0.0 {
          self / mag
        } else {
          self
        }
      }

      #[inline]
      pub fn direction(self, towards: Self) -> Self { (self - towards).normalized() }

      #[inline]
      pub fn with_magnitude(self, magnitude: $ty) -> Self { self.normalized() * magnitude }
      #[inline]
      pub fn clamp_magnitude(self, max_magnitude: $ty) -> Self {
        let sqr_magnitude = self.sqr_magnitude();
        if sqr_magnitude > max_magnitude * max_magnitude {
          // NOTE: The minimum value of `max_magnitude * max_magnitude` is
          // zero, therefore (due to the strict greater-than comparison)
          // `sqr_magnitude` can be assumed to be non-zero.
          (self / sqr_magnitude.sqrt()) * max_magnitude
        } else {
          self
        }
      }

      #[inline]
      pub fn angle_from_x_axis(self) -> $ty { self.y.atan2(self.x) }

      #[inline]
      pub fn angle_normalized(self, rhs: Self) -> $ty { self.dot(rhs).clamp2(-1.0, 1.0).acos() }
      #[inline]
      pub fn signed_angle_normalized(self, rhs: Self) -> $ty {
        self.angle_normalized(rhs) * self.angle_sign(rhs)
      }

      #[inline]
      pub fn angle(self, rhs: Self) -> $ty {
        let mag = (self.sqr_magnitude() * rhs.sqr_magnitude()).sqrt();
        if mag != 0.0 {
          (self.dot(rhs) / mag).clamp2(-1.0, 1.0).acos()
        } else {
          0.0
        }
      }
      #[inline]
      pub fn signed_angle(self, rhs: Self) -> $ty { self.angle(rhs) * self.angle_sign(rhs) }

      #[inline]
      pub fn rotated(self, angle: $ty) -> Self {
        let s = angle.sin();
        let c = angle.cos();
        Self {
          x: c * self.x - s * self.y,
          y: s * self.x + c * self.y,
        }
      }
    }
  };
}

impl_vec2!(u8);
impl_vec2!(i8, signed);
impl_vec2!(u16);
impl_vec2!(i16, signed);
impl_vec2!(u32);
impl_vec2!(i32, signed);
impl_vec2!(u64);
impl_vec2!(i64, signed);
impl_vec2!(u128);
impl_vec2!(i128, signed);
impl_vec2!(usize);
impl_vec2!(isize, signed);
impl_vec2!(f32, float);
impl_vec2!(f64, float);

impl<T> From<(T, T)> for Vec2<T> {
  #[inline(always)]
  fn from((x, y): (T, T)) -> Self { Self { x, y } }
}

impl<T> From<[T; 2]> for Vec2<T> {
  #[inline(always)]
  fn from([x, y]: [T; 2]) -> Self { Self { x, y } }
}

impl<T: Copy> From<T> for Vec2<T> {
  #[inline(always)]
  fn from(n: T) -> Self { Self { x: n, y: n } }
}

impl<T> Into<(T, T)> for Vec2<T> {
  #[inline(always)]
  fn into(self) -> (T, T) { (self.x, self.y) }
}

impl<T> Into<[T; 2]> for Vec2<T> {
  #[inline(always)]
  fn into(self) -> [T; 2] { [self.x, self.y] }
}

impl<T> Lerp<Self, T> for Vec2<T>
where
  Self: Lerp<Output = Self>,
  T: Copy,
{
  type Output = Self;
  #[inline]
  fn lerp(self, rhs: Self, t: T) -> Self::Output { self.lerp(rhs, Self { x: t, y: t }) }
}

impl<T> Clamp2 for Vec2<T>
where
  T: Clamp2<Output = T>,
{
  type Output = Self;
  #[inline]
  fn clamp2(self, min: Self, max: Self) -> Self::Output {
    Self { x: self.x.clamp2(min.x, max.x), y: self.y.clamp2(min.y, max.y) }
  }
}

impl<T> Clamp2<T> for Vec2<T>
where
  Self: Clamp2<Output = Self>,
  T: Copy,
{
  type Output = Self;
  #[inline]
  fn clamp2(self, min: T, max: T) -> Self::Output {
    self.clamp2(Self { x: min, y: min }, Self { x: max, y: max })
  }
}

impl<T, U> NumCastFrom<Vec2<U>> for Vec2<T>
where
  T: NumCastFrom<U>,
{
  #[inline]
  fn cast_from(v: Vec2<U>) -> Self {
    Self { x: NumCastFrom::cast_from(v.x), y: NumCastFrom::cast_from(v.y) }
  }
}
