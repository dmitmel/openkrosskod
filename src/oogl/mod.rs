macro_rules! gl_enum {
  (
    $(#[$meta_outer:meta])*
    $visibility:vis enum $enum_name:ident {
      $(
        $(#[$meta_inner:meta])*
        $rust_variant:ident = $gl_variant:ident
      ),+ $(,)?
    }
  ) => {
    $(#[$meta_outer])*
    $visibility enum $enum_name {
      $(
        $(#[$meta_inner])*
        $rust_variant,
      )+
    }

    impl $enum_name {
      $visibility fn from_raw(raw: $crate::gl::types::GLenum) -> Option<Self> {
        match raw {
          $($crate::gl::$gl_variant => Some(Self::$rust_variant),)+
          _ => None,
        }
      }

      $visibility fn to_raw(&self) -> $crate::gl::types::GLenum {
        match self {
          $(Self::$rust_variant => $crate::gl::$gl_variant,)+
        }
      }
    }
  };
}

pub struct BindingContext {}
impl BindingContext {
  pub fn new() -> BindingContext { Self {} }
}

pub mod buffer;
pub mod context;
pub mod debug;
pub mod shader;
pub mod texture;

pub use buffer::{
  BoundBuffer, BoundElementBuffer, BoundVertexBuffer, Buffer, BufferUsageHint, DrawPrimitive,
};
pub use context::Context;
pub use shader::{SetUniform, Shader, ShaderProgram, ShaderType, Uniform};
pub use texture::{
  BoundTexture, BoundTexture2D, Texture, TextureFilter, TextureInputDataType, TextureInputFormat,
  TextureInternalFormat, TextureWrappingMode,
};
