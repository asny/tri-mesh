pub use crate::math::*;

mod io;
#[doc(inline)]
pub use io::*;

mod utility;
#[doc(inline)]
pub use utility::*;

mod append;
#[doc(inline)]
pub use append::*;

mod cleanup;
#[doc(inline)]
pub use cleanup::*;

mod ids;
#[doc(inline)]
pub use ids::*;

mod iterators;
#[doc(inline)]
pub use iterators::*;

mod traversal;
#[doc(inline)]
pub use traversal::*;

mod edit;
#[doc(inline)]
pub use edit::*;

mod orientation;
#[doc(inline)]
pub use orientation::*;

mod connectivity_info;

use crate::mesh::connectivity_info::ConnectivityInfo;
use std::collections::HashMap;

///
/// A representation of a triangle mesh which is efficient for calculating on and making changes to a mesh.
/// Use [Mesh::new] to construct a new mesh.
/// Use [Mesh::export] to export the mesh to a format that is efficient for visualization.
///
/// ## Basic functionality:
/// - [Iterators](#iterators)
/// - [Traversal](#traversal)
/// - [Edit](#edit)
/// - [Orientation](#orientation)
///
/// ## Simple operations
/// - [Connectivity](#connectivity)
/// - [Vertex measures](#vertex-measures)
/// - [Edge measures](#edge-measures)
/// - [Face measures](#face-measures)
/// - [Transformations](#transformations)
/// - [Bounding box](#bounding-box)
/// - [Validity](#validity)
///
/// ## Advanced operations
/// - [Quality](#quality)
/// - [Connected components](#connected-components)
/// - [Intersection](#intersection)
/// - [Merge](#merge)
/// - [Split](#split)
///
#[derive(Debug, Clone)]
pub struct Mesh {
    connectivity_info: ConnectivityInfo,
}
