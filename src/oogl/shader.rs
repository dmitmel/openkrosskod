use super::{RawGL, SharedContext};
use crate::math::{Color, Vec2};
use ::gl::prelude::*;
use prelude_plus::*;

gl_enum!({
  pub enum ShaderType {
    Vertex = VERTEX_SHADER,
    Fragment = FRAGMENT_SHADER,
  }
});

#[derive(Debug)]
pub struct Shader {
  ctx: SharedContext,
  addr: u32,
  type_: ShaderType,
}

impl !Send for Shader {}
impl !Sync for Shader {}

impl Shader {
  pub fn ctx(&self) -> &SharedContext { &self.ctx }
  pub fn raw_gl(&self) -> &RawGL { self.ctx.raw_gl() }
  pub fn addr(&self) -> u32 { self.addr }
  pub fn type_(&self) -> ShaderType { self.type_ }

  pub fn new(ctx: SharedContext, type_: ShaderType) -> Self {
    let addr = unsafe { ctx.raw_gl().CreateShader(type_.as_raw()) };
    Self { ctx, addr, type_ }
  }

  pub fn set_source(&self, src: &[u8]) {
    let gl = self.raw_gl();

    let c_src = CString::new(src).unwrap();
    unsafe { gl.ShaderSource(self.addr, 1, &c_src.as_ptr(), ptr::null()) };
  }

  pub fn compile(&self) -> bool {
    let gl = self.raw_gl();

    unsafe { gl.CompileShader(self.addr) };
    let mut status = gl::FALSE as GLint;
    unsafe { gl.GetShaderiv(self.addr, gl::COMPILE_STATUS, &mut status) };
    status == gl::TRUE as GLint
  }

  pub fn get_info_log(&self) -> Vec<u8> {
    let gl = self.raw_gl();

    let mut buf_len = 0 as GLint;
    unsafe { gl.GetShaderiv(self.addr, gl::INFO_LOG_LENGTH, &mut buf_len) };
    let mut buf: Vec<u8> = Vec::with_capacity(buf_len as usize);

    if buf_len != 0 {
      let mut text_len = 0 as GLint;
      unsafe {
        gl.GetShaderInfoLog(self.addr, buf_len, &mut text_len, buf.as_mut_ptr() as *mut GLchar);
        buf.set_len(text_len as usize);
      }
    }

    buf
  }
}

impl Drop for Shader {
  fn drop(&mut self) { unsafe { self.raw_gl().DeleteShader(self.addr) }; }
}

#[derive(Debug)]
pub struct Program {
  ctx: SharedContext,
  addr: u32,
}

impl !Send for Program {}
impl !Sync for Program {}

pub const INACTIVE_UNIFORM_LOCATION: i32 = -1;
pub const INACTIVE_ATTRIBUTE_LOCATION: u32 = -1i32 as u32;

impl Program {
  pub fn ctx(&self) -> &SharedContext { &self.ctx }
  pub fn raw_gl(&self) -> &RawGL { self.ctx.raw_gl() }
  pub fn addr(&self) -> u32 { self.addr }

  pub fn new(ctx: SharedContext) -> Self {
    let addr = unsafe { ctx.raw_gl().CreateProgram() };
    Self { ctx, addr }
  }

  pub fn bind(&'_ mut self) -> ProgramBinding<'_> {
    self.ctx.bound_program.bind_if_needed(self.ctx.raw_gl(), self.addr);
    ProgramBinding { program: self }
  }

  pub fn attach_shader(&self, shader: &Shader) {
    unsafe { self.raw_gl().AttachShader(self.addr, shader.addr) };
  }

  pub fn detach_shader(&self, shader: &Shader) {
    unsafe { self.raw_gl().DetachShader(self.addr, shader.addr) };
  }

  pub fn link(&self) -> bool {
    let gl = self.raw_gl();

    unsafe { gl.LinkProgram(self.addr) };
    let mut status = gl::FALSE as GLint;
    unsafe { gl.GetProgramiv(self.addr, gl::LINK_STATUS, &mut status) };
    status == gl::TRUE as GLint
  }

  pub fn get_info_log(&self) -> Vec<u8> {
    let gl = self.raw_gl();

    let mut buf_len = 0 as GLint;
    unsafe { gl.GetProgramiv(self.addr, gl::INFO_LOG_LENGTH, &mut buf_len) };
    let mut buf: Vec<u8> = Vec::with_capacity(buf_len as usize);

    if buf_len != 0 {
      let mut text_len = 0 as GLint;
      unsafe {
        gl.GetProgramInfoLog(self.addr, buf_len, &mut text_len, buf.as_mut_ptr() as *mut GLchar);
        buf.set_len(text_len as usize);
      }
    }

    buf
  }

  pub fn get_uniform_location(&self, name: &[u8]) -> i32 {
    let gl = self.raw_gl();
    let c_name = CString::new(name).unwrap();
    unsafe { gl.GetUniformLocation(self.addr, c_name.as_ptr()) }
  }

  pub fn get_uniform<T>(&self, name: &[u8]) -> Uniform<T> {
    let gl = self.raw_gl();
    let location = self.get_uniform_location(name);

    let data_type: Option<(UniformType, u32)> = if location != INACTIVE_UNIFORM_LOCATION {
      let mut data_type = 0;
      let mut data_array_len = 0;
      unsafe {
        gl.GetActiveUniform(
          self.addr,
          location as u32,
          0,               // name buffer size
          ptr::null_mut(), // name length without the \0
          &mut data_array_len,
          &mut data_type,
          ptr::null_mut(), // name buffer (we already know the name)
        )
      };
      assert!(data_type > 0);
      assert!(data_array_len > 0);

      if data_array_len != 1 {
        todo!("array uniforms");
      }

      Some((
        UniformType::from_raw(data_type)
          .unwrap_or_else(|| panic!("Unknown uniform data type: 0x{:x}", data_type)),
        data_array_len as u32,
      ))
    } else {
      None
    };

    Uniform { location, program_addr: self.addr, data_type, phantom: PhantomData }
  }

  pub fn get_attribute_location(&self, name: &[u8]) -> u32 {
    let gl = self.raw_gl();
    let c_name = CString::new(name).unwrap();
    unsafe { gl.GetAttribLocation(self.addr, c_name.as_ptr()) as u32 }
  }

  pub fn get_attribute(&self, name: &[u8]) -> Attribute {
    let gl = self.raw_gl();
    let location = self.get_attribute_location(name);

    let data_type: Option<(AttributeType, u32)> = if location != INACTIVE_ATTRIBUTE_LOCATION {
      let mut data_type = 0;
      let mut data_array_len = 0;
      unsafe {
        gl.GetActiveAttrib(
          self.addr,
          location,
          0,               // name buffer size
          ptr::null_mut(), // name length without the \0
          &mut data_array_len,
          &mut data_type,
          ptr::null_mut(), // name buffer (we already know the name)
        )
      };
      assert!(data_type > 0);
      assert!(data_array_len > 0);

      if data_array_len != 1 {
        todo!("array attributes");
      }

      Some((
        AttributeType::from_raw(data_type)
          .unwrap_or_else(|| panic!("Unknown attribute data type: 0x{:x}", data_type)),
        data_array_len as u32,
      ))
    } else {
      None
    };

    Attribute { location, program_addr: self.addr, data_type }
  }
}

impl Drop for Program {
  fn drop(&mut self) { unsafe { self.raw_gl().DeleteProgram(self.addr) }; }
}

#[derive(Debug)]
pub struct ProgramBinding<'program> {
  program: &'program mut Program,
}

impl<'program> ProgramBinding<'program> {
  pub fn ctx(&self) -> &SharedContext { &self.program.ctx }
  pub fn raw_gl(&self) -> &RawGL { self.program.ctx.raw_gl() }
  pub fn program(&self) -> &Program { &self.program }

  pub fn unbind_completely(self) {
    self.ctx().bound_program.unbind_unconditionally(self.raw_gl());
  }
}

#[derive(Debug)]
pub struct Uniform<T> {
  location: i32,
  program_addr: u32,
  data_type: Option<(UniformType, u32)>,
  phantom: PhantomData<*mut T>,
}

impl<T> !Send for Uniform<T> {}
impl<T> !Sync for Uniform<T> {}

impl<T> Uniform<T> {
  pub fn location(&self) -> i32 { self.location }
  pub fn is_active(&self) -> bool { self.location != INACTIVE_UNIFORM_LOCATION }
  pub fn program_addr(&self) -> u32 { self.program_addr }
  pub fn data_type(&self) -> &Option<(UniformType, u32)> { &self.data_type }
}

macro_rules! impl_set_uniform {
  ($data_type:ty, $arg_pattern:pat, $gl_uniform_func_name:ident($($gl_uniform_func_arg:expr),+)) => {
    impl Uniform<$data_type> {
      pub fn set(&self, program_binding: &ProgramBinding<'_>, $arg_pattern: $data_type) {
        let program = &program_binding.program;
        let gl = program.raw_gl();
        assert_eq!(self.program_addr, program.addr);
        unsafe { gl.$gl_uniform_func_name(self.location, $($gl_uniform_func_arg),+) };
      }
    }
  };
}

impl_set_uniform!(u32, x, Uniform1i(x as i32));
impl_set_uniform!(i32, x, Uniform1i(x));
impl_set_uniform!(f32, x, Uniform1f(x));
impl_set_uniform!((f32, f32), (x, y), Uniform2f(x, y));
impl_set_uniform!((i32, i32), (x, y), Uniform2i(x, y));
impl_set_uniform!((u32, u32), (x, y), Uniform2i(x as i32, y as i32));
impl_set_uniform!(Vec2<f32>, Vec2 { x, y }, Uniform2f(x, y));
impl_set_uniform!(Color<f32>, Color { r, g, b, a }, Uniform4f(r, g, b, a));
impl_set_uniform!(super::Texture2DBinding<'_>, tex, Uniform1i(tex.unit() as i32));

gl_enum!({
  pub enum UniformType {
    Float = FLOAT,
    Vec2 = FLOAT_VEC2,
    Vec3 = FLOAT_VEC3,
    Vec4 = FLOAT_VEC4,
    Int = INT,
    IVec2 = INT_VEC2,
    IVec3 = INT_VEC3,
    IVec4 = INT_VEC4,
    Bool = BOOL,
    BVec2 = BOOL_VEC2,
    BVec3 = BOOL_VEC3,
    BVec4 = BOOL_VEC4,
    Mat2 = FLOAT_MAT2,
    Mat3 = FLOAT_MAT3,
    Mat4 = FLOAT_MAT4,
    Sampler2D = SAMPLER_2D,
    SamplerCube = SAMPLER_CUBE,
  }
});

impl UniformType {
  pub fn components(&self) -> u8 {
    match self {
      Self::Float | Self::Int | Self::Bool => 1,
      Self::Vec2 | Self::IVec2 | Self::BVec2 => 2,
      Self::Vec3 | Self::IVec3 | Self::BVec3 => 3,
      Self::Vec4 | Self::IVec4 | Self::BVec4 => 4,
      Self::Mat2 => 2 * 2,
      Self::Mat3 => 3 * 3,
      Self::Mat4 => 4 * 4,
      Self::Sampler2D | Self::SamplerCube => 1,
    }
  }
}

#[derive(Debug)]
pub struct Attribute {
  location: u32,
  program_addr: u32,
  data_type: Option<(AttributeType, u32)>,
}

impl !Send for Attribute {}
impl !Sync for Attribute {}

impl Attribute {
  pub fn location(&self) -> u32 { self.location }
  pub fn is_active(&self) -> bool { self.location != INACTIVE_ATTRIBUTE_LOCATION }
  pub fn program_addr(&self) -> u32 { self.program_addr }
  pub fn data_type(&self) -> &Option<(AttributeType, u32)> { &self.data_type }

  #[inline(always)]
  pub fn to_pointer(&self, config: super::AttributePtrConfig) -> super::AttributePtr {
    if let Some((data_type, data_array_len)) = self.data_type {
      assert_eq!(config.len as u32, data_type.components() as u32 * data_array_len);
    }
    super::AttributePtr::new(self.location, config)
  }
}

gl_enum!({
  pub enum AttributeType {
    Float = FLOAT,
    Vec2 = FLOAT_VEC2,
    Vec3 = FLOAT_VEC3,
    Vec4 = FLOAT_VEC4,
    // Mat2 = FLOAT_MAT2,
    // Mat3 = FLOAT_MAT3,
    // Mat4 = FLOAT_MAT4,
  }
});

impl AttributeType {
  pub fn components(&self) -> u8 {
    match self {
      Self::Float => 1,
      Self::Vec2 => 2,
      Self::Vec3 => 3,
      Self::Vec4 => 4,
    }
  }
}
