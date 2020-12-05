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
    unsafe {
      self.raw_gl().ShaderSource(
        self.addr,
        1,
        &(src.as_ptr() as *const c_char),
        &(i32::try_from(src.len()).unwrap()),
      );
    }
  }

  pub fn get_source(&self) -> Vec<u8> {
    let gl = self.raw_gl();

    let mut buf_size: i32 = 0;
    unsafe { gl.GetShaderiv(self.addr, gl::SHADER_SOURCE_LENGTH, &mut buf_size) };
    let mut buf: Vec<u8> = Vec::with_capacity(buf_size as usize);

    if buf_size != 0 {
      let mut text_len: i32 = 0;
      unsafe {
        gl.GetShaderSource(self.addr, buf_size, &mut text_len, buf.as_mut_ptr() as *mut c_char);
        buf.set_len(text_len as usize);
      }
    }

    buf
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
        gl.GetShaderInfoLog(self.addr, buf_size, &mut text_len, buf.as_mut_ptr() as *mut c_char);
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
  uniform_descriptors: RefCell<HashMap<String, UniformDescriptor>>,
}

impl !Send for Program {}
impl !Sync for Program {}

pub const INACTIVE_UNIFORM_LOCATION: i32 = -1;
pub const INACTIVE_ATTRIBUTE_LOCATION: u32 = -1_i32 as u32;

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
  #[inline(always)]
  pub fn uniform_descriptors(&self) -> Ref<HashMap<String, UniformDescriptor>> {
    self.uniform_descriptors.borrow()
  }

  pub fn new(ctx: SharedContext) -> Self {
    let addr = unsafe { ctx.raw_gl().CreateProgram() };
    Self { ctx, addr, uniform_descriptors: RefCell::new(HashMap::new()) }
  }

  pub fn bind(&'_ mut self) -> ProgramBinding<'_> {
    let binding_target = &self.ctx.bound_program;
    binding_target.on_binding_created(self.addr);
    binding_target.bind_if_needed(self.raw_gl(), self.addr);
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
        gl.GetProgramInfoLog(self.addr, buf_size, &mut text_len, buf.as_mut_ptr() as *mut c_char);
        buf.set_len(text_len as usize);
      }
    }

    // TODO: see Shader::info_log

    buf
  }

  pub fn load_uniform_descriptors(&self) {
    let gl = self.raw_gl();
    let mut uniform_descriptors = self.uniform_descriptors.borrow_mut();
    uniform_descriptors.clear();
    uniform_descriptors.shrink_to_fit();

    let mut active_uniforms_count: i32 = 0;
    unsafe { gl.GetProgramiv(self.addr, gl::ACTIVE_UNIFORMS, &mut active_uniforms_count) };
    assert!(active_uniforms_count >= 0);
    uniform_descriptors.reserve(active_uniforms_count as usize);

    let mut max_name_len: i32 = 0;
    unsafe { gl.GetProgramiv(self.addr, gl::ACTIVE_UNIFORM_MAX_LENGTH, &mut max_name_len) };
    assert!(max_name_len > 0);

    for uniform_index in 0..active_uniforms_count {
      let mut name_buf = Vec::<u8>::with_capacity(max_name_len as usize);
      let mut name_len: i32 = 0;
      let mut data_type_name: u32 = 0;
      let mut data_array_len: i32 = 0;

      unsafe {
        self.raw_gl().GetActiveUniform(
          self.addr,
          uniform_index as u32,
          max_name_len,
          &mut name_len,
          &mut data_array_len,
          &mut data_type_name,
          name_buf.as_mut_ptr() as *mut c_char,
        );
      }
      assert!(name_len > 0 && data_type_name > 0 && data_array_len > 0);

      unsafe { name_buf.set_len(name_len as usize) };

      // NOTE: The string returned by `GetActiveUniform` is NUL-terminated in
      // the memory (though the terminator doesn't count towards `name_len`),
      // hence it is possible to pull off the following maneuver and skip
      // validation by `CString`.
      let location =
        unsafe { gl.GetUniformLocation(self.addr, name_buf.as_ptr() as *const c_char) };
      // Now we can safely mutate the name_buf, as the fact of its
      // NUL-termination is now useless.

      let mut name = String::from_utf8(name_buf).unwrap();

      const ARRAY_NAME_MARKER: &str = "[0]";
      let is_array = name.ends_with(ARRAY_NAME_MARKER);
      if is_array {
        name.truncate(name.len() - ARRAY_NAME_MARKER.len());
      }

      let data_type = UniformType {
        name: UniformTypeName::from_raw(data_type_name).unwrap(),
        array_len: if is_array { Some(data_array_len as u32) } else { None },
      };

      uniform_descriptors.insert(name, UniformDescriptor { location, data_type });
    }
  }

  pub fn get_uniform<T: CorrespondingUniformType>(&self, name: &str) -> Uniform<T> {
    #[inline(never)]
    #[track_caller]
    fn check_uniform_type(
      this: &Program,
      name: &str,
      corresponding_types: &'static [UniformTypeName],
      rust_type_name: &'static str,
    ) -> (i32, Option<UniformType>) {
      if let Some(descriptor) = this.uniform_descriptors.borrow().get(name) {
        let data_type = &descriptor.data_type;
        assert!(
          corresponding_types.contains(&data_type.name),
          "mismatched uniform types: values of the Rust type `{}` are not assignable to \
          the uniform `{}` with the GLSL type `{}`",
          rust_type_name,
          name,
          data_type,
        );
        (descriptor.location, Some(*data_type))
      } else {
        (INACTIVE_UNIFORM_LOCATION, None)
      }
    }

    let (location, data_type) =
      check_uniform_type(&self, name, T::CORRESPONDING_UNIFORM_TYPES, std::any::type_name::<T>());
    Uniform { location, program_addr: self.addr, data_type, phantom: PhantomData }
  }

  pub fn get_uniform_location(&self, name: &str) -> i32 {
    self.uniform_descriptors.borrow().get(name).map_or(INACTIVE_UNIFORM_LOCATION, |d| d.location)
  }

  pub fn get_attribute<T: CorrespondingAttributePtrType>(&self, name: &str) -> Attribute<T> {
    let location = self.get_attribute_location(name);
    // TODO: Add type validation for attributes as well.
    let data_type = self.get_attribute_data_type(location);
    Attribute { location, program_addr: self.addr, data_type, phantom: PhantomData }
  }

  pub fn get_attribute_location(&self, name: &str) -> u32 {
    let gl = self.raw_gl();
    let c_name = CString::new(name).unwrap();
    unsafe { gl.GetAttribLocation(self.addr, c_name.as_ptr()) as u32 }
  }

  pub fn get_attribute_data_type(&self, location: u32) -> Option<AttributeType> {
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
    assert!(data_type > 0 && data_array_len > 0);

    if data_array_len != 1 {
      todo!("array attributes");
    }

    Some(AttributeType {
      name: AttributeTypeName::from_raw(data_type)
        .unwrap_or_else(|| panic!("Unknown attribute data type: 0x{:x}", data_type)),
      array_len: data_array_len as u32,
    })
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

impl<'obj> Drop for ProgramBinding<'obj> {
  fn drop(&mut self) { self.ctx().bound_program.on_binding_dropped(); }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct UniformDescriptor {
  pub location: i32,
  pub data_type: UniformType,
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
  pub fn reflect_from(program: &Program, name: &str) -> Self { program.get_uniform(name) }
}

macro_rules! impl_set_uniform {
  (
    $data_type:ty, [$($corresponding_type_name:ident),+],
    $arg_pattern:pat, $gl_uniform_func_name:ident($($gl_uniform_func_arg:expr),+) $(,)?
  ) => {
    impl CorrespondingUniformType for $data_type {
      const CORRESPONDING_UNIFORM_TYPES: &'static [UniformTypeName] =
        &[$(UniformTypeName::$corresponding_type_name),+];
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

impl_set_uniform!(f32, [Float], x, Uniform1f(x));
impl_set_uniform!(u32, [Int], x, Uniform1i(x as i32));
impl_set_uniform!(i32, [Int], x, Uniform1i(x));
impl_set_uniform!(bool, [Bool], x, Uniform1i(x as i32));
impl_set_uniform!(Vec2<f32>, [Vec2], Vec2 { x, y }, Uniform2f(x, y));
impl_set_uniform!(Vec2<i32>, [IVec2], Vec2 { x, y }, Uniform2i(x, y));
impl_set_uniform!(Vec2<u32>, [IVec2], Vec2 { x, y }, Uniform2i(x as i32, y as i32));
impl_set_uniform!(Vec2<bool>, [BVec2], Vec2 { x, y }, Uniform2i(x as i32, y as i32));
impl_set_uniform!(Color<f32>, [Vec4], Color { r, g, b, a }, Uniform4f(r, g, b, a));
impl_set_uniform!(crate::TextureUnit, [Sampler2D, SamplerCube], unit, Uniform1i(unit.id() as i32));

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct UniformType {
  pub name: UniformTypeName,
  pub array_len: Option<u32>,
}

impl UniformType {
  #[inline(always)]
  pub fn is_array(&self) -> bool { self.array_len.is_some() }
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
}

impl fmt::Display for UniformType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.name)?;
    if let Some(array_len) = self.array_len {
      write!(f, "[{}]", array_len)?;
    }
    Ok(())
  }
}

impl fmt::Display for UniformTypeName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use UniformTypeName::*;
    write!(
      f,
      "{}",
      match self {
        Float => "float",
        Vec2 => "vec2",
        Vec3 => "vec3",
        Vec4 => "vec4",
        Int => "int",
        IVec2 => "ivec2",
        IVec3 => "ivec3",
        IVec4 => "ivec4",
        Bool => "bool",
        BVec2 => "bvec2",
        BVec3 => "bvec3",
        BVec4 => "bvec4",
        Mat2 => "mat2",
        Mat3 => "mat3",
        Mat4 => "mat4",
        Sampler2D => "sampler2D",
        SamplerCube => "samplerCube",
      }
    )
  }
}

pub trait CorrespondingUniformType {
  const CORRESPONDING_UNIFORM_TYPES: &'static [UniformTypeName];
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
  pub fn reflect_from(program: &Program, name: &str) -> Self { program.get_attribute(name) }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct AttributeType {
  pub name: AttributeTypeName,
  pub array_len: u32,
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
          $($field_name: <$field_type>::reflect_from(program, stringify!($field_name))),+
        }
      }
    }
  };
}
