//!
//! Module containing advanced functionality working on the [Mesh](crate::Mesh) struct.
//!

mod bounding_box;
#[doc(inline)]
pub use bounding_box::*;

mod connected_components;
#[doc(inline)]
pub use connected_components::*;

mod connectivity;
#[doc(inline)]
pub use connectivity::*;

mod edge_measures;
#[doc(inline)]
pub use edge_measures::*;

mod face_measures;
#[doc(inline)]
pub use face_measures::*;

mod intersection;
#[doc(inline)]
pub use intersection::*;

mod quality;
#[doc(inline)]
pub use quality::*;

mod transformations;
#[doc(inline)]
pub use transformations::*;

mod vertex_measures;
#[doc(inline)]
pub use vertex_measures::*;

mod merge;
#[doc(inline)]
pub use merge::*;

mod split;
#[doc(inline)]
pub use split::*;
