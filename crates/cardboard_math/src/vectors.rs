// See also:
// <https://github.com/rustgd/cgmath/blob/a691de871493f652836281e71e2c86c1eb5b50ca/src/vector.rs>
// <https://github.com/rustgd/cgmath/blob/a691de871493f652836281e71e2c86c1eb5b50ca/src/macros.rs>
// <https://github.com/rustgd/cgmath/blob/a691de871493f652836281e71e2c86c1eb5b50ca/src/structure.rs>

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::*;

use crate::ops::*;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct Vec2<T> {
  pub x: T,
  pub y: T,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct Vec3<T> {
  pub x: T,
  pub y: T,
  pub z: T,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct Vec4<T> {
  pub x: T,
  pub y: T,
  pub z: T,
  pub w: T,
}

pub type Vec2f = Vec2<f32>;
pub type Vec2d = Vec2<f64>;
pub type Vec3f = Vec3<f32>;
pub type Vec3d = Vec3<f64>;
pub type Vec4f = Vec4<f32>;
pub type Vec4d = Vec4<f64>;

pub type Vec2bool = Vec2<bool>;
pub type Vec2u8 = Vec2<u8>;
pub type Vec2i8 = Vec2<i8>;
pub type Vec2u16 = Vec2<u16>;
pub type Vec2i16 = Vec2<i16>;
pub type Vec2u32 = Vec2<u32>;
pub type Vec2i32 = Vec2<i32>;
pub type Vec2u64 = Vec2<u64>;
pub type Vec2i64 = Vec2<i64>;
pub type Vec2u128 = Vec2<u128>;
pub type Vec2i128 = Vec2<i128>;
pub type Vec2usize = Vec2<usize>;
pub type Vec2isize = Vec2<isize>;
pub type Vec2f32 = Vec2<f32>;
pub type Vec2f64 = Vec2<f64>;

// https://stackoverflow.com/a/60187870/12005228
macro_rules! skip_first_tt {
  ($head:tt $($rest:tt)*) => { $($rest)* };
}

macro_rules! impl_vec_shorthands {
  ($shorthand_name:ident, $n_shorthand_name:ident, $VecTy:ident { $($field:ident),+ }) => {
    #[inline(always)]
    pub const fn $shorthand_name<T>($($field: T),+) -> $VecTy<T> { $VecTy { $($field),+ } }

    #[cfg(not(feature = "const_fn"))]
    #[inline(always)]
    pub fn $n_shorthand_name<T: Copy>(n: T) -> $VecTy<T> { $VecTy { $($field: n),+ } }
    #[cfg(feature = "const_fn")]
    #[inline(always)]
    pub const fn $n_shorthand_name<T: Copy>(n: T) -> $VecTy<T> { $VecTy { $($field: n),+ } }
  };
}

macro_rules! impl_vec_n {
  ($fields:literal, $VecTy:ident { $($field:ident),+ }) => {
    impl<T: fmt::Debug> fmt::Debug for $VecTy<T> {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(stringify!($VecTy)) $(.field(&self.$field))+ .finish()
      }
    }

    impl<T> $VecTy<T> {
      #[inline(always)]
      pub const fn new($($field: T),+) -> Self { Self { $($field),+ } }

      #[inline]
      pub fn from2<U>(v: $VecTy<U>) -> Self
      where
        T: From<U>,
      {
        Self { $($field: T::from(v.$field)),+ }
      }

      #[inline]
      pub fn into2<U>(self) -> $VecTy<U>
      where
        T: Into<U>,
      {
        $VecTy { $($field: T::into(self.$field)),+ }
      }

      #[inline]
      pub fn try_from2<U, E>(v: $VecTy<U>) -> Result<Self, E>
      where
        T: TryFrom<U, Error = E>,
      {
        Ok(Self { $($field: T::try_from(v.$field)?),+ })
      }

      #[inline]
      pub fn try_into2<U, E>(self) -> Result<$VecTy<U>, E>
      where
        T: TryInto<U, Error = E>,
      {
        Ok($VecTy { $($field: T::try_into(self.$field)?),+ })
      }

      #[inline]
      pub fn map<U, F: FnMut(T) -> U>(self, mut op: F) -> $VecTy<U> {
        $VecTy { $($field: op(self.$field)),+ }
      }

      #[inline]
      pub fn zip<U, V, F: FnMut(T, U) -> V>(self, other: $VecTy<U>, mut op: F) -> $VecTy<V> {
        $VecTy { $($field: op(self.$field, other.$field)),+ }
      }
    }

    impl<T> From<[T; $fields]> for $VecTy<T> {
      #[inline(always)]
      fn from([$($field),+]: [T; $fields]) -> Self { Self { $($field),+ } }
    }

    impl<T> From<$VecTy<T>> for [T; $fields] {
      #[inline(always)]
      fn from(v: $VecTy<T>) -> Self { [$(v.$field),+] }
    }

    impl<T> AsRef<[T; $fields]> for $VecTy<T> {
      #[inline(always)]
      fn as_ref(&self) -> &[T; $fields] {
        unsafe { &*(self as *const _ as *const [T; $fields]) }
      }
    }

    impl<T> AsMut<[T; $fields]> for $VecTy<T> {
      #[inline(always)]
      fn as_mut(&mut self) -> &mut [T; $fields] {
        unsafe { &mut *(self as *mut _ as *mut [T; $fields]) }
      }
    }

    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, u8);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, i8, signed);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, u16);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, i16, signed);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, u32);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, i32, signed);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, u64);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, i64, signed);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, u128);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, i128, signed);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, usize);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, isize, signed);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, f32, float);
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, f64, float);

    impl<T> Lerp<Self, T> for $VecTy<T>
    where
      Self: Lerp<Output = Self>,
      T: Copy,
    {
      type Output = Self;
      #[inline]
      fn lerp(self, rhs: Self, t: T) -> Self::Output { self.lerp(rhs, Self { $($field: t),+ }) }
    }

    impl<T> Clamp2 for $VecTy<T>
    where
      T: Clamp2<Output = T>,
    {
      type Output = Self;
      #[inline]
      fn clamp2(self, min: Self, max: Self) -> Self::Output {
        Self { $($field: self.$field.clamp2(min.$field, max.$field)),+ }
      }
    }

    impl<T> Clamp2<T> for $VecTy<T>
    where
      Self: Clamp2<Output = Self>,
      T: Copy,
    {
      type Output = Self;
      #[inline]
      fn clamp2(self, min: T, max: T) -> Self::Output {
        self.clamp2(Self { $($field: min),+ }, Self { $($field: max),+ })
      }
    }

    impl<T, U> NumCastFrom<$VecTy<U>> for $VecTy<T>
    where
      T: NumCastFrom<U>,
    {
      #[inline]
      fn cast_from(v: $VecTy<U>) -> Self {
        Self { $($field: NumCastFrom::cast_from(v.$field)),+ }
      }
    }
  };
}

macro_rules! impl_vec_n_for_t {
  ($fields:literal, $VecTy:ident { $($field:ident),+ }, $NumTy:ident) => {
    impl $VecTy<$NumTy> {
      pub const ONE: Self = Self { $($field: 1 as _),+ };
      pub const ZERO: Self = Self { $($field: 0 as _),+ };

      #[inline]
      pub fn is_zero(self) -> bool { self == Self::ZERO }

      #[inline]
      pub fn sqr_magnitude(self) -> $NumTy { skip_first_tt!($(+ self.$field * self.$field)+) }
      #[inline]
      pub fn sqr_distance(self, rhs: Self) -> $NumTy { (rhs - self).sqr_magnitude() }
      #[inline]
      pub fn dot(self, rhs: Self) -> $NumTy { skip_first_tt!($(+ self.$field * rhs.$field)+) }

      #[inline]
      pub fn min_components(self, rhs: Self) -> Self {
        Self { $($field: self.$field.min(rhs.$field)),+ }
      }
      #[inline]
      pub fn max_components(self, rhs: Self) -> Self {
        Self { $($field: self.$field.max(rhs.$field)),+ }
      }

      #[inline]
      pub fn mul_components(self) -> $NumTy { skip_first_tt!($(+ self.$field)+) }

      #[inline]
      pub fn reflected_normal(self, normal: Self) -> Self {
        self - (2 as $NumTy) * self.dot(normal) * normal
      }
    }

    impl_vec_n_operator!(binary, $VecTy<$NumTy>, Add, fn add(a, b) { Self { $($field: a.$field + b.$field),+ } });
    impl_vec_n_operator!(binary, $VecTy<$NumTy>, Sub, fn sub(a, b) { Self { $($field: a.$field - b.$field),+ } });
    impl_vec_n_operator!(binary, $VecTy<$NumTy>, Mul, fn mul(a, b) { Self { $($field: a.$field * b.$field),+ } });
    impl_vec_n_operator!(binary, $VecTy<$NumTy>, Div, fn div(a, b) { Self { $($field: a.$field / b.$field),+ } });
    impl_vec_n_operator!(binary, $VecTy<$NumTy>, Rem, fn rem(a, b) { Self { $($field: a.$field % b.$field),+ } });

    impl_vec_n_operator!(binary_assign, $VecTy<$NumTy>, AddAssign, fn add_assign(a, b) { $(a.$field += b.$field);+ });
    impl_vec_n_operator!(binary_assign, $VecTy<$NumTy>, SubAssign, fn sub_assign(a, b) { $(a.$field -= b.$field);+ });
    impl_vec_n_operator!(binary_assign, $VecTy<$NumTy>, MulAssign, fn mul_assign(a, b) { $(a.$field *= b.$field);+ });
    impl_vec_n_operator!(binary_assign, $VecTy<$NumTy>, DivAssign, fn div_assign(a, b) { $(a.$field /= b.$field);+ });
    impl_vec_n_operator!(binary_assign, $VecTy<$NumTy>, RemAssign, fn rem_assign(a, b) { $(a.$field %= b.$field);+ });

    impl_vec_n_operator!(binary_scalar, $VecTy<$NumTy>, Mul, fn mul(v, s) { $VecTy { $($field: v.$field * s),+ } });
    impl_vec_n_operator!(binary_scalar, $VecTy<$NumTy>, Div, fn div(v, s) { $VecTy { $($field: v.$field / s),+ } });
    impl_vec_n_operator!(binary_scalar, $VecTy<$NumTy>, Rem, fn rem(v, s) { $VecTy { $($field: v.$field % s),+ } });

    impl_vec_n_operator!(binary_scalar_assign, $VecTy<$NumTy>, MulAssign, fn mul_assign(v, s) { $(v.$field *= s);+ });
    impl_vec_n_operator!(binary_scalar_assign, $VecTy<$NumTy>, DivAssign, fn div_assign(v, s) { $(v.$field /= s);+ });
    impl_vec_n_operator!(binary_scalar_assign, $VecTy<$NumTy>, RemAssign, fn rem_assign(v, s) { $(v.$field %= s);+ });

    impl_vec_n_specific!($VecTy, $NumTy);
  };

  ($fields:literal, $VecTy:ident { $($field:ident),+ }, $NumTy:ident, signed) => {
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, $NumTy);
    impl_vec_n_operator!(unary, $VecTy<$NumTy>, Neg, fn neg(a) { Self { $($field: -a.$field),+ } });

    impl $VecTy<$NumTy> {
      #[inline]
      pub fn abs(self) -> Self { Self { $($field: self.$field.abs()),+ } }
      #[inline]
      pub fn signum(self) -> Self { Self { $($field: self.$field.signum()),+ } }
    }

    impl Clamp2Abs<$NumTy> for $VecTy<$NumTy> {
      type Output = Self;
      #[inline]
      fn clamp2_abs(self, max: $NumTy) -> Self::Output {
        Self { $($field: self.$field.clamp2_abs(max)),+ }
      }
    }

    impl_vec_n_specific!($VecTy, $NumTy, signed);
  };

  ($fields:literal, $VecTy:ident { $($field:ident),+ }, $NumTy:ident, float) => {
    impl_vec_n_for_t!($fields, $VecTy { $($field),+ }, $NumTy, signed);

    impl $VecTy<$NumTy> {
      #[inline]
      pub fn magnitude(self) -> $NumTy { self.sqr_magnitude().sqrt() }
      #[inline]
      pub fn distance(self, rhs: Self) -> $NumTy { self.sqr_distance(rhs).sqrt() }

      #[inline]
      pub fn normalized(self) -> Self {
        let mag = self.magnitude();
        if mag != 0.0 { self / mag } else { self }
      }
      #[inline]
      pub fn direction(self, towards: Self) -> Self { (self - towards).normalized() }

      #[inline]
      pub fn with_magnitude(self, magnitude: $NumTy) -> Self { self.normalized() * magnitude }
      #[inline]
      pub fn clamp_magnitude(self, max_magnitude: $NumTy) -> Self {
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
    }

    impl_vec_n_specific!($VecTy, $NumTy, float);
  };
}

macro_rules! impl_vec_n_operator {
  (unary, $VecTy:ident<$NumTy:ident>, $op:ident, fn $op_fn:ident($myself:ident) $op_fn_body:block) => {
    impl $op for $VecTy<$NumTy> {
      type Output = Self;
      #[inline]
      fn $op_fn(self) -> Self::Output {
        let $myself = self;
        $op_fn_body
      }
    }
  };

  (binary, $VecTy:ident<$NumTy:ident>, $op:ident, fn $op_fn:ident($lhs:ident, $rhs:ident) $op_fn_body:block) => {
    impl $op for $VecTy<$NumTy> {
      type Output = Self;
      #[inline]
      fn $op_fn(self, other: Self) -> Self::Output {
        let ($lhs, $rhs) = (self, other);
        $op_fn_body
      }
    }
  };

  (binary_scalar, $VecTy:ident<$NumTy:ident>, $op:ident, fn $op_fn:ident($lhs:ident, $rhs:ident) $op_fn_body:block) => {
    impl $op<$NumTy> for $VecTy<$NumTy> {
      type Output = Self;
      #[inline]
      fn $op_fn(self, other: $NumTy) -> Self::Output {
        let ($lhs, $rhs) = (self, other);
        $op_fn_body
      }
    }
    impl $op<$VecTy<$NumTy>> for $NumTy {
      type Output = $VecTy<$NumTy>;
      #[inline]
      fn $op_fn(self, other: $VecTy<$NumTy>) -> Self::Output {
        let ($lhs, $rhs) = (other, self);
        $op_fn_body
      }
    }
  };

  (binary_assign, $VecTy:ident<$NumTy:ident>, $op:ident, fn $op_fn:ident($lhs:ident, $rhs:ident) $op_fn_body:block) => {
    impl $op for $VecTy<$NumTy> {
      #[inline]
      fn $op_fn(&mut self, other: Self) {
        let (mut $lhs, $rhs) = (self, other);
        $op_fn_body
      }
    }
  };

  (binary_scalar_assign, $VecTy:ident<$NumTy:ident>, $op:ident, fn $op_fn:ident($lhs:ident, $rhs:ident) $op_fn_body:block) => {
    impl $op<$NumTy> for $VecTy<$NumTy> {
      #[inline]
      fn $op_fn(&mut self, other: $NumTy) {
        let (mut $lhs, $rhs) = (self, other);
        $op_fn_body
      }
    }
  };
}

macro_rules! impl_vec_n_tuple_conversions {
  ($VecTy:ident { $($field:ident),+ }, $generic:ident, $tuple:ty) => {
    impl<$generic> From<$tuple> for $VecTy<T> {
      #[inline(always)]
      fn from(($($field),+): $tuple) -> Self { Self { $($field),+ } }
    }

    impl<T> From<$VecTy<T>> for $tuple {
      #[inline(always)]
      fn from(v: $VecTy<T>) -> Self { ($(v.$field),+) }
    }
  };
}

macro_rules! impl_vec_n_specific {
  (Vec2, $NumTy:ident) => {};

  (Vec2, $NumTy:ident, signed) => {
#[rustfmt::skip]
    impl Vec2<$NumTy> {
      pub const UP:    Self = vec2( 0 as _,  1 as _);
      pub const DOWN:  Self = vec2( 0 as _, -1 as _);
      pub const RIGHT: Self = vec2( 1 as _,  0 as _);
      pub const LEFT:  Self = vec2(-1 as _,  0 as _);
    }

    impl Vec2<$NumTy> {
      #[inline]
      pub fn perpendicular_cw(self) -> Self { Self { x: self.y, y: -self.x } }
      #[inline]
      pub fn perpendicular_ccw(self) -> Self { Self { x: -self.y, y: self.x } }

      #[inline]
      pub fn angle_sign(self, rhs: Self) -> $NumTy { (self.x * rhs.y - self.y * rhs.x).signum() }
    }
  };

  (Vec3, $NumTy:ident, signed) => {
#[rustfmt::skip]
    impl Vec3<$NumTy> {
      pub const UP:       Self = vec3( 0 as _,  1 as _,  0 as _);
      pub const DOWN:     Self = vec3( 0 as _, -1 as _,  0 as _);
      pub const RIGHT:    Self = vec3( 1 as _,  0 as _,  0 as _);
      pub const LEFT:     Self = vec3(-1 as _,  0 as _,  0 as _);
      pub const FORWARD:  Self = vec3( 0 as _,  0 as _,  1 as _);
      pub const BACKWARD: Self = vec3( 0 as _,  0 as _, -1 as _);
    }

    impl Vec3<$NumTy> {
      #[inline]
      pub fn cross(self, other: Self) -> Self {
        Self {
          x: (self.y * other.z) - (self.z * other.y),
          y: (self.z * other.x) - (self.x * other.z),
          z: (self.x * other.y) - (self.y * other.x),
        }
      }
    }
  };

  (Vec2, $NumTy:ident, float) => {
    // TODO: add the angles stuff to Vec3
    impl Vec2<$NumTy> {
      #[inline]
      pub fn angle_from_x_axis(self) -> $NumTy { self.y.atan2(self.x) }
      #[inline]
      pub fn rotated_from_x_axis(angle: $NumTy) -> Self {
        let (y, x) = angle.sin_cos();
        Self { x, y }
      }

      #[inline]
      pub fn angle_normalized(self, rhs: Self) -> $NumTy { self.dot(rhs).clamp2(-1.0, 1.0).acos() }
      #[inline]
      pub fn signed_angle_normalized(self, rhs: Self) -> $NumTy {
        self.angle_normalized(rhs) * self.angle_sign(rhs)
      }

      #[inline]
      pub fn angle(self, rhs: Self) -> $NumTy {
        let mag = (self.sqr_magnitude() * rhs.sqr_magnitude()).sqrt();
        if mag != 0.0 {
          (self.dot(rhs) / mag).clamp2(-1.0, 1.0).acos()
        } else {
          0.0
        }
      }
      #[inline]
      pub fn signed_angle(self, rhs: Self) -> $NumTy { self.angle(rhs) * self.angle_sign(rhs) }

      #[inline]
      pub fn rotated(self, angle: $NumTy) -> Self {
        let (s, c) = angle.sin_cos();
        Self { x: c * self.x - s * self.y, y: s * self.x + c * self.y }
      }
    }
  };

  ($($anything:tt)*) => {};
}

impl_vec_shorthands!(vec2, vec2n, Vec2 { x, y });
impl_vec_shorthands!(vec3, vec3n, Vec3 { x, y, z });
impl_vec_shorthands!(vec4, vec4n, Vec4 { x, y, z, w });

impl_vec_n!(2, Vec2 { x, y });
impl_vec_n!(3, Vec3 { x, y, z });
impl_vec_n!(4, Vec4 { x, y, z, w });

impl_vec_n_tuple_conversions!(Vec2 { x, y }, T, (T, T));
impl_vec_n_tuple_conversions!(Vec3 { x, y, z }, T, (T, T, T));
impl_vec_n_tuple_conversions!(Vec4 { x, y, z, w }, T, (T, T, T, T));
