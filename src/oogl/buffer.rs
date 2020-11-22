use super::shader::ShaderProgramBinding;
use super::Context;
use crate::gl_prelude::*;
use crate::prelude::*;

#[derive(Debug)]
pub struct Buffer<T> {
  ctx: Rc<Context>,
  addr: GLaddr,
  phantom: PhantomData<*const T>,
}

impl<T> !Send for Buffer<T> {}
impl<T> !Sync for Buffer<T> {}

impl<T> Buffer<T> {
  pub fn new(ctx: Rc<Context>) -> Self {
    let mut addr = 0;
    unsafe {
      ctx.gl.GenBuffers(1, &mut addr);
    }
    Self { ctx, addr, phantom: PhantomData }
  }

  pub fn addr(&self) -> GLaddr { self.addr }

  pub fn bind(&'_ mut self, target: BufferBindTarget) -> BufferBinding<'_, T> {
    BufferBinding::new(self, target)
  }
}

impl<T> Drop for Buffer<T> {
  fn drop(&mut self) {
    unsafe {
      self.ctx.gl.DeleteBuffers(1, &self.addr);
    }
  }
}

#[derive(Debug)]
pub struct BufferBinding<'buf, T> {
  buffer: &'buf mut Buffer<T>,
  target: BufferBindTarget,
}

impl<'buf, T> BufferBinding<'buf, T> {
  fn new(buffer: &'buf mut Buffer<T>, target: BufferBindTarget) -> Self {
    unsafe {
      buffer.ctx.gl.BindBuffer(target.to_raw(), buffer.addr);
    }
    Self { buffer, target }
  }

  pub fn buffer(&self) -> &Buffer<T> { self.buffer }
  pub fn target(&self) -> BufferBindTarget { self.target }
}

#[cfg(feature = "gl_unbind_bindings_on_drop")]
impl<'buf, T> Drop for BufferBinding<'buf, T> {
  fn drop(&mut self) {
    unsafe {
      self.buffer.ctx.gl.BindBuffer(self.target.to_raw(), 0);
    }
  }
}

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum BufferBindTarget {
    Array = ARRAY_BUFFER,
    ElementArray = ELEMENT_ARRAY_BUFFER,
  }
}

pub trait BoundBuffer<'buf, T> {
  fn binding(&self) -> &BufferBinding<'buf, T>;
  fn buffer(&'buf self) -> &'buf Buffer<T> { self.binding().buffer() }

  fn set_data(&self, data: &[T], usage_hint: BufferUsageHint)
  where
    T: 'buf,
  {
    let binding = self.binding();
    unsafe {
      binding.buffer().ctx.gl.BufferData(
        binding.target().to_raw(),
        GLsizeiptr::try_from(data.len() * mem::size_of::<T>()).unwrap(),
        data.as_ptr() as *const GLvoid,
        usage_hint.to_raw(),
      );
    }
  }
}

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum BufferUsageHint {
    StreamDraw = STREAM_DRAW,
    StaticDraw = STATIC_DRAW,
    DynamicDraw = DYNAMIC_DRAW,
  }
}

#[derive(Debug)]
pub struct BoundVertexBuffer<'buf, T> {
  binding: BufferBinding<'buf, T>,
}

impl<'buf, T> BoundVertexBuffer<'buf, T> {
  pub fn new(buffer: &'buf mut Buffer<T>) -> Self {
    Self { binding: buffer.bind(BufferBindTarget::Array) }
  }

  pub fn draw(
    &self,
    _program_binding: &ShaderProgramBinding,
    mode: DrawPrimitive,
    start: u32,
    count: u32,
  ) {
    unsafe {
      self.binding.buffer.ctx.gl.DrawArrays(
        mode.to_raw(),
        GLint::try_from(start).unwrap(),
        GLint::try_from(count).unwrap(),
      );
    }
  }
}

impl<'buf, T> BoundBuffer<'buf, T> for BoundVertexBuffer<'buf, T> {
  fn binding(&self) -> &BufferBinding<'buf, T> { &self.binding }
}

#[derive(Debug)]
pub struct BoundElementBuffer<'buf, T: ElementBufferIndex> {
  binding: BufferBinding<'buf, T>,
}

impl<'buf, T: ElementBufferIndex> BoundElementBuffer<'buf, T> {
  pub fn new(buffer: &'buf mut Buffer<T>) -> Self {
    Self { binding: buffer.bind(BufferBindTarget::ElementArray) }
  }

  pub fn draw(
    &self,
    _program_binding: &ShaderProgramBinding,
    mode: DrawPrimitive,
    start: u32,
    count: u32,
  ) {
    unsafe {
      self.binding.buffer.ctx.gl.DrawElements(
        mode.to_raw(),
        GLint::try_from(count).unwrap(),
        T::GL_DRAW_ELEMENTS_DATA_TYPE.to_raw(),
        (start as usize * mem::size_of::<T>()) as *const GLvoid,
      );
    }
  }
}

impl<'buf, T: ElementBufferIndex> BoundBuffer<'buf, T> for BoundElementBuffer<'buf, T> {
  fn binding(&self) -> &BufferBinding<'buf, T> { &self.binding }
}

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum DrawPrimitive {
    Points = POINTS,
    LineStrip = LINE_STRIP,
    LineLoop = LINE_LOOP,
    Lines = LINES,
    TriangleStrip = TRIANGLE_STRIP,
    TriangleFan = TRIANGLE_FAN,
    Triangles = TRIANGLES,
  }
}

pub trait ElementBufferIndex {
  const GL_DRAW_ELEMENTS_DATA_TYPE: DrawElementsDataType;
}

impl ElementBufferIndex for GLubyte {
  const GL_DRAW_ELEMENTS_DATA_TYPE: DrawElementsDataType = DrawElementsDataType::UnsignedByte;
}

impl ElementBufferIndex for GLushort {
  const GL_DRAW_ELEMENTS_DATA_TYPE: DrawElementsDataType = DrawElementsDataType::UnsignedShort;
}

gl_enum! {
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub enum DrawElementsDataType {
    UnsignedByte = UNSIGNED_BYTE,
    UnsignedShort = UNSIGNED_SHORT,
  }
}
