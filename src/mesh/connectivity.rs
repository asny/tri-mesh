
use crate::mesh::Mesh;
use crate::mesh::ids::*;

/// # Connectivity functionality
impl Mesh
{
    pub fn connecting_edge(&self, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Option<HalfEdgeID>
    {
        for walker in self.vertex_halfedge_iter(vertex_id1) {
            if &walker.vertex_id().unwrap() == vertex_id2 {
                return walker.halfedge_id()
            }
        }
        None
    }

    pub fn find_edge(&self, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Option<HalfEdgeID>
    {
        let mut walker = self.walker();
        for halfedge_id in self.halfedge_iter() {
            walker.as_halfedge_walker(&halfedge_id);
            if &walker.vertex_id().unwrap() == vertex_id2 && &walker.as_twin().vertex_id().unwrap() == vertex_id1
            {
                return Some(halfedge_id)
            }
        }
        None
    }

    pub fn vertex_on_boundary(&self, vertex_id: &VertexID) -> bool
    {
        for mut walker in self.vertex_halfedge_iter(vertex_id) {
            if walker.face_id().is_none() || walker.as_twin().face_id().is_none()
            {
                return true;
            }
        }
        false
    }

    pub fn on_boundary(&self, halfedge_id: &HalfEdgeID) -> bool
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        walker.face_id().is_none() || walker.as_twin().face_id().is_none()
    }

    pub fn edge_vertices(&self, halfedge_id: &HalfEdgeID) -> (VertexID, VertexID)
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let v1 = walker.vertex_id().unwrap();
        let v2 = walker.as_twin().vertex_id().unwrap();
        (v1, v2)
    }

    pub fn ordered_edge_vertices(&self, halfedge_id: &HalfEdgeID) -> (VertexID, VertexID)
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let v1 = walker.vertex_id().unwrap();
        let v2 = walker.as_twin().vertex_id().unwrap();
        if v1 < v2 { (v1, v2) } else { (v2, v1) }
    }

    pub fn face_vertices(&self, face_id: &FaceID) -> (VertexID, VertexID, VertexID)
    {
        let mut walker = self.walker_from_face(face_id);
        let v1 = walker.vertex_id().unwrap();
        walker.as_next();
        let v2 = walker.vertex_id().unwrap();
        walker.as_next();
        let v3 = walker.vertex_id().unwrap();
        (v1, v2, v3)
    }

    pub fn ordered_face_vertices(&self, face_id: &FaceID) -> (VertexID, VertexID, VertexID)
    {
        let mut walker = self.walker_from_face(face_id);
        let v1 = walker.vertex_id().unwrap();
        walker.as_next();
        let v2 = walker.vertex_id().unwrap();
        walker.as_next();
        let v3 = walker.vertex_id().unwrap();
        if v1 < v2 {
            if v2 < v3 { (v1, v2, v3) }
            else { if v1 < v3 { (v1, v3, v2) } else { (v3, v1, v2) } }
        }
        else {
            if v1 < v3 { (v2, v1, v3) }
            else { if v2 < v3 { (v2, v3, v1) } else { (v3, v2, v1) } }
        }
    }
}