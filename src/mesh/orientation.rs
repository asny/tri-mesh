use crate::mesh::Mesh;
use crate::mesh::ids::*;

/// # Orientation functionality
impl Mesh {
    pub fn flip_orientation(&mut self)
    {
        for face_id in self.face_iter() {
            self.flip_orientation_of_face(&face_id);
        }
    }

    pub(crate) fn flip_orientation_of_face(&mut self, face_id: &FaceID)
    {
        let mut update_list = [(None, None, None); 3];

        let mut i = 0;
        for mut walker in self.face_halfedge_iter(face_id) {
            let halfedge_id = walker.halfedge_id();
            let vertex_id = walker.vertex_id();
            walker.as_previous();
            update_list[i] = (halfedge_id.clone(), walker.vertex_id(), walker.halfedge_id());
            i += 1;

            self.connectivity_info.set_vertex_halfedge(&walker.vertex_id().unwrap(), walker.halfedge_id());

            walker.as_next().as_twin();
            if walker.face_id().is_none() {
                self.connectivity_info.set_vertex_halfedge(&walker.vertex_id().unwrap(), walker.halfedge_id());
                self.connectivity_info.set_halfedge_vertex(&walker.halfedge_id().unwrap(), vertex_id.unwrap());
            }
        }

        for (halfedge_id, new_vertex_id, new_next_id) in update_list.iter() {
            self.connectivity_info.set_halfedge_vertex(&halfedge_id.unwrap(), new_vertex_id.unwrap());
            self.connectivity_info.set_halfedge_next(&halfedge_id.unwrap(), *new_next_id);
        }
    }

    pub fn fix_orientation(&mut self)
    {
        let mut visited_faces = std::collections::HashSet::new();
        for face_id in self.face_iter() {
            self.correct_orientation_of(&face_id, &mut visited_faces);
        }
    }

    fn correct_orientation_of(&mut self, face_id: &FaceID, visited_faces: &mut std::collections::HashSet<FaceID>)
    {
        if !visited_faces.contains(face_id)
        {
            visited_faces.insert(*face_id);
            for mut walker in self.face_halfedge_iter(face_id) {
                let vertex_id = walker.vertex_id();
                if let Some(face_id_to_test) = walker.as_twin().face_id()
                {
                    if vertex_id == walker.vertex_id() {
                        self.flip_orientation_of_face(&face_id_to_test)
                    }
                    self.correct_orientation_of(&face_id_to_test, visited_faces);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utility::*;

    #[test]
    fn test_flip_orientation_of_face()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  1, 2, 3];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5];
        let mut mesh = crate::MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap();

        mesh.flip_orientation_of_face(&mesh.face_iter().next().unwrap());
        test_is_valid(&mesh).unwrap();

    }

    #[test]
    fn test_flip_orientation()
    {
        let mut mesh = crate::MeshBuilder::new().cube().build().unwrap();

        let mut map = std::collections::HashMap::new();
        for face_id in mesh.face_iter() {
            map.insert(face_id, mesh.face_normal(&face_id));
        }
        mesh.flip_orientation();

        test_is_valid(&mesh).unwrap();
        for face_id in mesh.face_iter() {
            assert_eq!(mesh.face_normal(&face_id), -*map.get(&face_id).unwrap());
        }
    }
}
