use crate::impl_prelude::*;
use cardboard_math::*;
use prelude_plus::*;

pub type RawGL = Gles2;

pub type SharedContext = Rc<Context>;

pub struct Context {
  raw_gl: RawGL,
  sdl_gl_context: sdl2::video::GLContext,
  capabilities: ContextCapabilities,

  pub(crate) bound_program: BindingTarget<ProgramBindingTarget>,
  pub(crate) bound_vertex_buffer: BindingTarget<BufferBindingTarget>,
  pub(crate) bound_element_buffer: BindingTarget<BufferBindingTarget>,
  pub(crate) bound_texture_2d: BindingTarget<TextureBindingTarget>,
  pub(crate) bound_framebuffer: BindingTarget<FramebufferBindingTarget>,

  active_texture_unit: Cell<u32>,
  free_texture_units: UnsafeCell<Vec<u32>>,
}

impl !Send for Context {}
impl !Sync for Context {}

impl Context {
  #[inline(always)]
  pub fn raw_gl(&self) -> &RawGL { &self.raw_gl }
  #[inline(always)]
  pub fn sdl_gl_context(&self) -> &sdl2::video::GLContext { &self.sdl_gl_context }
  #[inline(always)]
  pub fn capabilities(&self) -> &ContextCapabilities { &self.capabilities }

  pub fn load_with(
    video_subsystem: &sdl2::VideoSubsystem,
    sdl_gl_context: sdl2::video::GLContext,
  ) -> Self {
    let gl = Gles2::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const c_void);

    // This has to be done first!!!
    crate::debug::init(&gl);

    let capabilities = ContextCapabilities::load(&gl);
    assert!(capabilities.extensions.gl_oes_texture_npot);

    // TODO: put this into some kind of GL configuration struct
    const MAX_USABLE_TEXTURE_UNITS: u32 = 16;
    let free_texture_units = UnsafeCell::new(
      (0..capabilities.max_texture_units.min(MAX_USABLE_TEXTURE_UNITS)).rev().collect(),
    );

    Self {
      raw_gl: gl,
      sdl_gl_context,
      capabilities,

      // programs are a special case, the binding target value doesn't matter
      // because there is no such thing as binding a program to a target
      bound_program: BindingTarget::new(gl::NONE),
      bound_vertex_buffer: BindingTarget::new(crate::BindBufferTarget::Vertex.as_raw()),
      bound_element_buffer: BindingTarget::new(crate::BindBufferTarget::Element.as_raw()),
      bound_texture_2d: BindingTarget::new(crate::BindTextureTarget::Texture2D.as_raw()),
      bound_framebuffer: BindingTarget::new(crate::BindFramebufferTarget::Default.as_raw()),

      active_texture_unit: Cell::new(0),
      free_texture_units,
    }
  }

  #[inline(always)]
  pub fn active_texture_unit(&self) -> u32 { self.active_texture_unit.get() }

  pub(crate) unsafe fn set_active_texture_unit(&self, unit: u32) {
    self.raw_gl.ActiveTexture(gl::TEXTURE0 + unit as u32);
    self.active_texture_unit.set(unit);
  }

  pub(crate) fn alloc_texture_unit(&self) -> u32 {
    unsafe { &mut *self.free_texture_units.get() }.pop().expect("no free texture units left")
  }

  pub(crate) fn free_texture_unit(&self, unit: u32) {
    unsafe { &mut *self.free_texture_units.get() }.push(unit);
  }

  pub fn set_clear_color(&self, color: Colorf) {
    unsafe { self.raw_gl.ClearColor(color.r, color.g, color.b, color.a) };
  }

  pub fn clear(&self, flags: ClearFlags) { unsafe { self.raw_gl.Clear(flags.bits()) }; }

  pub fn set_viewport(&self, pos: Vec2i32, size: Vec2i32) {
    unsafe { self.raw_gl.Viewport(pos.x, pos.y, size.x, size.y) };
  }

  unsafe fn set_feature_enabled(&self, feature: u32, enabled: bool) {
    if enabled {
      self.raw_gl.Enable(feature);
    } else {
      self.raw_gl.Disable(feature);
    }
  }

  #[inline(always)]
  pub fn set_blending_enabled(&self, enabled: bool) {
    unsafe { self.set_feature_enabled(gl::BLEND, enabled) };
  }

  pub fn set_blending_factors(&self, src: BlendingFactor, dest: BlendingFactor) {
    unsafe { self.raw_gl.BlendFunc(src.as_raw(), dest.as_raw()) };
  }

  pub fn set_blending_equation(&self, equation: BlendingEquation) {
    unsafe { self.raw_gl.BlendEquation(equation.as_raw()) };
  }

  pub fn set_blending_color(&self, color: Colorf) {
    unsafe { self.raw_gl.BlendColor(color.r, color.g, color.b, color.a) };
  }

  pub fn release_shader_compiler(&self) { unsafe { self.raw_gl.ReleaseShaderCompiler() }; }
}

// TODO: Implement Debug properly with wrappers for non-debuggable stuff
impl fmt::Debug for Context {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Context#<{:p}>", unsafe { self.sdl_gl_context.raw() })
  }
}

#[derive(Debug)]
pub(crate) struct BindingTarget<T> {
  target: u32,
  bound_addr: Cell<u32>,
  is_binding_alive: Cell<bool>,
  phantom: PhantomData<*mut T>,
}

#[allow(dead_code)]
impl<T> BindingTarget<T> {
  #[inline(always)]
  pub(crate) fn target(&self) -> u32 { self.target }
  #[inline(always)]
  pub(crate) fn bound_addr(&self) -> u32 { self.bound_addr.get() }
  #[inline(always)]
  pub(crate) fn is_anything_bound(&self) -> bool { self.bound_addr() != 0 }
  #[inline(always)]
  pub(crate) fn is_binding_alive(&self) -> bool { self.is_binding_alive.get() }

  pub(crate) fn new(target: u32) -> Self {
    Self {
      target,
      bound_addr: Cell::new(0),
      is_binding_alive: Cell::new(false),
      phantom: PhantomData,
    }
  }

  #[inline]
  pub(crate) fn on_binding_created(&self, addr: u32) {
    if self.is_binding_alive.get() {
      #[inline(never)]
      #[cold]
      #[track_caller]
      fn on_binding_created_fail(addr_new: u32, addr_old: u32) {
        panic!(
          "attempt to bind object #{} while the binding of object #{} is still alive",
          addr_new, addr_old,
        );
      }
      on_binding_created_fail(addr, self.bound_addr.get());
    }
    self.is_binding_alive.set(true);
  }

  #[inline(always)]
  pub(crate) fn on_binding_dropped(&self) { self.is_binding_alive.set(false); }
}

macro_rules! impl_binding_target_state {
  ($target_enum:ident, $gl_bind_fn:ident ($($target:ident)?)) => {
    #[derive(Debug)]
    pub(crate) enum $target_enum {}

    #[allow(dead_code)]
    impl BindingTarget<$target_enum> {
      #[inline]
      pub(crate) fn bind_unconditionally(&self, gl: &RawGL, addr: u32) {
        unsafe { gl.$gl_bind_fn($(self.$target, )? addr) };
        self.bound_addr.set(addr);
      }

      #[inline(always)]
      pub(crate) fn unbind_unconditionally(&self, gl: &RawGL) {
        self.bind_unconditionally(gl, 0)
      }

      #[inline]
      pub(crate) fn bind_if_needed(&self, gl: &RawGL, addr: u32) {
        if self.bound_addr.get() != addr {
          self.bind_unconditionally(gl, addr);
        }
      }
    }
  };
}

impl_binding_target_state!(ProgramBindingTarget, UseProgram());
impl_binding_target_state!(BufferBindingTarget, BindBuffer(target));
impl_binding_target_state!(TextureBindingTarget, BindTexture(target));
impl_binding_target_state!(FramebufferBindingTarget, BindFramebuffer(target));

#[derive(Debug, Eq, PartialEq, Clone, Hash, Default)]
pub struct ContextCapabilities {
  pub renderer: String,
  pub vendor: String,
  pub gl_version: String,
  pub glsl_version: String,
  pub extensions: ContextExtensions,

  pub max_texture_units: u32,
  pub max_texture_size: u32,

  pub max_debug_object_label_len: i32,
}

impl ContextCapabilities {
  pub fn load(gl: &RawGL) -> Self {
    fn get_bool_1(gl: &RawGL, name: u32) -> u8 {
      let mut value = 0;
      unsafe { gl.GetBooleanv(name, &mut value) }
      value
    }

    fn get_i32_1(gl: &RawGL, name: u32) -> i32 {
      let mut value = 0;
      unsafe { gl.GetIntegerv(name, &mut value) }
      value
    }

    #[inline(always)]
    fn get_u32_1(gl: &RawGL, name: u32) -> u32 { get_i32_1(gl, name) as _ }

    fn get_string(gl: &RawGL, name: u32) -> String {
      let ptr: *const u8 = unsafe { gl.GetString(name) };
      assert!(!ptr.is_null());
      let c_str = unsafe { CStr::from_ptr(ptr as *const i8) };
      String::from_utf8(c_str.to_bytes().to_vec()).expect("GetString returned a non-UTF8 string")
    }

    let renderer = get_string(gl, gl::RENDERER);
    info!("GL renderer:    {}", renderer);
    let vendor = get_string(gl, gl::VENDOR);
    info!("GL vendor:      {}", vendor);
    let gl_version = get_string(gl, gl::VERSION);
    info!("GL version:     {}", gl_version);
    let glsl_version = get_string(gl, gl::SHADING_LANGUAGE_VERSION);
    info!("GLSL version:   {}", glsl_version);

    let extensions = ContextExtensions::new(get_string(gl, gl::EXTENSIONS).split(' '));
    info!("GL extensions:  {:?}", extensions);

    // TODO:
    // fn get_number_precision(
    //   gl: &RawGL,
    //   shader_type: ShaderType,
    //   precision_type: NumberPrecisionType,
    // ) {
    //   let mut range = [0; 2];
    //   let mut precision = [0; 2];
    //   unsafe {
    //     gl.GetShaderPrecisionFormat(
    //       shader_type.as_raw(),
    //       precision_type.as_raw(),
    //       range.as_mut_ptr(),
    //       precision.as_mut_ptr(),
    //     )
    //   }
    //   println!(
    //     "shader_type = {:?}, precision_type = {:?}, range = {:?}, precision = {:?}",
    //     shader_type, precision_type, range, precision
    //   );
    // }

    assert!(get_bool_1(gl, gl::SHADER_COMPILER) == gl::TRUE);

    // for shader_type in ShaderType::VARIANTS {
    //   for precision_type in NumberPrecisionType::VARIANTS {
    //     get_number_precision(gl, *shader_type, *precision_type);
    //   }
    // }

    Self {
      renderer,
      vendor,
      gl_version,
      glsl_version,
      extensions,

      max_texture_units: get_u32_1(gl, gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS),
      max_texture_size: get_u32_1(gl, gl::MAX_TEXTURE_SIZE),

      max_debug_object_label_len: if gl.ObjectLabel.is_loaded() {
        get_i32_1(gl, gl::MAX_LABEL_LENGTH)
      } else {
        0
      },
    }
  }
}

macro_rules! generate_context_extensions_struct {
  ($(($name:literal, $field:ident)),* $(,)?) => {
    #[derive(Debug, Eq, PartialEq, Clone, Hash, Default)]
    pub struct ContextExtensions {
      $(pub $field: bool),*
    }

    impl ContextExtensions {
      fn new<'a, T: IntoIterator<Item = &'a str>>(loaded_extension_names_iter: T) -> Self {
        let mut extensions = ContextExtensions {
          $($field: false),*
        };

        for name in loaded_extension_names_iter {
          match name {
            $($name => extensions.$field = true,)*
            _ => {}
          }
        }

        extensions
      }
    }
  };
}

generate_context_extensions_struct![
  ("GL_KHR_debug", gl_khr_debug),
  ("GL_OES_texture_npot", gl_oes_texture_npot),
];

// gl_enum!({
//   pub enum NumberPrecisionType {
//     LowFloat = LOW_FLOAT,
//     MediumFloat = MEDIUM_FLOAT,
//     HighFloat = HIGH_FLOAT,
//     LowInt = LOW_INT,
//     MediumInt = MEDIUM_INT,
//     HighInt = HIGH_INT,
//   }
// });

gl_enum!({
  pub enum BlendingFactor {
    Zero = ZERO,
    One = ONE,

    SrcColor = SRC_COLOR,
    SrcAlpha = SRC_ALPHA,
    OneMinusSrcColor = ONE_MINUS_SRC_COLOR,
    OneMinusSrcAlpha = ONE_MINUS_SRC_ALPHA,

    DestColor = DST_COLOR,
    DestAlpha = DST_ALPHA,
    OneMinusDestColor = ONE_MINUS_DST_COLOR,
    OneMinusDestAlpha = ONE_MINUS_DST_ALPHA,

    ConstColor = CONSTANT_COLOR,
    ConstAlpha = CONSTANT_ALPHA,
    OneMinusConstColor = ONE_MINUS_CONSTANT_COLOR,
    OneMinusConstAlpha = ONE_MINUS_CONSTANT_ALPHA,
    // SrcAlphaSaturate = SRC_ALPHA_SATURATE, // min(src.a, 1 - dest.a)
  }
});

gl_enum!({
  pub enum BlendingEquation {
    Add = FUNC_ADD,
    Sub = FUNC_SUBTRACT,
    SubRev = FUNC_REVERSE_SUBTRACT,
  }
});

bitflags! {
  pub struct ClearFlags: u32 {
    const COLOR = gl::COLOR_BUFFER_BIT;
    const DEPTH = gl::DEPTH_BUFFER_BIT;
    const STENCIL = gl::STENCIL_BUFFER_BIT;
  }
}
