use crate::impl_prelude::*;
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
  internal_state_acquired: bool,
}

impl !Send for Framebuffer {}
impl !Sync for Framebuffer {}

unsafe impl Object for Framebuffer {
  const DEBUG_TYPE_IDENTIFIER: u32 = gl::FRAMEBUFFER;

  #[inline(always)]
  fn ctx(&self) -> &SharedContext { &self.ctx }
  #[inline(always)]
  fn addr(&self) -> u32 { self.addr }
  #[inline(always)]
  fn internal_state_acquired(&self) -> bool { self.internal_state_acquired }
}

impl Framebuffer {
  pub const BIND_TARGET: BindFramebufferTarget = BindFramebufferTarget::Default;

  pub fn new(ctx: SharedContext) -> Self {
    let mut addr = 0;
    unsafe { ctx.raw_gl().GenFramebuffers(1, &mut addr) };
    Self { ctx, addr, internal_state_acquired: false }
  }

  pub fn bind(&'_ mut self) -> FramebufferBinding<'_> {
    let binding_target = &self.ctx.bound_framebuffer;
    binding_target.on_binding_created(self.addr);
    binding_target.bind_if_needed(self.raw_gl(), self.addr);
    self.internal_state_acquired = true;
    FramebufferBinding { framebuffer: self }
  }
}

impl Drop for Framebuffer {
  fn drop(&mut self) { unsafe { self.raw_gl().DeleteFramebuffers(1, &self.addr) }; }
}

#[derive(Debug)]
pub struct FramebufferBinding<'obj> {
  framebuffer: &'obj mut Framebuffer,
}

unsafe impl<'obj> ObjectBinding<'obj, Framebuffer> for FramebufferBinding<'obj> {
  #[inline(always)]
  fn object(&self) -> &Framebuffer { &self.framebuffer }

  fn unbind_completely(self) {
    self.ctx().bound_framebuffer.unbind_unconditionally(self.raw_gl());
  }
}

impl<'obj> Drop for FramebufferBinding<'obj> {
  fn drop(&mut self) { self.ctx().bound_framebuffer.on_binding_dropped(); }
}

impl<'obj> FramebufferBinding<'obj> {
  pub const BIND_TARGET: BindFramebufferTarget = Framebuffer::BIND_TARGET;

  pub fn status(&self) -> FramebufferStatus {
    FramebufferStatus::from_raw_unwrap(unsafe {
      self.raw_gl().CheckFramebufferStatus(Self::BIND_TARGET.as_raw())
    })
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
