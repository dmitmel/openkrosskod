use crate::debug;
use crate::impl_prelude::*;
use prelude_plus::*;

pub use crate::{Buffer, BufferBinding, DrawableBufferBinding, Texture, TextureBinding};

pub unsafe trait Object {
  const DEBUG_TYPE_ID: u32;

  fn ctx(&self) -> &SharedContext;
  fn addr(&self) -> u32;

  #[inline(always)]
  fn raw_gl(&self) -> &RawGL { self.ctx().raw_gl() }

  fn set_debug_label(&self, label: &[u8]) {
    unsafe { debug::set_object_debug_label(self.ctx(), Self::DEBUG_TYPE_ID, self.addr(), label) };
  }

  fn get_debug_label(&self) -> Vec<u8> {
    unsafe { debug::get_object_debug_label(self.ctx(), Self::DEBUG_TYPE_ID, self.addr()) }
  }
}

// Not quite sure about how lifetimes work in this case
pub unsafe trait ObjectBinding<'obj, Obj: 'obj>
where
  Obj: Object,
{
  fn object(&'obj self) -> &Obj;

  #[inline(always)]
  fn ctx(&'obj self) -> &SharedContext { self.object().ctx() }
  #[inline(always)]
  fn raw_gl(&'obj self) -> &RawGL { self.object().raw_gl() }

  fn unbind_completely(self);
}
