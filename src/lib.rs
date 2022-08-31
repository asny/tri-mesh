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
//! - [Export](mesh/struct.Mesh.html#export) functionality (methods for extracting raw float buffers which can be used for visualisation)
//! - And more..
//!
//! All functionality is implemented as methods on the [Mesh](crate::mesh::Mesh) struct, so take a look at that rather long list of methods for a complete overview.
//! Also, to construct a new mesh, use the [Mesh builder](crate::mesh_builder::MeshBuilder).
//!

#![warn(missing_docs)]

pub mod math;
pub use math::*;

pub mod mesh;
pub use mesh::*;

pub mod operations;
pub use operations::*;

pub mod mesh_builder;
pub use mesh_builder::*;

pub mod prelude;

/// Result returned from an `tri-mesh` operation.
pub type TriMeshResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(test)]
mod test_utility {
    use super::*;
    /// Creates three connected triangles in `x = [-3, 3]`, `y = [-1, 2]` and `z = 0`
    /// which covers a square in `x = [-1, 1]`, `y = [-1, 1]` and `z = 0`
    /// and has a common vertex in `(0, 0, 0)`.
    pub(crate) fn subdivided_triangle() -> Mesh {
        RawMesh {
            indices: Some(Indices::U8(vec![0, 2, 3, 0, 3, 1, 0, 1, 2])),
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
        RawMesh {
            indices: Some(Indices::U8(vec![0, 1, 2])),
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
        RawMesh {
            indices: Some(Indices::U8(vec![0, 1, 2, 2, 1, 3])),
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
        RawMesh {
            indices: Some(Indices::U8(vec![0, 1, 2, 2, 1, 3, 3, 1, 4, 3, 4, 5])),
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
        RawMesh {
            indices: Some(Indices::U8(vec![
                0, 1, 2, 0, 2, 3, 4, 7, 6, 4, 6, 5, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2,
                7, 3, 4, 0, 3, 4, 3, 7,
            ])),
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
