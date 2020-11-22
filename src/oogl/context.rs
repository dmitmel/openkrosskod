use super::debug;
use crate::gl_prelude::*;
use crate::prelude::*;

pub struct Context {
  pub gl: Gles2,
}

impl Context {
  pub fn load_with<F>(load_fn: F) -> Self
  where
    F: FnMut(&'static str) -> *const GLvoid,
  {
    let gl = Gles2::load_with(load_fn);

    if gl.DebugMessageCallback.is_loaded() {
      unsafe {
        gl.Enable(gl::DEBUG_OUTPUT);
        gl.DebugMessageCallback(Some(debug::internal_debug_message_callback), ptr::null());
      }
    }

    Self { gl }
  }

  pub fn clear_color(&self, r: GLfloat, g: GLfloat, b: GLfloat, a: GLfloat) {
    unsafe {
      self.gl.ClearColor(r, g, b, a);
      self.gl.Clear(gl::COLOR_BUFFER_BIT);
    }
  }

  pub fn set_viewport(&self, x: GLint, y: GLint, w: GLsizei, h: GLsizei) {
    unsafe {
      self.gl.Viewport(x, y, w, h);
    }
  }
}

impl fmt::Debug for Context {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("Context").finish_non_exhaustive()
  }
}
