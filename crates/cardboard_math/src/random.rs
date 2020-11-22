pub use ::getrandom::{self, getrandom};
use std::mem::size_of;
use std::ops::Range;

pub use oorandom::{Rand32, Rand64};

pub fn init_rand32() -> Result<Rand32, getrandom::Error> {
  let mut seed_bytes = [0u8; size_of::<u64>()];
  getrandom(&mut seed_bytes)?;
  Ok(Rand32::new(u64::from_le_bytes(seed_bytes)))
}

pub fn init_rand64() -> Result<Rand64, getrandom::Error> {
  let mut seed_bytes = [0u8; size_of::<u128>()];
  getrandom(&mut seed_bytes)?;
  Ok(Rand64::new(u128::from_le_bytes(seed_bytes)))
}

pub fn init_rand_size() -> Result<RandSize, getrandom::Error> {
  let mut seed_bytes = [0u8; size_of::<RandSizeSeed>()];
  getrandom(&mut seed_bytes)?;
  Ok(RandSize::new(RandSizeSeed::from_le_bytes(seed_bytes)))
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct RandSize(pub RandSizeImpl);

macro_rules! impl_rand_size {
  ($rand_t:ty, $num_t:ty, $float_num_t:ty, $seed_t:ty, $rand_usize_fn:ident, $rand_isize_fn:ident) => {
    pub type RandSizeImpl = $rand_t;
    pub type RandSizeNum = $num_t;
    pub type RandSizeFloatNum = $float_num_t;
    pub type RandSizeSeed = $seed_t;

    impl RandSize {
      /// See [`Rand32::DEFAULT_INC`] and [`Rand64::DEFAULT_INC`].
      pub const DEFAULT_INC: RandSizeSeed = RandSizeImpl::DEFAULT_INC;
      /// See [`Rand32::new`] and [`Rand64::new`].
      #[inline(always)]
      pub fn new(seed: RandSizeSeed) -> Self { Self(RandSizeImpl::new(seed)) }
      /// See [`Rand32::new_inc`] and [`Rand64::new_inc`].
      pub fn new_inc(seed: RandSizeSeed, increment: RandSizeSeed) -> Self {
        Self(RandSizeImpl::new_inc(seed, increment))
      }
      /// See [`Rand32::state`] and [`Rand64::state`].
      #[inline(always)]
      pub fn state(&self) -> (RandSizeSeed, RandSizeSeed) { self.0.state() }
      /// See [`Rand32::from_state`] and [`Rand64::from_state`].
      #[inline(always)]
      pub fn from_state(state: (RandSizeSeed, RandSizeSeed)) -> Self {
        Self(RandSizeImpl::from_state(state))
      }
      /// See [`Rand32::rand_u32`] and [`Rand64::rand_u64`].
      #[inline(always)]
      pub fn rand_usize(&mut self) -> usize { self.0.$rand_usize_fn() as usize }
      /// See [`Rand32::rand_i32`] and [`Rand64::rand_i64`].
      #[inline(always)]
      pub fn rand_isize(&mut self) -> isize { self.0.$rand_isize_fn() as isize }
      /// See [`Rand32::rand_float`] and [`Rand64::rand_float`].
      #[inline(always)]
      pub fn rand_float(&mut self) -> RandSizeFloatNum { self.0.rand_float() }
      /// See [`Rand32::rand_range`] and [`Rand64::rand_range`].
      #[inline]
      pub fn rand_range(&mut self, range: Range<usize>) -> usize {
        let mut rng = self.0;
        rng.rand_range((range.start as RandSizeNum)..(range.end as RandSizeNum)) as usize
      }
    }
  };
}

cfg_if::cfg_if! {
  if #[cfg(target_pointer_width = "32")] {
    impl_rand_size!(Rand32, u32, f32, u64, rand_u32, rand_i32);
  } else if #[cfg(target_pointer_width = "64")] {
    impl_rand_size!(Rand64, u64, f64, u128, rand_u64, rand_i64);
  }
}
