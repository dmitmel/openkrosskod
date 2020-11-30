use crate::impl_prelude::*;
use prelude_plus::*;

// TODO: implement size handling system similar to that of Texture2D

gl_enum!({
  pub enum BindBufferTarget {
    Vertex = ARRAY_BUFFER,
    Element = ELEMENT_ARRAY_BUFFER,
  }
});

#[derive(Debug)]
pub struct VertexBuffer<T> {
  ctx: SharedContext,
  addr: u32,
  internal_state_acquired: bool,

  attributes: Vec<AttributePtr>,
  stride: u32,
  len: Cell<usize>,

  phantom: PhantomData<*mut T>,
}

impl<T> !Send for VertexBuffer<T> {}
impl<T> !Sync for VertexBuffer<T> {}

impl<T> Object for VertexBuffer<T> {
  const DEBUG_TYPE_IDENTIFIER: u32 = gl::BUFFER;

  #[inline(always)]
  fn ctx(&self) -> &SharedContext { &self.ctx }
  #[inline(always)]
  fn addr(&self) -> u32 { self.addr }
  #[inline(always)]
  fn internal_state_acquired(&self) -> bool { self.internal_state_acquired }
}

impl<T> VertexBuffer<T> {
  #[inline(always)]
  pub fn attributes(&self) -> &[AttributePtr] { &self.attributes }
  #[inline(always)]
  pub fn stride(&self) -> u32 { self.stride }

  pub fn new(ctx: SharedContext, attributes: Vec<AttributePtr>) -> Self {
    let mut stride = 0;
    for attrib in &attributes {
      assert!(1 <= attrib.config.len && attrib.config.len <= 4);
      stride += attrib.size as u32;
    }
    assert!(stride <= i32::MAX as u32); // for quick conversion to GLsizei

    assert_eq!(mem::size_of::<T>(), stride as usize);

    let mut addr = 0;
    unsafe { ctx.raw_gl().GenBuffers(1, &mut addr) };
    Self {
      ctx,
      addr,
      internal_state_acquired: false,

      attributes,
      stride,
      len: Cell::new(0),

      phantom: PhantomData,
    }
  }

  pub fn bind(&mut self) -> VertexBufferBinding<'_, T> {
    self.ctx.bound_vertex_buffer.bind_if_needed(
      self.ctx.raw_gl(),
      self.addr,
      &mut self.internal_state_acquired,
    );
    VertexBufferBinding { buffer: self }
  }
}

impl<T> Drop for VertexBuffer<T> {
  fn drop(&mut self) { unsafe { self.raw_gl().DeleteBuffers(1, &self.addr) }; }
}

#[derive(Debug)]
pub struct VertexBufferBinding<'obj, T> {
  buffer: &'obj mut VertexBuffer<T>,
}

impl<'obj, T> ObjectBinding<'obj, VertexBuffer<T>> for VertexBufferBinding<'obj, T> {
  #[inline(always)]
  fn object(&self) -> &VertexBuffer<T> { &self.buffer }

  fn unbind_completely(self) {
    self.ctx().bound_vertex_buffer.unbind_unconditionally(self.raw_gl());
  }
}

impl<'obj, T> VertexBufferBinding<'obj, T> {
  pub fn configure_attributes(&self) {
    let gl = self.raw_gl();
    let attributes = &self.buffer.attributes;
    let stride = self.buffer.stride;

    let mut offset = 0;
    for attrib in attributes {
      if attrib.is_active() {
        unsafe {
          gl.VertexAttribPointer(
            attrib.location as u32,
            attrib.config.len as i32,
            attrib.config.type_.as_raw(),
            attrib.config.normalize as u8,
            stride as i32,
            offset as *const c_void,
          )
        };
      }
      offset += attrib.size as isize;
    }
  }

  // https://stackoverflow.com/q/39264296/12005228
  pub fn enable_attributes(&self) {
    let gl = self.raw_gl();
    let attributes = &self.buffer.attributes;
    for attrib in attributes {
      if attrib.is_active() {
        unsafe { gl.EnableVertexAttribArray(attrib.location as u32) };
      }
    }
  }

  pub fn disable_attributes(&self) {
    let gl = self.raw_gl();
    let attributes = &self.buffer.attributes;
    for attrib in attributes {
      if attrib.is_active() {
        unsafe { gl.DisableVertexAttribArray(attrib.location as u32) };
      }
    }
  }
}

#[derive(Debug)]
pub struct ElementBuffer<T: BufferIndex> {
  ctx: SharedContext,
  addr: u32,
  internal_state_acquired: bool,

  len: Cell<usize>,

  phantom: PhantomData<*mut T>,
}

impl<T: BufferIndex> !Send for ElementBuffer<T> {}
impl<T: BufferIndex> !Sync for ElementBuffer<T> {}

impl<T: BufferIndex> Object for ElementBuffer<T> {
  const DEBUG_TYPE_IDENTIFIER: u32 = gl::BUFFER;

  #[inline(always)]
  fn ctx(&self) -> &SharedContext { &self.ctx }
  #[inline(always)]
  fn addr(&self) -> u32 { self.addr }
  #[inline(always)]
  fn internal_state_acquired(&self) -> bool { self.internal_state_acquired }
}

impl<T: BufferIndex> ElementBuffer<T> {
  pub fn new(ctx: SharedContext) -> Self {
    let mut addr = 0;
    unsafe { ctx.raw_gl().GenBuffers(1, &mut addr) };
    Self { ctx, addr, internal_state_acquired: false, len: Cell::new(0), phantom: PhantomData }
  }

  pub fn bind(&mut self) -> ElementBufferBinding<'_, T> {
    self.ctx.bound_element_buffer.bind_if_needed(
      self.ctx.raw_gl(),
      self.addr,
      &mut self.internal_state_acquired,
    );
    ElementBufferBinding { buffer: self }
  }
}

impl<T: BufferIndex> Drop for ElementBuffer<T> {
  fn drop(&mut self) { unsafe { self.raw_gl().DeleteBuffers(1, &self.addr) }; }
}

#[derive(Debug)]
pub struct ElementBufferBinding<'obj, T: BufferIndex> {
  buffer: &'obj mut ElementBuffer<T>,
}

impl<'obj, T> ObjectBinding<'obj, ElementBuffer<T>> for ElementBufferBinding<'obj, T>
where
  T: BufferIndex,
{
  #[inline(always)]
  fn object(&self) -> &ElementBuffer<T> { &self.buffer }

  fn unbind_completely(self) {
    self.ctx().bound_element_buffer.unbind_unconditionally(self.raw_gl());
  }
}

gl_enum!({
  pub enum BufferUsageHint {
    StreamDraw = STREAM_DRAW,
    StaticDraw = STATIC_DRAW,
    DynamicDraw = DYNAMIC_DRAW,
  }
});

gl_enum!({
  pub enum DrawPrimitive {
    Points = POINTS,
    LineStrip = LINE_STRIP,
    LineLoop = LINE_LOOP,
    Lines = LINES,
    TriangleStrip = TRIANGLE_STRIP,
    TriangleFan = TRIANGLE_FAN,
    Triangles = TRIANGLES,
  }
});

pub trait BufferIndex {
  const GL_DRAW_ELEMENTS_DATA_TYPE: DrawElementsDataType;
}

impl BufferIndex for u8 {
  const GL_DRAW_ELEMENTS_DATA_TYPE: DrawElementsDataType = DrawElementsDataType::U8;
}

impl BufferIndex for u16 {
  const GL_DRAW_ELEMENTS_DATA_TYPE: DrawElementsDataType = DrawElementsDataType::U16;
}

gl_enum!({
  pub enum DrawElementsDataType {
    U8 = UNSIGNED_BYTE,
    U16 = UNSIGNED_SHORT,
  }
});

pub trait Buffer<T>: Object {
  fn len(&self) -> usize;
  #[inline(always)]
  fn is_empty(&self) -> bool { self.len() == 0 }

  unsafe fn __impl_set_len(&self, len: usize);
}

impl<T> Buffer<T> for VertexBuffer<T> {
  #[inline(always)]
  fn len(&self) -> usize { self.len.get() }
  #[inline(always)]
  unsafe fn __impl_set_len(&self, len: usize) { self.len.set(len) }
}

impl<T: BufferIndex> Buffer<T> for ElementBuffer<T> {
  #[inline(always)]
  fn len(&self) -> usize { self.len.get() }
  #[inline(always)]
  unsafe fn __impl_set_len(&self, len: usize) { self.len.set(len) }
}

pub trait BufferBinding<'obj, Obj: 'obj, T>: ObjectBinding<'obj, Obj>
where
  Obj: Buffer<T>,
{
  const BIND_TARGET: BindBufferTarget;

  #[inline(always)]
  fn len(&'obj self) -> usize { self.object().len() }
  #[inline(always)]
  fn is_empty(&'obj self) -> bool { self.object().is_empty() }

  fn reserve_and_set(&'obj self, usage_hint: BufferUsageHint, data: &[T]) {
    unsafe { self.__impl_buffer_data(data.len(), data.as_ptr(), usage_hint) };
  }

  fn reserve(&'obj self, usage_hint: BufferUsageHint, data_len: usize) {
    unsafe { self.__impl_buffer_data(data_len, ptr::null(), usage_hint) };
  }

  fn set(&'obj self, data: &[T]) {
    let self_len = self.len();
    let slice_len = data.len();
    assert_eq!(slice_len, self_len);
    unsafe { self.__impl_buffer_sub_data(0, slice_len, data.as_ptr()) };
  }

  fn set_slice(&'obj self, offset: usize, data: &[T]) {
    let self_len = self.len();
    let slice_len = data.len();
    assert!(offset < self_len);
    assert!(offset + slice_len <= self_len);
    unsafe { self.__impl_buffer_sub_data(offset, slice_len, data.as_ptr()) };
  }

  unsafe fn __impl_buffer_data(
    &'obj self,
    len: usize,
    data: *const T,
    usage_hint: BufferUsageHint,
  ) {
    self.object().__impl_set_len(len);
    self.raw_gl().BufferData(
      Self::BIND_TARGET.as_raw(),
      isize::try_from(len * mem::size_of::<T>()).unwrap(),
      data as *const c_void,
      usage_hint.as_raw(),
    );
  }

  unsafe fn __impl_buffer_sub_data(&'obj self, offset: usize, len: usize, data: *const T) {
    self.raw_gl().BufferSubData(
      Self::BIND_TARGET.as_raw(),
      isize::try_from(offset * mem::size_of::<T>()).unwrap(),
      isize::try_from(len * mem::size_of::<T>()).unwrap(),
      data as *const c_void,
    );
  }
}

impl<'obj, T> BufferBinding<'obj, VertexBuffer<T>, T> for VertexBufferBinding<'obj, T> {
  const BIND_TARGET: BindBufferTarget = BindBufferTarget::Vertex;
}

impl<'obj, T> BufferBinding<'obj, ElementBuffer<T>, T> for ElementBufferBinding<'obj, T>
where
  T: BufferIndex,
{
  const BIND_TARGET: BindBufferTarget = BindBufferTarget::Element;
}

pub trait DrawableBufferBinding<'obj, Obj: 'obj, T>: BufferBinding<'obj, Obj, T>
where
  Obj: Buffer<T>,
{
  fn draw(&'obj self, _program_binding: &crate::ProgramBinding, mode: DrawPrimitive) {
    unsafe { self.__impl_draw(mode, 0, self.len()) }
  }

  fn draw_slice(
    &'obj self,
    _program_binding: &crate::ProgramBinding,
    mode: DrawPrimitive,
    start: usize,
    count: usize,
  ) {
    assert!(start < self.len());
    assert!(start + count <= self.len());
    unsafe { self.__impl_draw(mode, start, count) }
  }

  unsafe fn __impl_draw(&'obj self, mode: DrawPrimitive, start: usize, count: usize);
}

impl<'obj, T> DrawableBufferBinding<'obj, VertexBuffer<T>, T> for VertexBufferBinding<'obj, T> {
  unsafe fn __impl_draw(&'obj self, mode: DrawPrimitive, start: usize, count: usize) {
    self.raw_gl().DrawArrays(
      mode.as_raw(),
      i32::try_from(start).unwrap(),
      i32::try_from(count).unwrap(),
    )
  }
}

impl<'obj, T> DrawableBufferBinding<'obj, ElementBuffer<T>, T> for ElementBufferBinding<'obj, T>
where
  T: BufferIndex,
{
  unsafe fn __impl_draw(&'obj self, mode: DrawPrimitive, start: usize, count: usize) {
    self.raw_gl().DrawElements(
      mode.as_raw(),
      i32::try_from(count).unwrap(),
      T::GL_DRAW_ELEMENTS_DATA_TYPE.as_raw(),
      (start as usize * mem::size_of::<T>()) as *const c_void,
    )
  }
}

#[derive(Debug)]
pub struct AttributePtrConfig {
  pub type_: AttributePtrDataType,
  pub len: u16,
  pub normalize: bool,
}

#[derive(Debug)]
pub struct AttributePtr {
  location: u32,
  config: AttributePtrConfig,
  size: u16,
}

impl AttributePtr {
  #[inline(always)]
  pub fn location(&self) -> u32 { self.location }
  #[inline(always)]
  pub fn is_active(&self) -> bool { self.location != crate::INACTIVE_ATTRIBUTE_LOCATION }
  #[inline(always)]
  pub fn config(&self) -> &AttributePtrConfig { &self.config }
  #[inline(always)]
  pub fn size(&self) -> u16 { self.size }

  pub fn new(location: u32, config: AttributePtrConfig) -> Self {
    assert!(config.len > 0);
    let size = config.type_.size() as u16 * config.len;
    Self { location, config, size }
  }
}

gl_enum!({
  pub enum AttributePtrDataType {
    I8 = BYTE,
    U8 = UNSIGNED_BYTE,
    I16 = SHORT,
    U16 = UNSIGNED_SHORT,
    // Fixed = FIXED,
    F32 = FLOAT,
  }
});

impl AttributePtrDataType {
  pub fn size(&self) -> u8 {
    use mem::size_of;
    let size: usize = match self {
      Self::I8 => size_of::<i8>(),
      Self::U8 => size_of::<u8>(),
      Self::I16 => size_of::<i16>(),
      Self::U16 => size_of::<u16>(),
      // Self::Fixed => size_of::<GLfixed>(),
      Self::F32 => size_of::<f32>(),
    };
    assert!(size <= u8::MAX as usize);
    size as u8
  }
}
