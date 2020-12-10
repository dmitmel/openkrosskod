use crate::impl_prelude::*;
use cardboard_math::*;
use prelude_plus::*;

// TODO: implement size handling system similar to that of Texture2D

gl_enum!({
  pub enum BindBufferTarget {
    Vertex = ARRAY_BUFFER,
    Element = ELEMENT_ARRAY_BUFFER,
  }
});

#[derive(Debug)]
pub struct VertexBuffer<T: Copy> {
  ctx: SharedContext,
  addr: u32,
  internal_state_acquired: bool,

  attribs: Vec<AttribPtr>,
  stride: u32,
  len: Cell<usize>,

  phantom: PhantomData<*mut T>,
}

impl<T> !Send for VertexBuffer<T> {}
impl<T> !Sync for VertexBuffer<T> {}

unsafe impl<T: Copy> Object for VertexBuffer<T> {
  const DEBUG_TYPE_IDENTIFIER: u32 = gl::BUFFER;

  #[inline(always)]
  fn ctx(&self) -> &SharedContext { &self.ctx }
  #[inline(always)]
  fn addr(&self) -> u32 { self.addr }
  #[inline(always)]
  fn internal_state_acquired(&self) -> bool { self.internal_state_acquired }
}

impl<T: Copy> VertexBuffer<T> {
  #[inline(always)]
  pub fn attribs(&self) -> &[AttribPtr] { &self.attribs }
  #[inline(always)]
  pub fn stride(&self) -> u32 { self.stride }

  pub fn new(ctx: SharedContext, attribs: Vec<AttribPtr>) -> Self {
    let mut stride = 0;
    for attrib in &attribs {
      assert!(1 <= attrib.type_.len && attrib.type_.len <= 4);
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

      attribs,
      stride,
      len: Cell::new(0),

      phantom: PhantomData,
    }
  }

  pub fn bind(&mut self) -> VertexBufferBinding<'_, T> {
    let binding_target = &self.ctx.bound_vertex_buffer;
    binding_target.on_binding_created(self.addr);
    binding_target.bind_if_needed(self.raw_gl(), self.addr);
    self.internal_state_acquired = true;
    VertexBufferBinding { buffer: self }
  }
}

impl<T: Copy> Drop for VertexBuffer<T> {
  fn drop(&mut self) { unsafe { self.raw_gl().DeleteBuffers(1, &self.addr) }; }
}

#[derive(Debug)]
pub struct VertexBufferBinding<'obj, T: Copy> {
  buffer: &'obj mut VertexBuffer<T>,
}

unsafe impl<'obj, T> ObjectBinding<'obj, VertexBuffer<T>> for VertexBufferBinding<'obj, T>
where
  T: Copy,
{
  #[inline(always)]
  fn object(&self) -> &VertexBuffer<T> { &self.buffer }

  fn unbind_completely(self) {
    self.ctx().bound_vertex_buffer.unbind_unconditionally(self.raw_gl());
  }
}

impl<'obj, T: Copy> Drop for VertexBufferBinding<'obj, T> {
  fn drop(&mut self) { self.ctx().bound_vertex_buffer.on_binding_dropped(); }
}

impl<'obj, T: Copy> VertexBufferBinding<'obj, T> {
  pub fn configure_attribs(&self) {
    let gl = self.raw_gl();
    let attribs = &self.buffer.attribs;
    let stride = self.buffer.stride;

    let mut offset = 0;
    for attrib in attribs {
      if attrib.is_active() {
        unsafe {
          gl.VertexAttribPointer(
            attrib.location as u32,
            attrib.type_.len as i32,
            attrib.type_.name.as_raw(),
            attrib.type_.normalize as u8,
            stride as i32,
            offset as *const c_void,
          )
        };
      }
      offset += attrib.size as isize;
    }
  }

  // https://stackoverflow.com/q/39264296/12005228
  pub fn enable_attribs(&self) {
    let gl = self.raw_gl();
    let attribs = &self.buffer.attribs;
    for attrib in attribs {
      if attrib.is_active() {
        unsafe { gl.EnableVertexAttribArray(attrib.location as u32) };
      }
    }
  }

  pub fn disable_attribs(&self) {
    let gl = self.raw_gl();
    let attribs = &self.buffer.attribs;
    for attrib in attribs {
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

unsafe impl<T: BufferIndex> Object for ElementBuffer<T> {
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
    let binding_target = &self.ctx.bound_element_buffer;
    binding_target.on_binding_created(self.addr);
    binding_target.bind_if_needed(self.raw_gl(), self.addr);
    self.internal_state_acquired = true;
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

unsafe impl<'obj, T> ObjectBinding<'obj, ElementBuffer<T>> for ElementBufferBinding<'obj, T>
where
  T: BufferIndex,
{
  #[inline(always)]
  fn object(&self) -> &ElementBuffer<T> { &self.buffer }

  fn unbind_completely(self) {
    self.ctx().bound_element_buffer.unbind_unconditionally(self.raw_gl());
  }
}

impl<'obj, T: BufferIndex> Drop for ElementBufferBinding<'obj, T> {
  fn drop(&mut self) { self.ctx().bound_element_buffer.on_binding_dropped(); }
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

pub trait BufferIndex: Copy {
  const GL_DRAW_ELEMENTS_TYPE: DrawElementsType;
}

impl BufferIndex for u8 {
  const GL_DRAW_ELEMENTS_TYPE: DrawElementsType = DrawElementsType::U8;
}

impl BufferIndex for u16 {
  const GL_DRAW_ELEMENTS_TYPE: DrawElementsType = DrawElementsType::U16;
}

gl_enum!({
  pub enum DrawElementsType {
    U8 = UNSIGNED_BYTE,
    U16 = UNSIGNED_SHORT,
  }
});

pub unsafe trait Buffer<T>: Object {
  fn len(&self) -> usize;
  #[inline(always)]
  fn is_empty(&self) -> bool { self.len() == 0 }

  unsafe fn __impl_set_len(&self, len: usize);
}

unsafe impl<T: Copy> Buffer<T> for VertexBuffer<T> {
  #[inline(always)]
  fn len(&self) -> usize { self.len.get() }
  #[inline(always)]
  unsafe fn __impl_set_len(&self, len: usize) { self.len.set(len) }
}

unsafe impl<T: BufferIndex> Buffer<T> for ElementBuffer<T> {
  #[inline(always)]
  fn len(&self) -> usize { self.len.get() }
  #[inline(always)]
  unsafe fn __impl_set_len(&self, len: usize) { self.len.set(len) }
}

pub unsafe trait BufferBinding<'obj, Obj: 'obj, T>: ObjectBinding<'obj, Obj>
where
  Obj: Buffer<T>,
  T: Copy,
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

unsafe impl<'obj, T> BufferBinding<'obj, VertexBuffer<T>, T> for VertexBufferBinding<'obj, T>
where
  T: Copy,
{
  const BIND_TARGET: BindBufferTarget = BindBufferTarget::Vertex;
}

unsafe impl<'obj, T> BufferBinding<'obj, ElementBuffer<T>, T> for ElementBufferBinding<'obj, T>
where
  T: BufferIndex,
{
  const BIND_TARGET: BindBufferTarget = BindBufferTarget::Element;
}

pub unsafe trait DrawableBufferBinding<'obj, Obj: 'obj, T>:
  BufferBinding<'obj, Obj, T>
where
  Obj: Buffer<T>,
  T: Copy,
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

unsafe impl<'obj, T> DrawableBufferBinding<'obj, VertexBuffer<T>, T>
  for VertexBufferBinding<'obj, T>
where
  T: Copy,
{
  unsafe fn __impl_draw(&'obj self, mode: DrawPrimitive, start: usize, count: usize) {
    self.raw_gl().DrawArrays(
      mode.as_raw(),
      i32::try_from(start).unwrap(),
      i32::try_from(count).unwrap(),
    )
  }
}

unsafe impl<'obj, T> DrawableBufferBinding<'obj, ElementBuffer<T>, T>
  for ElementBufferBinding<'obj, T>
where
  T: BufferIndex,
{
  unsafe fn __impl_draw(&'obj self, mode: DrawPrimitive, start: usize, count: usize) {
    self.raw_gl().DrawElements(
      mode.as_raw(),
      i32::try_from(count).unwrap(),
      T::GL_DRAW_ELEMENTS_TYPE.as_raw(),
      (start as usize * mem::size_of::<T>()) as *const c_void,
    )
  }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct AttribPtrType {
  pub name: AttribPtrTypeName,
  pub len: u32,
  pub normalize: bool,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct AttribPtr {
  location: u32,
  type_: AttribPtrType,
  size: u32,
}

impl AttribPtr {
  #[inline(always)]
  pub fn location(&self) -> u32 { self.location }
  #[inline(always)]
  pub fn is_active(&self) -> bool { self.location != crate::INACTIVE_ATTRIB_LOCATION }
  #[inline(always)]
  pub fn type_(&self) -> &AttribPtrType { &self.type_ }
  #[inline(always)]
  pub fn size(&self) -> u32 { self.size }

  pub fn new(location: u32, type_: AttribPtrType) -> Self {
    assert!(type_.len > 0);
    let size = type_.name.size() as u32 * type_.len;
    Self { location, type_, size }
  }
}

impl<T: CorrespondingAttribPtrType> crate::Attrib<T> {
  pub fn to_pointer(&self, type_: AttribPtrType) -> AttribPtr {
    assert_eq!(type_.len, T::CORRESPONDING_ATTRIB_PTR_TYPE.len);
    if let Some(data_type) = self.data_type() {
      assert_eq!(type_.len as u32, data_type.name.components() as u32 * data_type.array_len);
    }
    AttribPtr::new(self.location(), type_)
  }

  pub fn to_pointer_simple(&self) -> AttribPtr {
    self.to_pointer(T::CORRESPONDING_ATTRIB_PTR_TYPE)
  }
}

gl_enum!({
  pub enum AttribPtrTypeName {
    I8 = BYTE,
    U8 = UNSIGNED_BYTE,
    I16 = SHORT,
    U16 = UNSIGNED_SHORT,
    // Fixed = FIXED,
    F32 = FLOAT,
  }
});

impl AttribPtrTypeName {
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

pub trait CorrespondingAttribPtrType {
  const CORRESPONDING_ATTRIB_PTR_TYPE: AttribPtrType;
}

macro_rules! impl_attr_type {
  ($data_type:ty, ($corresponding_type_name:ident, $corresponding_type_len:literal)) => {
    impl CorrespondingAttribPtrType for $data_type {
      const CORRESPONDING_ATTRIB_PTR_TYPE: AttribPtrType = AttribPtrType {
        name: AttribPtrTypeName::$corresponding_type_name,
        len: $corresponding_type_len,
        normalize: false,
      };
    }
  };
}

impl_attr_type!(f32, (F32, 1));
impl_attr_type!(Vec2<f32>, (F32, 2));
impl_attr_type!(Color<f32>, (F32, 4));
