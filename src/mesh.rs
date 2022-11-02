pub use crate::math::*;

mod io;
#[doc(inline)]
pub use io::*;

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

mod cleanup;
#[doc(inline)]
pub use cleanup::*;

mod append;
#[doc(inline)]
pub use append::*;

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
/// - [Clean-up](#clean-up)
/// - [Append](#append)
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
#[derive(Debug)]
pub struct Mesh {
    connectivity_info: ConnectivityInfo,
}

impl Mesh {
    /// Returns the vertex position.
    pub fn vertex_position(&self, vertex_id: VertexID) -> Vec3 {
        self.connectivity_info.position(vertex_id)
    }

    /// Returns the number of vertices in the mesh.
    pub fn no_vertices(&self) -> usize {
        self.connectivity_info.no_vertices()
    }

    /// Returns the number of edges in the mesh.
    pub fn no_edges(&self) -> usize {
        self.connectivity_info.no_halfedges() / 2
    }

    /// Returns the number of half-edges in the mesh.
    pub fn no_halfedges(&self) -> usize {
        self.connectivity_info.no_halfedges()
    }

    /// Returns the number of faces in the mesh.
    pub fn no_faces(&self) -> usize {
        self.connectivity_info.no_faces()
    }
}

impl Clone for Mesh {
    fn clone(&self) -> Mesh {
        Mesh {
            connectivity_info: self.connectivity_info.clone(),
        }
    }
}
