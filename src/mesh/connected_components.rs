//! See [Mesh](crate::mesh::Mesh).

use crate::prelude::*;
use std::collections::HashSet;

/// # Connected components
impl Mesh
{
    ///
    /// Finds the connected set of faces starting from the given face.
    ///
    pub fn connected_component(&self, start_face_id: FaceID) -> HashSet<FaceID>
    {
        self.connected_component_with_limit(start_face_id, &|_| false )
    }

    ///
    /// Finds all the sets of connected faces.
    ///
    pub fn connected_components(&self) -> Vec<HashSet<FaceID>>
    {
        self.connected_components_with_limit(&|_| false )
    }

    ///
    /// Finds the connected set of faces starting from the given face and limited by the given limit function.
    ///
    pub fn connected_component_with_limit(&self, start_face_id: FaceID, limit: &Fn(HalfEdgeID) -> bool) -> HashSet<FaceID>
    {
        let mut component = HashSet::new();
        component.insert(start_face_id);
        let mut to_be_tested = vec![start_face_id];
        while let Some(test_face) = to_be_tested.pop()
        {
            for halfedge_id in self.face_halfedge_iter(test_face) {
                if !limit(halfedge_id) {
                    if let Some(face_id) = self.walker_from_halfedge(halfedge_id).as_twin().face_id() {
                        if !component.contains(&face_id)
                        {
                            component.insert(face_id);
                            to_be_tested.push(face_id);
                        }
                    }
                }
            }
        }
        component
    }

    ///
    /// Finds all the sets of connected faces which are limited by the given limit function.
    ///
    pub fn connected_components_with_limit(&self, limit: &Fn(HalfEdgeID) -> bool) -> Vec<HashSet<FaceID>>
    {
        let mut components: Vec<HashSet<FaceID>> = Vec::new();
        for face_id in self.face_iter() {
            if components.iter().find(|com| com.contains(&face_id)).is_none() {
                components.push(self.connected_component_with_limit(face_id, limit));
            }
        }
        components
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh_builder::MeshBuilder;

    #[test]
    fn test_one_connected_component()
    {
        let mesh = create_connected_test_object();
        let cc = mesh.connected_component(mesh.face_iter().next().unwrap());
        assert_eq!(cc.len(), mesh.no_faces());
    }

    #[test]
    fn test_connected_components()
    {
        let mesh = create_unconnected_test_object();
        let cc = mesh.connected_components();

        assert_eq!(cc.len(), 3);

        assert_eq!(cc[0].len() + cc[1].len() + cc[2].len(), 15);
        assert!(cc.iter().find(|vec| vec.len() == 12).is_some());
        assert!(cc.iter().find(|vec| vec.len() == 2).is_some());
        assert!(cc.iter().find(|vec| vec.len() == 1).is_some());
    }

    fn create_connected_test_object() -> Mesh
    {
        let positions: Vec<f64> = vec![
            1.0, -1.0, -1.0,
            1.0, -1.0, 1.0,
            -1.0, -1.0, 1.0,
            -1.0, -1.0, -1.0,
            1.0, 1.0, -1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0,
            -1.0, 1.0, -1.0
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2,
            0, 2, 3,
            4, 7, 6,
            4, 6, 5,
            0, 4, 5,
            0, 5, 1,
            1, 5, 6,
            1, 6, 2,
            2, 6, 7,
            2, 7, 3,
            4, 0, 3,
            4, 3, 7
        ];
        MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap()
    }

    fn create_unconnected_test_object() -> Mesh
    {
        let positions: Vec<f64> = vec![
            1.0, -1.0, -1.0,
            1.0, -1.0, 1.0,
            -1.0, -1.0, 1.0,
            -1.0, -1.0, -1.0,
            1.0, 1.0, -1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0,
            -1.0, 1.0, -1.0,

            -1.0, 2.0, -1.0,
            -1.0, 3.0, -1.0,
            -2.0, 4.0, -1.0,
            -2.0, 1.0, -1.0,

            -1.0, 3.0, -2.0,
            -2.0, 4.0, -3.0,
            -2.0, 1.0, -4.0
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2,
            0, 2, 3,
            4, 7, 6,
            4, 6, 5,
            0, 4, 5,
            0, 5, 1,
            1, 5, 6,
            1, 6, 2,
            2, 6, 7,
            2, 7, 3,
            4, 0, 3,
            4, 3, 7,

            8, 9, 10,
            8, 10, 11,

            12, 13, 14
        ];

        MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap()
    }
}