#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Vec2<T> {
  pub x: T,
  pub y: T,
}

pub type Vec2d = Vec2<f64>;
pub type Vec2f = Vec2<f32>;

pub const fn vec2<T>(x: T, y: T) -> Vec2<T> { Vec2 { x, y } }
pub const fn vec2n<T: Copy>(n: T) -> Vec2<T> { Vec2 { x: n, y: n } }

macro_rules! define_consts {
  ($float_type:ident) => {
    pub mod $float_type {
      use super::{vec2, vec2n, Vec2};
      pub const UP: Vec2<$float_type> = vec2(0.0, 1.0);
      pub const RIGHT: Vec2<$float_type> = vec2(1.0, 0.0);
      pub const DOWN: Vec2<$float_type> = vec2(0.0, -1.0);
      pub const LEFT: Vec2<$float_type> = vec2(-1.0, 0.0);
      pub const ONE: Vec2<$float_type> = vec2n(1.0);
      pub const ZERO: Vec2<$float_type> = vec2n(0.0);
    }
  };
}

define_consts!(f32);
define_consts!(f64);

impl<T> Vec2<T> {
  // pub fn from<U>(v: Vec2<U>) -> Self
  // where
  //   T: From<U>,
  // {
  //   Self { x: T::from(v.x), y: T::from(v.y) }
  // }

  // pub fn into<U>(self) -> Vec2<U>
  // where
  //   T: Into<U>,
  // {
  //   Vec2 { x: T::into(self.x), y: T::into(self.y) }
  // }

  // pub fn try_from<U, E>(v: Vec2<U>) -> Result<Self, E>
  // where
  //   T: TryFrom<U, Error = E>,
  // {
  //   Ok(Self { x: T::try_from(v.x)?, y: T::try_from(v.y)? })
  // }

  // pub fn try_into<U, E>(self) -> Result<Vec2<U>, E>
  // where
  //   T: TryInto<U, Error = E>,
  // {
  //   Ok(Vec2 { x: T::try_into(self.x)?, y: T::try_into(self.y)? })
  // }

  pub fn map<U, F: FnMut(T) -> U>(self, mut op: F) -> Vec2<U> {
    Vec2 { x: op(self.x), y: op(self.y) }
  }

  pub fn sqr_length(self) -> T
  where
    T: Add<Output = T> + Mul<Output = T> + Copy,
  {
    self.x * self.x + self.y * self.y
  }

  pub fn sqr_distance(self, rhs: Self) -> T
  where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Copy,
  {
    (rhs - self).sqr_length()
  }

  pub fn dot(self, rhs: Self) -> T
  where
    T: Add<Output = T> + Mul<Output = T>,
  {
    self.x * rhs.x + self.y * rhs.y
  }
}

macro_rules! impl_float {
  ($ty:ident, [$(($cast_to_type:ident, $cast_fn_name:ident)),+]) => {
    impl Vec2<$ty> {
      $(pub fn $cast_fn_name(self) -> Vec2<$cast_to_type> {
        Vec2 { x: self.x as $cast_to_type, y: self.y as $cast_to_type }
      })+

      pub fn min_components(self, rhs: Self) -> Self {
        Self { x: self.x.min(rhs.x), y: self.y.min(rhs.y) }
      }

      pub fn max_components(self, rhs: Self) -> Self {
        Self { x: self.x.max(rhs.x), y: self.y.max(rhs.y) }
      }

      pub fn length(self) -> $ty { self.sqr_length().sqrt() }
      pub fn distance(self, rhs: Self) -> $ty { self.sqr_distance(rhs).sqrt() }

      pub fn normalize(self) -> Self {
        let len = self.length();
        if len != 0.0 {
          self / len
        } else {
          self
        }
      }
    }
  };
}

impl_float!(f64, [(f32, as_f32)]);
impl_float!(f32, [(f64, as_f64)]);

impl<T> From<(T, T)> for Vec2<T> {
  fn from((x, y): (T, T)) -> Self { Self { x, y } }
}

impl<T: Copy> From<T> for Vec2<T> {
  fn from(n: T) -> Self { Self { x: n, y: n } }
}

macro_rules! impl_operator {
  (unary: $($op_name:ident $op_fn_name:ident),+ $(,)?) => { $(
      use std::ops::$op_name;
      impl<T: $op_name<Output = T>> $op_name for Vec2<T> {
      type Output = Self;
      fn $op_fn_name(self) -> Self::Output {
        Self { x: $op_name::$op_fn_name(self.x), y: $op_name::$op_fn_name(self.y) }
      }
    }
  )+ };

  (binary: $($op_name:ident $op_fn_name:ident),+ $(,)?) => { $(
    use std::ops::$op_name;
    impl<T: $op_name<Output = T>> $op_name for Vec2<T> {
      type Output = Self;
      fn $op_fn_name(self, rhs: Self) -> Self::Output {
        Self { x: $op_name::$op_fn_name(self.x, rhs.x), y: $op_name::$op_fn_name(self.y, rhs.y) }
      }
    }
    impl<T: $op_name<Output = T> + Copy> $op_name<T> for Vec2<T> {
      type Output = Self;
      fn $op_fn_name(self, rhs: T) -> Self::Output {
        Self { x: $op_name::$op_fn_name(self.x, rhs), y: $op_name::$op_fn_name(self.y, rhs) }
      }
    }
  )+ };

  (binary_assign: $($op_name:ident $op_fn_name:ident),+ $(,)?) => { $(
    use std::ops::$op_name;
    impl<T: $op_name> $op_name for Vec2<T> {
      fn $op_fn_name(&mut self, rhs: Self) {
        $op_name::$op_fn_name(&mut self.x, rhs.x);
        $op_name::$op_fn_name(&mut self.y, rhs.y);
      }
    }
    impl<T: $op_name + Copy> $op_name<T> for Vec2<T> {
      fn $op_fn_name(&mut self, rhs: T) {
        $op_name::$op_fn_name(&mut self.x, rhs);
        $op_name::$op_fn_name(&mut self.y, rhs);
      }
    }
  )+ };
}

impl_operator!(unary: Neg neg);
impl_operator!(binary: Add add, Sub sub, Mul mul, Div div);
impl_operator!(binary_assign:
  AddAssign add_assign, SubAssign sub_assign, MulAssign mul_assign, DivAssign div_assign);

use super::ops::Lerp;
impl<T: Lerp<Output = T> + Copy> Lerp<Vec2<T>, T> for Vec2<T> {
  type Output = Self;
  fn lerp(self, rhs: Self, t: T) -> Self::Output {
    Self { x: self.x.lerp(rhs.x, t), y: self.y.lerp(rhs.y, t) }
  }
}
