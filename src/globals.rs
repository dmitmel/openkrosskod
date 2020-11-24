use crate::game_fs::GameFs;
use crate::input::InputState;
use crate::oogl;
use cardboard_math::*;
use prelude_plus::*;

pub type SharedGlobals = Rc<Globals>;

#[derive(Debug)]
pub struct Globals {
  pub gl: oogl::SharedContext,
  pub game_fs: GameFs,
  pub random: GlobalRandom,

  pub should_stop_game_loop: Cell<bool>,
  pub first_game_loop_tick: bool,
  pub time: f64,
  pub delta_time: f64,
  pub fixed_time: f64,
  pub fixed_delta_time: f64,

  pub window_size_i: Vec2<u32>,
  pub window_size: Vec2f,
  pub window_was_resized: bool,
  pub window_is_focused: bool,

  pub input_state: InputState,
}

impl Globals {
  #[inline(always)]
  pub fn share_gl(&self) -> oogl::SharedContext { Rc::clone(&self.gl) }
}

#[derive(Debug)]
pub struct GlobalRandom(UnsafeCell<Rand64>);

impl GlobalRandom {
  pub fn init() -> Result<Self, getrandom::Error> {
    let mut seed_bytes = [0u8; mem::size_of::<u128>()];
    getrandom(&mut seed_bytes)?;

    let mut seed_hex_str = String::with_capacity(seed_bytes.len() * 2);
    // taken from <https://github.com/KokaKiwi/rust-hex/blob/76a83021a1d38cd0e11416c57e50579d1e567054/src/lib.rs#L68-L77>
    static CHARS: &[u8] = b"0123456789abcdef";
    for byte in &seed_bytes {
      seed_hex_str.push(CHARS[(byte >> 4) as usize] as char);
      seed_hex_str.push(CHARS[(byte & 0xF) as usize] as char);
    }

    debug!("RNG Seed: 0x{}", seed_hex_str);

    let inner = Rand64::new(u128::from_le_bytes(seed_bytes));
    Ok(Self(UnsafeCell::new(inner)))
  }

  #[inline]
  pub fn next_u64(&self) -> u64 { unsafe { &mut *self.0.get() }.rand_u64() }
  #[inline]
  pub fn next_i64(&self) -> i64 { unsafe { &mut *self.0.get() }.rand_i64() }
  #[inline]
  pub fn next_f64(&self) -> f64 { unsafe { &mut *self.0.get() }.rand_float() }

  #[inline]
  pub fn next_bool(&self) -> bool { self.next_u64() & 1 != 0 }

  #[inline]
  pub fn next_u64_in_range(&self, range: Range<u64>) -> u64 {
    assert!(range.start < range.end);
    unsafe { &mut *self.0.get() }.rand_range(range)
  }

  #[inline]
  pub fn next_i64_in_range(&self, range: Range<i64>) -> i64 {
    assert!(range.start < range.end);
    // Map the range from `(-(1 << 63))..((1 << 63) - 1)` (signed range) to
    // `0..((1 << 64) - 1)` (unsigned range). Note that both ranges have the same size.
    let unsigned_range =
      (range.start.wrapping_add(i64::MIN) as u64)..(range.end.wrapping_add(i64::MIN) as u64);
    // Now the logic for `u64`s can be used
    let unsigned_random = self.next_u64_in_range(unsigned_range);
    // And finally, map the random value from the unsigned range to the signed
    // range. Note that because both ranges have the same size neither the
    // resulting values, nor their distribution are distorted.
    (unsigned_random as i64).wrapping_sub(i64::MIN)
  }

  pub fn fill_bytes(&self, mut out: &mut [u8]) {
    // This is the most optimal implementation I could come up with. Index
    // bounds checks are eliminated by the optimizer in the generated assembly
    // for this function (unlike in an implementation with an index incremented
    // by the block size on each iteration), hence it is basically equivalent
    // to using unsafe code, namely `ptr::copy_nonoverlapping`.
    while !out.is_empty() {
      let block = self.next_u64().to_le_bytes();
      let out_block_len = out.len().min(block.len());
      out[..out_block_len].copy_from_slice(&block[..out_block_len]);
      out = &mut out[out_block_len..];
    }
  }
}

#[rustfmt::skip]
impl GlobalRandom {
  #[inline(always)] pub fn next_u8 (&self) -> u8  { self.next_u64() as _ }
  #[inline(always)] pub fn next_i8 (&self) -> i8  { self.next_i64() as _ }
  #[inline(always)] pub fn next_u16(&self) -> u16 { self.next_u64() as _ }
  #[inline(always)] pub fn next_i16(&self) -> i16 { self.next_i64() as _ }
  #[inline(always)] pub fn next_u32(&self) -> u32 { self.next_u64() as _ }
  #[inline(always)] pub fn next_i32(&self) -> i32 { self.next_i64() as _ }

  // TODO: copy the implementation of rand_float from Rand32?
  #[inline(always)] pub fn next_f32(&self) -> f32 { self.next_f64() as _ }
}

// No 128-bit ISA support, as you can see
#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
#[rustfmt::skip]
impl GlobalRandom {
  #[inline(always)] pub fn next_usize(&self) -> usize { self.next_u64() as _ }
  #[inline(always)] pub fn next_isize(&self) -> isize { self.next_i64() as _ }
}
