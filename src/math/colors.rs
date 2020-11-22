#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Color<T> {
  pub r: T,
  pub g: T,
  pub b: T,
  pub a: T,
}

pub type Colord = Color<f64>;
pub type Colorf = Color<f32>;

#[inline(always)]
pub const fn color<T>(r: T, g: T, b: T, a: T) -> Color<T> { Color { r, g, b, a } }
#[inline(always)]
pub const fn colorn<T: Copy>(n: T, a: T) -> Color<T> { Color { r: n, g: n, b: n, a } }
