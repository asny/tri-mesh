//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;

/// # Merge
impl Mesh {
    ///
    /// Merges the mesh together with the `other` mesh.
    /// The `other` mesh primitives are copied to the current mesh (and `other` is therefore not changed)
    /// followed by merging of overlapping primitives.
    ///
    pub fn merge_with(&mut self, other: &Self) {
        self.append(other);
        self.merge_overlapping_primitives();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use three_d_asset::{Positions, TriMesh};

    #[test]
    fn test_face_face_merging_at_edge() {
        let mut mesh1: Mesh = TriMesh {
            positions: Positions::F64(vec![
                vec3(-2.0, 0.0, -2.0),
                vec3(-2.0, 0.0, 2.0),
                vec3(2.0, 0.0, 0.0),
            ]),
            ..Default::default()
        }
        .into();

        let mesh2: Mesh = TriMesh {
            positions: Positions::F64(vec![
                vec3(-2.0, 0.0, 2.0),
                vec3(-2.0, 0.0, -2.0),
                vec3(-2.0, 0.5, 0.0),
            ]),
            ..Default::default()
        }
        .into();

        mesh1.merge_with(&mesh2);

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
        result.merge_with(meshes2.first().unwrap());

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
        result.is_valid().unwrap();
    }
}
