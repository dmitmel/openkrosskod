#![deny(missing_debug_implementations)]
#![allow(clippy::return_self_not_must_use)]

pub mod colors;
pub mod matrices;
pub mod ops;
pub mod random;
pub mod vectors;

pub use colors::*;
pub use matrices::*;
pub use ops::*;
pub use random::*;
pub use vectors::*;
