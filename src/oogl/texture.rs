use super::{RawGL, SharedContext};
use ::gl::prelude::*;
use prelude_plus::*;

gl_enum!({
  pub enum BindTextureTarget {
    Texture2D = TEXTURE_2D,
    CubeMap = TEXTURE_CUBE_MAP,
  }
});

#[derive(Debug)]
pub struct Texture2D {
  ctx: SharedContext,
  addr: u32,
}

impl Texture2D {
  pub const BIND_TARGET: BindTextureTarget = BindTextureTarget::Texture2D;

  pub fn ctx(&self) -> &SharedContext { &self.ctx }
  pub fn raw_gl(&self) -> &RawGL { self.ctx.raw_gl() }
  pub fn addr(&self) -> u32 { self.addr }

  pub fn new(ctx: SharedContext) -> Self {
    let mut addr = 0;
    unsafe { ctx.raw_gl().GenTextures(1, &mut addr) };
    Self { ctx, addr }
  }

  pub fn bind(&'_ mut self, unit: Option<u32>) -> Texture2DBinding<'_> {
    #[allow(clippy::or_fun_call)]
    let unit = unit.unwrap_or(self.ctx.active_texture_unit.get());
    assert!(unit < self.ctx.capabilities().max_texture_units);

    let different_texture_was_bound = self.ctx.bound_texture_2d.bound_addr() != self.addr;
    let different_unit_was_selected = self.ctx.active_texture_unit.get() != unit;

    if different_texture_was_bound || different_unit_was_selected {
      let gl = self.ctx.raw_gl();

      if different_unit_was_selected {
        unsafe { gl.ActiveTexture(gl::TEXTURE0 + unit as GLenum) };
        self.ctx.active_texture_unit.set(unit);
      }

      self.ctx.bound_texture_2d.bind_unconditionally(gl, self.addr);
    }
    Texture2DBinding { texture: self, unit }
  }
}

impl Drop for Texture2D {
  fn drop(&mut self) { unsafe { self.ctx.raw_gl().DeleteTextures(1, &self.addr) }; }
}

#[derive(Debug)]
pub struct Texture2DBinding<'tex> {
  texture: &'tex mut Texture2D,
  unit: u32,
}

impl<'tex> Texture2DBinding<'tex> {
  pub const BIND_TARGET: BindTextureTarget = Texture2D::BIND_TARGET;

  pub fn ctx(&self) -> &SharedContext { &self.texture.ctx }
  pub fn raw_gl(&self) -> &RawGL { self.texture.ctx.raw_gl() }
  pub fn texture(&self) -> &Texture2D { &self.texture }
  pub fn unit(&self) -> u32 { self.unit }

  pub fn unbind_completely(self) {
    self.ctx().bound_framebuffer.unbind_unconditionally(self.raw_gl());
  }

  pub fn generate_mipmap(&self) {
    unsafe { self.raw_gl().GenerateMipmap(Self::BIND_TARGET.as_raw()) };
  }

  pub fn set_wrapping_mode(&self, mode_s: TextureWrappingMode, mode_t: TextureWrappingMode) {
    let gl = self.raw_gl();
    let gl_target = Self::BIND_TARGET.as_raw();
    unsafe {
      gl.TexParameteri(gl_target, gl::TEXTURE_WRAP_S, mode_s.as_raw() as GLint);
      gl.TexParameteri(gl_target, gl::TEXTURE_WRAP_T, mode_t.as_raw() as GLint);
    }
  }

  pub fn set_wrapping_modes(&self, mode: TextureWrappingMode) {
    self.set_wrapping_mode(mode, mode)
  }

  pub fn set_minifying_filter(&self, filter: TextureFilter, mipmap_filter: Option<TextureFilter>) {
    let gl = self.raw_gl();
    let gl_target = Self::BIND_TARGET.as_raw();

    use TextureFilter::*;
    let gl_enum = match (filter, mipmap_filter) {
      (_, None) => filter.as_raw(),
      (Nearest, Some(Nearest)) => gl::NEAREST_MIPMAP_NEAREST,
      (Linear, Some(Nearest)) => gl::LINEAR_MIPMAP_NEAREST,
      (Nearest, Some(Linear)) => gl::NEAREST_MIPMAP_LINEAR,
      (Linear, Some(Linear)) => gl::LINEAR_MIPMAP_LINEAR,
    };

    unsafe { gl.TexParameteri(gl_target, gl::TEXTURE_MIN_FILTER, gl_enum as GLint) };
  }

  pub fn set_magnifying_filter(&self, filter: TextureFilter) {
    let gl = self.raw_gl();
    let gl_target = Self::BIND_TARGET.as_raw();
    unsafe { gl.TexParameteri(gl_target, gl::TEXTURE_MAG_FILTER, filter.as_raw() as GLint) };
  }

  pub fn set_filters(&self, filter: TextureFilter, mipmap_filter: Option<TextureFilter>) {
    self.set_minifying_filter(filter, mipmap_filter);
    self.set_magnifying_filter(filter);
  }

  pub fn set_data(
    &self,
    level_of_detail: u32,
    format: TextureInputFormat,
    internal_format: TextureInternalFormat,
    size: (u32, u32),
    data: &[u8],
  ) {
    let ctx = self.ctx();

    let (width, height) = size;
    let max_size = ctx.capabilities().max_texture_size;
    assert!(width <= max_size);
    assert!(height <= max_size);
    assert_eq!(data.len(), width as usize * height as usize * format.color_components() as usize);

    self.set_data_internal(
      level_of_detail,
      format,
      internal_format,
      size,
      data.as_ptr() as *const _,
    );
  }

  pub fn reserve_data(
    &self,
    level_of_detail: u32,
    format: TextureInputFormat,
    internal_format: TextureInternalFormat,
    size: (u32, u32),
  ) {
    let ctx = self.ctx();

    let (width, height) = size;
    let max_size = ctx.capabilities().max_texture_size;
    assert!(width <= max_size);
    assert!(height <= max_size);

    self.set_data_internal(level_of_detail, format, internal_format, size, ptr::null());
  }

  fn set_data_internal(
    &self,
    level_of_detail: u32,
    format: TextureInputFormat,
    internal_format: TextureInternalFormat,
    (width, height): (u32, u32),
    data_ptr: *const GLvoid,
  ) {
    unsafe {
      self.ctx().raw_gl().TexImage2D(
        Self::BIND_TARGET.as_raw(),
        GLint::try_from(level_of_detail).unwrap(),
        internal_format.as_raw() as GLint,
        GLint::try_from(width).unwrap(),
        GLint::try_from(height).unwrap(),
        0, // border, must be zero
        format.as_raw(),
        TextureInputDataType::U8.as_raw(),
        data_ptr,
      );
    }
  }

  pub fn set_sub_data(
    &self,
    level_of_detail: u32,
    format: TextureInputFormat,
    (x_offset, y_offset): (u32, u32),
    (width, height): (u32, u32),
    data: &[u8],
  ) {
    assert_eq!(data.len(), width as usize * height as usize * format.color_components() as usize);

    unsafe {
      self.ctx().raw_gl().TexSubImage2D(
        Self::BIND_TARGET.as_raw(),
        GLint::try_from(level_of_detail).unwrap(),
        GLint::try_from(x_offset).unwrap(),
        GLint::try_from(y_offset).unwrap(),
        GLint::try_from(width).unwrap(),
        GLint::try_from(height).unwrap(),
        format.as_raw(),
        TextureInputDataType::U8.as_raw(),
        data.as_ptr() as *const _,
      );
    }
  }
}

gl_enum!({
  pub enum TextureFilter {
    Nearest = NEAREST,
    Linear = LINEAR,
  }
});

gl_enum!({
  pub enum TextureWrappingMode {
    ClampToEdge = CLAMP_TO_EDGE,
    MirroredRepeat = MIRRORED_REPEAT,
    Repeat = REPEAT,
  }
});

gl_enum!({
  pub enum TextureInternalFormat {
    Alpha = ALPHA,
    Luminance = LUMINANCE,
    LuminanceAlpha = LUMINANCE_ALPHA,
    RGB = RGB,
    RGBA = RGBA,
  }
});

impl TextureInternalFormat {
  pub fn color_components(&self) -> u8 {
    match self {
      Self::Alpha | Self::Luminance => 1,
      Self::LuminanceAlpha => 2,
      Self::RGB => 3,
      Self::RGBA => 4,
    }
  }
}

gl_enum!({
  pub enum TextureInputFormat {
    Alpha = ALPHA,
    RGB = RGB,
    RGBA = RGBA,
    Luminance = LUMINANCE,
    LuminanceAlpha = LUMINANCE_ALPHA,
  }
});

impl TextureInputFormat {
  pub fn color_components(&self) -> u8 {
    match self {
      Self::Alpha | Self::Luminance => 1,
      Self::LuminanceAlpha => 2,
      Self::RGB => 3,
      Self::RGBA => 4,
    }
  }
}

gl_enum!({
  pub enum TextureInputDataType {
    U8 = UNSIGNED_BYTE,
    U16_5_6_5 = UNSIGNED_SHORT_5_6_5,
    U16_4_4_4_4 = UNSIGNED_SHORT_4_4_4_4,
    U16_5_5_5_1 = UNSIGNED_SHORT_5_5_5_1,
  }
});
