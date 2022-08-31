//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;

/// # Orientation
impl Mesh {
    /// Flip the orientation of all faces in the mesh, ie. such that the normal points in the opposite direction.
    pub fn flip_orientation(&mut self) {
        for face_id in self.face_iter() {
            self.flip_orientation_of_face(face_id);
        }
    }

    /// Fix the orientation of all faces in the mesh such that the orientation of each pair of neighbouring faces is aligned.
    pub fn fix_orientation(&mut self) {
        let mut visited_faces = std::collections::HashMap::new();
        for face_id in self.face_iter() {
            self.find_faces_to_flip_orientation(face_id, &mut visited_faces, false);
        }
        for (face_id, should_flip) in visited_faces {
            if should_flip {
                self.flip_orientation_of_face(face_id)
            }
        }
    }

    fn flip_orientation_of_face(&mut self, face_id: FaceID) {
        let mut update_list = [(None, None, None); 3];

        let mut i = 0;
        for halfedge_id in self.face_halfedge_iter(face_id) {
            let mut walker = self.walker_from_halfedge(halfedge_id);
            let vertex_id = walker.vertex_id();
            walker.as_previous();
            update_list[i] = (Some(halfedge_id), walker.vertex_id(), walker.halfedge_id());
            i += 1;

            self.connectivity_info
                .set_vertex_halfedge(walker.vertex_id().unwrap(), walker.halfedge_id());

            walker.as_next().as_twin();
            if walker.face_id().is_none() {
                self.connectivity_info
                    .set_vertex_halfedge(walker.vertex_id().unwrap(), walker.halfedge_id());
                self.connectivity_info
                    .set_halfedge_vertex(walker.halfedge_id().unwrap(), vertex_id.unwrap());
            }
        }

        for (halfedge_id, new_vertex_id, new_next_id) in update_list.iter() {
            self.connectivity_info
                .set_halfedge_vertex(halfedge_id.unwrap(), new_vertex_id.unwrap());
            self.connectivity_info
                .set_halfedge_next(halfedge_id.unwrap(), *new_next_id);
        }
    }

    fn find_faces_to_flip_orientation(
        &self,
        face_id: FaceID,
        visited_faces: &mut std::collections::HashMap<FaceID, bool>,
        should_flip: bool,
    ) {
        if !visited_faces.contains_key(&face_id) {
            visited_faces.insert(face_id, should_flip);
            for halfedge_id in self.face_halfedge_iter(face_id) {
                let mut walker = self.walker_from_halfedge(halfedge_id);
                let vertex_id = walker.vertex_id();
                if let Some(face_id_to_test) = walker.as_twin().face_id() {
                    let is_opposite = vertex_id == walker.vertex_id();
                    self.find_faces_to_flip_orientation(
                        face_id_to_test,
                        visited_faces,
                        is_opposite && !should_flip || !is_opposite && should_flip,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_orientation_of_face() {
        let indices: Vec<u32> = vec![0, 1, 2, 1, 2, 3];
        let positions: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.5, 1.0, 0.0, 1.5];
        let mut mesh = crate::MeshBuilder::new()
            .with_indices(indices)
            .with_positions(positions)
            .build()
            .unwrap();

        mesh.flip_orientation_of_face(mesh.face_iter().next().unwrap());
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_flip_orientation() {
        let mut mesh: Mesh = three_d_asset::TriMesh::sphere(4).into();

        let mut map = std::collections::HashMap::new();
        for face_id in mesh.face_iter() {
            map.insert(face_id, mesh.face_normal(face_id));
        }
        mesh.flip_orientation();

        mesh.is_valid().unwrap();
        for face_id in mesh.face_iter() {
            assert!((mesh.face_normal(face_id) - -*map.get(&face_id).unwrap()).magnitude() < 0.001);
        }
    }
}
