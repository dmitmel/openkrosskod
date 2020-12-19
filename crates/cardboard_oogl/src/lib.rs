#![feature(negative_impls)]
#![deny(missing_debug_implementations)]
#![allow(clippy::missing_safety_doc)]

pub use gl as raw_gl;
pub type RawGL = gl::Gles2;

#[doc(hidden)]
#[inline(never)]
#[cold]
#[track_caller]
pub fn _gl_enum_unknown_raw_value_fail(name: &'static str, raw: ::gl::types::GLenum) -> ! {
  panic!("unknown raw value for enum {}: 0x{:08x}", name, raw)
}

macro_rules! gl_enum {
  // a wrapper for autoformatting purposes
  ({$($tt:tt)+}) => { gl_enum! { $($tt)+ } };

  (
    $(#[$enum_meta:meta])* $visibility:vis enum $enum_name:ident {
      $($(#[$variant_meta:meta])* $rust_variant:ident = $gl_variant:ident),+ $(,)?
    }
  ) => {
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    $(#[$enum_meta])*
    $visibility enum $enum_name {
      $($(#[$variant_meta])* $rust_variant = ::gl::$gl_variant,)+
    }

    impl $enum_name {
      $visibility const VARIANTS: &'static [Self] = &[$(Self::$rust_variant),+];

      #[inline]
      $visibility const fn from_raw(raw: ::gl::types::GLenum) -> Option<Self> {
        Some(match raw {
          $(::gl::$gl_variant => Self::$rust_variant,)+
            _ => return None,
        })
      }

      #[inline]
      $visibility fn from_raw_unwrap(raw: ::gl::types::GLenum) -> Self {
        Self::from_raw(raw)
          .unwrap_or_else(|| $crate::_gl_enum_unknown_raw_value_fail(stringify!($enum_name), raw))
      }

      #[inline(always)]
      $visibility const fn as_raw(&self) -> ::gl::types::GLenum {
        *self as ::gl::types::GLenum
      }
    }
  };
}

mod impl_prelude;

pub mod buffer;
pub mod context;
pub mod debug;
pub mod framebuffer;
pub mod shader;
pub mod texture;
pub mod traits;

pub use buffer::*;
pub use context::*;
pub use debug::*;
pub use framebuffer::*;
pub use shader::*;
pub use texture::*;
pub use traits::*;
