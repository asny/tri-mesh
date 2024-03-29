//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;
use std::collections::HashSet;

impl Mesh {
    ///
    /// Merges overlapping faces, edges and vertices if it is possible without creating a non-manifold mesh.
    ///
    pub fn merge_overlapping_primitives(&mut self) {
        let set_of_vertices_to_merge = self.find_overlapping_vertices();
        let set_of_edges_to_merge = self.find_overlapping_edges(&set_of_vertices_to_merge);
        let set_of_faces_to_merge = self.find_overlapping_faces(&set_of_vertices_to_merge);

        for faces_to_merge in set_of_faces_to_merge {
            let mut iter = faces_to_merge.iter();
            iter.next();
            for face_id2 in iter {
                self.remove_face_unsafe(*face_id2);
            }
        }

        for vertices_to_merge in set_of_vertices_to_merge {
            let mut iter = vertices_to_merge.iter();
            let mut vertex_id1 = *iter.next().unwrap();
            for vertex_id2 in iter {
                vertex_id1 = self.merge_vertices(vertex_id1, *vertex_id2);
            }
        }

        for edges_to_merge in set_of_edges_to_merge {
            let mut iter = edges_to_merge.iter();
            let mut edge_id1 = *iter.next().unwrap();
            for edge_id2 in iter {
                if let Some(e) = self.merge_halfedges(edge_id1, *edge_id2) {
                    edge_id1 = e;
                }
            }
        }

        self.fix_orientation();
    }

    fn merge_halfedges(
        &mut self,
        halfedge_id1: HalfEdgeID,
        halfedge_id2: HalfEdgeID,
    ) -> Option<HalfEdgeID> {
        let mut walker1 = self.walker_from_halfedge(halfedge_id1);
        let mut walker2 = self.walker_from_halfedge(halfedge_id2);

        let edge1_alone = walker1.face_id().is_none() && walker1.as_twin().face_id().is_none();
        let edge1_interior = walker1.face_id().is_some() && walker1.as_twin().face_id().is_some();
        let edge1_boundary = !edge1_alone && !edge1_interior;

        let edge2_alone = walker2.face_id().is_none() && walker2.as_twin().face_id().is_none();
        let edge2_interior = walker2.face_id().is_some() && walker2.as_twin().face_id().is_some();
        let edge2_boundary = !edge2_alone && !edge2_interior;

        if edge1_interior && !edge2_alone || edge2_interior && !edge1_alone {
            // Skip since merging these halfedges will create a non-manifold mesh
            return None;
        }

        let mut halfedge_to_remove1 = None;
        let mut halfedge_to_remove2 = None;
        let mut halfedge_to_survive1 = None;
        let mut halfedge_to_survive2 = None;
        let mut vertex_id1 = None;
        let mut vertex_id2 = None;

        if edge1_boundary {
            if walker1.face_id().is_none() {
                walker1.as_twin();
            };
            halfedge_to_remove1 = walker1.twin_id();
            halfedge_to_survive1 = walker1.halfedge_id();
            vertex_id1 = walker1.vertex_id();
        }
        if edge2_boundary {
            if walker2.face_id().is_none() {
                walker2.as_twin();
            };
            halfedge_to_remove2 = walker2.twin_id();
            halfedge_to_survive2 = walker2.halfedge_id();
            vertex_id2 = walker2.vertex_id();
        }
        if edge1_alone {
            if edge2_interior {
                halfedge_to_remove1 = walker1.twin_id();
                halfedge_to_remove2 = walker1.halfedge_id();

                halfedge_to_survive1 = walker2.halfedge_id();
                vertex_id1 = walker2.vertex_id();
                walker2.as_twin();
                halfedge_to_survive2 = walker2.halfedge_id();
                vertex_id2 = walker2.vertex_id();
            } else {
                if vertex_id2 == walker1.vertex_id() {
                    walker1.as_twin();
                }
                halfedge_to_remove1 = walker1.twin_id();
                halfedge_to_survive1 = walker1.halfedge_id();
                vertex_id1 = walker1.vertex_id();
            }
        }
        if edge2_alone {
            if edge1_interior {
                halfedge_to_remove1 = walker2.twin_id();
                halfedge_to_remove2 = walker2.halfedge_id();

                halfedge_to_survive1 = walker1.halfedge_id();
                vertex_id1 = walker1.vertex_id();
                walker1.as_twin();
                halfedge_to_survive2 = walker1.halfedge_id();
                vertex_id2 = walker1.vertex_id();
            } else {
                if vertex_id1 == walker2.vertex_id() {
                    walker2.as_twin();
                }
                halfedge_to_remove2 = walker2.twin_id();
                halfedge_to_survive2 = walker2.halfedge_id();
                vertex_id2 = walker2.vertex_id();
            }
        }

        self.connectivity_info
            .remove_halfedge(halfedge_to_remove1.unwrap());
        self.connectivity_info
            .remove_halfedge(halfedge_to_remove2.unwrap());
        self.connectivity_info
            .set_halfedge_twin(halfedge_to_survive1.unwrap(), halfedge_to_survive2.unwrap());
        self.connectivity_info
            .set_vertex_halfedge(vertex_id1.unwrap(), halfedge_to_survive2);
        self.connectivity_info
            .set_vertex_halfedge(vertex_id2.unwrap(), halfedge_to_survive1);
        Some(halfedge_to_survive1.unwrap())
    }

    fn merge_vertices(&mut self, vertex_id1: VertexID, vertex_id2: VertexID) -> VertexID {
        for halfedge_id in self.halfedge_iter() {
            let walker = self.walker_from_halfedge(halfedge_id);
            if walker.vertex_id().unwrap() == vertex_id2 {
                self.connectivity_info
                    .set_halfedge_vertex(walker.halfedge_id().unwrap(), vertex_id1);
            }
        }
        self.connectivity_info.remove_vertex(vertex_id2);

        vertex_id1
    }

    fn find_overlapping_vertices(&self) -> Vec<Vec<VertexID>> {
        let mut to_check = HashSet::new();
        self.vertex_iter().for_each(|v| {
            to_check.insert(v);
        });

        let mut set_to_merge = Vec::new();
        while !to_check.is_empty() {
            let id1 = *to_check.iter().next().unwrap();
            to_check.remove(&id1);

            let mut to_merge = Vec::new();
            for id2 in to_check.iter() {
                if (self.vertex_position(id1) - self.vertex_position(*id2)).magnitude() < 0.00001 {
                    to_merge.push(*id2);
                }
            }
            if !to_merge.is_empty() {
                for id in to_merge.iter() {
                    to_check.remove(id);
                }
                to_merge.push(id1);
                set_to_merge.push(to_merge);
            }
        }
        set_to_merge
    }

    fn find_overlapping_faces(
        &self,
        set_of_vertices_to_merge: &Vec<Vec<VertexID>>,
    ) -> Vec<Vec<FaceID>> {
        let vertices_to_merge = |vertex_id| {
            set_of_vertices_to_merge
                .iter()
                .find(|vec| vec.contains(&vertex_id))
        };
        let mut to_check = HashSet::new();
        self.face_iter().for_each(|id| {
            to_check.insert(id);
        });

        let mut set_to_merge = Vec::new();
        while !to_check.is_empty() {
            let id1 = *to_check.iter().next().unwrap();
            to_check.remove(&id1);

            let (v0, v1, v2) = self.face_vertices(id1);
            if let Some(vertices_to_merge0) = vertices_to_merge(v0) {
                if let Some(vertices_to_merge1) = vertices_to_merge(v1) {
                    if let Some(vertices_to_merge2) = vertices_to_merge(v2) {
                        let mut to_merge = Vec::new();
                        for id2 in to_check.iter() {
                            let (v3, v4, v5) = self.face_vertices(*id2);
                            if (vertices_to_merge0.contains(&v3)
                                || vertices_to_merge0.contains(&v4)
                                || vertices_to_merge0.contains(&v5))
                                && (vertices_to_merge1.contains(&v3)
                                    || vertices_to_merge1.contains(&v4)
                                    || vertices_to_merge1.contains(&v5))
                                && (vertices_to_merge2.contains(&v3)
                                    || vertices_to_merge2.contains(&v4)
                                    || vertices_to_merge2.contains(&v5))
                            {
                                to_merge.push(*id2);
                            }
                        }
                        if !to_merge.is_empty() {
                            for id in to_merge.iter() {
                                to_check.remove(id);
                            }
                            to_merge.push(id1);
                            set_to_merge.push(to_merge);
                        }
                    }
                }
            }
        }
        set_to_merge
    }

    fn find_overlapping_edges(
        &self,
        set_of_vertices_to_merge: &Vec<Vec<VertexID>>,
    ) -> Vec<Vec<HalfEdgeID>> {
        let vertices_to_merge = |vertex_id| {
            set_of_vertices_to_merge
                .iter()
                .find(|vec| vec.contains(&vertex_id))
        };
        let mut to_check = HashSet::new();
        self.edge_iter().for_each(|e| {
            to_check.insert(e);
        });

        let mut set_to_merge = Vec::new();
        while !to_check.is_empty() {
            let id1 = *to_check.iter().next().unwrap();
            to_check.remove(&id1);

            let (v0, v1) = self.edge_vertices(id1);
            if let Some(vertices_to_merge0) = vertices_to_merge(v0) {
                if let Some(vertices_to_merge1) = vertices_to_merge(v1) {
                    let mut to_merge = Vec::new();
                    for id2 in to_check.iter() {
                        let (v0, v1) = self.edge_vertices(*id2);
                        if vertices_to_merge0.contains(&v0) && vertices_to_merge1.contains(&v1)
                            || vertices_to_merge1.contains(&v0) && vertices_to_merge0.contains(&v1)
                        {
                            to_merge.push(*id2);
                        }
                    }
                    if !to_merge.is_empty() {
                        for id in to_merge.iter() {
                            to_check.remove(id);
                        }
                        to_merge.push(id1);
                        set_to_merge.push(to_merge);
                    }
                }
            }
        }
        set_to_merge
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use three_d_asset::{Indices, Positions, TriMesh};

    #[test]
    fn test_remove_lonely_vertices() {
        let mut mesh = crate::test_utility::subdivided_triangle();
        let mut iter = mesh.face_iter();
        let face_id1 = iter.next().unwrap();
        let face_id2 = iter.next().unwrap();
        mesh.remove_face_unsafe(face_id1);
        mesh.remove_face_unsafe(face_id2);

        mesh.remove_lonely_primitives();

        assert_eq!(3, mesh.no_vertices());
        assert_eq!(6, mesh.no_halfedges());
        assert_eq!(1, mesh.no_faces());
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_merge_overlapping_primitives() {
        let positions = vec![
            vec3(0.0, 0.0, 0.0),
            vec3(1.0, 0.0, -0.5),
            vec3(-1.0, 0.0, -0.5),
            vec3(0.0, 0.0, 0.0),
            vec3(-1.0, 0.0, -0.5),
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            vec3(1.0, 0.0, -0.5),
        ];

        let mut mesh: Mesh = TriMesh {
            positions: Positions::F64(positions),
            ..Default::default()
        }
        .into();
        mesh.merge_overlapping_primitives();

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(12, mesh.no_halfedges());
        assert_eq!(3, mesh.no_faces());
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_merge_overlapping_primitives_of_cube() {
        let mut mesh: Mesh = TriMesh::cube().into();
        mesh.merge_overlapping_primitives();

        assert_eq!(8, mesh.no_vertices());
        assert_eq!(36, mesh.no_halfedges());
        assert_eq!(12, mesh.no_faces());
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_merge_overlapping_individual_faces() {
        let mut mesh: Mesh = TriMesh {
            positions: Positions::F64(vec![
                vec3(0.0, 0.0, 0.0),
                vec3(1.0, 0.0, -0.5),
                vec3(-1.0, 0.0, -0.5),
                vec3(0.0, 0.0, 0.0),
                vec3(-1.0, 0.0, -0.5),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 0.0, 0.0),
                vec3(-1.0, 0.0, -0.5),
                vec3(0.0, 0.0, 1.0),
            ]),
            ..Default::default()
        }
        .into();
        mesh.merge_overlapping_primitives();

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(10, mesh.no_halfedges());
        assert_eq!(2, mesh.no_faces());
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_merge_two_overlapping_faces() {
        let mut mesh: Mesh = TriMesh {
            indices: Indices::U8(vec![0, 1, 2, 1, 3, 2, 4, 6, 5, 6, 7, 5]),
            positions: Positions::F64(vec![
                vec3(0.0, 0.0, 0.0),
                vec3(-1.0, 0.0, 0.0),
                vec3(-0.5, 0.0, 1.0),
                vec3(-1.5, 0.0, 1.0),
                vec3(-1.0, 0.0, 0.0),
                vec3(-0.5, 0.0, 1.0),
                vec3(-1.5, 0.0, 1.0),
                vec3(-1.0, 0.0, 1.5),
            ]),
            ..Default::default()
        }
        .into();
        mesh.merge_overlapping_primitives();

        assert_eq!(5, mesh.no_vertices());
        assert_eq!(14, mesh.no_halfedges());
        assert_eq!(3, mesh.no_faces());
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_merge_three_overlapping_faces() {
        let mut mesh: Mesh = TriMesh {
            indices: Indices::U8(vec![0, 1, 2, 1, 3, 2, 4, 6, 5, 6, 7, 5, 8, 10, 9]),
            positions: Positions::F64(vec![
                vec3(0.0, 0.0, 0.0),
                vec3(-1.0, 0.0, 0.0),
                vec3(-0.5, 0.0, 1.0),
                vec3(-1.5, 0.0, 1.0),
                vec3(-1.0, 0.0, 0.0),
                vec3(-0.5, 0.0, 1.0),
                vec3(-1.5, 0.0, 1.0),
                vec3(-1.0, 0.0, 1.5),
                vec3(-1.0, 0.0, 0.0),
                vec3(-0.5, 0.0, 1.0),
                vec3(-1.5, 0.0, 1.0),
            ]),
            ..Default::default()
        }
        .into();
        mesh.merge_overlapping_primitives();

        assert_eq!(5, mesh.no_vertices());
        assert_eq!(14, mesh.no_halfedges());
        assert_eq!(3, mesh.no_faces());
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_merge_vertices() {
        let mut mesh: Mesh = TriMesh {
            positions: Positions::F64(vec![
                vec3(0.0, 0.0, 0.0),
                vec3(1.0, 0.0, -0.5),
                vec3(-1.0, 0.0, -0.5),
                vec3(0.0, 0.0, 0.0),
                vec3(-1.0, 0.0, -0.5),
                vec3(0.0, 0.0, 1.0),
            ]),
            ..Default::default()
        }
        .into();

        let mut vertex_id1 = None;
        for vertex_id in mesh.vertex_iter() {
            if mesh.vertex_position(vertex_id) == vec3(0.0, 0.0, 0.0) {
                if vertex_id1.is_none() {
                    vertex_id1 = Some(vertex_id);
                } else {
                    mesh.merge_vertices(vertex_id1.unwrap(), vertex_id);
                    break;
                }
            }
        }

        assert_eq!(5, mesh.no_vertices());
        assert_eq!(12, mesh.no_halfedges());
        assert_eq!(2, mesh.no_faces());
    }

    #[test]
    fn test_merge_halfedges() {
        let mut mesh: Mesh = TriMesh {
            positions: Positions::F64(vec![
                vec3(1.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, -1.0),
                vec3(0.0, 0.0, 0.0),
                vec3(1.0, 0.0, 0.0),
                vec3(0.0, 0.0, 1.0),
            ]),
            ..Default::default()
        }
        .into();

        let mut heid1 = None;
        for halfedge_id in mesh.edge_iter() {
            let (v0, v1) = mesh.edge_vertices(halfedge_id);
            if mesh.vertex_position(v0)[2] == 0.0 && mesh.vertex_position(v1)[2] == 0.0 {
                let halfedge_id = mesh.connecting_edge(v0, v1).unwrap();
                if heid1.is_none() {
                    heid1 = Some((halfedge_id, v0, v1));
                } else {
                    let (halfedge_id1, v10, v11) = heid1.unwrap();
                    mesh.merge_vertices(v0, v11);
                    mesh.merge_vertices(v1, v10);
                    mesh.merge_halfedges(halfedge_id1, halfedge_id).unwrap();
                    break;
                }
            }
        }

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(10, mesh.no_halfedges());
        assert_eq!(2, mesh.no_faces());
    }

    #[test]
    fn test_merge_overlapping_primitives_with_cube() {
        let mut mesh: Mesh = TriMesh::cube().into();
        mesh.merge_overlapping_primitives();
        assert_eq!(mesh.no_faces(), 12);
        assert_eq!(mesh.no_vertices(), 8);
        mesh.is_valid().unwrap();
    }
}
