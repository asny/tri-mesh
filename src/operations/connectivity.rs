//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;

/// # Connectivity
impl Mesh {
    ///
    /// Returns the connecting edge between the two vertices or `None` if no edge is found.
    ///
    /// **Note:** This method assumes that the mesh is properly connected.
    /// See [vertex_halfedge_iter](#method.vertex_halfedge_iter) for more information.
    ///
    pub fn connecting_edge(
        &self,
        vertex_id1: VertexID,
        vertex_id2: VertexID,
    ) -> Option<HalfEdgeID> {
        for halfedge_id in self.vertex_halfedge_iter(vertex_id1) {
            if self.walker_from_halfedge(halfedge_id).vertex_id().unwrap() == vertex_id2 {
                return Some(halfedge_id);
            }
        }
        None
    }

    /// Returns whether or not the vertex is on a boundary.
    pub fn is_vertex_on_boundary(&self, vertex_id: VertexID) -> bool {
        for halfedge_id in self.vertex_halfedge_iter(vertex_id) {
            let mut walker = self.walker_from_halfedge(halfedge_id);
            if walker.face_id().is_none() || walker.as_twin().face_id().is_none() {
                return true;
            }
        }
        false
    }

    /// Returns whether or not the edge is on a boundary.
    pub fn is_edge_on_boundary(&self, halfedge_id: HalfEdgeID) -> bool {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        walker.face_id().is_none() || walker.as_twin().face_id().is_none()
    }

    /// Returns the vertex id of the two adjacent vertices to the given edge.
    pub fn edge_vertices(&self, halfedge_id: HalfEdgeID) -> (VertexID, VertexID) {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let v1 = walker.vertex_id().unwrap();
        let v2 = walker.as_twin().vertex_id().unwrap();
        (v1, v2)
    }

    /// Returns the vertex id of the two adjacent vertices to the given edge
    /// and ordered such that `ordered_edge_vertices.0 < ordered_edge_vertices.1`.
    pub fn ordered_edge_vertices(&self, halfedge_id: HalfEdgeID) -> (VertexID, VertexID) {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let v1 = walker.vertex_id().unwrap();
        let v2 = walker.as_twin().vertex_id().unwrap();
        if v1 < v2 {
            (v1, v2)
        } else {
            (v2, v1)
        }
    }

    /// Returns the vertex id of the three connected vertices to the given face.
    pub fn face_vertices(&self, face_id: FaceID) -> (VertexID, VertexID, VertexID) {
        let mut walker = self.walker_from_face(face_id);
        let v1 = walker.vertex_id().unwrap();
        walker.as_next();
        let v2 = walker.vertex_id().unwrap();
        walker.as_next();
        let v3 = walker.vertex_id().unwrap();
        (v1, v2, v3)
    }

    /// Returns the vertex id of the three connected vertices to the given face
    /// and ordered such that `ordered_face_vertices.0 < ordered_face_vertices.1 < ordered_face_vertices.2`.
    pub fn ordered_face_vertices(&self, face_id: FaceID) -> (VertexID, VertexID, VertexID) {
        let mut walker = self.walker_from_face(face_id);
        let v1 = walker.vertex_id().unwrap();
        walker.as_next();
        let v2 = walker.vertex_id().unwrap();
        walker.as_next();
        let v3 = walker.vertex_id().unwrap();
        if v1 < v2 {
            if v2 < v3 {
                (v1, v2, v3)
            } else {
                if v1 < v3 {
                    (v1, v3, v2)
                } else {
                    (v3, v1, v2)
                }
            }
        } else {
            if v1 < v3 {
                (v2, v1, v3)
            } else {
                if v2 < v3 {
                    (v2, v3, v1)
                } else {
                    (v3, v2, v1)
                }
            }
        }
    }
}
