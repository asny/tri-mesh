
use crate::mesh::Mesh;
use crate::mesh::math::*;
use crate::mesh::ids::*;

/// # Transformations
impl Mesh
{
    pub fn move_vertex_to(&mut self, vertex_id: VertexID, value: Vec3)
    {
        self.positions.insert(vertex_id, value);
    }

    pub fn move_vertex_by(&mut self, vertex_id: VertexID, value: Vec3)
    {
        let mut p = value;
        {
            p = p + *self.positions.get(&vertex_id).unwrap();
        }
        self.positions.insert(vertex_id, p);
    }

    pub fn scale(&mut self, scale: f32)
    {
        for vertex_id in self.vertex_iter() {
            let p = *self.vertex_position(&vertex_id);
            self.move_vertex_to(vertex_id, p * scale);
        }
    }

    pub fn translate(&mut self, translation: &Vec3)
    {
        for vertex_id in self.vertex_iter() {
            self.move_vertex_by(vertex_id, *translation);
        }
    }
}