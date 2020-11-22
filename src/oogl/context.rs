use super::debug;
use crate::math::*;
use ::gl::prelude::*;
use prelude_plus::*;

pub type RawGL = Gles2;

pub type SharedContext = Rc<Context>;

pub struct Context {
  raw_gl: RawGL,
  sdl_gl_context: sdl2::video::GLContext,
  capabilities: ContextCapabilities,

  pub(super) bound_program: BindingTarget<ProgramBindingTarget>,
  pub(super) bound_vertex_buffer: BindingTarget<BufferBindingTarget>,
  pub(super) bound_element_buffer: BindingTarget<BufferBindingTarget>,
  pub(super) active_texture_unit: Cell<u32>,
  pub(super) bound_texture_2d: BindingTarget<TextureBindingTarget>,
  pub(super) bound_framebuffer: BindingTarget<FramebufferBindingTarget>,
}

impl !Send for Context {}
impl !Sync for Context {}

impl Context {
  pub fn raw_gl(&self) -> &RawGL { &self.raw_gl }
  pub fn sdl_gl_context(&self) -> &sdl2::video::GLContext { &self.sdl_gl_context }
  pub fn capabilities(&self) -> &ContextCapabilities { &self.capabilities }

  pub fn load_with(
    video_subsystem: &sdl2::VideoSubsystem,
    sdl_gl_context: sdl2::video::GLContext,
  ) -> Self {
    let gl = Gles2::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const GLvoid);

    // This has to be done first!!!
    if gl.DebugMessageCallback.is_loaded() {
      unsafe {
        gl.Enable(gl::DEBUG_OUTPUT);
        gl.DebugMessageCallback(Some(debug::internal_debug_message_callback), ptr::null());
      }
    }

    unsafe { gl.Enable(gl::BLEND) };

    let capabilities = ContextCapabilities::load(&gl);

    Self {
      raw_gl: gl,
      sdl_gl_context,
      capabilities,

      // programs are a special case, the binding target value doesn't matter
      // because there is no such thing as binding a program to a target
      bound_program: BindingTarget::new(gl::NONE),
      bound_vertex_buffer: BindingTarget::new(super::BindBufferTarget::Vertex.as_raw()),
      bound_element_buffer: BindingTarget::new(super::BindBufferTarget::Element.as_raw()),
      active_texture_unit: Cell::new(0),
      bound_texture_2d: BindingTarget::new(super::BindTextureTarget::Texture2D.as_raw()),
      bound_framebuffer: BindingTarget::new(super::BindFramebufferTarget::Default.as_raw()),
    }
  }

  pub fn clear_color(&self, color: Colorf) {
    unsafe {
      self.raw_gl.ClearColor(color.r, color.g, color.b, color.a);
      self.raw_gl.Clear(gl::COLOR_BUFFER_BIT);
    }
  }

  pub fn set_viewport(&self, x: i32, y: i32, w: i32, h: i32) {
    unsafe { self.raw_gl.Viewport(x, y, w, h) };
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
}

// TODO: Perhaps implement this properly later?
impl fmt::Debug for Context {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Context#<{:p}>", &self.sdl_gl_context)
  }
}

#[derive(Debug)]
pub(super) struct BindingTarget<T> {
  target: GLenum,
  bound_addr: Cell<u32>,
  phantom: PhantomData<*mut T>,
}
impl<T> BindingTarget<T> {
  pub(super) fn bound_addr(&self) -> u32 { self.bound_addr.get() }
  #[allow(dead_code)]
  pub(super) fn is_anything_bound(&self) -> bool { self.bound_addr.get() != 0 }
  pub(super) fn new(target: GLenum) -> Self {
    Self { target, bound_addr: Cell::new(0), phantom: PhantomData }
  }
}

macro_rules! impl_binding_target_state {
  ($target_enum:ident, $gl_bind_fn:ident ($($target:ident)?)) => {
    pub(super) enum $target_enum {}

    #[allow(dead_code)]
    impl BindingTarget<$target_enum> {
      #[inline(always)]
      pub(super) fn bind_unconditionally(&self, gl: &RawGL, addr: u32) {
        unsafe { gl.$gl_bind_fn($(self.$target, )? addr) };
        self.bound_addr.set(addr);
      }

      #[inline(always)]
      pub(super) fn unbind_unconditionally(&self, gl: &RawGL) {
        self.bind_unconditionally(gl, 0)
      }

      #[inline(always)]
      pub(super) fn bind_if_needed(&self, gl: &RawGL, addr: u32) {
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

// #[derive(Debug)]
// pub struct BindingContext<T> {
//   phantom: PhantomData<*mut T>,
// }
// impl<T> BindingContext<T> {
//   pub fn new() -> Self { Self { phantom: PhantomData } }
// }

#[derive(Debug)]
pub struct ContextCapabilities {
  pub max_texture_units: u32,
  pub max_texture_size: u32,
}

impl ContextCapabilities {
  pub fn load(gl: &RawGL) -> Self {
    fn get_bool_1(gl: &RawGL, name: GLenum) -> GLboolean {
      let mut value = 0;
      unsafe { gl.GetBooleanv(name, &mut value) }
      value
    }

    fn get_int_1(gl: &RawGL, name: GLenum) -> GLint {
      let mut value = 0;
      unsafe { gl.GetIntegerv(name, &mut value) }
      value
    }

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
      max_texture_units: get_int_1(gl, gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS) as u32,
      max_texture_size: get_int_1(gl, gl::MAX_TEXTURE_SIZE) as u32,
    }
  }
}

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
