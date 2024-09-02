//!
//! This crate contains a [Mesh] struct which represents a triangle mesh.
//! It is implemented using a half-edge data structure which is efficient for creating, editing, traversing and computing on that mesh.
//! Also, the mesh can easily be created from and exported into a format that is efficient for visualization.
//! Finally, operations on the mesh is implemented as methods on the [Mesh] struct, so take a look at that rather long list of methods for a complete overview.
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
    #[error("action {0} will produce a non-manifold mesh")]
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
