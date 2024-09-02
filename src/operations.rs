//!
//! Module containing advanced functionality working on the [Mesh](crate::Mesh) struct.
//!

// Simple
mod connectivity;

mod vertex_measures;

mod edge_measures;

mod face_measures;

mod transformations;

mod bounding_box;
#[doc(inline)]
pub use bounding_box::*;

mod validity;

// Advanced
mod quality;

mod connected_components;

mod intersection;
#[doc(inline)]
pub use intersection::*;

mod merge;

mod split;
