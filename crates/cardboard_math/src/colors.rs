use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Deserialize, Serialize)]
#[repr(C)] // TODO: do we lose efficiency with repr(C) in this case?
pub struct Color<T> {
  pub r: T,
  pub g: T,
  pub b: T,
  pub a: T,
}

impl<T: fmt::Debug> fmt::Debug for Color<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("Color").field(&self.r).field(&self.g).field(&self.b).field(&self.a).finish()
  }
}

pub type Colord = Color<f64>;
pub type Colorf = Color<f32>;

#[inline(always)]
pub const fn color<T>(r: T, g: T, b: T, a: T) -> Color<T> { Color { r, g, b, a } }

#[cfg(not(feature = "const_fn"))]
#[inline]
pub fn colorn<T: Copy>(n: T, a: T) -> Color<T> { Color { r: n, g: n, b: n, a } }
#[cfg(feature = "const_fn")]
#[inline]
pub const fn colorn<T: Copy>(n: T, a: T) -> Color<T> { Color { r: n, g: n, b: n, a } }

impl<T> Color<T> {
  #[inline]
  pub fn with_alpha(self, a: T) -> Self { Self { r: self.r, g: self.g, b: self.b, a } }
}
