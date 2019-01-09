
use crate::mesh::Mesh;
use crate::mesh::math::*;
use crate::mesh::ids::*;

/// # Edge measures
impl Mesh
{
    pub fn edge_positions(&self, halfedge_id: &HalfEdgeID) -> (&Vec3, &Vec3)
    {
        let vertices = self.ordered_edge_vertices(halfedge_id);
        (self.vertex_position(&vertices.0), self.vertex_position(&vertices.1))
    }

    pub fn edge_length(&self, halfedge_id: &HalfEdgeID) -> f32
    {
        let (p0, p1) = self.edge_positions(halfedge_id);
        (p0 - p1).magnitude()
    }

    pub fn edge_sqr_length(&self, halfedge_id: &HalfEdgeID) -> f32
    {
        let (p0, p1) = self.edge_positions(halfedge_id);
        (p0 - p1).magnitude2()
    }
}