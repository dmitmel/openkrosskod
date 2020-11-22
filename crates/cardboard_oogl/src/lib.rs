#![feature(negative_impls)]

// TODO: INLINE TRIVIAL STUFF!!!

macro_rules! gl_enum {
  // a wrapper for autoformatting purposes
  ({$($tt:tt)+}) => { gl_enum! { $($tt)+ } };

  (
    $(#[$enum_meta:meta])* $visibility:vis enum $enum_name:ident {
      $($(#[$variant_meta:meta])* $rust_variant:ident = $gl_variant:ident),+ $(,)?
    }
  ) => {
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    $(#[$enum_meta])*
    $visibility enum $enum_name {
      $($(#[$variant_meta])* $rust_variant = ::gl::$gl_variant,)+
    }

    impl $enum_name {
      $visibility const VARIANTS: &'static [Self] = &[$(Self::$rust_variant),+];

      // #[inline(always)] // TODO: consider inlining
      $visibility const fn from_raw(raw: ::gl::types::GLenum) -> Option<Self> {
        Some(match raw {
          $(::gl::$gl_variant => Self::$rust_variant,)+
            _ => return None,
        })
      }

      #[inline(always)]
      $visibility const fn as_raw(&self) -> ::gl::types::GLenum {
        *self as ::gl::types::GLenum
      }
    }
  };
}

pub mod buffer;
pub mod context;
pub mod debug;
pub mod framebuffer;
pub mod shader;
pub mod texture;

pub use buffer::*;
pub use context::*;
pub use framebuffer::*;
pub use shader::*;
pub use texture::*;
