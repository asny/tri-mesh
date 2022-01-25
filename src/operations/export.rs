//! See [Mesh](crate::mesh::Mesh).

use crate::prelude::*;

///
/// # Export
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
///     println!("The vertex positions of face with index {} is:", face);
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
/// ## Non-index based arrays
///
/// ```
/// # let mesh = tri_mesh::MeshBuilder::new().cube().build().unwrap();
/// // Get vertex positions and vertex normals for each corner of each face as float arrays..
/// let positions = mesh.non_indexed_positions_buffer();
/// let normals = mesh.non_indexed_normals_buffer();
/// # assert_eq!(positions.len(), mesh.no_faces() * 3 * 3);
/// # assert_eq!(normals.len(), mesh.no_faces() * 3 * 3);
///
/// // .. the face attributes are extracted by
/// for face in 0..positions.len()/9
/// {
///     let vertices = (9*face, 9*face+3, 9*face+6);
///     println!("The vertex positions of face with index {} is:", face);
///     println!("({}, {}, {}), ({}, {}, {}) and ({}, {}, {})",
///         positions[vertices.0], positions[vertices.0+1], positions[vertices.0+2],
///         positions[vertices.1], positions[vertices.1+1], positions[vertices.1+2],
///         positions[vertices.2], positions[vertices.2+1], positions[vertices.2+2]);
///     println!("The vertex normals of face with index {} is:", face);
///     println!("({}, {}, {}), ({}, {}, {}) and ({}, {}, {})",
///         normals[vertices.0], normals[vertices.0+1], normals[vertices.0+2],
///         normals[vertices.1], normals[vertices.1+1], normals[vertices.1+2],
///         normals[vertices.2], normals[vertices.2+1], normals[vertices.2+2]);
/// }
/// ```
///
///
impl Mesh {
    ///
    /// Returns the face indices in an array `(i0, i1, i2) = (indices[3*x], indices[3*x+1], indices[3*x+2])` which is meant to be used for visualisation.
    /// Use the `positions_buffer` method and `normals_buffer` method to get the positions and normals of the vertices.
    /// See [this](#index-based-arrays) example.
    ///
    pub fn indices_buffer(&self) -> Vec<u32> {
        let vertices: Vec<VertexID> = self.vertex_iter().collect();
        let mut indices = Vec::with_capacity(self.no_faces() * 3);
        for face_id in self.face_iter() {
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
    pub fn positions_buffer(&self) -> Vec<f64> {
        let mut positions = Vec::with_capacity(self.no_vertices() * 3);
        for position in self
            .vertex_iter()
            .map(|vertex_id| self.vertex_position(vertex_id))
        {
            push_vec3(&mut positions, position);
        }
        positions
    }

    ///
    /// Returns the positions of the vertices in an array which is meant to be used for visualisation.
    /// See [this](#index-based-arrays) example.
    ///
    /// **Note:** The connectivity of the vertices are attained by the `indices_buffer` method.
    ///
    pub fn positions_buffer_f32(&self) -> Vec<f32> {
        let mut positions = Vec::with_capacity(self.no_vertices() * 3);
        for position in self
            .vertex_iter()
            .map(|vertex_id| self.vertex_position(vertex_id))
        {
            for i in 0..3 {
                positions.push(position[i] as f32);
            }
        }
        positions
    }

    ///
    /// Returns the normals of the vertices in an array which is meant to be used for visualisation.
    /// See [this](#index-based-arrays) example.
    ///
    /// **Note:** The connectivity of the vertices are attained by the `indices_buffer` method.
    ///
    /// **Note:** The normal of a vertex is computed as the average of the normals of the adjacent faces.
    ///
    /// **Note:** The normals are computed from the connectivity and positions each time this method is invoked.
    ///
    pub fn normals_buffer(&self) -> Vec<f64> {
        let mut normals = Vec::with_capacity(self.no_vertices() * 3);
        for vertex_id in self.vertex_iter() {
            push_vec3(&mut normals, self.vertex_normal(vertex_id));
        }
        normals
    }

    ///
    /// Returns the normals of the vertices in an array which is meant to be used for visualisation.
    /// See [this](#index-based-arrays) example.
    ///
    /// **Note:** The connectivity of the vertices are attained by the `indices_buffer` method.
    ///
    /// **Note:** The normal of a vertex is computed as the average of the normals of the adjacent faces.
    ///
    /// **Note:** The normals are computed from the connectivity and positions each time this method is invoked.
    ///
    pub fn normals_buffer_f32(&self) -> Vec<f32> {
        let mut normals = Vec::with_capacity(self.no_vertices() * 3);
        for vertex_id in self.vertex_iter() {
            let n = self.vertex_normal(vertex_id);
            for i in 0..3 {
                normals.push(n[i] as f32);
            }
        }
        normals
    }

    ///
    /// Returns the positions of the face corners in an array which is meant to be used for visualisation.
    /// See [this](#non-index-based-arrays) example.
    ///
    pub fn non_indexed_positions_buffer(&self) -> Vec<f64> {
        let mut positions = Vec::with_capacity(self.no_faces() * 3 * 3);
        for face_id in self.face_iter() {
            let (p0, p1, p2) = self.face_positions(face_id);
            push_vec3(&mut positions, p0);
            push_vec3(&mut positions, p1);
            push_vec3(&mut positions, p2);
        }
        positions
    }

    ///
    /// Returns the normals of the face corners in an array which is meant to be used for visualisation.
    /// See [this](#non-index-based-arrays) example.
    ///
    /// **Note:** The normal of a vertex is computed as the average of the normals of the adjacent faces.
    ///
    /// **Note:** The normals are computed from the connectivity and positions each time this method is invoked.
    ///
    pub fn non_indexed_normals_buffer(&self) -> Vec<f64> {
        let mut normals = Vec::with_capacity(self.no_faces() * 3 * 3);
        for face_id in self.face_iter() {
            let (v0, v1, v2) = self.face_vertices(face_id);
            push_vec3(&mut normals, self.vertex_normal(v0));
            push_vec3(&mut normals, self.vertex_normal(v1));
            push_vec3(&mut normals, self.vertex_normal(v2));
        }
        normals
    }
}

fn push_vec3(vec: &mut Vec<f64>, vec3: Vec3) {
    for i in 0..3 {
        vec.push(vec3[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexed_export() {
        let mesh = MeshBuilder::new().cylinder(3, 16).build().unwrap();
        let indices = mesh.indices_buffer();
        let positions = mesh.positions_buffer();
        let normals = mesh.normals_buffer();

        assert_eq!(indices.len(), mesh.no_faces() * 3);
        assert_eq!(positions.len(), mesh.no_vertices() * 3);
        assert_eq!(normals.len(), mesh.no_vertices() * 3);

        for face in 0..positions.len() / 9 {
            let vertices = (
                indices[3 * face] as usize,
                indices[3 * face + 1] as usize,
                indices[3 * face + 2] as usize,
            );
            let p0 = vec3(
                positions[3 * vertices.0],
                positions[3 * vertices.0 + 1],
                positions[3 * vertices.0 + 2],
            );
            let p1 = vec3(
                positions[3 * vertices.1],
                positions[3 * vertices.1 + 1],
                positions[3 * vertices.1 + 2],
            );
            let p2 = vec3(
                positions[3 * vertices.2],
                positions[3 * vertices.2 + 1],
                positions[3 * vertices.2 + 2],
            );
            let center = (p0 + p1 + p2) / 3.0;
            let face_id = mesh
                .face_iter()
                .find(|face_id| (mesh.face_center(*face_id) - center).magnitude() < 0.00001);
            assert!(face_id.is_some());

            let n0 = vec3(
                normals[3 * vertices.0],
                normals[3 * vertices.0 + 1],
                normals[3 * vertices.0 + 2],
            );
            let n1 = vec3(
                normals[3 * vertices.1],
                normals[3 * vertices.1 + 1],
                normals[3 * vertices.1 + 2],
            );
            let n2 = vec3(
                normals[3 * vertices.2],
                normals[3 * vertices.2 + 1],
                normals[3 * vertices.2 + 2],
            );

            let (v0, v1, v2) = mesh.face_vertices(face_id.unwrap());

            assert!(
                n0 == mesh.vertex_normal(v0)
                    || n1 == mesh.vertex_normal(v0)
                    || n2 == mesh.vertex_normal(v0)
            );
            assert!(
                n0 == mesh.vertex_normal(v1)
                    || n1 == mesh.vertex_normal(v1)
                    || n2 == mesh.vertex_normal(v1)
            );
            assert!(
                n0 == mesh.vertex_normal(v2)
                    || n1 == mesh.vertex_normal(v2)
                    || n2 == mesh.vertex_normal(v2)
            );
        }
    }

    #[test]
    fn test_non_indexed_export() {
        let mesh = MeshBuilder::new().cylinder(3, 16).build().unwrap();
        let positions = mesh.non_indexed_positions_buffer();
        let normals = mesh.non_indexed_normals_buffer();

        assert_eq!(positions.len(), mesh.no_faces() * 3 * 3);
        assert_eq!(normals.len(), mesh.no_faces() * 3 * 3);

        for face in 0..positions.len() / 9 {
            let vertices = (9 * face, 9 * face + 3, 9 * face + 6);
            let p0 = vec3(
                positions[vertices.0],
                positions[vertices.0 + 1],
                positions[vertices.0 + 2],
            );
            let p1 = vec3(
                positions[vertices.1],
                positions[vertices.1 + 1],
                positions[vertices.1 + 2],
            );
            let p2 = vec3(
                positions[vertices.2],
                positions[vertices.2 + 1],
                positions[vertices.2 + 2],
            );
            let center = (p0 + p1 + p2) / 3.0;

            let face_id = mesh
                .face_iter()
                .find(|face_id| (mesh.face_center(*face_id) - center).magnitude() < 0.00001);
            assert!(face_id.is_some());

            let n0 = vec3(
                normals[vertices.0],
                normals[vertices.0 + 1],
                normals[vertices.0 + 2],
            );
            let n1 = vec3(
                normals[vertices.1],
                normals[vertices.1 + 1],
                normals[vertices.1 + 2],
            );
            let n2 = vec3(
                normals[vertices.2],
                normals[vertices.2 + 1],
                normals[vertices.2 + 2],
            );

            let (v0, v1, v2) = mesh.face_vertices(face_id.unwrap());

            assert!(
                n0 == mesh.vertex_normal(v0)
                    || n1 == mesh.vertex_normal(v0)
                    || n2 == mesh.vertex_normal(v0)
            );
            assert!(
                n0 == mesh.vertex_normal(v1)
                    || n1 == mesh.vertex_normal(v1)
                    || n2 == mesh.vertex_normal(v1)
            );
            assert!(
                n0 == mesh.vertex_normal(v2)
                    || n1 == mesh.vertex_normal(v2)
                    || n2 == mesh.vertex_normal(v2)
            );
        }
    }
}
