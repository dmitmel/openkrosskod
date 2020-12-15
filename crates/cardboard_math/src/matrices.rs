#![allow(clippy::too_many_arguments)]

//! NOTE: Matrix implementation is currently very basic and is intended only to
//! work with one example in `cardboard_oogl`! **DO NOT USE!!!**

// See also:
// <https://github.com/rustgd/cgmath/blob/8e0d5ece92ddccd1cbd9670b2bf3007ca9ada986/src/matrix.rs>
// <https://github.com/rustgd/cgmath/blob/8e0d5ece92ddccd1cbd9670b2bf3007ca9ada986/src/macros.rs>
// <https://github.com/rustgd/cgmath/blob/8e0d5ece92ddccd1cbd9670b2bf3007ca9ada986/src/structure.rs>

use serde::{Deserialize, Serialize};
use std::ops::*;

use crate::vectors::*;

#[repr(C)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Deserialize, Serialize)]
pub struct Mat4<T> {
  pub x: Vec4<T>,
  pub y: Vec4<T>,
  pub z: Vec4<T>,
  pub w: Vec4<T>,
}

pub type Mat4f = Mat4<f32>;

impl<T> Mat4<T> {
  #[inline]
  #[rustfmt::skip]
  pub const fn new(
    c0r0: T, c0r1: T, c0r2: T, c0r3: T,
    c1r0: T, c1r1: T, c1r2: T, c1r3: T,
    c2r0: T, c2r1: T, c2r2: T, c2r3: T,
    c3r0: T, c3r1: T, c3r2: T, c3r3: T,
  ) -> Self  {
    Self {
      x: vec4(c0r0, c0r1, c0r2, c0r3),
      y: vec4(c1r0, c1r1, c1r2, c1r3),
      z: vec4(c2r0, c2r1, c2r2, c2r3),
      w: vec4(c3r0, c3r1, c3r2, c3r3),
    }
  }
}

impl<T> AsRef<[T; 4 * 4]> for Mat4<T> {
  #[inline(always)]
  fn as_ref(&self) -> &[T; 4 * 4] { unsafe { &*(self as *const _ as *const [T; 4 * 4]) } }
}

impl<T> AsMut<[T; 4 * 4]> for Mat4<T> {
  #[inline(always)]
  fn as_mut(&mut self) -> &mut [T; 4 * 4] { unsafe { &mut *(self as *mut _ as *mut [T; 4 * 4]) } }
}

impl Mat4<f32> {
  #[rustfmt::skip]
  pub fn identity() -> Self {
    Self::new(
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0,
    )
  }

  #[rustfmt::skip]
  pub fn from_axis_angle(axis: Vec3<f32>, angle: f32) -> Self {
    let Vec3 { x: ax, y: ay, z: az } = axis;
    let (s, c) = angle.sin_cos();
    let tmp = 1.0 - c;

    Self::new(
      tmp * ax * ax + c,      tmp * ax * ay + s * az, tmp * ax * az - s * ay, 0.0,
      tmp * ax * ay - s * az, tmp * ay * ay + c,      tmp * ay * az + s * ax, 0.0,
      tmp * ax * az + s * ay, tmp * ay * az - s * ax, tmp * az * az + c,      0.0,
      0.0,                    0.0,                    0.0,                    1.0,
    )
  }

  #[rustfmt::skip]
  pub fn look_to_rh(eye: Vec3<f32>, dir: Vec3<f32>, up: Vec3<f32>) -> Self {
    let f = dir.normalized();
    let s = f.cross(up).normalized();
    let u = s.cross(f);

    Self::new(
        s.x,         u.x,         -f.x,       0.0,
        s.y,         u.y,         -f.y,       0.0,
        s.z,         u.z,         -f.z,       0.0,
        -eye.dot(s), -eye.dot(u), eye.dot(f), 1.0,
    )
  }

  #[rustfmt::skip]
  pub fn perspective_rh_no(fov_y: f32, aspect: f32, z_near: f32, z_far: f32) -> Self {
    let tan_half_fov_y = (fov_y / 2.0).tan();
    let c0r0 = 1.0 / (aspect * tan_half_fov_y);
    let c1r1 = 1.0 / tan_half_fov_y;
    let c2r2 = -(z_far + z_near) / (z_far - z_near);
    let c2r3 = -1.0;
    let c3r2 = -(2.0 * z_far * z_near) / (z_far - z_near);
    Self::new(
      c0r0, 0.0,  0.0,  0.0,
      0.0,  c1r1, 0.0,  0.0,
      0.0,  0.0,  c2r2, c2r3,
      0.0,  0.0,  c3r2, 0.0,
    )
  }
}

impl Mul<Mat4<f32>> for Mat4<f32> {
  type Output = Self;
  #[rustfmt::skip]
  fn mul(self, rhs: Self) -> Self {
    let Self { x: a, y: b, z: c, w: d } = rhs;
    Self {
      x: a*rhs.x.x + b*rhs.x.y + c*rhs.x.z + d*rhs.x.w,
      y: a*rhs.y.x + b*rhs.y.y + c*rhs.y.z + d*rhs.y.w,
      z: a*rhs.z.x + b*rhs.z.y + c*rhs.z.z + d*rhs.z.w,
      w: a*rhs.w.x + b*rhs.w.y + c*rhs.w.z + d*rhs.w.w,
    }
  }
}
