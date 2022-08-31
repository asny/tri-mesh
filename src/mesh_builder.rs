//!
//! Module containing [MeshBuilder] which has functionality to build a new [Mesh] instance.
//!

use crate::mesh::Mesh;
use crate::TriMeshResult;

use thiserror::Error;
///
/// Error when building a mesh
///
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum MeshBuilderError {
    #[error("the positions haven't been specified before calling the build function")]
    NoPositionsSpecified,
}

///
/// `MeshBuilder` contains functionality to build a mesh from either raw data (indices, positions, normals)
/// or from simple geometric shapes (box, icosahedron, cylinder, ..).
///
#[derive(Debug, Default)]
pub struct MeshBuilder {
    indices: Option<Vec<u32>>,
    positions: Option<Vec<f64>>,
}

impl MeshBuilder {
    /// Creates a new [MeshBuilder] instance.
    pub fn new() -> Self {
        MeshBuilder {
            indices: None,
            positions: None,
        }
    }

    ///
    /// Set the indices of each face, where the indices of face `x` is `(i0, i1, i2) = (indices[3*x], indices[3*x+1], indices[3*x+2])`.
    ///
    /// # Examples
    /// ```
    /// # use tri_mesh::*;
    /// #
    /// # fn main() -> tri_mesh::TriMeshResult<()> {
    /// let indices: Vec<u32> = vec![0, 1, 2,  0, 2, 3,  0, 3, 1];
    /// let positions: Vec<f64> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0];
    /// let mesh = MeshBuilder::new().with_indices(indices).with_positions(positions).build()?;
    ///
    /// assert_eq!(mesh.no_faces(), 3);
    /// assert_eq!(mesh.no_vertices(), 4);
    ///
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn with_indices(mut self, indices: Vec<u32>) -> Self {
        self.indices = Some(indices);
        self
    }

    ///
    /// Set the positions of each vertex, where the position of vertex `x` is `(x, y, z) = (positions[3*x], positions[3*x+1], positions[3*x+2])`;
    ///
    /// # Examples
    ///
    /// Build from positions (note: Use [merge_overlapping_primitives](crate::mesh::Mesh::merge_overlapping_primitives) if you want to merge
    /// unconnected but overlapping parts of the mesh):
    /// ```
    /// # use tri_mesh::*;
    /// #
    /// # fn main() -> tri_mesh::TriMeshResult<()> {
    /// let positions: Vec<f64> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
    ///                                    0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
    ///                                    0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5];
    /// let mesh = MeshBuilder::new().with_positions(positions).build()?;
    ///
    /// assert_eq!(mesh.no_faces(), 3);
    /// assert_eq!(mesh.no_vertices(), 9);
    ///
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn with_positions(mut self, positions: Vec<f64>) -> Self {
        self.positions = Some(positions);
        self
    }

    ///
    /// Builds the mesh. Returns the mesh if the definition is valid and otherwise an error.
    ///
    /// # Errors
    ///
    /// If no positions are specified, [MeshBuilderError::NoPositionsSpecified] error is returned.
    ///
    pub fn build(self) -> TriMeshResult<Mesh> {
        let positions = self
            .positions
            .ok_or(MeshBuilderError::NoPositionsSpecified)?;
        let indices = self
            .indices
            .unwrap_or((0..positions.len() as u32 / 3).collect());
        Ok(Mesh::new(indices, positions))
    }
}
