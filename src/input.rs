use cardboard_math::*;
use prelude_plus::*;
use sdl2::keyboard::Scancode;

#[derive(Debug)]
pub struct InputState {
  pub mouse_pos: Vec2f,
  pub keyboard: KeyboardStateTable,
}

impl InputState {
  pub fn new() -> Self { Self { mouse_pos: vec2n(0.0), keyboard: KeyboardStateTable::new() } }

  pub fn is_key_down(&self, scancode: Scancode) -> bool {
    self.keyboard.get(scancode).map_or(false, |b| *b)
  }

  #[inline(always)]
  pub fn is_key_up(&self, scancode: Scancode) -> bool { !self.is_key_down(scancode) }
}

macro_rules! generate_keyboard_state_table {
  ($($scancode:ident),+ $(,)?) => {
    #[allow(non_snake_case)]
    #[derive(Debug)]
    pub struct KeyboardStateTable {
      $(pub $scancode: bool),+
    }

    impl KeyboardStateTable {
      pub fn new() -> Self { Self { $($scancode: false),+ } }

      pub fn get(&self, scancode: Scancode) -> Option<&'_ bool> {
        Some(match scancode {
          $(Scancode::$scancode => &self.$scancode,)+
          _ => return None,
        })
      }

      pub fn get_mut(&mut self, scancode: Scancode) -> Option<&'_ mut bool> {
        Some(match scancode {
          $(Scancode::$scancode => &mut self.$scancode,)+
          _ => return None,
        })
      }

      pub fn set(&mut self, scancode: Scancode, new_state: bool) {
        if let Some(state) = self.get_mut(scancode) {
          *state = new_state;
        }
      }
    }
  };
}

generate_keyboard_state_table![Q, W, S, Up, Down];
