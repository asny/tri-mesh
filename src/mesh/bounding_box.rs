//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::Mesh;
use crate::mesh::math::*;

/// # Bounding box
impl Mesh
{
    /// Returns minimum and maximum coordinates of the axis aligned bounding box of the mesh.
    pub fn extreme_coordinates(&self) -> (Vec3, Vec3)
    {
        let mut min_coordinates = vec3(std::f32::MAX, std::f32::MAX, std::f32::MAX);
        let mut max_coordinates = vec3(std::f32::MIN, std::f32::MIN, std::f32::MIN);
        for vertex_id in self.vertex_iter()
        {
            let position = self.vertex_position(vertex_id);
            for i in 0..3 {
                min_coordinates[i] = min_coordinates[i].min(position[i]);
                max_coordinates[i] = max_coordinates[i].max(position[i]);
            }
        }
        (min_coordinates, max_coordinates)
    }

    /// Returns the center of the smallest axis aligned box which contains the entire mesh, ie. the axis aligned bounding box.
    pub fn axis_aligned_bounding_box_center(&self) -> Vec3
    {
        let (min_coord, max_coord) = self.extreme_coordinates();
        0.5 * (max_coord + min_coord)
    }

    /// Returns the smallest axis aligned box which contains the entire mesh, ie. the axis aligned bounding box.
    pub fn axis_aligned_bounding_box(&self) -> Mesh
    {
        let (min_coord, max_coord) = self.extreme_coordinates();
        let mut mesh = crate::MeshBuilder::new().cube().build().unwrap();
        let scale = 0.5 * (max_coord - min_coord);
        mesh.non_uniform_scale(scale.x, scale.y, scale.z);
        let translation = 0.5 * (max_coord + min_coord);
        mesh.translate(translation);
        mesh
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MeshBuilder;

    #[test]
    fn test_axis_aligned_bounding_box() {
        let mut mesh = MeshBuilder::new().cylinder(3, 16).build().unwrap();
        mesh.non_uniform_scale(4.5, 0.1, -4.5);
        mesh.translate(vec3(-1.5, 3.7, 9.1));

        let bb = mesh.axis_aligned_bounding_box();

        assert_eq!(bb.extreme_coordinates(), mesh.extreme_coordinates());
    }
}