//!
//! A triangle mesh data structure including basic operations.
//!
//! # Features
//!
//! - The main struct [Mesh](crate::mesh::Mesh) implements the half-edge mesh data structure for easy and efficient traversal
//! - Half-edge [Walker] to traverse the mesh
//! - [Iterators](mesh/struct.Mesh.html#iterators) over primitives (vertices, half-edges, edges, faces)
//! - Convenient [connectivity](mesh/struct.Mesh.html#connectivity) functionality (e.g. vertices of a face, edge between two vertices)
//! - Measures on [vertices](mesh/struct.Mesh.html#vertex-measures), [edges](mesh/struct.Mesh.html#edge-measures) and [faces](mesh/struct.Mesh.html#face-measures) (e.g. position of vertex, area of face)
//! - [Bounding box](mesh/struct.Mesh.html#bounding-box) functionality (e.g. constructing the axis aligned bounding box)
//! - [Edit](mesh/struct.Mesh.html#edit) functionality (e.g. split edge, collapse edge, flip edge)
//! - [Quality](mesh/struct.Mesh.html#quality) functionality (e.g. flip edges recursively to improve triangle quality, collapse small faces)
//! - [Orientation](mesh/struct.Mesh.html#orientation) functionality (e.g. flip orientation of all faces)
//! - [Transformations](mesh/struct.Mesh.html#transformations) affecting the vertex positions (e.g. moving a single vertex or rotate the entire mesh)
//! - [Intersection](mesh/struct.Mesh.html#intersection) functionality (e.g. face/ray intersection, edge/point intersection)
//! - [Merge](mesh/struct.Mesh.html#merge) used for merging of entire meshes (e.g. append one mesh to another or merge overlapping primitives in a mesh)
//! - [Split](mesh/struct.Mesh.html#split) functionality (e.g. clone a subset of a mesh or split two meshes at their intersection)
//! - And more..
//!
//! All functionality is implemented as methods on the [Mesh](crate::mesh::Mesh) struct, so take a look at that rather long list of methods for a complete overview.
//!

#![warn(missing_docs)]

pub mod math;

mod mesh;
pub use mesh::*;

mod operations;
pub use operations::*;

use thiserror::Error;
///
/// Error when performing a mesh operation
///
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("configuration is not valid: {0}")]
    ActionWillResultInInvalidMesh(String),
    #[error("action will produce a non-manifold mesh: {0}")]
    ActionWillResultInNonManifoldMesh(String),
    #[error("the mesh has ended up in an invalid state: {0}")]
    MeshIsInvalid(String),
}

#[cfg(test)]
mod test_utility {
    use super::*;
    use three_d_asset::{Indices, Positions, TriMesh};
    /// Creates three connected triangles in `x = [-3, 3]`, `y = [-1, 2]` and `z = 0`
    /// which covers a square in `x = [-1, 1]`, `y = [-1, 1]` and `z = 0`
    /// and has a common vertex in `(0, 0, 0)`.
    pub(crate) fn subdivided_triangle() -> Mesh {
        TriMesh {
            indices: Indices::U8(vec![0, 2, 3, 0, 3, 1, 0, 1, 2]),
            positions: Positions::F64(vec![
                vec3(0.0, 0.0, 0.0),
                vec3(-3.0, -1.0, 0.0),
                vec3(3.0, -1.0, 0.0),
                vec3(0.0, 2.0, 0.0),
            ]),
            ..Default::default()
        }
        .into()
    }

    /// Creates a triangle in `x = [-3, 3]`, `y = [-1, 2]` and `z = 0` which covers a square in `x = [-1, 1]`, `y = [-1, 1]` and `z = 0`.
    pub(crate) fn triangle() -> Mesh {
        TriMesh {
            indices: Indices::U8(vec![0, 1, 2]),
            positions: Positions::F64(vec![
                vec3(-3.0, -1.0, 0.0),
                vec3(3.0, -1.0, 0.0),
                vec3(0.0, 2.0, 0.0),
            ]),
            ..Default::default()
        }
        .into()
    }

    /// Creates a square in `x = [-1, 1]`, `y = [-1, 1]` and `z = 0`.
    pub(crate) fn square() -> Mesh {
        TriMesh {
            indices: Indices::U8(vec![0, 1, 2, 2, 1, 3]),
            positions: Positions::F64(vec![
                vec3(-1.0, -1.0, 0.0),
                vec3(1.0, -1.0, 0.0),
                vec3(-1.0, 1.0, 0.0),
                vec3(1.0, 1.0, 0.0),
            ]),
            ..Default::default()
        }
        .into()
    }

    pub(crate) fn triangle_strip() -> Mesh {
        TriMesh {
            indices: Indices::U8(vec![0, 1, 2, 2, 1, 3, 3, 1, 4, 3, 4, 5]),
            positions: Positions::F64(vec![
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(1.0, 0.0, 0.5),
                vec3(1.0, 0.0, 1.5),
                vec3(0.0, 0.0, 2.0),
                vec3(0.0, 0.0, 2.5),
            ]),
            ..Default::default()
        }
        .into()
    }

    pub(crate) fn cube() -> Mesh {
        TriMesh {
            indices: Indices::U8(vec![
                0, 1, 2, 0, 2, 3, 4, 7, 6, 4, 6, 5, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2,
                7, 3, 4, 0, 3, 4, 3, 7,
            ]),
            positions: Positions::F64(vec![
                vec3(1.0, -1.0, -1.0),
                vec3(1.0, -1.0, 1.0),
                vec3(-1.0, -1.0, 1.0),
                vec3(-1.0, -1.0, -1.0),
                vec3(1.0, 1.0, -1.0),
                vec3(1.0, 1.0, 1.0),
                vec3(-1.0, 1.0, 1.0),
                vec3(-1.0, 1.0, -1.0),
            ]),
            ..Default::default()
        }
        .into()
    }
}
