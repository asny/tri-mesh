//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;
use crate::operations::*;
use std::collections::{HashMap, HashSet};

/// # Split
impl Mesh {
    /// Clones a subset of this mesh defined by the is_included function.
    pub fn clone_subset(&self, is_included: &dyn Fn(&Mesh, FaceID) -> bool) -> Mesh {
        let mut clone = self.clone();
        for face_id in clone.face_iter() {
            if !is_included(&clone, face_id) {
                let edges: Vec<HalfEdgeID> = clone.face_halfedge_iter(face_id).collect();
                clone.remove_face_unsafe(face_id);
                for halfedge_id in edges {
                    let mut walker = clone.walker_from_halfedge(halfedge_id);
                    walker.as_twin();
                    if walker.face_id().is_none() {
                        clone
                            .connectivity_info
                            .remove_halfedge(walker.halfedge_id().unwrap());
                        clone.connectivity_info.remove_halfedge(halfedge_id);
                    }
                }
            }
        }

        for vertex_id in clone.vertex_iter() {
            clone.connectivity_info.set_vertex_halfedge(vertex_id, None);
        }

        for halfedge_id in clone.halfedge_iter() {
            let walker = clone.walker_from_halfedge(halfedge_id);
            clone
                .connectivity_info
                .set_vertex_halfedge(walker.vertex_id().unwrap(), walker.twin_id());
        }
        for vertex_id in clone.vertex_iter() {
            clone.remove_vertex_if_lonely(vertex_id);
        }
        clone
    }

    ///
    /// Splits the mesh into subsets bounded by the edges where the is_at_split function returns true.
    ///
    pub fn split(&self, is_at_split: &dyn Fn(&Mesh, HalfEdgeID) -> bool) -> Vec<Mesh> {
        let components =
            self.connected_components_with_limit(&|halfedge_id| is_at_split(self, halfedge_id));
        components
            .iter()
            .map(|cc| self.clone_subset(&|_, face_id| cc.contains(&face_id)))
            .collect()
    }

    ///
    /// Splits the two meshes into subsets bounded by the intersection between the two meshes.
    ///
    pub fn split_at_intersection(&mut self, other: &mut Mesh) -> (Vec<Mesh>, Vec<Mesh>) {
        let stitches = self.split_primitives_at_intersection_internal(other);
        let mut map1 = HashMap::new();
        let mut map2 = HashMap::new();
        stitches.iter().for_each(|(v0, v1)| {
            map1.insert(*v0, *v1);
            map2.insert(*v1, *v0);
        });

        let meshes1 =
            self.split(&|_, halfedge_id| is_at_intersection(self, other, halfedge_id, &map1));
        let meshes2 =
            other.split(&|_, halfedge_id| is_at_intersection(other, self, halfedge_id, &map2));
        (meshes1, meshes2)
    }

    ///
    /// Splits the primitives of the two meshes at the intersection between the two meshes.
    ///
    pub fn split_primitives_at_intersection(&mut self, other: &mut Mesh) {
        self.split_primitives_at_intersection_internal(other);
    }

    fn split_primitives_at_intersection_internal(
        &mut self,
        other: &mut Mesh,
    ) -> Vec<(VertexID, VertexID)> {
        let mut intersections = find_intersections(self, other);
        let mut stitches = Vec::new();
        while let Some((ref new_edges1, ref new_edges2)) =
            split_at_intersections(self, other, &intersections, &mut stitches)
        {
            intersections =
                find_intersections_between_edge_face(self, new_edges1, other, new_edges2);
        }
        stitches
    }
}

fn is_at_intersection(
    mesh1: &Mesh,
    mesh2: &Mesh,
    halfedge_id: HalfEdgeID,
    stitches: &HashMap<VertexID, VertexID>,
) -> bool {
    let (va, vb) = mesh1.ordered_edge_vertices(halfedge_id);
    if let (Some(vc), Some(vd)) = (stitches.get(&va), stitches.get(&vb)) {
        if let Some(halfedge_id2) = mesh2.connecting_edge(*vc, *vd) {
            if mesh1.is_edge_on_boundary(halfedge_id) || mesh2.is_edge_on_boundary(halfedge_id2) {
                return true;
            }
            let mut walker1 = mesh1.walker_from_halfedge(halfedge_id);
            let mut walker2 = mesh2.walker_from_halfedge(halfedge_id2);
            let face_id10 = walker1.face_id().unwrap();
            let face_id11 = walker1.as_twin().face_id().unwrap();
            let face_id20 = walker2.face_id().unwrap();
            let face_id21 = walker2.as_twin().face_id().unwrap();
            if (!face_and_face_overlaps(&mesh1, face_id10, mesh2, face_id20)
                && !face_and_face_overlaps(&mesh1, face_id10, mesh2, face_id21))
                || (!face_and_face_overlaps(&mesh1, face_id11, mesh2, face_id20)
                    && !face_and_face_overlaps(&mesh1, face_id11, mesh2, face_id21))
            {
                return true;
            }
        }
    }
    false
}

fn face_and_face_overlaps(mesh1: &Mesh, face_id1: FaceID, mesh2: &Mesh, face_id2: FaceID) -> bool {
    let (v0, v1, v2) = mesh1.face_vertices(face_id1);
    let (p0, p1, p2) = mesh2.face_positions(face_id2);

    (mesh1.vertex_point_intersection(v0, &p0).is_some()
        || mesh1.vertex_point_intersection(v1, &p0).is_some()
        || mesh1.vertex_point_intersection(v2, &p0).is_some())
        && (mesh1.vertex_point_intersection(v0, &p1).is_some()
            || mesh1.vertex_point_intersection(v1, &p1).is_some()
            || mesh1.vertex_point_intersection(v2, &p1).is_some())
        && (mesh1.vertex_point_intersection(v0, &p2).is_some()
            || mesh1.vertex_point_intersection(v1, &p2).is_some()
            || mesh1.vertex_point_intersection(v2, &p2).is_some())
}

fn split_at_intersections(
    mesh1: &mut Mesh,
    mesh2: &mut Mesh,
    intersections: &HashMap<(Primitive, Primitive), Vec3>,
    stitches: &mut Vec<(VertexID, VertexID)>,
) -> Option<(Vec<HalfEdgeID>, Vec<HalfEdgeID>)> {
    let mut new_edges1 = Vec::new();
    let mut new_edges2 = Vec::new();

    // Split faces
    let mut new_intersections: HashMap<(Primitive, Primitive), Vec3> = HashMap::new();
    let mut face_splits1 = HashMap::new();
    let mut face_splits2 = HashMap::new();
    for ((id1, id2), point) in intersections.iter() {
        if let Primitive::Face(face_id) = id1 {
            match find_face_primitive_to_split(&face_splits1, mesh1, *face_id, point) {
                Primitive::Vertex(vertex_id) => {
                    new_intersections.insert((Primitive::Vertex(vertex_id), *id2), *point);
                }
                Primitive::Edge(edge) => {
                    new_intersections.insert((Primitive::Edge(edge), *id2), *point);
                }
                Primitive::Face(split_face_id) => {
                    let vertex_id = mesh1.split_face(split_face_id, point.clone());
                    insert_faces(&mut face_splits1, mesh1, *face_id, vertex_id);
                    for halfedge_id in mesh1.vertex_halfedge_iter(vertex_id) {
                        new_edges1.push(halfedge_id);
                    }
                    new_intersections.insert((Primitive::Vertex(vertex_id), *id2), *point);
                }
            }
        } else if let Primitive::Face(face_id) = id2 {
            match find_face_primitive_to_split(&face_splits2, mesh2, *face_id, point) {
                Primitive::Vertex(vertex_id) => {
                    new_intersections.insert((*id1, Primitive::Vertex(vertex_id)), *point);
                }
                Primitive::Edge(edge) => {
                    new_intersections.insert((*id1, Primitive::Edge(edge)), *point);
                }
                Primitive::Face(split_face_id) => {
                    let vertex_id = mesh2.split_face(split_face_id, point.clone());
                    insert_faces(&mut face_splits2, mesh2, *face_id, vertex_id);
                    for halfedge_id in mesh2.vertex_halfedge_iter(vertex_id) {
                        new_edges2.push(halfedge_id);
                    }
                    new_intersections.insert((*id1, Primitive::Vertex(vertex_id)), *point);
                }
            }
        } else {
            new_intersections.insert((*id1, *id2), *point);
        }
    }

    // Split edges
    let mut edge_splits1 = HashMap::new();
    let mut edge_splits2 = HashMap::new();
    for ((id1, id2), point) in new_intersections.drain() {
        let v0 = match id1 {
            Primitive::Vertex(vertex_id) => vertex_id,
            Primitive::Edge(edge) => {
                match find_edge_primitive_to_split(&edge_splits1, mesh1, edge, &point) {
                    Primitive::Vertex(vertex_id) => vertex_id,
                    Primitive::Edge(split_edge) => {
                        let (v0, v1) = mesh1.edge_vertices(split_edge);
                        let vertex_id = mesh1.split_edge(split_edge, point);

                        if !edge_splits1.contains_key(&edge) {
                            edge_splits1.insert(edge, HashSet::new());
                        }
                        let list = edge_splits1.get_mut(&edge).unwrap();

                        list.remove(&split_edge);
                        for halfedge_id in mesh1.vertex_halfedge_iter(vertex_id) {
                            let vid = mesh1.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                            if vid != v0 && vid != v1 {
                                new_edges1.push(halfedge_id);
                            } else {
                                list.insert(halfedge_id);
                            }
                        }
                        vertex_id
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
            _ => {
                unreachable!()
            }
        };
        let v1 = match id2 {
            Primitive::Vertex(vertex_id) => vertex_id,
            Primitive::Edge(edge) => {
                match find_edge_primitive_to_split(&edge_splits2, mesh2, edge, &point) {
                    Primitive::Vertex(vertex_id) => vertex_id,
                    Primitive::Edge(split_edge) => {
                        let (v0, v1) = mesh2.edge_vertices(split_edge);
                        let vertex_id = mesh2.split_edge(split_edge, point);

                        if !edge_splits2.contains_key(&edge) {
                            edge_splits2.insert(edge, HashSet::new());
                        }
                        let list = edge_splits2.get_mut(&edge).unwrap();

                        list.remove(&split_edge);
                        for halfedge_id in mesh2.vertex_halfedge_iter(vertex_id) {
                            let vid = mesh2.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                            if vid != v0 && vid != v1 {
                                new_edges2.push(halfedge_id);
                            } else {
                                list.insert(halfedge_id);
                            }
                        }
                        vertex_id
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
            _ => {
                unreachable!()
            }
        };
        stitches.push((v0, v1));
    }
    if new_edges1.len() > 0 && new_edges2.len() > 0 {
        Some((new_edges1, new_edges2))
    } else {
        None
    }
}

fn find_face_primitive_to_split(
    face_splits: &HashMap<FaceID, HashSet<FaceID>>,
    mesh: &Mesh,
    face_id: FaceID,
    point: &Vec3,
) -> Primitive {
    if let Some(new_faces) = face_splits.get(&face_id) {
        for new_face_id in new_faces {
            if let Some(Intersection::Point { primitive, .. }) =
                mesh.face_point_intersection(*new_face_id, point)
            {
                return primitive;
            }
        }
        unreachable!()
    }
    Primitive::Face(face_id)
}

fn find_edge_primitive_to_split(
    edge_splits: &HashMap<HalfEdgeID, HashSet<HalfEdgeID>>,
    mesh: &Mesh,
    edge: HalfEdgeID,
    point: &Vec3,
) -> Primitive {
    if let Some(new_edges) = edge_splits.get(&edge) {
        for new_edge in new_edges {
            if let Some(Intersection::Point { primitive, .. }) =
                mesh.edge_point_intersection(*new_edge, point)
            {
                return primitive;
            }
        }
        unreachable!()
    }
    Primitive::Edge(edge)
}

fn insert_faces(
    face_list: &mut HashMap<FaceID, HashSet<FaceID>>,
    mesh: &Mesh,
    face_id: FaceID,
    vertex_id: VertexID,
) {
    if !face_list.contains_key(&face_id) {
        face_list.insert(face_id, HashSet::new());
    }
    let list = face_list.get_mut(&face_id).unwrap();

    let mut iter = mesh.vertex_halfedge_iter(vertex_id);
    list.insert(
        mesh.walker_from_halfedge(iter.next().unwrap())
            .face_id()
            .unwrap(),
    );
    list.insert(
        mesh.walker_from_halfedge(iter.next().unwrap())
            .face_id()
            .unwrap(),
    );
    list.insert(
        mesh.walker_from_halfedge(iter.next().unwrap())
            .face_id()
            .unwrap(),
    );
}

fn find_intersections(mesh1: &Mesh, mesh2: &Mesh) -> HashMap<(Primitive, Primitive), Vec3> {
    let edges1: Vec<HalfEdgeID> = mesh1.edge_iter().collect();
    let edges2: Vec<HalfEdgeID> = mesh2.edge_iter().collect();
    find_intersections_between_edge_face(mesh1, &edges1, mesh2, &edges2)
}

fn find_intersections_between_edge_face(
    mesh1: &Mesh,
    edges1: &Vec<HalfEdgeID>,
    mesh2: &Mesh,
    edges2: &Vec<HalfEdgeID>,
) -> HashMap<(Primitive, Primitive), Vec3> {
    let mut intersections: HashMap<(Primitive, Primitive), Vec3> = HashMap::new();
    for edge1 in edges1 {
        for face_id2 in mesh2.face_iter() {
            let (p0, p1) = mesh1.edge_positions(*edge1);
            if let Some(intersection) = mesh2.face_line_piece_intersection(face_id2, &p0, &p1) {
                match intersection {
                    Intersection::Point {
                        primitive: primitive2,
                        point,
                    } => {
                        if let Some(Intersection::Point {
                            primitive: primitive1,
                            ..
                        }) = mesh1.edge_point_intersection(*edge1, &point)
                        {
                            intersections.insert((primitive1, primitive2), point);
                        } else {
                            unreachable!()
                        }
                    }
                    Intersection::LinePiece {
                        primitive0: primitive20,
                        primitive1: primitive21,
                        point0,
                        point1,
                    } => {
                        if let Some(Intersection::Point {
                            primitive: primitive1,
                            ..
                        }) = mesh1.edge_point_intersection(*edge1, &point0)
                        {
                            intersections.insert((primitive1, primitive20), point0);
                        } else {
                            unreachable!()
                        }

                        if let Some(Intersection::Point {
                            primitive: primitive1,
                            ..
                        }) = mesh1.edge_point_intersection(*edge1, &point1)
                        {
                            intersections.insert((primitive1, primitive21), point1);
                        } else {
                            unreachable!()
                        }
                    }
                }
            }
        }
    }
    for edge2 in edges2 {
        for face_id1 in mesh1.face_iter() {
            let (p0, p1) = mesh2.edge_positions(*edge2);
            if let Some(intersection) = mesh1.face_line_piece_intersection(face_id1, &p0, &p1) {
                match intersection {
                    Intersection::Point {
                        primitive: primitive1,
                        point,
                    } => {
                        if let Some(Intersection::Point {
                            primitive: primitive2,
                            ..
                        }) = mesh2.edge_point_intersection(*edge2, &point)
                        {
                            intersections.insert((primitive1, primitive2), point);
                        } else {
                            unreachable!()
                        }
                    }
                    Intersection::LinePiece {
                        primitive0: primitive10,
                        primitive1: primitive11,
                        point0,
                        point1,
                    } => {
                        if let Some(Intersection::Point {
                            primitive: primitive2,
                            ..
                        }) = mesh2.edge_point_intersection(*edge2, &point0)
                        {
                            intersections.insert((primitive10, primitive2), point0);
                        } else {
                            unreachable!()
                        }

                        if let Some(Intersection::Point {
                            primitive: primitive2,
                            ..
                        }) = mesh2.edge_point_intersection(*edge2, &point1)
                        {
                            intersections.insert((primitive11, primitive2), point1);
                        } else {
                            unreachable!()
                        }
                    }
                }
            }
        }
    }
    intersections
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MeshBuilder;

    #[test]
    fn test_clone_subset() {
        let indices: Vec<u32> = vec![0, 1, 2, 2, 1, 3, 3, 1, 4, 3, 4, 5];
        let positions: Vec<f64> = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.5, 1.0, 0.0, 1.5, 0.0, 0.0, 2.0, 1.0, 0.0,
            2.5,
        ];
        let mesh = MeshBuilder::new()
            .with_indices(indices)
            .with_positions(positions)
            .build()
            .unwrap();

        let mut faces = std::collections::HashSet::new();
        for face_id in mesh.face_iter() {
            faces.insert(face_id);
            break;
        }

        let sub_mesh = mesh.clone_subset(&|_, face_id| faces.contains(&face_id));

        assert_eq!(sub_mesh.no_faces(), 1);
        assert_eq!(sub_mesh.no_halfedges(), 6);
        assert_eq!(sub_mesh.no_vertices(), 3);
        mesh.is_valid().unwrap();
        sub_mesh.is_valid().unwrap();
    }

    #[test]
    fn test_split() {
        let indices: Vec<u32> = vec![0, 1, 2, 2, 1, 3, 3, 1, 4, 3, 4, 5];
        let positions: Vec<f64> = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.5, 1.0, 0.0, 1.5, 0.0, 0.0, 2.0, 1.0, 0.0,
            2.5,
        ];
        let mesh = MeshBuilder::new()
            .with_indices(indices)
            .with_positions(positions)
            .build()
            .unwrap();

        let meshes = mesh.split(&|mesh, he_id| {
            let (p0, p1) = mesh.edge_positions(he_id);
            p0.z > 0.75 && p0.z < 1.75 && p1.z > 0.75 && p1.z < 1.75
        });

        assert_eq!(meshes.len(), 2);
        let m1 = &meshes[0];
        let m2 = &meshes[1];

        mesh.is_valid().unwrap();
        m1.is_valid().unwrap();
        m2.is_valid().unwrap();

        assert_eq!(m1.no_faces(), 2);
        assert_eq!(m2.no_faces(), 2);
    }

    #[test]
    fn test_face_face_stitching_at_edge() {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f64> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new()
            .with_positions(positions1)
            .with_indices(indices1)
            .build()
            .unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f64> = vec![-2.0, 0.0, 2.0, -2.0, 0.0, -2.0, -2.0, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new()
            .with_positions(positions2)
            .with_indices(indices2)
            .build()
            .unwrap();

        let (meshes1, meshes2) = mesh1.split_at_intersection(&mut mesh2);
        assert_eq!(meshes1.len(), 1);
        assert_eq!(meshes2.len(), 1);

        let mut m1 = meshes1[0].clone();
        let m2 = meshes2[0].clone();
        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        assert_eq!(m1.no_faces(), 2);
        assert_eq!(m1.no_vertices(), 4);

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_face_face_stitching_at_mid_edge() {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f64> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new()
            .with_positions(positions1)
            .with_indices(indices1)
            .build()
            .unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f64> = vec![-2.0, 0.0, 1.0, -2.0, 0.0, -1.0, -2.0, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new()
            .with_positions(positions2)
            .with_indices(indices2)
            .build()
            .unwrap();

        let (meshes1, meshes2) = mesh1.split_at_intersection(&mut mesh2);
        assert_eq!(meshes1.len(), 1);
        assert_eq!(meshes2.len(), 1);

        let mut m1 = meshes1[0].clone();
        let m2 = meshes2[0].clone();
        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        assert_eq!(m1.no_faces(), 4);
        assert_eq!(m1.no_vertices(), 6);

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_box_box_stitching() {
        let mut mesh1 = MeshBuilder::new().cube().build().unwrap();
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        mesh2.translate(vec3(0.5, 0.5, 0.5));

        let (meshes1, meshes2) = mesh1.split_at_intersection(&mut mesh2);
        assert_eq!(meshes1.len(), 2);
        assert_eq!(meshes2.len(), 2);

        let mut m1 = if meshes1[0].no_faces() > meshes1[1].no_faces() {
            meshes1[0].clone()
        } else {
            meshes1[1].clone()
        };
        let m2 = if meshes2[0].no_faces() > meshes2[1].no_faces() {
            meshes2[0].clone()
        } else {
            meshes2[1].clone()
        };

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();

        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_sphere_box_stitching() {
        let mut mesh1 = MeshBuilder::new().icosahedron().build().unwrap();
        for _ in 0..1 {
            for face_id in mesh1.face_iter() {
                let p = mesh1.face_center(face_id).normalize();
                mesh1.split_face(face_id, p);
            }
            mesh1.smooth_vertices(1.0);
            for vertex_id in mesh1.vertex_iter() {
                let p = mesh1.vertex_position(vertex_id).normalize();
                mesh1.move_vertex_to(vertex_id, p)
            }
            mesh1.flip_edges(0.5);
        }
        mesh1.translate(vec3(0.0, 1.5, 0.0));
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        mesh2.translate(vec3(0.5, 2.0, 0.5));

        let (meshes1, meshes2) = mesh1.split_at_intersection(&mut mesh2);
        assert_eq!(meshes1.len(), 2);
        assert_eq!(meshes2.len(), 2);

        let mut m1 = if meshes1[0].no_faces() > meshes1[1].no_faces() {
            meshes1[0].clone()
        } else {
            meshes1[1].clone()
        };
        let m2 = if meshes2[0].no_faces() > meshes2[1].no_faces() {
            meshes2[0].clone()
        } else {
            meshes2[1].clone()
        };

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();

        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_is_at_intersection_cube_cube() {
        let mesh1 = MeshBuilder::new().cube().build().unwrap();
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        mesh2.translate(vec3(0.0, 2.0, 0.0));

        let mut map = HashMap::new();
        for vertex_id1 in mesh1.vertex_iter() {
            for vertex_id2 in mesh2.vertex_iter() {
                if (mesh1.vertex_position(vertex_id1) - mesh2.vertex_position(vertex_id2))
                    .magnitude()
                    < 0.0001
                {
                    map.insert(vertex_id1, vertex_id2);
                }
            }
        }
        let result = mesh1.connected_components_with_limit(&|halfedge_id| {
            is_at_intersection(&mesh1, &mesh2, halfedge_id, &map)
        });

        assert_eq!(result.len(), 2);
        assert!(result.iter().find(|cc| cc.len() == 2).is_some());
        assert!(result.iter().find(|cc| cc.len() == 10).is_some());
    }

    #[test]
    fn test_is_at_intersection() {
        let mesh1 = MeshBuilder::new().cube().build().unwrap();

        let positions = vec![
            -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 0.0, 2.0, 0.0,
        ];
        let indices = vec![0, 1, 2, 0, 2, 3, 0, 3, 4];
        let mesh2 = MeshBuilder::new()
            .with_positions(positions)
            .with_indices(indices)
            .build()
            .unwrap();

        let mut map = HashMap::new();
        for vertex_id1 in mesh1.vertex_iter() {
            for vertex_id2 in mesh2.vertex_iter() {
                if (mesh1.vertex_position(vertex_id1) - mesh2.vertex_position(vertex_id2))
                    .magnitude()
                    < 0.0001
                {
                    map.insert(vertex_id2, vertex_id1);
                }
            }
        }
        let result = mesh2.connected_components_with_limit(&|halfedge_id| {
            is_at_intersection(&mesh2, &mesh1, halfedge_id, &map)
        });

        assert_eq!(result.len(), 2);
        assert!(result.iter().find(|cc| cc.len() == 1).is_some());
        assert!(result.iter().find(|cc| cc.len() == 2).is_some());
    }

    #[test]
    fn test_finding_edge_edge_intersections() {
        let mesh1 = create_simple_mesh_x_z();
        let mesh2 = create_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 5);

        assert!(intersections
            .iter()
            .any(|pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 0.25));
        assert!(intersections
            .iter()
            .any(|pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 0.75));
        assert!(intersections
            .iter()
            .any(|pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 1.25));
        assert!(intersections
            .iter()
            .any(|pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 1.75));
        assert!(intersections
            .iter()
            .any(|pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 2.25));
    }

    #[test]
    fn test_finding_face_edge_intersections() {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f64> = vec![0.5, -0.5, 0.0, 0.5, 0.5, 0.75, 0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new()
            .with_positions(positions)
            .with_indices(indices)
            .build()
            .unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 2);
    }

    #[test]
    fn test_finding_face_vertex_intersections() {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f64> = vec![0.5, 0.0, 0.5, 0.5, 0.5, 0.75, 0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new()
            .with_positions(positions)
            .with_indices(indices)
            .build()
            .unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_finding_edge_vertex_intersections() {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f64> = vec![0.5, 0.0, 0.25, 0.5, 0.5, 0.75, 0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new()
            .with_positions(positions)
            .with_indices(indices)
            .build()
            .unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_finding_vertex_vertex_intersections() {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f64> = vec![1.0, 0.0, 0.5, 0.5, 0.5, 0.75, 0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new()
            .with_positions(positions)
            .with_indices(indices)
            .build()
            .unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_split_edges() {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);
        let mut stitches = Vec::new();
        let (new_edges1, new_edges2) =
            split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap();

        assert_eq!(mesh1.no_vertices(), 11);
        assert_eq!(mesh1.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh1.no_faces(), 12);

        assert_eq!(mesh2.no_vertices(), 11);
        assert_eq!(mesh2.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh2.no_faces(), 12);

        assert_eq!(new_edges1.len(), 8);
        assert_eq!(new_edges2.len(), 8);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_split_faces() {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_shifted_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 8);

        let mut stitches = Vec::new();
        let (new_edges1, new_edges2) =
            split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap();

        assert_eq!(mesh1.no_vertices(), 14);
        assert_eq!(mesh1.no_faces(), 19);
        assert_eq!(mesh1.no_halfedges(), 19 * 3 + 7);

        assert_eq!(mesh2.no_vertices(), 14);
        assert_eq!(mesh2.no_faces(), 19);
        assert_eq!(mesh2.no_halfedges(), 19 * 3 + 7);

        assert_eq!(new_edges1.len(), 19);
        assert_eq!(new_edges2.len(), 19);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_split_face_two_times() {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f64> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new()
            .with_positions(positions1)
            .with_indices(indices1)
            .build()
            .unwrap();
        let area1 = mesh1.face_area(mesh1.face_iter().next().unwrap());

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f64> = vec![0.2, -0.2, 0.5, 0.5, 0.5, 0.75, 0.5, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new()
            .with_positions(positions2)
            .with_indices(indices2)
            .build()
            .unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 2);

        let mut stitches = Vec::new();
        let (new_edges1, new_edges2) =
            split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap();

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh1.no_faces(), 5);
        assert_eq!(mesh1.no_halfedges(), 5 * 3 + 3);

        let mut area_test1 = 0.0;
        for face_id in mesh1.face_iter() {
            area_test1 = area_test1 + mesh1.face_area(face_id);
        }
        assert!((area1 - area_test1).abs() < 0.001);

        assert_eq!(mesh2.no_vertices(), 5);
        assert_eq!(mesh2.no_faces(), 3);
        assert_eq!(mesh2.no_halfedges(), 3 * 3 + 5);

        assert_eq!(new_edges1.len(), 6);
        assert_eq!(new_edges2.len(), 2);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_split_edge_two_times() {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new()
            .with_positions(positions1)
            .with_indices(indices1)
            .build()
            .unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f64> = vec![0.0, -0.2, 0.5, 0.0, -0.2, 1.5, 0.0, 1.5, 0.0];
        let mut mesh2 = MeshBuilder::new()
            .with_positions(positions2)
            .with_indices(indices2)
            .build()
            .unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 2);

        let mut stitches = Vec::new();
        let (new_edges1, new_edges2) =
            split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap();

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh1.no_faces(), 3);
        assert_eq!(mesh1.no_halfedges(), 3 * 3 + 5);

        assert_eq!(mesh2.no_vertices(), 5);
        assert_eq!(mesh2.no_faces(), 3);
        assert_eq!(mesh2.no_halfedges(), 3 * 3 + 5);

        assert_eq!(new_edges1.len(), 2);
        assert_eq!(new_edges2.len(), 2);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_face_face_splitting() {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f64> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new()
            .with_positions(positions1)
            .with_indices(indices1)
            .build()
            .unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f64> = vec![0.2, -0.2, 0.5, 0.5, 0.5, 0.75, 0.5, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new()
            .with_positions(positions2)
            .with_indices(indices2)
            .build()
            .unwrap();

        mesh1.split_primitives_at_intersection(&mut mesh2);

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh2.no_vertices(), 5);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_simple_simple_splitting() {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_shifted_simple_mesh_y_z();

        mesh1.split_primitives_at_intersection(&mut mesh2);

        assert_eq!(mesh1.no_vertices(), 14);
        assert_eq!(mesh2.no_vertices(), 14);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_box_box_splitting() {
        let mut mesh1 = MeshBuilder::new().cube().build().unwrap();
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        mesh2.translate(vec3(0.5, 0.5, 0.5));

        mesh1.split_primitives_at_intersection(&mut mesh2);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    fn create_simple_mesh_x_z() -> Mesh {
        let indices: Vec<u32> = vec![0, 1, 2, 2, 1, 3, 3, 1, 4, 3, 4, 5];
        let positions: Vec<f64> = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.5, 1.0, 0.0, 1.5, 0.0, 0.0, 2.0, 1.0, 0.0,
            2.5,
        ];
        MeshBuilder::new()
            .with_positions(positions)
            .with_indices(indices)
            .build()
            .unwrap()
    }

    fn create_simple_mesh_y_z() -> Mesh {
        let indices: Vec<u32> = vec![0, 1, 2, 2, 1, 3, 3, 1, 4, 3, 4, 5];
        let positions: Vec<f64> = vec![
            0.5, -0.5, 0.0, 0.5, -0.5, 1.0, 0.5, 0.5, 0.5, 0.5, 0.5, 1.5, 0.5, -0.5, 2.0, 0.5, 0.5,
            2.5,
        ];
        MeshBuilder::new()
            .with_positions(positions)
            .with_indices(indices)
            .build()
            .unwrap()
    }

    fn create_shifted_simple_mesh_y_z() -> Mesh {
        let indices: Vec<u32> = vec![0, 1, 2, 2, 1, 3, 3, 1, 4, 3, 4, 5];
        let positions: Vec<f64> = vec![
            0.5, -0.5, -0.2, 0.5, -0.5, 0.8, 0.5, 0.5, 0.3, 0.5, 0.5, 1.3, 0.5, -0.5, 1.8, 0.5,
            0.5, 2.3,
        ];
        MeshBuilder::new()
            .with_positions(positions)
            .with_indices(indices)
            .build()
            .unwrap()
    }
}
