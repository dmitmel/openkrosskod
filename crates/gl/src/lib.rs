pub use self::all::*;

pub mod all {
  #![allow(clippy::all)]
  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod prelude {
  pub use super::all as gl;
  pub use super::all::types::*;
  pub use super::all::Gles2;
}
