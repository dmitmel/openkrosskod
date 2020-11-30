use crate::impl_prelude::*;
use crate::CorrespondingAttributePtrType;
use cardboard_math::*;
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

impl Object for Shader {
  const DEBUG_TYPE_IDENTIFIER: u32 = gl::SHADER;

  #[inline(always)]
  fn ctx(&self) -> &SharedContext { &self.ctx }
  #[inline(always)]
  fn addr(&self) -> u32 { self.addr }
  #[inline(always)]
  fn internal_state_acquired(&self) -> bool { true }
}

impl Shader {
  #[inline(always)]
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
    let mut status = gl::FALSE as i32;
    unsafe { gl.GetShaderiv(self.addr, gl::COMPILE_STATUS, &mut status) };
    status == gl::TRUE as i32
  }

  pub fn get_info_log(&self) -> Vec<u8> {
    let gl = self.raw_gl();

    let mut buf_size: i32 = 0;
    unsafe { gl.GetShaderiv(self.addr, gl::INFO_LOG_LENGTH, &mut buf_size) };
    let mut buf: Vec<u8> = Vec::with_capacity(buf_size as usize);

    if buf_size != 0 {
      let mut text_len: i32 = 0;
      unsafe {
        gl.GetShaderInfoLog(self.addr, buf_size, &mut text_len, buf.as_mut_ptr() as *mut i8);
        buf.set_len(text_len as usize);
      }
    }

    // TODO: Parse error messages and print corresponding source code lines. See
    // <https://github.com/krux02/opengl-sandbox/blob/dbb100bb0bbad96e53b1844c5a5ab7be1673e706/fancygl/glwrapper.nim#L915-L946>

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

impl Object for Program {
  const DEBUG_TYPE_IDENTIFIER: u32 = gl::PROGRAM;

  #[inline(always)]
  fn ctx(&self) -> &SharedContext { &self.ctx }
  #[inline(always)]
  fn addr(&self) -> u32 { self.addr }
  #[inline(always)]
  fn internal_state_acquired(&self) -> bool { true }
}

impl Program {
  pub fn new(ctx: SharedContext) -> Self {
    let addr = unsafe { ctx.raw_gl().CreateProgram() };
    Self { ctx, addr }
  }

  pub fn bind(&'_ mut self) -> ProgramBinding<'_> {
    self.ctx.bound_program.bind_if_needed(self.ctx.raw_gl(), self.addr, &mut false);
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
    let mut status = gl::FALSE as i32;
    unsafe { gl.GetProgramiv(self.addr, gl::LINK_STATUS, &mut status) };
    status == gl::TRUE as i32
  }

  pub fn get_info_log(&self) -> Vec<u8> {
    let gl = self.raw_gl();

    let mut buf_size: i32 = 0;
    unsafe { gl.GetProgramiv(self.addr, gl::INFO_LOG_LENGTH, &mut buf_size) };
    let mut buf: Vec<u8> = Vec::with_capacity(buf_size as usize);

    if buf_size != 0 {
      let mut text_len: i32 = 0;
      unsafe {
        gl.GetProgramInfoLog(self.addr, buf_size, &mut text_len, buf.as_mut_ptr() as *mut i8);
        buf.set_len(text_len as usize);
      }
    }

    // TODO: see Shader::info_log

    buf
  }

  pub fn get_uniform<T: CorrespondingUniformType>(&self, name: &[u8]) -> Uniform<T> {
    // TODO: extract into a different function to avoid code bloat due to
    // monomorphization
    let location = self.get_uniform_location(name);
    let data_type = self.get_uniform_type(location);
    if let Some(data_type) = &data_type {
      assert!(T::CORRESPONDING_UNIFORM_TYPE.is_assignable_to(data_type.name));
    }
    Uniform { location, program_addr: self.addr, data_type, phantom: PhantomData }
  }

  pub fn get_uniform_location(&self, name: &[u8]) -> i32 {
    let gl = self.raw_gl();
    let c_name = CString::new(name).unwrap();
    unsafe { gl.GetUniformLocation(self.addr, c_name.as_ptr()) }
  }

  pub fn get_uniform_type(&self, location: i32) -> Option<UniformType> {
    if location == INACTIVE_UNIFORM_LOCATION {
      return None;
    }

    let mut data_type = 0;
    let mut data_array_len = 0;
    unsafe {
      self.raw_gl().GetActiveUniform(
        self.addr,
        location as u32,
        0,               // name buffer size
        ptr::null_mut(), // name length without the \0
        &mut data_array_len,
        &mut data_type,
        ptr::null_mut(), // name buffer
      )
    };
    assert!(data_type > 0);
    assert!(data_array_len > 0);

    if data_array_len != 1 {
      todo!("array uniforms");
    }

    Some(UniformType {
      name: UniformTypeName::from_raw(data_type)
        .unwrap_or_else(|| panic!("Unknown uniform data type: 0x{:x}", data_type)),
      len: data_array_len as u32,
    })
  }

  pub fn get_attribute_location(&self, name: &[u8]) -> u32 {
    let gl = self.raw_gl();
    let c_name = CString::new(name).unwrap();
    unsafe { gl.GetAttribLocation(self.addr, c_name.as_ptr()) as u32 }
  }

  pub fn get_attribute_type(&self, location: u32) -> Option<AttributeType> {
    if location == INACTIVE_ATTRIBUTE_LOCATION {
      return None;
    }

    let mut data_type = 0;
    let mut data_array_len = 0;
    unsafe {
      self.raw_gl().GetActiveAttrib(
        self.addr,
        location,
        0,               // name buffer size
        ptr::null_mut(), // name length without the \0
        &mut data_array_len,
        &mut data_type,
        ptr::null_mut(), // name buffer
      )
    };
    assert!(data_type > 0);
    assert!(data_array_len > 0);

    if data_array_len != 1 {
      todo!("array attributes");
    }

    Some(AttributeType {
      name: AttributeTypeName::from_raw(data_type)
        .unwrap_or_else(|| panic!("Unknown attribute data type: 0x{:x}", data_type)),
      len: data_array_len as u32,
    })
  }

  pub fn get_attribute<T: CorrespondingAttributePtrType>(&self, name: &[u8]) -> Attribute<T> {
    let location = self.get_attribute_location(name);
    // TODO: See get_uniform
    let data_type = self.get_attribute_type(location);
    Attribute { location, program_addr: self.addr, data_type, phantom: PhantomData }
  }
}

impl Drop for Program {
  fn drop(&mut self) { unsafe { self.raw_gl().DeleteProgram(self.addr) }; }
}

#[derive(Debug)]
pub struct ProgramBinding<'obj> {
  program: &'obj mut Program,
}

impl<'obj> ObjectBinding<'obj, Program> for ProgramBinding<'obj> {
  #[inline(always)]
  fn object(&self) -> &Program { &self.program }

  fn unbind_completely(self) { self.ctx().bound_program.unbind_unconditionally(self.raw_gl()); }
}

#[derive(Debug)]
pub struct Uniform<T: CorrespondingUniformType> {
  location: i32,
  program_addr: u32,
  data_type: Option<UniformType>,
  phantom: PhantomData<*mut T>,
}

impl<T> !Send for Uniform<T> {}
impl<T> !Sync for Uniform<T> {}

impl<T: CorrespondingUniformType> Uniform<T> {
  #[inline(always)]
  pub fn location(&self) -> i32 { self.location }
  #[inline(always)]
  pub fn is_active(&self) -> bool { self.location != INACTIVE_UNIFORM_LOCATION }
  #[inline(always)]
  pub fn program_addr(&self) -> u32 { self.program_addr }
  #[inline(always)]
  pub fn data_type(&self) -> &Option<UniformType> { &self.data_type }

  #[inline(always)]
  pub fn reflect_from(program: &Program, name: &[u8]) -> Self
  where
    T: CorrespondingUniformType,
  {
    program.get_uniform(name)
  }
}

macro_rules! impl_set_uniform {
  (
    $data_type:ty, $corresponding_type_name:ident,
    |$arg_pattern:pat| $gl_uniform_func_name:ident($($gl_uniform_func_arg:expr),+) $(,)?
  ) => {
    impl CorrespondingUniformType for $data_type {
      const CORRESPONDING_UNIFORM_TYPE: UniformTypeName =
        UniformTypeName::$corresponding_type_name;
    }

    impl Uniform<$data_type> {
      pub fn set(&self, program_binding: &ProgramBinding<'_>, value: $data_type) {
        // For some reason applying the pattern directly in the method
        // signature causes rust-analyzer to report false-positive "argument
        // count mismatch" errors. Well, nevermind, I'll do that with a local
        // variable binding then (`value` is inaccessible from the macro
        // invocation thanks to macro hygiene).
        let $arg_pattern = value;
        let program = &program_binding.program;
        assert_eq!(self.program_addr, program.addr);
        unsafe { program.raw_gl().$gl_uniform_func_name(self.location, $($gl_uniform_func_arg),+) };
      }
    }
  };
}

impl_set_uniform!(f32, Float, |x| Uniform1f(x));
impl_set_uniform!(u32, Int, |x| Uniform1i(x as i32));
impl_set_uniform!(i32, Int, |x| Uniform1i(x));
impl_set_uniform!(bool, Bool, |x| Uniform1i(x as i32));
impl_set_uniform!(Vec2<f32>, Vec2, |Vec2 { x, y }| Uniform2f(x, y));
impl_set_uniform!(Vec2<i32>, IVec2, |Vec2 { x, y }| Uniform2i(x, y));
impl_set_uniform!(Vec2<u32>, IVec2, |Vec2 { x, y }| Uniform2i(x as i32, y as i32));
impl_set_uniform!(Vec2<bool>, BVec2, |Vec2 { x, y }| Uniform2i(x as i32, y as i32));
impl_set_uniform!(Color<f32>, Vec4, |Color { r, g, b, a }| Uniform4f(r, g, b, a));

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct UniformType {
  pub name: UniformTypeName,
  pub len: u32,
}

gl_enum!({
  pub enum UniformTypeName {
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

impl UniformTypeName {
  pub fn components(self) -> u8 {
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

  pub fn is_assignable_to(self, other: UniformTypeName) -> bool {
    use UniformTypeName::*;
    match (self, other) {
      _ if self == other => true,
      (Int, Sampler2D) => true,
      _ => false,
    }
  }
}

pub trait CorrespondingUniformType {
  const CORRESPONDING_UNIFORM_TYPE: UniformTypeName;
}

#[derive(Debug)]
pub struct Attribute<T: CorrespondingAttributePtrType> {
  location: u32,
  program_addr: u32,
  data_type: Option<AttributeType>,
  phantom: PhantomData<*mut T>,
}

impl<T> !Send for Attribute<T> {}
impl<T> !Sync for Attribute<T> {}

impl<T: CorrespondingAttributePtrType> Attribute<T> {
  #[inline(always)]
  pub fn location(&self) -> u32 { self.location }
  #[inline(always)]
  pub fn is_active(&self) -> bool { self.location != INACTIVE_ATTRIBUTE_LOCATION }
  #[inline(always)]
  pub fn program_addr(&self) -> u32 { self.program_addr }
  #[inline(always)]
  pub fn data_type(&self) -> &Option<AttributeType> { &self.data_type }

  #[inline(always)]
  pub fn reflect_from(program: &Program, name: &[u8]) -> Self { program.get_attribute(name) }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct AttributeType {
  pub name: AttributeTypeName,
  pub len: u32,
}

gl_enum!({
  pub enum AttributeTypeName {
    Float = FLOAT,
    Vec2 = FLOAT_VEC2,
    Vec3 = FLOAT_VEC3,
    Vec4 = FLOAT_VEC4,
    // Mat2 = FLOAT_MAT2,
    // Mat3 = FLOAT_MAT3,
    // Mat4 = FLOAT_MAT4,
  }
});

impl AttributeTypeName {
  pub fn components(&self) -> u8 {
    match self {
      Self::Float => 1,
      Self::Vec2 => 2,
      Self::Vec3 => 3,
      Self::Vec4 => 4,
      // Self::Mat2 => 2 * 2,
      // Self::Mat3 => 3 * 3,
      // Self::Mat4 => 4 * 4,
    }
  }
}

#[macro_export]
macro_rules! program_reflection_block {
  // a wrapper for autoformatting purposes
  ({$($tt:tt)+}) => { $crate::program_reflection_block! { $($tt)+ } };

  (
    $(#[$struct_meta:meta])* $struct_visibility:vis struct $struct_name:ident {
      $($(#[$field_meta:meta])* $field_visibility:vis $field_name:ident: $field_type:ty),+ $(,)?
    }
  ) => {
    $(#[$struct_meta])*
    $struct_visibility struct $struct_name {
      $($(#[$field_meta])* $field_visibility $field_name: $field_type),+
    }

    impl $struct_name {
      $struct_visibility fn new(program: &$crate::Program) -> Self {
        Self {
          $($field_name: <$field_type>::reflect_from(
            program, stringify!($field_name).as_bytes())),+
        }
      }
    }
  };
}
