//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;
use crate::TriMeshResult;
use std::collections::HashMap;

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

    /// Appends the `other` mesh to this mesh without creating a connection between them.
    /// Use `merge_with` if merging of overlapping primitives is desired, thereby creating a connection.
    /// All the primitives of the `other` mesh are copied to the current mesh and the `other` mesh is therefore not changed.
    pub fn append(&mut self, other: &Self) {
        let mut mapping: HashMap<VertexID, VertexID> = HashMap::new();
        let mut get_or_create_vertex = |mesh: &mut Mesh, vertex_id| -> VertexID {
            if let Some(vid) = mapping.get(&vertex_id) {
                return *vid;
            }
            let p = other.vertex_position(vertex_id);
            let vid = mesh.connectivity_info.new_vertex(p.clone());
            mapping.insert(vertex_id, vid);
            vid
        };

        let mut face_mapping: HashMap<FaceID, FaceID> = HashMap::new();
        for other_face_id in other.face_iter() {
            let vertex_ids = other.face_vertices(other_face_id);

            let vertex_id0 = get_or_create_vertex(self, vertex_ids.0);
            let vertex_id1 = get_or_create_vertex(self, vertex_ids.1);
            let vertex_id2 = get_or_create_vertex(self, vertex_ids.2);
            let new_face_id = self
                .connectivity_info
                .create_face(vertex_id0, vertex_id1, vertex_id2);

            for halfedge_id in other.face_halfedge_iter(other_face_id) {
                if let Some(fid) = other.walker_from_halfedge(halfedge_id).as_twin().face_id() {
                    if let Some(self_face_id) = face_mapping.get(&fid) {
                        for halfedge_id1 in self.face_halfedge_iter(*self_face_id) {
                            let mut walker1 = self.walker_from_halfedge(halfedge_id1);
                            let source_vertex_id = walker1.vertex_id().unwrap();
                            let sink_vertex_id = walker1.as_next().vertex_id().unwrap();

                            for halfedge_id2 in self.face_halfedge_iter(new_face_id) {
                                let mut walker2 = self.walker_from_halfedge(halfedge_id2);
                                if sink_vertex_id == walker2.vertex_id().unwrap()
                                    && source_vertex_id == walker2.as_next().vertex_id().unwrap()
                                {
                                    self.connectivity_info.set_halfedge_twin(
                                        walker1.halfedge_id().unwrap(),
                                        walker2.halfedge_id().unwrap(),
                                    );
                                }
                            }
                        }
                    }
                }
            }

            face_mapping.insert(other_face_id, new_face_id);
        }

        self.create_boundary_edges();
    }

    fn create_boundary_edges(&mut self) {
        let mut walker = self.walker();
        for halfedge_id in self.halfedge_iter() {
            walker.as_halfedge_walker(halfedge_id);
            if walker.twin_id().is_none() {
                let boundary_halfedge_id = self.connectivity_info.new_halfedge(
                    walker.as_previous().vertex_id(),
                    None,
                    None,
                );
                self.connectivity_info
                    .set_halfedge_twin(halfedge_id, boundary_halfedge_id);
            }
        }
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
    fn test_face_face_merging_at_edge_when_orientation_is_opposite() {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f64> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new()
            .with_indices(indices1)
            .with_positions(positions1)
            .build()
            .unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f64> = vec![-2.0, 0.0, 2.0, -2.0, 0.5, 0.0, -2.0, 0.0, -2.0];
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
    fn test_box_icosahedron_append() {
        let mut mesh1 = MeshBuilder::new().cube().build().unwrap();
        let mut mesh2 = MeshBuilder::new().icosahedron().build().unwrap();
        mesh2.translate(vec3(0.5, 0.5, 0.5));

        mesh1.append(&mesh2);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        assert_eq!(mesh1.no_vertices(), mesh2.no_vertices() + 8);
        assert_eq!(mesh1.no_halfedges(), mesh2.no_halfedges() + 36);
        assert_eq!(mesh1.no_faces(), mesh2.no_faces() + 12);

        for pos in mesh2.vertex_iter().map(|v| mesh2.vertex_position(v)) {
            assert!(mesh1
                .vertex_iter()
                .find(|v| mesh1.vertex_position(*v) == pos)
                .is_some());
        }
    }

    #[test]
    fn test_box_box_merge() {
        let mut mesh1 = MeshBuilder::new().cube().build().unwrap();
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        mesh2.translate(vec3(0.5, 0.5, 0.5));

        let (meshes1, meshes2) = mesh1.split_at_intersection(&mut mesh2);

        let mut result = meshes1.first().unwrap().clone();
        result.merge_with(meshes2.first().unwrap()).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
        result.is_valid().unwrap();
    }
}
