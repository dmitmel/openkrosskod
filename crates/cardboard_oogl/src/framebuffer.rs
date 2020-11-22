use crate::{RawGL, SharedContext};
// use ::gl::prelude::*;
use prelude_plus::*;

gl_enum!({
  pub enum BindFramebufferTarget {
    Default = FRAMEBUFFER,
  }
});

#[derive(Debug)]
pub struct Framebuffer {
  ctx: SharedContext,
  addr: u32,
}

impl !Send for Framebuffer {}
impl !Sync for Framebuffer {}

impl Framebuffer {
  pub const BIND_TARGET: BindFramebufferTarget = BindFramebufferTarget::Default;

  pub fn ctx(&self) -> &SharedContext { &self.ctx }
  pub fn raw_gl(&self) -> &RawGL { self.ctx.raw_gl() }
  pub fn addr(&self) -> u32 { self.addr }

  pub fn new(ctx: SharedContext) -> Self {
    let mut addr = 0;
    unsafe { ctx.raw_gl().GenFramebuffers(1, &mut addr) };
    Self { ctx, addr }
  }

  pub fn bind(&'_ mut self) -> FramebufferBinding<'_> {
    self.ctx.bound_framebuffer.bind_if_needed(&self.ctx.raw_gl(), self.addr);
    FramebufferBinding { framebuffer: self }
  }
}

impl Drop for Framebuffer {
  fn drop(&mut self) { unsafe { self.ctx.raw_gl().DeleteFramebuffers(1, &self.addr) }; }
}

#[derive(Debug)]
pub struct FramebufferBinding<'tex> {
  framebuffer: &'tex mut Framebuffer,
}

impl<'tex> FramebufferBinding<'tex> {
  pub const BIND_TARGET: BindFramebufferTarget = Framebuffer::BIND_TARGET;

  pub fn ctx(&self) -> &SharedContext { &self.framebuffer.ctx }
  pub fn raw_gl(&self) -> &RawGL { self.framebuffer.ctx.raw_gl() }
  pub fn framebuffer(&self) -> &Framebuffer { &self.framebuffer }

  pub fn unbind_completely(self) {
    self.ctx().bound_framebuffer.unbind_unconditionally(self.raw_gl());
  }

  pub fn status(&self) -> FramebufferStatus {
    FramebufferStatus::from_raw(unsafe {
      self.raw_gl().CheckFramebufferStatus(Self::BIND_TARGET.as_raw())
    })
    .unwrap()
  }
}

gl_enum!({
  pub enum FramebufferStatus {
    Complete = FRAMEBUFFER_COMPLETE,
    IncompleteAttachment = FRAMEBUFFER_INCOMPLETE_ATTACHMENT,
    IncompleteDimensions = FRAMEBUFFER_INCOMPLETE_DIMENSIONS,
    IncompleteMissingAttachment = FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT,
    IncompleteUnsupported = FRAMEBUFFER_UNSUPPORTED,
  }
});
