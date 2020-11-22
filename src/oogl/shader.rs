use super::Context;
use crate::gl_prelude::*;
use crate::math::Vec2;
use crate::prelude::*;

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum ShaderType {
    Vertex = VERTEX_SHADER,
    Fragment = FRAGMENT_SHADER,
  }
}

#[derive(Debug)]
pub struct Shader {
  ctx: Rc<Context>,
  addr: GLaddr,
  type_: ShaderType,
}

impl !Send for Shader {}
impl !Sync for Shader {}

impl Shader {
  pub fn new(ctx: Rc<Context>, type_: ShaderType) -> Self {
    let addr = unsafe { ctx.gl.CreateShader(type_.to_raw()) };
    Self { ctx, addr, type_ }
  }

  pub fn addr(&self) -> GLaddr { self.addr }
  pub fn type_(&self) -> ShaderType { self.type_ }

  pub fn set_source(&self, src: &[u8]) {
    unsafe {
      let c_src = CString::new(src).unwrap();
      self.ctx.gl.ShaderSource(self.addr, 1, &c_src.as_ptr(), ptr::null());
    }
  }

  pub fn compile(&self) -> bool {
    unsafe {
      self.ctx.gl.CompileShader(self.addr);
      let mut status = gl::FALSE as GLint;
      self.ctx.gl.GetShaderiv(self.addr, gl::COMPILE_STATUS, &mut status);
      status == gl::TRUE as GLint
    }
  }

  pub fn get_info_log(&self) -> Vec<u8> {
    unsafe {
      let mut buf_len = 0 as GLint;
      self.ctx.gl.GetShaderiv(self.addr, gl::INFO_LOG_LENGTH, &mut buf_len);
      let mut buf: Vec<u8> = Vec::with_capacity(buf_len as usize);

      if buf_len != 0 {
        let mut text_len = 0 as GLint;
        self.ctx.gl.GetShaderInfoLog(
          self.addr,
          buf_len,
          &mut text_len,
          buf.as_mut_ptr() as *mut GLchar,
        );
        buf.set_len(text_len as usize);
      }

      buf
    }
  }
}

impl Drop for Shader {
  fn drop(&mut self) {
    unsafe {
      self.ctx.gl.DeleteShader(self.addr);
    }
  }
}

#[derive(Debug)]
pub struct ShaderProgram {
  ctx: Rc<Context>,
  addr: GLaddr,
}

impl !Send for ShaderProgram {}
impl !Sync for ShaderProgram {}

impl ShaderProgram {
  pub fn new(ctx: Rc<Context>) -> Self {
    let addr = unsafe { ctx.gl.CreateProgram() };
    Self { ctx, addr }
  }

  pub fn addr(&self) -> GLaddr { self.addr }

  pub fn bind(&'_ mut self) -> ShaderProgramBinding<'_> { ShaderProgramBinding::new(self) }

  pub fn attach_shader(&self, shader: &Shader) {
    unsafe {
      self.ctx.gl.AttachShader(self.addr, shader.addr());
    }
  }

  pub fn detach_shader(&self, shader: &Shader) {
    unsafe {
      self.ctx.gl.DetachShader(self.addr, shader.addr());
    }
  }

  pub fn link(&self) -> bool {
    unsafe {
      self.ctx.gl.LinkProgram(self.addr);
      let mut status = gl::FALSE as GLint;
      self.ctx.gl.GetProgramiv(self.addr, gl::LINK_STATUS, &mut status);
      status == gl::TRUE as GLint
    }
  }

  pub fn get_info_log(&self) -> Vec<u8> {
    unsafe {
      let mut buf_len = 0 as GLint;
      self.ctx.gl.GetProgramiv(self.addr, gl::INFO_LOG_LENGTH, &mut buf_len);
      let mut buf: Vec<u8> = Vec::with_capacity(buf_len as usize);

      if buf_len != 0 {
        let mut text_len = 0 as GLint;
        self.ctx.gl.GetProgramInfoLog(
          self.addr,
          buf_len,
          &mut text_len,
          buf.as_mut_ptr() as *mut GLchar,
        );
        buf.set_len(text_len as usize);
      }

      buf
    }
  }

  pub fn get_uniform_location(&self, name: &[u8]) -> Option<GLint> {
    let c_name = CString::new(name).unwrap();
    let loc = unsafe { self.ctx.gl.GetUniformLocation(self.addr, c_name.as_ptr()) };
    if loc == -1 {
      None
    } else {
      Some(loc)
    }
  }

  pub fn get_uniform<T>(&self, name: &[u8]) -> Option<Uniform<T>>
  where
    Uniform<T>: SetUniform<T>,
  {
    Some(Uniform::new(self.get_uniform_location(name)?))
  }

  pub fn get_attribute_location(&self, name: &[u8]) -> Option<GLuint> {
    let c_name = CString::new(name).unwrap();
    let loc = unsafe { self.ctx.gl.GetAttribLocation(self.addr, c_name.as_ptr()) };
    if loc == -1 {
      None
    } else {
      Some(loc as GLuint)
    }
  }
}

impl Drop for ShaderProgram {
  fn drop(&mut self) {
    unsafe {
      self.ctx.gl.DeleteProgram(self.addr);
    }
  }
}

#[derive(Debug)]
pub struct ShaderProgramBinding<'shprg> {
  program: &'shprg mut ShaderProgram,
}

impl<'shprg> ShaderProgramBinding<'shprg> {
  fn new(program: &'shprg mut ShaderProgram) -> Self {
    unsafe {
      program.ctx.gl.UseProgram(program.addr);
    }
    Self { program }
  }

  pub fn program(&self) -> &ShaderProgram { self.program }
}

#[cfg(feature = "gl_unbind_bindings_on_drop")]
impl<'shprg> Drop for ShaderProgramBinding<'shprg> {
  fn drop(&mut self) {
    unsafe {
      self.program.ctx.gl.UseProgram(0);
    }
  }
}

#[derive(Debug)]
pub struct Uniform<T> {
  addr: GLint,
  phantom: PhantomData<*const T>,
}

impl<T> !Send for Uniform<T> {}
impl<T> !Sync for Uniform<T> {}

impl<T> Uniform<T> {
  fn new(addr: GLint) -> Self { Self { addr, phantom: PhantomData } }

  pub fn addr(&self) -> GLint { self.addr }
}

pub trait SetUniform<T> {
  fn set(&self, ctx: &Context, value: T);
}

macro_rules! impl_set_uniform {
  ($data_type:ty, $arg_pattern:pat, $gl_uniform_func_name:ident, $($gl_uniform_func_arg:expr),+) => {
    impl SetUniform<$data_type> for Uniform<$data_type> {
      fn set(&self, ctx: &Context, $arg_pattern: $data_type) {
        unsafe {
          ctx.gl.$gl_uniform_func_name(self.addr, $($gl_uniform_func_arg),+);
        }
      }
    }
  };
}

impl_set_uniform!(u32, v1, Uniform1i, v1 as i32);
impl_set_uniform!(i32, v1, Uniform1i, v1);
impl_set_uniform!(f32, v1, Uniform1f, v1);
impl_set_uniform!((f32, f32), (v1, v2), Uniform2f, v1, v2);
impl_set_uniform!((i32, i32), (v1, v2), Uniform2i, v1, v2);
impl_set_uniform!((u32, u32), (v1, v2), Uniform2i, v1 as i32, v2 as i32);
impl_set_uniform!(Vec2<f32>, Vec2 { x, y }, Uniform2f, x, y);
