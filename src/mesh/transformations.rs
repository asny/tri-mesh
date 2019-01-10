//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::Mesh;
use crate::mesh::math::*;
use crate::mesh::ids::*;

/// # Transformations
impl Mesh
{
    /// Moves the vertex to the specified position.
    pub fn move_vertex_to(&mut self, vertex_id: VertexID, value: Vec3)
    {
        self.positions.insert(vertex_id, value);
    }

    /// Moves the vertex by the specified vector, i.e. the new position is `mesh.vertex_position(vertex_id) + value`.
    pub fn move_vertex_by(&mut self, vertex_id: VertexID, value: Vec3)
    {
        let p = value + *self.vertex_position(&vertex_id);
        self.move_vertex_to(vertex_id, p);
    }

    /// Scales the entire mesh by `scale`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::prelude::*;
    /// #
    /// # fn main() -> Result<(), Box<tri_mesh::mesh_builder::Error>> {
    ///     let mut mesh = MeshBuilder::new().cube().build()?;
    /// #   let first_face_id = mesh.face_iter().next().unwrap();
    /// #   let face_area_before_scale = mesh.face_area(&first_face_id);
    ///     mesh.scale(2.0);
    /// #   let face_area_after_scale = mesh.face_area(&first_face_id);
    /// #   assert_eq!(4.0 * face_area_before_scale, face_area_after_scale);
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn scale(&mut self, scale: f32)
    {
        for vertex_id in self.vertex_iter() {
            let p = *self.vertex_position(&vertex_id);
            self.move_vertex_to(vertex_id, p * scale);
        }
    }

    /// Translates the entire mesh by `translation`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::prelude::*;
    /// #
    /// # fn main() -> Result<(), Box<tri_mesh::mesh_builder::Error>> {
    ///     let mut mesh = MeshBuilder::new().cube().build()?;
    /// #   let first_vertex_id = mesh.vertex_iter().next().unwrap();
    /// #   let vertex_position_before_scale = *mesh.vertex_position(&first_vertex_id);
    ///     mesh.translate(vec3(2.5, -1.0, 0.0));
    /// #   let vertex_position_after_scale = *mesh.vertex_position(&first_vertex_id);
    /// #   assert_eq!(vertex_position_before_scale + vec3(2.5, -1.0, 0.0), vertex_position_after_scale);
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn translate(&mut self, translation: Vec3)
    {
        for vertex_id in self.vertex_iter() {
            self.move_vertex_by(vertex_id, translation);
        }
    }

    pub fn rotate(&mut self, rotation: Mat3)
    {
        for vertex_id in self.vertex_iter() {
            let p = *self.vertex_position(&vertex_id);
            self.move_vertex_to(vertex_id, rotation * p);
        }
    }

    pub fn apply_transformation(&mut self, transformation: Mat4)
    {
        for vertex_id in self.vertex_iter() {
            let p = *self.vertex_position(&vertex_id);
            let p_new = (transformation * p.extend(1.0)).truncate();
            self.move_vertex_to(vertex_id, p_new);
        }
    }
}