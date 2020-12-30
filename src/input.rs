use cardboard_math::*;
use prelude_plus::*;
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;

#[derive(Debug)]
pub struct InputState {
  pub mouse_pos: Vec2f,
  pub prev_mouse_pos: Vec2f,
  pub delta_mouse_pos: Vec2f,

  pub prev_keyboard_state_table: [bool; Key::VARIANTS.len()],
  pub keyboard_state_table: [bool; Key::VARIANTS.len()],
}

impl InputState {
  pub fn new() -> Self {
    Self {
      mouse_pos: Vec2f::ZERO,
      prev_mouse_pos: Vec2f::ZERO,
      delta_mouse_pos: Vec2f::ZERO,

      prev_keyboard_state_table: [false; Key::VARIANTS.len()],
      keyboard_state_table: [false; Key::VARIANTS.len()],
    }
  }

  #[inline(always)]
  pub fn set_key_down(&mut self, key: Key, down: bool) {
    self.keyboard_state_table[key as usize] = down
  }

  #[inline(always)]
  pub fn is_key_down(&self, key: Key) -> bool { self.keyboard_state_table[key as usize] }

  #[inline(always)]
  pub fn is_key_up(&self, key: Key) -> bool { !self.is_key_down(key) }

  #[inline]
  pub fn is_key_pressed(&self, key: Key) -> bool {
    !self.prev_keyboard_state_table[key as usize] && self.keyboard_state_table[key as usize]
  }

  #[inline]
  pub fn is_key_unpressed(&self, key: Key) -> bool {
    self.prev_keyboard_state_table[key as usize] && !self.keyboard_state_table[key as usize]
  }

  #[inline]
  pub fn axis(&self, key_less: Key, key_more: Key) -> i8 {
    let mut dir = 0;
    if self.is_key_down(key_less) {
      dir -= 1;
    }
    if self.is_key_down(key_more) {
      dir += 1;
    }
    dir
  }
}

macro_rules! generate_key_enum {
  ($($key:ident),+ $(,)?) => {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    #[repr(u8)]
    pub enum Key {
      $($key),+
    }
    impl Key {
      pub const VARIANTS: &'static [Self] = &[$(Self::$key),+];
    }
  };
}

#[rustfmt::skip]
generate_key_enum! [
  Backspace, Tab, Return, Pause, CapsLock, Escape, Space, PageUp, PageDown,
  End, Left, Up, Right, Down, Home, Insert, Delete, Num0, Num1, Num2, Num3,
  Num4, Num5, Num6, Num7, Num8, Num9, A, B, C, D, E, F, G, H, I, J, K, L, M, N,
  O, P, Q, R, S, T, U, V, W, X, Y, Z, Kp0, Kp1, Kp2, Kp3, Kp4, Kp5, Kp6, Kp7,
  Kp8, Kp9, KpMultiply, KpPlus, KpMinus, KpDecimal, KpDivide, F1, F2, F3, F4,
  F5, F6, F7, F8, F9, F10, F11, F12,
  // TODO: handle left/right modifiers as the same single key?
  LShift, RShift, LCtrl, RCtrl, LAlt, RAlt, Equals, Comma, Minus, Period,
  Semicolon, Grave, Slash, LeftBracket, Backslash, RightBracket, Apostrophe,

  MouseLeft, MouseMiddle, MouseRight, MouseX1, MouseX2,
];

impl Key {
  #[rustfmt::skip]
  pub fn from_sdl2_mouse_button(value: MouseButton) -> Option<Self> {
    Some(match value {
      MouseButton::Left   => Self::MouseLeft,
      MouseButton::Middle => Self::MouseMiddle,
      MouseButton::Right  => Self::MouseRight,
      MouseButton::X1     => Self::MouseX1,
      MouseButton::X2     => Self::MouseX2,
      _ => return None,
    })
  }

  #[rustfmt::skip]
  pub fn from_sdl2_scancode(value: Scancode) -> Option<Self> {
    macro_rules! helper {
      ($($key:ident),+ $(,)?) => {
        Some(match value {
          $(Scancode::$key => Self::$key,)+
          _ => return None,
        })
      };
    }
    helper![
      Backspace, Tab, Return, Pause, CapsLock, Escape, Space, PageUp, PageDown,
      End, Left, Up, Right, Down, Home, Insert, Delete, Num0, Num1, Num2, Num3,
      Num4, Num5, Num6, Num7, Num8, Num9, A, B, C, D, E, F, G, H, I, J, K, L,
      M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Kp0, Kp1, Kp2, Kp3, Kp4, Kp5,
      Kp6, Kp7, Kp8, Kp9, KpMultiply, KpPlus, KpMinus, KpDecimal, KpDivide, F1,
      F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, LShift, RShift, LCtrl,
      RCtrl, LAlt, RAlt, Equals, Comma, Minus, Period, Semicolon, Grave, Slash,
      LeftBracket, Backslash, RightBracket, Apostrophe,
    ]
  }
}
