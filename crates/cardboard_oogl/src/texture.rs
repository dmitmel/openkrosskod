use crate::impl_prelude::*;
use cardboard_math::*;
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
  internal_state_acquired: bool,
}

impl !Send for Texture2D {}
impl !Sync for Texture2D {}

impl Object for Texture2D {
  const DEBUG_TYPE_IDENTIFIER: u32 = gl::TEXTURE;

  #[inline(always)]
  fn ctx(&self) -> &SharedContext { &self.ctx }
  #[inline(always)]
  fn addr(&self) -> u32 { self.addr }
  #[inline(always)]
  fn internal_state_acquired(&self) -> bool { self.internal_state_acquired }
}

impl Texture2D {
  pub const BIND_TARGET: BindTextureTarget = BindTextureTarget::Texture2D;

  pub fn new(ctx: SharedContext) -> Self {
    let mut addr = 0;
    unsafe { ctx.raw_gl().GenTextures(1, &mut addr) };
    Self { ctx, addr, internal_state_acquired: false }
  }

  pub fn bind(&'_ mut self, unit_preference: Option<u32>) -> Texture2DBinding<'_> {
    #[allow(clippy::or_fun_call)]
    let unit = unit_preference.unwrap_or(self.ctx.active_texture_unit.get());
    assert!(unit < self.ctx.capabilities().max_texture_units);

    let different_texture_was_bound = self.ctx.bound_texture_2d.bound_addr() != self.addr;
    let different_unit_was_selected = self.ctx.active_texture_unit.get() != unit;

    if different_texture_was_bound || different_unit_was_selected {
      let gl = self.ctx.raw_gl();

      if different_unit_was_selected {
        unsafe { gl.ActiveTexture(gl::TEXTURE0 + unit as u32) };
        self.ctx.active_texture_unit.set(unit);
      }

      self.ctx.bound_texture_2d.bind_unconditionally(gl, self.addr);
      self.internal_state_acquired = true;
    }
    Texture2DBinding { texture: self, unit }
  }
}

impl Drop for Texture2D {
  fn drop(&mut self) { unsafe { self.ctx.raw_gl().DeleteTextures(1, &self.addr) }; }
}

#[derive(Debug)]
pub struct Texture2DBinding<'obj> {
  texture: &'obj mut Texture2D,
  unit: u32,
}

impl<'obj> ObjectBinding<'obj, Texture2D> for Texture2DBinding<'obj> {
  #[inline(always)]
  fn object(&self) -> &Texture2D { &self.texture }

  fn unbind_completely(self) { self.ctx().bound_texture_2d.unbind_unconditionally(self.raw_gl()); }
}

impl<'obj> Texture2DBinding<'obj> {
  pub const BIND_TARGET: BindTextureTarget = Texture2D::BIND_TARGET;

  #[inline(always)]
  pub fn unit(&self) -> u32 { self.unit }

  pub fn generate_mipmap(&self) {
    unsafe { self.raw_gl().GenerateMipmap(Self::BIND_TARGET.as_raw()) };
  }

  pub fn set_wrapping_mode(&self, mode_s: TextureWrappingMode, mode_t: TextureWrappingMode) {
    let gl = self.raw_gl();
    let gl_target = Self::BIND_TARGET.as_raw();
    unsafe {
      gl.TexParameteri(gl_target, gl::TEXTURE_WRAP_S, mode_s.as_raw() as i32);
      gl.TexParameteri(gl_target, gl::TEXTURE_WRAP_T, mode_t.as_raw() as i32);
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

    unsafe { gl.TexParameteri(gl_target, gl::TEXTURE_MIN_FILTER, gl_enum as i32) };
  }

  pub fn set_magnifying_filter(&self, filter: TextureFilter) {
    let gl = self.raw_gl();
    let gl_target = Self::BIND_TARGET.as_raw();
    unsafe { gl.TexParameteri(gl_target, gl::TEXTURE_MAG_FILTER, filter.as_raw() as i32) };
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
    size: Vec2<u32>,
    data: &[u8],
  ) {
    let ctx = self.ctx();

    let max_size = ctx.capabilities().max_texture_size;
    assert!(size.x <= max_size);
    assert!(size.y <= max_size);
    assert_eq!(data.len(), size.x as usize * size.y as usize * format.color_components() as usize);

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
    size: Vec2<u32>,
  ) {
    let ctx = self.ctx();

    let max_size = ctx.capabilities().max_texture_size;
    assert!(size.x <= max_size);
    assert!(size.y <= max_size);

    self.set_data_internal(level_of_detail, format, internal_format, size, ptr::null());
  }

  fn set_data_internal(
    &self,
    level_of_detail: u32,
    format: TextureInputFormat,
    internal_format: TextureInternalFormat,
    size: Vec2<u32>,
    data_ptr: *const c_void,
  ) {
    unsafe {
      self.ctx().raw_gl().TexImage2D(
        Self::BIND_TARGET.as_raw(),
        i32::try_from(level_of_detail).unwrap(),
        internal_format.as_raw() as i32,
        i32::try_from(size.x).unwrap(),
        i32::try_from(size.y).unwrap(),
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
    offset: Vec2<u32>,
    size: Vec2<u32>,
    data: &[u8],
  ) {
    assert_eq!(data.len(), size.x as usize * size.y as usize * format.color_components() as usize);

    unsafe {
      self.ctx().raw_gl().TexSubImage2D(
        Self::BIND_TARGET.as_raw(),
        i32::try_from(level_of_detail).unwrap(),
        i32::try_from(offset.x).unwrap(),
        i32::try_from(offset.y).unwrap(),
        i32::try_from(size.x).unwrap(),
        i32::try_from(size.y).unwrap(),
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
