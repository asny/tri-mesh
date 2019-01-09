
use crate::mesh::Mesh;
use crate::mesh::math::*;
use crate::mesh::ids::*;
use std::collections::{HashSet, HashMap};

#[derive(Debug)]
pub enum Error {
    MergeWillCreateNonManifoldMesh {message: String},
    FailedToMergeVertices {message: String},
}

/// # Merging & splitting
impl Mesh
{
    pub fn merge_with(&mut self, other: &Self) -> Result<(), Error>
    {
        self.append(other);
        self.merge_overlapping_primitives()?;
        Ok(())
    }

    pub fn append(&mut self, other: &Self)
    {
        let mut mapping: HashMap<VertexID, VertexID> = HashMap::new();
        let mut get_or_create_vertex = |mesh: &mut Mesh, vertex_id| -> VertexID {
            if let Some(vid) = mapping.get(&vertex_id) {return vid.clone();}
            let p = other.vertex_position(&vertex_id);
            let vid = mesh.create_vertex(p.clone());
            mapping.insert(vertex_id, vid);
            vid
        };

        let mut face_mapping: HashMap<FaceID, FaceID> = HashMap::new();
        for other_face_id in other.face_iter() {
            let vertex_ids = other.face_vertices(&other_face_id);

            let vertex_id0 = get_or_create_vertex(self, vertex_ids.0);
            let vertex_id1 = get_or_create_vertex(self, vertex_ids.1);
            let vertex_id2 = get_or_create_vertex(self, vertex_ids.2);
            let new_face_id = self.connectivity_info.create_face(&vertex_id0, &vertex_id1, &vertex_id2);

            for mut walker in other.face_halfedge_iter(&other_face_id) {
                if let Some(fid) = walker.as_twin().face_id()
                {
                    if let Some(self_face_id) = face_mapping.get(&fid)
                    {
                        for mut walker1 in self.face_halfedge_iter(&self_face_id)
                        {
                            let source_vertex_id = walker1.vertex_id().unwrap();
                            let sink_vertex_id = walker1.as_next().vertex_id().unwrap();

                            for mut walker2 in self.face_halfedge_iter(&new_face_id)
                            {
                                if sink_vertex_id == walker2.vertex_id().unwrap() && source_vertex_id == walker2.as_next().vertex_id().unwrap() {
                                    self.connectivity_info.set_halfedge_twin(walker1.halfedge_id().unwrap(), walker2.halfedge_id().unwrap());
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

    pub fn clone_subset(&self, faces: &std::collections::HashSet<FaceID>) -> Mesh
    {
        let info = crate::mesh::ConnectivityInfo::new(faces.len(), faces.len());
        for face_id in faces {
            let face = self.connectivity_info.face(face_id).unwrap();
            for mut walker in self.face_halfedge_iter(face_id) {
                let halfedge_id = walker.halfedge_id().unwrap();
                let halfedge = self.connectivity_info.halfedge(&halfedge_id).unwrap();
                info.add_halfedge(halfedge_id, halfedge);

                let vertex_id = walker.vertex_id().unwrap();
                let vertex = self.connectivity_info.vertex(&vertex_id).unwrap();
                info.add_vertex(vertex_id, vertex);
                info.set_vertex_halfedge(&vertex_id, walker.next_id());

                walker.as_twin();
                if walker.face_id().is_none()
                {
                    let twin_id = walker.halfedge_id().unwrap();
                    let twin = self.connectivity_info.halfedge(&twin_id).unwrap();
                    info.add_halfedge(twin_id, twin);

                }
                else if !faces.contains(&walker.face_id().unwrap())
                {
                    let twin_id = walker.halfedge_id().unwrap();
                    let mut twin = self.connectivity_info.halfedge(&twin_id).unwrap();
                    twin.face = None;
                    twin.next = None;
                    info.add_halfedge(twin_id, twin);
                }
            }

            info.add_face(face_id.clone(), face);
        }

        let mut positions = HashMap::with_capacity(info.no_vertices());
        for vertex_id in info.vertex_iterator() {
            positions.insert(vertex_id.clone(), self.vertex_position(&vertex_id).clone());
        }

        Mesh::new_internal(positions, std::rc::Rc::new(info))
    }

    pub fn merge_overlapping_primitives(&mut self) -> Result<(), Error>
    {
        let set_of_vertices_to_merge = self.find_overlapping_vertices();
        let set_of_edges_to_merge = self.find_overlapping_edges(&set_of_vertices_to_merge);
        let set_of_faces_to_merge = self.find_overlapping_faces(&set_of_vertices_to_merge);

        for faces_to_merge in set_of_faces_to_merge {
            let mut iter = faces_to_merge.iter();
            iter.next();
            for face_id2 in iter {
                self.remove_face_unsafe(&face_id2);
            }
        }

        for vertices_to_merge in set_of_vertices_to_merge {
            let mut iter = vertices_to_merge.iter();
            let mut vertex_id1 = *iter.next().unwrap();
            for vertex_id2 in iter {
                vertex_id1 = self.merge_vertices(&vertex_id1, vertex_id2)?;
            }
        }

        for edges_to_merge in set_of_edges_to_merge {
            let mut iter = edges_to_merge.iter();
            let mut edge_id1 = *iter.next().unwrap();
            for edge_id2 in iter {
                edge_id1 = self.merge_halfedges(&edge_id1, edge_id2)?;
            }
        }

        self.fix_orientation();

        Ok(())
    }

    fn merge_halfedges(&mut self, halfedge_id1: &HalfEdgeID, halfedge_id2: &HalfEdgeID) -> Result<HalfEdgeID, Error>
    {
        let mut walker1 = self.walker_from_halfedge(halfedge_id1);
        let mut walker2 = self.walker_from_halfedge(halfedge_id2);

        let edge1_alone =  walker1.face_id().is_none() && walker1.as_twin().face_id().is_none();
        let edge1_interior =  walker1.face_id().is_some() && walker1.as_twin().face_id().is_some();
        let edge1_boundary = !edge1_alone && !edge1_interior;

        let edge2_alone =  walker2.face_id().is_none() && walker2.as_twin().face_id().is_none();
        let edge2_interior =  walker2.face_id().is_some() && walker2.as_twin().face_id().is_some();
        let edge2_boundary = !edge2_alone && !edge2_interior;

        if edge1_interior && !edge2_alone || edge2_interior && !edge1_alone {
            return Err(Error::FailedToMergeVertices { message: format!("Merging halfedges {} and {} will create a non-manifold mesh", halfedge_id1, halfedge_id2) });
        }

        let mut halfedge_to_remove1 = None;
        let mut halfedge_to_remove2 = None;
        let mut halfedge_to_survive1 = None;
        let mut halfedge_to_survive2 = None;
        let mut vertex_id1 = None;
        let mut vertex_id2 = None;

        if edge1_boundary {
            if walker1.face_id().is_none() { walker1.as_twin(); };
            halfedge_to_remove1 = walker1.twin_id();
            halfedge_to_survive1 = walker1.halfedge_id();
            vertex_id1 = walker1.vertex_id();
        }
        if edge2_boundary {
            if walker2.face_id().is_none() { walker2.as_twin(); };
            halfedge_to_remove2 = walker2.twin_id();
            halfedge_to_survive2 = walker2.halfedge_id();
            vertex_id2 = walker2.vertex_id();
        }
        if edge1_alone
        {
            if edge2_interior
            {
                halfedge_to_remove1 = walker1.twin_id();
                halfedge_to_remove2 = walker1.halfedge_id();

                halfedge_to_survive1 = walker2.halfedge_id();
                vertex_id1 = walker2.vertex_id();
                walker2.as_twin();
                halfedge_to_survive2 = walker2.halfedge_id();
                vertex_id2 = walker2.vertex_id();
            }
            else {
                if vertex_id2 == walker1.vertex_id() { walker1.as_twin(); }
                halfedge_to_remove1 = walker1.twin_id();
                halfedge_to_survive1 = walker1.halfedge_id();
                vertex_id1 = walker1.vertex_id();
            }
        }
        if edge2_alone
        {
            if edge1_interior {
                halfedge_to_remove1 = walker2.twin_id();
                halfedge_to_remove2 = walker2.halfedge_id();

                halfedge_to_survive1 = walker1.halfedge_id();
                vertex_id1 = walker1.vertex_id();
                walker1.as_twin();
                halfedge_to_survive2 = walker1.halfedge_id();
                vertex_id2 = walker1.vertex_id();
            }
            else {
                if vertex_id1 == walker2.vertex_id() { walker2.as_twin(); }
                halfedge_to_remove2 = walker2.twin_id();
                halfedge_to_survive2 = walker2.halfedge_id();
                vertex_id2 = walker2.vertex_id();
            }
        }

        self.connectivity_info.remove_halfedge(&halfedge_to_remove1.unwrap());
        self.connectivity_info.remove_halfedge(&halfedge_to_remove2.unwrap());
        self.connectivity_info.set_halfedge_twin(halfedge_to_survive1.unwrap(), halfedge_to_survive2.unwrap());
        self.connectivity_info.set_vertex_halfedge(&vertex_id1.unwrap(), halfedge_to_survive2);
        self.connectivity_info.set_vertex_halfedge(&vertex_id2.unwrap(), halfedge_to_survive1);
        Ok(halfedge_to_survive1.unwrap())
    }

    fn merge_vertices(&mut self, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Result<VertexID, Error>
    {
        for halfedge_id in self.halfedge_iter() {
            let walker = self.walker_from_halfedge(&halfedge_id);
            if walker.vertex_id().unwrap() == *vertex_id2 {
                self.connectivity_info.set_halfedge_vertex(&walker.halfedge_id().unwrap(), *vertex_id1);
            }
        }
        self.connectivity_info.remove_vertex(vertex_id2);

        Ok(vertex_id1.clone())
    }

    fn find_overlapping_vertices(&self) -> Vec<Vec<VertexID>>
    {
        let mut to_check = HashSet::new();
        self.vertex_iter().for_each(|v| { to_check.insert(v); } );

        let mut set_to_merge = Vec::new();
        while !to_check.is_empty() {
            let id1 = *to_check.iter().next().unwrap();
            to_check.remove(&id1);

            let mut to_merge = Vec::new();
            for id2 in to_check.iter()
            {
                if (self.vertex_position(&id1) - self.vertex_position(id2)).magnitude() < 0.00001
                {
                    to_merge.push(*id2);
                }
            }
            if !to_merge.is_empty()
            {
                for id in to_merge.iter()
                {
                    to_check.remove(id);
                }
                to_merge.push(id1);
                set_to_merge.push(to_merge);
            }
        }
        set_to_merge
    }

    fn find_overlapping_faces(&self, set_of_vertices_to_merge: &Vec<Vec<VertexID>>) -> Vec<Vec<FaceID>>
    {
        let vertices_to_merge = |vertex_id| {
            set_of_vertices_to_merge.iter().find(|vec| vec.contains(&vertex_id))
        };
        let mut to_check = HashSet::new();
        self.face_iter().for_each(|id| { to_check.insert(id); } );

        let mut set_to_merge = Vec::new();
        while !to_check.is_empty() {
            let id1 = *to_check.iter().next().unwrap();
            to_check.remove(&id1);

            let (v0, v1, v2) = self.face_vertices(&id1);
            if let Some(vertices_to_merge0) = vertices_to_merge(v0)
            {
                if let Some(vertices_to_merge1) = vertices_to_merge(v1)
                {
                    if let Some(vertices_to_merge2) = vertices_to_merge(v2)
                    {
                        let mut to_merge = Vec::new();
                        for id2 in to_check.iter()
                        {
                            let (v3, v4, v5) = self.face_vertices(&id2);
                            if (vertices_to_merge0.contains(&v3) || vertices_to_merge0.contains(&v4) || vertices_to_merge0.contains(&v5))
                                && (vertices_to_merge1.contains(&v3) || vertices_to_merge1.contains(&v4) || vertices_to_merge1.contains(&v5))
                                && (vertices_to_merge2.contains(&v3) || vertices_to_merge2.contains(&v4) || vertices_to_merge2.contains(&v5))
                                {
                                    to_merge.push(*id2);
                                }
                        }
                        if !to_merge.is_empty()
                            {
                                for id in to_merge.iter()
                                {
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

    fn find_overlapping_edges(&self, set_of_vertices_to_merge: &Vec<Vec<VertexID>>) -> Vec<Vec<HalfEdgeID>>
    {
        let vertices_to_merge = |vertex_id| {
            set_of_vertices_to_merge.iter().find(|vec| vec.contains(&vertex_id))
        };
        let mut to_check = HashSet::new();
        self.edge_iter().for_each(|e| { to_check.insert(e); } );

        let mut set_to_merge = Vec::new();
        while !to_check.is_empty() {
            let id1 = *to_check.iter().next().unwrap();
            to_check.remove(&id1);

            if let Some(vertices_to_merge0) = vertices_to_merge(id1.0)
            {
                if let Some(vertices_to_merge1) = vertices_to_merge(id1.1)
                {
                    let mut to_merge = Vec::new();
                    for id2 in to_check.iter()
                    {
                        if vertices_to_merge0.contains(&id2.0) && vertices_to_merge1.contains(&id2.1)
                            || vertices_to_merge1.contains(&id2.0) && vertices_to_merge0.contains(&id2.1)
                        {
                            to_merge.push(self.connecting_edge(&id2.0, &id2.1).unwrap());
                        }
                    }
                    if !to_merge.is_empty()
                    {
                        for id in to_merge.iter()
                        {
                            to_check.remove(&self.ordered_edge_vertices(id));
                        }
                        to_merge.push(self.connecting_edge(&id1.0, &id1.1).unwrap());
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
    use crate::test_utility::*;
    use crate::MeshBuilder;

    #[test]
    fn test_clone_subset()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5,  0.0, 0.0, 2.0,  1.0, 0.0, 2.5];
        let mesh = MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap();

        let mut faces = std::collections::HashSet::new();
        for face_id in mesh.face_iter() {
            faces.insert(face_id);
            break;
        }

        let sub_mesh = mesh.clone_subset(&faces);

        test_is_valid(&mesh).unwrap();
        test_is_valid(&sub_mesh).unwrap();
    }

    #[test]
    fn test_merge_overlapping_primitives()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
                                       0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5];

        let mut mesh = Mesh::new((0..9).collect(), positions);
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(12, mesh.no_halfedges());
        assert_eq!(3, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_overlapping_primitives_of_cube()
    {
        let mut mesh = create_unconnected_cube();
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(8, mesh.no_vertices());
        assert_eq!(36, mesh.no_halfedges());
        assert_eq!(12, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_overlapping_individual_faces()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0];

        let mut mesh = Mesh::new((0..9).collect(), positions);
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(10, mesh.no_halfedges());
        assert_eq!(2, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_two_overlapping_faces()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  1, 3, 2,  4, 6, 5,  6, 7, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  -1.0, 0.0, 0.0,  -0.5, 0.0, 1.0,  -1.5, 0.0, 1.0,
                                       -1.0, 0.0, 0.0,  -0.5, 0.0, 1.0,  -1.5, 0.0, 1.0,  -1.0, 0.0, 1.5];

        let mut mesh = Mesh::new(indices, positions);
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(5, mesh.no_vertices());
        assert_eq!(14, mesh.no_halfedges());
        assert_eq!(3, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_three_overlapping_faces()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  1, 3, 2,  4, 6, 5,  6, 7, 5,  8, 10, 9];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  -1.0, 0.0, 0.0,  -0.5, 0.0, 1.0,  -1.5, 0.0, 1.0,
                                       -1.0, 0.0, 0.0,  -0.5, 0.0, 1.0,  -1.5, 0.0, 1.0,  -1.0, 0.0, 1.5,
                                        -1.0, 0.0, 0.0,  -0.5, 0.0, 1.0,  -1.5, 0.0, 1.0];

        let mut mesh = Mesh::new(indices, positions);
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(5, mesh.no_vertices());
        assert_eq!(14, mesh.no_halfedges());
        assert_eq!(3, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_vertices()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0];
        let mut mesh = Mesh::new((0..6).collect(), positions);

        let mut vertex_id1 = None;
        for vertex_id in mesh.vertex_iter() {
            if *mesh.vertex_position(&vertex_id) == vec3(0.0, 0.0, 0.0)
            {
                if vertex_id1.is_none() { vertex_id1 = Some(vertex_id); }
                else {
                    mesh.merge_vertices(&vertex_id1.unwrap(), &vertex_id).unwrap();
                    break;
                }
            }
        }

        assert_eq!(5, mesh.no_vertices());
        assert_eq!(12, mesh.no_halfedges());
        assert_eq!(2, mesh.no_faces());
    }

    #[test]
    fn test_merge_halfedges()
    {
        let positions: Vec<f32> = vec![1.0, 0.0, 0.0,  0.0, 0.0, 0.0,  0.0, 0.0, -1.0,
                                       0.0, 0.0, 0.0,  1.0, 0.0, 0.0,  0.0, 0.0, 1.0];
        let mut mesh = Mesh::new((0..6).collect(), positions);

        let mut heid1 = None;
        for (v0, v1) in mesh.edge_iter() {
            if mesh.vertex_position(&v0)[2] == 0.0 && mesh.vertex_position(&v1)[2] == 0.0
            {
                let halfedge_id = mesh.connecting_edge(&v0, &v1).unwrap();
                if heid1.is_none() { heid1 = Some((halfedge_id, v0, v1)); }
                else {
                    let (halfedge_id1, v10, v11) = heid1.unwrap();
                    mesh.merge_vertices(&v0, &v11).unwrap();
                    mesh.merge_vertices(&v1, &v10).unwrap();
                    mesh.merge_halfedges(&halfedge_id1, &halfedge_id).unwrap();
                    break;
                }
            }
        }

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(10, mesh.no_halfedges());
        assert_eq!(2, mesh.no_faces());
    }

    #[test]
    fn test_face_face_merging_at_edge()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_indices(indices1).with_positions(positions1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0, -2.0, 0.0, -2.0, -2.0, 0.5, 0.0];
        let mesh2 = MeshBuilder::new().with_indices(indices2).with_positions(positions2).build().unwrap();

        mesh1.merge_with(&mesh2).unwrap();

        assert_eq!(mesh1.no_faces(), 2);
        assert_eq!(mesh1.no_vertices(), 4);

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();
    }

    #[test]
    fn test_face_face_merging_at_edge_when_orientation_is_opposite()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_indices(indices1).with_positions(positions1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0, -2.0, 0.5, 0.0, -2.0, 0.0, -2.0];
        let mesh2 = MeshBuilder::new().with_indices(indices2).with_positions(positions2).build().unwrap();

        mesh1.merge_with(&mesh2).unwrap();

        assert_eq!(mesh1.no_faces(), 2);
        assert_eq!(mesh1.no_vertices(), 4);

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();
    }
}