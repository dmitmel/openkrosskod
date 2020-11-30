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
pub struct Texture2D<T: TextureDataType = u8> {
  ctx: SharedContext,
  addr: u32,
  internal_state_acquired: bool,

  input_format: TextureInputFormat,
  internal_format: TextureInternalFormat,
  size: Cell<Vec2u32>,

  phantom: PhantomData<*mut T>,
}

impl<T: TextureDataType> !Send for Texture2D<T> {}
impl<T: TextureDataType> !Sync for Texture2D<T> {}

impl<T: TextureDataType> Object for Texture2D<T> {
  const DEBUG_TYPE_IDENTIFIER: u32 = gl::TEXTURE;

  #[inline(always)]
  fn ctx(&self) -> &SharedContext { &self.ctx }
  #[inline(always)]
  fn addr(&self) -> u32 { self.addr }
  #[inline(always)]
  fn internal_state_acquired(&self) -> bool { self.internal_state_acquired }
}

impl<T: TextureDataType> Texture2D<T> {
  pub const BIND_TARGET: BindTextureTarget = BindTextureTarget::Texture2D;

  #[inline(always)]
  pub fn input_format(&self) -> TextureInputFormat { self.input_format }
  #[inline(always)]
  pub fn internal_format(&self) -> TextureInternalFormat { self.internal_format }

  pub fn new(
    ctx: SharedContext,
    input_format: TextureInputFormat,
    internal_format_preference: Option<TextureInternalFormat>,
  ) -> Self {
    let mut addr = 0;
    unsafe { ctx.raw_gl().GenTextures(1, &mut addr) };

    Self {
      ctx,
      addr,
      internal_state_acquired: false,

      input_format,
      internal_format: internal_format_preference
        .unwrap_or_else(|| input_format.ideal_internal_format()),
      size: Cell::new(vec2n(0)),

      phantom: PhantomData,
    }
  }

  pub fn size_at_level_of_detail(&self, level_of_detail: u32) -> Vec2u32 {
    let size = self.size();
    // `a >> k` is equivalent to `a / 2**k`, max value with 1 is taken because
    // the texture size can't be zero on any dimension
    vec2((size.x >> level_of_detail).max(1), (size.y >> level_of_detail).max(1))
  }

  pub fn levels_of_detail_count(&self) -> u32 {
    /// Essentially returns the value of `floor(log2(max(n, 1))) + 1` where `n`
    /// is a positive integer. Explanation as for why this exact expression is
    /// used:
    ///
    /// `floor(log2(m))` can be interpreted as the number of times `m` has to
    /// be bitshifted to the right to get 1, in other words to get to the
    /// minimum level of detail on this axis (see also
    /// `size_at_level_of_detail`). A maximum value with 1 is taken because the
    /// result of `0.leading_zeros()` is undefined, as are the logarithms of
    /// non-positive numbers. This is however done as a sanity check because
    /// the texture size can be zero only if it wasn't set with `set_size`
    /// before (`set_size` doesn't allow zero sizes on any axis). Finally, we
    /// add 1 to include the default level of detail, i.e. level 0.
    fn lod_count_for_axis(n: u32) -> u32 {
      // <https://users.rust-lang.org/t/logarithm-of-integers/8506/5>
      // <https://github.com/rust-lang/rust/pull/70835/files#diff-8b5d068f3b1614f253a7bf9920ad9e8528eb6d623b57b69b09c90799cebaf9b1R4358>
      (1u32.leading_zeros() - n.max(1).leading_zeros()) + 1
    }

    let size = self.size();
    lod_count_for_axis(size.x).max(lod_count_for_axis(size.y))
  }

  pub fn bind(&'_ mut self, unit_preference: Option<u32>) -> Texture2DBinding<'_, T> {
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

impl<T: TextureDataType> Drop for Texture2D<T> {
  fn drop(&mut self) { unsafe { self.ctx.raw_gl().DeleteTextures(1, &self.addr) }; }
}

pub trait TextureDataType {
  const GL_TEXTURE_INPUT_DATA_TYPE: TextureInputDataType;
}

impl TextureDataType for u8 {
  const GL_TEXTURE_INPUT_DATA_TYPE: TextureInputDataType = TextureInputDataType::U8;
}

#[derive(Debug)]
pub struct Texture2DBinding<'obj, T: TextureDataType = u8> {
  texture: &'obj mut Texture2D<T>,
  unit: u32,
}

impl<'obj, T> ObjectBinding<'obj, Texture2D<T>> for Texture2DBinding<'obj, T>
where
  T: TextureDataType,
{
  #[inline(always)]
  fn object(&self) -> &Texture2D<T> { &self.texture }

  fn unbind_completely(self) { self.ctx().bound_texture_2d.unbind_unconditionally(self.raw_gl()); }
}

impl<'obj, T: TextureDataType> Texture2DBinding<'obj, T> {
  pub const BIND_TARGET: BindTextureTarget = Texture2D::<T>::BIND_TARGET;

  #[inline(always)]
  pub fn unit(&self) -> u32 { self.unit }

  #[inline(always)]
  pub fn size(&self) -> Vec2u32 { self.texture.size() }

  pub fn set_size(&self, size: Vec2u32) {
    let max_size = self.ctx().capabilities().max_texture_size;
    assert!(size.x > 0);
    assert!(size.y > 0);
    assert!(size.x <= max_size);
    assert!(size.y <= max_size);
    self.texture.size.set(size);
  }

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

  pub fn reserve_and_set(&self, level_of_detail: u32, data: &[T]) {
    let size = self.texture.size_at_level_of_detail(level_of_detail);
    assert_eq!(
      data.len(),
      size.x as usize * size.y as usize * self.texture.input_format.color_components() as usize
    );

    self.reserve_and_set_internal(level_of_detail, data.as_ptr());
  }

  pub fn reserve(&self, level_of_detail: u32) {
    self.reserve_and_set_internal(level_of_detail, ptr::null());
  }

  fn reserve_and_set_internal(&self, level_of_detail: u32, data_ptr: *const T) {
    let size = self.texture.size_at_level_of_detail(level_of_detail);
    unsafe {
      self.ctx().raw_gl().TexImage2D(
        Self::BIND_TARGET.as_raw(),
        i32::try_from(level_of_detail).unwrap(),
        self.texture.internal_format.as_raw() as i32,
        i32::try_from(size.x).unwrap(),
        i32::try_from(size.y).unwrap(),
        0, // border, must be zero
        self.texture.input_format.as_raw(),
        T::GL_TEXTURE_INPUT_DATA_TYPE.as_raw(),
        data_ptr as *const c_void,
      );
    }
  }

  pub fn set(&self, level_of_detail: u32, data: &[T]) {
    self.set_slice(level_of_detail, vec2n(0), self.texture.size(), data);
  }

  pub fn set_slice(&self, level_of_detail: u32, offset: Vec2u32, size: Vec2u32, data: &[T]) {
    assert_eq!(
      data.len(),
      size.x as usize * size.y as usize * self.texture.input_format.color_components() as usize
    );

    // TODO: Add check of the rectangle fromed by the offset and the size being
    // contained inside the texture size, somehow ignore it in `set`.

    unsafe {
      self.ctx().raw_gl().TexSubImage2D(
        Self::BIND_TARGET.as_raw(),
        i32::try_from(level_of_detail).unwrap(),
        i32::try_from(offset.x).unwrap(),
        i32::try_from(offset.y).unwrap(),
        i32::try_from(size.x).unwrap(),
        i32::try_from(size.y).unwrap(),
        self.texture.input_format.as_raw(),
        T::GL_TEXTURE_INPUT_DATA_TYPE.as_raw(),
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
  pub fn color_components(self) -> u8 {
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
  pub fn color_components(self) -> u8 {
    match self {
      Self::Alpha | Self::Luminance => 1,
      Self::LuminanceAlpha => 2,
      Self::RGB => 3,
      Self::RGBA => 4,
    }
  }

  pub fn ideal_internal_format(self) -> TextureInternalFormat {
    use TextureInternalFormat as Int;
    match self {
      Self::Alpha => Int::Alpha,
      Self::RGB => Int::RGB,
      Self::RGBA => Int::RGBA,
      Self::Luminance => Int::Luminance,
      Self::LuminanceAlpha => Int::LuminanceAlpha,
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

pub trait Texture<T: TextureDataType = u8>: Object {
  fn size(&self) -> Vec2u32;
  #[inline(always)]
  fn is_empty(&self) -> bool {
    let s = self.size();
    s.x == 0 || s.y == 0
  }
}

impl<T: TextureDataType> Texture<T> for Texture2D<T> {
  #[inline(always)]
  fn size(&self) -> Vec2u32 { self.size.get() }
}

pub trait TextureBinding<'obj, Obj: 'obj, T>: ObjectBinding<'obj, Obj>
where
  Obj: Texture<T>,
  T: TextureDataType,
{
  const BIND_TARGET: BindTextureTarget;

  #[inline(always)]
  fn size(&'obj self) -> Vec2u32 { self.object().size() }
  #[inline(always)]
  fn is_empty(&'obj self) -> bool { self.object().is_empty() }
}
