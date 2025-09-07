//! Allow defining gravity fields with various shapes

/// Module providing components usable in both 2D and 3D variants
pub mod gravity;

#[allow(unused_imports)]
use godot::prelude::*;

/// Root of the library
#[cfg(feature = "standalone")]
struct GravityFieldtExtension;

#[cfg(feature = "standalone")]
#[gdextension]
unsafe impl ExtensionLibrary for GravityFieldtExtension {}
