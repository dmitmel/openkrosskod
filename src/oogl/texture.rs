use super::Context;
use crate::gl_prelude::*;
use crate::prelude::*;

pub type TextureUnit = u8;

#[derive(Debug)]
pub struct Texture {
  ctx: Rc<Context>,
  addr: GLaddr,
}

impl !Send for Texture {}
impl !Sync for Texture {}

impl Texture {
  pub fn new(ctx: Rc<Context>) -> Self {
    let mut addr = 0;
    unsafe {
      ctx.gl.GenTextures(1, &mut addr);
    }
    Self { ctx, addr }
  }

  pub fn addr(&self) -> GLaddr { self.addr }

  pub fn bind(
    &'_ mut self,
    target: TextureBindTarget,
    unit: Option<TextureUnit>,
  ) -> TextureBinding<'_> {
    TextureBinding::new(self, target, unit)
  }
}

impl Drop for Texture {
  fn drop(&mut self) {
    unsafe {
      self.ctx.gl.DeleteTextures(1, &self.addr);
    }
  }
}

#[derive(Debug)]
pub struct TextureBinding<'tex> {
  texture: &'tex mut Texture,
  target: TextureBindTarget,
}

impl<'tex> TextureBinding<'tex> {
  fn new(
    texture: &'tex mut Texture,
    target: TextureBindTarget,
    unit: Option<TextureUnit>,
  ) -> Self {
    unsafe {
      if let Some(unit) = unit {
        texture.ctx.gl.ActiveTexture(gl::TEXTURE0 + unit as GLenum);
      }
      texture.ctx.gl.BindTexture(target.to_raw(), texture.addr);
    }
    Self { texture, target }
  }

  pub fn texture(&self) -> &Texture { self.texture }
  pub fn target(&self) -> TextureBindTarget { self.target }
}

#[cfg(feature = "gl_unbind_bindings_on_drop")]
impl<'tex> Drop for TextureBinding<'tex> {
  fn drop(&mut self) {
    unsafe {
      self.texture.ctx.gl.BindTexture(self.target.to_raw(), 0);
    }
  }
}

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum TextureBindTarget {
    Tex2D = TEXTURE_2D,
    CubeMap = TEXTURE_CUBE_MAP,
  }
}

pub trait BoundTexture<'tex> {
  fn binding(&self) -> &TextureBinding<'tex>;

  fn generate_mipmap(&self) {
    let binding = self.binding();
    let target = binding.target();
    let ctx = &binding.texture.ctx;
    unsafe {
      ctx.gl.GenerateMipmap(target.to_raw());
    }
  }

  fn set_wrapping_mode(&self, mode_s: TextureWrappingMode, mode_t: TextureWrappingMode) {
    let binding = self.binding();
    let target = binding.target();
    let ctx = &binding.texture.ctx;
    unsafe {
      ctx.gl.TexParameteri(target.to_raw(), gl::TEXTURE_WRAP_S, mode_s.to_raw() as GLint);
      ctx.gl.TexParameteri(target.to_raw(), gl::TEXTURE_WRAP_T, mode_t.to_raw() as GLint);
    }
  }

  fn set_wrapping_modes(&self, mode: TextureWrappingMode) { self.set_wrapping_mode(mode, mode) }

  fn set_minifying_filter(&self, filter: TextureFilter, mipmap_filter: Option<TextureFilter>) {
    let binding = self.binding();
    let target = binding.target();
    let ctx = &binding.texture.ctx;

    use TextureFilter::*;
    let gl_enum = match (filter, mipmap_filter) {
      (_, None) => filter.to_raw(),
      (Nearest, Some(Nearest)) => gl::NEAREST_MIPMAP_NEAREST,
      (Linear, Some(Nearest)) => gl::LINEAR_MIPMAP_NEAREST,
      (Nearest, Some(Linear)) => gl::NEAREST_MIPMAP_LINEAR,
      (Linear, Some(Linear)) => gl::LINEAR_MIPMAP_LINEAR,
    };

    unsafe {
      ctx.gl.TexParameteri(target.to_raw(), gl::TEXTURE_MIN_FILTER, gl_enum as GLint);
    }
  }

  fn set_magnifying_filter(&self, filter: TextureFilter) {
    let binding = self.binding();
    let target = binding.target();
    let ctx = &binding.texture.ctx;
    unsafe {
      ctx.gl.TexParameteri(target.to_raw(), gl::TEXTURE_MAG_FILTER, filter.to_raw() as GLint);
    }
  }

  fn set_filters(&self, filter: TextureFilter, mipmap_filter: Option<TextureFilter>) {
    self.set_minifying_filter(filter, mipmap_filter);
    self.set_magnifying_filter(filter);
  }
}

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum TextureFilter {
    Nearest = NEAREST,
    Linear = LINEAR,
  }
}

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum TextureWrappingMode {
    ClampToEdge = CLAMP_TO_EDGE,
    MirroredRepeat = MIRRORED_REPEAT,
    Repeat = REPEAT,
  }
}

#[derive(Debug)]
pub struct BoundTexture2D<'tex> {
  binding: TextureBinding<'tex>,
}

impl<'tex> BoundTexture2D<'tex> {
  pub fn new(texture: &'tex mut Texture, unit: Option<TextureUnit>) -> Self {
    Self { binding: texture.bind(TextureBindTarget::Tex2D, unit) }
  }

  pub fn set_data(
    &self,
    level: u32,
    format: TextureInputFormat,
    internal_format: TextureInternalFormat,
    size: (u32, u32),
    data: &[u8],
  ) {
    let (width, height) = size;
    assert_eq!(data.len(), width as usize * height as usize * format.color_components() as usize);
    unsafe {
      self.binding.texture.ctx.gl.TexImage2D(
        gl::TEXTURE_2D,
        GLint::try_from(level).unwrap(),
        internal_format.to_raw() as GLint,
        GLint::try_from(width).unwrap(),
        GLint::try_from(height).unwrap(),
        0, // border, must be zero
        format.to_raw(),
        TextureInputDataType::UnsignedByte.to_raw(),
        data.as_ptr() as *const GLvoid,
      );
    }
  }
}

impl<'tex> BoundTexture<'tex> for BoundTexture2D<'tex> {
  fn binding(&self) -> &TextureBinding<'tex> { &self.binding }
}

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum TextureInternalFormat {
    Alpha = ALPHA,
    Luminance = LUMINANCE,
    LuminanceAlpha = LUMINANCE_ALPHA,
    RGB = RGB,
    RGBA = RGBA,
  }
}

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

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum TextureInputFormat {
    Alpha = ALPHA,
    RGB = RGB,
    RGBA = RGBA,
    Luminance = LUMINANCE,
    LuminanceAlpha = LUMINANCE_ALPHA,
  }
}

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

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum TextureInputDataType {
    UnsignedByte = UNSIGNED_BYTE,
    UnsignedShort565 = UNSIGNED_SHORT_5_6_5,
    UnsignedShort4444 = UNSIGNED_SHORT_4_4_4_4,
    UnsignedShort5551 = UNSIGNED_SHORT_5_5_5_1,
  }
}
