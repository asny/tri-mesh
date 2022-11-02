//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;

impl Mesh {
    /// Returns the vertex position.
    pub fn position(&self, vertex_id: VertexID) -> Vec3 {
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
