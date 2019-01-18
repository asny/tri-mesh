//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::Mesh;
use crate::mesh::ids::*;

///
/// # Export functionality
///
/// Methods for extracting raw mesh data which for example can be used for visualisation.
///
/// # Examples
///
/// ## Index based arrays
///
/// ```
/// # let mesh = tri_mesh::MeshBuilder::new().cube().build().unwrap();
/// // Get face indices, vertex positions and vertex normals as float arrays..
/// let indices = mesh.indices_buffer();
/// let positions = mesh.positions_buffer();
/// let normals = mesh.normals_buffer();
/// # assert_eq!(positions.len(), 24);
/// # assert_eq!(normals.len(), 24);
///
/// // The vertex attributes are extracted by..
/// for vertex in 0..positions.len()/3
/// {
///     println!("The position and normal of vertex with index {} is:", vertex);
///     println!("({}, {}, {}) and ({}, {}, {})",
///         positions[3*vertex], positions[3*vertex+1], positions[3*vertex+2],
///         normals[3*vertex], normals[3*vertex+1], normals[3*vertex+2]);
/// }
///
/// // .. and the face attributes are extracted by
/// for face in 0..indices.len()/3
/// {
///     let vertices = (indices[3*face] as usize, indices[3*face + 1] as usize, indices[3*face + 2] as usize);
///     println!("The positions of face with index {} is:", face);
///     println!("({}, {}, {}), ({}, {}, {}) and ({}, {}, {})",
///         positions[3*vertices.0], positions[3*vertices.0+1], positions[3*vertices.0+2],
///         positions[3*vertices.1], positions[3*vertices.1+1], positions[3*vertices.1+2],
///         positions[3*vertices.2], positions[3*vertices.2+1], positions[3*vertices.2+2]);
///     println!("The normals of face with index {} is:", face);
///     println!("({}, {}, {}), ({}, {}, {}) and ({}, {}, {})",
///         normals[3*vertices.0], normals[3*vertices.0+1], normals[3*vertices.0+2],
///         normals[3*vertices.1], normals[3*vertices.1+1], normals[3*vertices.1+2],
///         normals[3*vertices.2], normals[3*vertices.2+1], normals[3*vertices.2+2]);
/// }
/// ```
///
///
impl Mesh
{
    ///
    /// Returns the face indices in an array `(i0, i1, i2) = (indices[3*x], indices[3*x+1], indices[3*x+2])` which is meant to be used for visualisation.
    /// Use the `positions_buffer` method and `normals_buffer` method to get the positions and normals of the vertices.
    /// See [this](#index-based-arrays) example.
    ///
    pub fn indices_buffer(&self) -> Vec<u32>
    {
        let vertices: Vec<VertexID> = self.vertex_iter().collect();
        let mut indices = Vec::with_capacity(self.no_faces() * 3);
        for face_id in self.face_iter()
        {
            for halfedge_id in self.face_halfedge_iter(face_id) {
                let vertex_id = self.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                let index = vertices.iter().position(|v| v == &vertex_id).unwrap();
                indices.push(index as u32);
            }
        }
        indices
    }

    ///
    /// Returns the positions of the vertices in an array which is meant to be used for visualisation.
    /// See [this](#index-based-arrays) example.
    ///
    /// **Note:** The connectivity of the vertices are attained by the `indices_buffer` method.
    ///
    pub fn positions_buffer(&self) -> Vec<f32>
    {
        let mut positions = Vec::with_capacity(self.no_vertices() * 3);
        for v3 in self.vertex_iter().map(|vertex_id| self.vertex_position(vertex_id)) {
            positions.push(v3.x); positions.push(v3.y); positions.push(v3.z);
        }
        positions
    }


    ///
    /// Returns the normals of the vertices in an array which is meant to be used for visualisation.
    /// See [this](#index-based-arrays) example.
    ///
    /// **Note:** The connectivity of the vertices are attained by the `indices_buffer` method.
    ///
    /// **Note:** The normals are computed from the connectivity and positions each time this method is invoked.
    ///
    pub fn normals_buffer(&self) -> Vec<f32>
    {
        let mut normals = Vec::with_capacity(self.no_vertices() * 3);
        for vertex_id in self.vertex_iter() {
            let normal = self.vertex_normal(vertex_id);
            normals.push(normal.x);
            normals.push(normal.y);
            normals.push(normal.z);
        }
        normals
    }

}