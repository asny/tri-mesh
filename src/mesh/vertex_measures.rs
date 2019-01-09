
use crate::mesh::Mesh;
use crate::mesh::math::*;
use crate::mesh::ids::*;

/// # Vertex measures
impl Mesh
{
    pub fn vertex_position(&self, vertex_id: &VertexID) -> &Vec3
    {
        self.positions.get(vertex_id).unwrap()
    }

    pub fn vertex_normal(&self, vertex_id: &VertexID) -> Vec3
    {
        let mut normal = vec3(0.0, 0.0, 0.0);
        for walker in self.vertex_halfedge_iter(&vertex_id) {
            if let Some(face_id) = walker.face_id() {
                normal = normal + self.face_normal(&face_id)
            }
        }
        normal.normalize()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utility::*;
    
    #[test]
    fn test_vertex_normal() {
        let mesh = create_three_connected_faces();
        let computed_normal = mesh.vertex_normal(&VertexID::new(0));
        assert_eq!(0.0, computed_normal.x);
        assert_eq!(1.0, computed_normal.y);
        assert_eq!(0.0, computed_normal.z);
    }
}