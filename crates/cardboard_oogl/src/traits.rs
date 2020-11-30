use crate::debug;
use crate::impl_prelude::*;
use prelude_plus::*;

pub use crate::{Buffer, BufferBinding, BufferIndex, CorrespondingUniformType, TextureDataType};

pub trait Object {
  const DEBUG_TYPE_IDENTIFIER: u32;

  fn ctx(&self) -> &SharedContext;
  fn addr(&self) -> u32;
  fn internal_state_acquired(&self) -> bool;

  #[inline(always)]
  fn raw_gl(&self) -> &RawGL { self.ctx().raw_gl() }

  fn set_debug_label(&self, label: &[u8]) {
    assert!(self.internal_state_acquired());
    let type_identifier = Self::DEBUG_TYPE_IDENTIFIER;
    unsafe { debug::set_object_debug_label(self.ctx(), type_identifier, self.addr(), label) };
  }

  fn get_debug_label(&self) -> Vec<u8> {
    let type_identifier = Self::DEBUG_TYPE_IDENTIFIER;
    unsafe { debug::get_object_debug_label(self.ctx(), type_identifier, self.addr()) }
  }
}

// Not quite sure about how lifetimes work in this case
pub trait ObjectBinding<'obj, Obj: 'obj>
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
