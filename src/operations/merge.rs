//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;
use crate::TriMeshResult;

/// # Merge
impl Mesh {
    ///
    /// Merges the mesh together with the `other` mesh.
    /// The `other` mesh primitives are copied to the current mesh (and `other` is therefore not changed)
    /// followed by merging of overlapping primitives.
    ///
    /// # Error
    ///
    /// Returns an error if the merging will result in a non-manifold mesh.
    ///
    pub fn merge_with(&mut self, other: &Self) -> TriMeshResult<()> {
        self.append(other);
        self.merge_overlapping_primitives()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MeshBuilder;

    #[test]
    fn test_face_face_merging_at_edge() {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f64> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new()
            .with_indices(indices1)
            .with_positions(positions1)
            .build()
            .unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f64> = vec![-2.0, 0.0, 2.0, -2.0, 0.0, -2.0, -2.0, 0.5, 0.0];
        let mesh2 = MeshBuilder::new()
            .with_indices(indices2)
            .with_positions(positions2)
            .build()
            .unwrap();

        mesh1.merge_with(&mesh2).unwrap();

        assert_eq!(mesh1.no_faces(), 2);
        assert_eq!(mesh1.no_vertices(), 4);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_box_box_merge() {
        let mut mesh1 = crate::test_utility::cube();
        let mut mesh2 = crate::test_utility::cube();
        mesh2.translate(vec3(0.5, 0.5, 0.5));

        let (meshes1, meshes2) = mesh1.split_at_intersection(&mut mesh2);

        let mut result = meshes1.first().unwrap().clone();
        result.merge_with(meshes2.first().unwrap()).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
        result.is_valid().unwrap();
    }
}
