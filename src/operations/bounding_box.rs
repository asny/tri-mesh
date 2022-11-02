//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;

pub use three_d_asset::AxisAlignedBoundingBox;

/// # Bounding box
impl Mesh {
    /// Returns the smallest axis aligned box which contains the entire mesh, ie. the axis aligned bounding box.
    pub fn axis_aligned_bounding_box(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox::new_with_positions(
            &self
                .vertex_iter()
                .map(|v| self.position(v).cast::<f32>().unwrap())
                .collect::<Vec<_>>(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use three_d_asset::TriMesh;

    #[test]
    fn test_axis_aligned_bounding_box() {
        let mut mesh: Mesh = TriMesh::cylinder(16).into();
        mesh.non_uniform_scale(4.5, 0.1, -4.5);
        mesh.translate(vec3(-1.5, 3.7, 9.1));

        let bb = mesh.axis_aligned_bounding_box();

        assert_eq!(bb.min(), Vector3::new(-1.5, 3.6, 4.6));
        assert_eq!(bb.max(), Vector3::new(3.0, 3.8, 13.6));
    }

    #[test]
    fn test_extreme_coordinates() {
        let mesh: Mesh = TriMesh::sphere(4).into();
        let bb = mesh.axis_aligned_bounding_box();
        assert_eq!(bb.min(), Vector3::new(-1.0, -1.0, -1.0));
        assert_eq!(bb.max(), Vector3::new(1.0, 1.0, 1.0));
    }
}
