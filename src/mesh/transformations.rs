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
        self.connectivity_info.set_position(vertex_id, value);
    }

    /// Moves the vertex by the specified vector, i.e. the new position is `mesh.vertex_position(vertex_id) + value`.
    pub fn move_vertex_by(&mut self, vertex_id: VertexID, value: Vec3)
    {
        let p = value + self.vertex_position(vertex_id);
        self.move_vertex_to(vertex_id, p);
    }

    /// Scales the entire mesh by multiplying `scale` to each vertex position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::prelude::*;
    /// #
    /// # fn main() -> Result<(), Box<tri_mesh::mesh_builder::Error>> {
    ///     let mut mesh = MeshBuilder::new().cube().build()?;
    /// #   let first_face_id = mesh.face_iter().next().unwrap();
    /// #   let face_area_before = mesh.face_area(first_face_id);
    ///     mesh.scale(2.0);
    /// #   let face_area_after = mesh.face_area(first_face_id);
    /// #   assert_eq!(4.0 * face_area_before, face_area_after);
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn scale(&mut self, scale: f64)
    {
        for vertex_id in self.vertex_iter() {
            let p = self.vertex_position(vertex_id);
            self.move_vertex_to(vertex_id, p * scale);
        }
    }

    /// Scales the entire mesh by multiplying `scale_x` to the x component of each vertex position, `scale_y` to the y component and `scale_z` to the z component.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::prelude::*;
    /// #
    /// # fn main() -> Result<(), Box<tri_mesh::mesh_builder::Error>> {
    ///     let mut mesh = MeshBuilder::new().cube().build()?;
    /// #   let first_face_id = mesh.face_iter().find(|f| mesh.face_normal(*f) == vec3(0.0, 1.0, 0.0)).unwrap();
    /// #   let second_face_id = mesh.face_iter().find(|f| mesh.face_normal(*f) == vec3(1.0, 0.0, 0.0)).unwrap();
    /// #   let face_area_before1 = mesh.face_area(first_face_id);
    /// #   let face_area_before2 = mesh.face_area(second_face_id);
    ///     mesh.non_uniform_scale(2.0, 1.0, 1.0);
    /// #   assert_eq!(2.0 * face_area_before1, mesh.face_area(first_face_id));
    /// #   assert_eq!(face_area_before2, mesh.face_area(second_face_id));
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn non_uniform_scale(&mut self, scale_x: f64, scale_y: f64, scale_z: f64)
    {
        for vertex_id in self.vertex_iter() {
            let p = self.vertex_position(vertex_id);
            self.move_vertex_to(vertex_id, vec3(p.x * scale_x, p.y * scale_y, p.z * scale_z));
        }
    }

    /// Translates the entire mesh by applying the `translation` to each vertex position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::prelude::*;
    /// #
    /// # fn main() -> Result<(), Box<tri_mesh::mesh_builder::Error>> {
    ///     let mut mesh = MeshBuilder::new().cube().build()?;
    /// #   let first_vertex_id = mesh.vertex_iter().next().unwrap();
    /// #   let vertex_position_before = mesh.vertex_position(first_vertex_id);
    ///     mesh.translate(vec3(2.5, -1.0, 0.0));
    /// #   let vertex_position_after = mesh.vertex_position(first_vertex_id);
    /// #   assert_eq!(vertex_position_before + vec3(2.5, -1.0, 0.0), vertex_position_after);
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

    ///
    /// Rotates the entire mesh by applying the given `rotation` to each vertex position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::prelude::*;
    /// #
    /// # fn main() -> Result<(), Box<tri_mesh::mesh_builder::Error>> {
    ///     let mut mesh = MeshBuilder::new().cube().build()?;
    /// #   let first_vertex_id = mesh.vertex_iter().next().unwrap();
    /// #   let vertex_position_before = mesh.vertex_position(first_vertex_id);
    ///     mesh.apply_transformation(Mat4::from_angle_y(Deg(360.0)));
    /// #   let vertex_position_after = mesh.vertex_position(first_vertex_id);
    /// #   assert!((vertex_position_before - vertex_position_after).magnitude() < 0.000001);
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn rotate(&mut self, rotation: Mat3)
    {
        for vertex_id in self.vertex_iter() {
            let p = self.vertex_position(vertex_id);
            self.move_vertex_to(vertex_id, rotation * p);
        }
    }

    ///
    /// Transforms the entire mesh by applying the `transformation` to each vertex position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::prelude::*;
    /// #
    /// # fn main() -> Result<(), Box<tri_mesh::mesh_builder::Error>> {
    ///     let mut mesh = MeshBuilder::new().cube().build()?;
    /// #   let first_vertex_id = mesh.vertex_iter().next().unwrap();
    /// #   let vertex_position_before = mesh.vertex_position(first_vertex_id);
    ///     mesh.apply_transformation(Mat4::from_translation(vec3(2.5, -1.0, 0.0)));
    /// #   let vertex_position_after = mesh.vertex_position(first_vertex_id);
    /// #   assert_eq!(vertex_position_before + vec3(2.5, -1.0, 0.0), vertex_position_after);
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn apply_transformation(&mut self, transformation: Mat4)
    {
        for vertex_id in self.vertex_iter() {
            let p = self.vertex_position(vertex_id);
            let p_new = (transformation * p.extend(1.0)).truncate();
            self.move_vertex_to(vertex_id, p_new);
        }
    }
}