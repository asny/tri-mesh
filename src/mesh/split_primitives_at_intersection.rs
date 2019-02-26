
use std::collections::{HashMap, HashSet};
use crate::prelude::*;

impl Mesh
{
    pub fn split_primitives_at_intersection(&mut self, other: &mut Mesh)
    {
        let mut intersections = find_intersections(self, other);
        while let Some((ref new_edges1, ref new_edges2)) = split_at_intersections(self, other, &intersections)
        {
            intersections = find_intersections_between_edge_face(self, new_edges1, other, new_edges2);
        }
    }
}

fn split_at_intersections(mesh1: &mut Mesh, mesh2: &mut Mesh, intersections: &HashMap<(Primitive, Primitive), Vec3>) -> Option<(Vec<HalfEdgeID>, Vec<HalfEdgeID>)>
{
    let mut new_edges1 = Vec::new();
    let mut new_edges2 = Vec::new();

    // Split faces
    let mut new_intersections: HashMap<(Primitive, Primitive), Vec3> = HashMap::new();
    let mut face_splits1 = HashMap::new();
    let mut face_splits2= HashMap::new();
    for ((id1, id2), point) in intersections.iter()
    {
        if let Primitive::Face(face_id) = id1
        {
            match find_face_primitive_to_split(&face_splits1, mesh1, *face_id, point) {
                Primitive::Vertex(vertex_id) => { new_intersections.insert((Primitive::Vertex(vertex_id), *id2), *point); },
                Primitive::Edge(edge) => { new_intersections.insert((Primitive::Edge(edge), *id2), *point); },
                Primitive::Face(split_face_id) => {
                    let vertex_id = mesh1.split_face(split_face_id, point.clone());
                    insert_faces(&mut face_splits1, mesh1, *face_id, vertex_id);
                    for halfedge_id in mesh1.vertex_halfedge_iter(vertex_id) {
                        new_edges1.push(halfedge_id);
                    }
                    new_intersections.insert((Primitive::Vertex(vertex_id), *id2), *point);
                }
            }
        }
        else if let Primitive::Face(face_id) = id2
        {
            match find_face_primitive_to_split(&face_splits2, mesh2, *face_id, point) {
                Primitive::Vertex(vertex_id) => { new_intersections.insert((*id1, Primitive::Vertex(vertex_id)), *point); },
                Primitive::Edge(edge) => { new_intersections.insert((*id1, Primitive::Edge(edge)), *point); },
                Primitive::Face(split_face_id) => {
                    let vertex_id = mesh2.split_face(split_face_id, point.clone());
                    insert_faces(&mut face_splits2, mesh2, *face_id, vertex_id);
                    for halfedge_id in mesh2.vertex_halfedge_iter(vertex_id) {
                        new_edges2.push(halfedge_id);
                    }
                    new_intersections.insert((*id1, Primitive::Vertex(vertex_id)), *point);
                }
            }
        }
        else {
            new_intersections.insert((*id1, *id2), *point);
        }
    }

    // Split edges
    let mut edge_splits1 = HashMap::new();
    let mut edge_splits2 = HashMap::new();
    for ((id1, id2), point) in new_intersections.drain()
    {
        match id1 {
            Primitive::Vertex(_) => {},
            Primitive::Edge(edge) => {
                match find_edge_primitive_to_split(&edge_splits1, mesh1, edge, &point) {
                    Primitive::Vertex(_) => {},
                    Primitive::Edge(split_edge) => {
                        let (v0, v1) = mesh1.edge_vertices(split_edge);
                        let vertex_id = mesh1.split_edge(split_edge, point);

                        if !edge_splits1.contains_key(&edge) { edge_splits1.insert(edge, HashSet::new()); }
                        let list = edge_splits1.get_mut(&edge).unwrap();

                        list.remove(&split_edge);
                        for halfedge_id in mesh1.vertex_halfedge_iter(vertex_id) {
                            let vid = mesh1.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                            if vid != v0 && vid != v1
                            {
                                new_edges1.push(halfedge_id);
                            }
                            else {
                                list.insert(halfedge_id);
                            }
                        }
                    },
                    _ => {unreachable!()}
                }
            },
            _ => {unreachable!()}
        };
        match id2 {
            Primitive::Vertex(_) => {},
            Primitive::Edge(edge) => {
                match find_edge_primitive_to_split(&edge_splits2, mesh2, edge, &point) {
                    Primitive::Vertex(_) => {},
                    Primitive::Edge(split_edge) => {
                        let (v0, v1) = mesh2.edge_vertices(split_edge);
                        let vertex_id = mesh2.split_edge(split_edge, point);

                        if !edge_splits2.contains_key(&edge) { edge_splits2.insert(edge, HashSet::new()); }
                        let list = edge_splits2.get_mut(&edge).unwrap();

                        list.remove(&split_edge);
                        for halfedge_id in mesh2.vertex_halfedge_iter(vertex_id) {
                            let vid = mesh2.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                            if vid != v0 && vid != v1
                            {
                                new_edges2.push(halfedge_id);
                            }
                            else {
                                list.insert(halfedge_id);
                            }
                        }
                    },
                    _ => {unreachable!()}
                }
            },
            _ => {unreachable!()}
        };
    }
    if new_edges1.len() > 0 && new_edges2.len() > 0 { Some((new_edges1, new_edges2)) }
    else {None}
}

fn find_face_primitive_to_split(face_splits: &HashMap<FaceID, HashSet<FaceID>>, mesh: &Mesh, face_id: FaceID, point: &Vec3) -> Primitive
{
    if let Some(new_faces) = face_splits.get(&face_id)
    {
        for new_face_id in new_faces
        {
            if let Some(Intersection::Point {primitive, ..}) = mesh.face_point_intersection(*new_face_id, point) { return primitive; }
        }
        unreachable!()
    }
    Primitive::Face(face_id)
}

fn find_edge_primitive_to_split(edge_splits: &HashMap<HalfEdgeID, HashSet<HalfEdgeID>>, mesh: &Mesh, edge: HalfEdgeID, point: &Vec3) -> Primitive
{
    if let Some(new_edges) = edge_splits.get(&edge)
    {
        for new_edge in new_edges
        {
            if let Some(Intersection::Point {primitive, ..}) = mesh.edge_point_intersection(*new_edge, point) { return primitive; }
        }
        unreachable!()
    }
    Primitive::Edge(edge)
}

fn insert_faces(face_list: &mut HashMap<FaceID, HashSet<FaceID>>, mesh: &Mesh, face_id: FaceID, vertex_id: VertexID)
{
    if !face_list.contains_key(&face_id) { face_list.insert(face_id, HashSet::new()); }
    let list = face_list.get_mut(&face_id).unwrap();

    let mut iter = mesh.vertex_halfedge_iter(vertex_id);
    list.insert(mesh.walker_from_halfedge(iter.next().unwrap()).face_id().unwrap());
    list.insert(mesh.walker_from_halfedge(iter.next().unwrap()).face_id().unwrap());
    list.insert(mesh.walker_from_halfedge(iter.next().unwrap()).face_id().unwrap());
}

fn find_intersections(mesh1: &Mesh, mesh2: &Mesh) -> HashMap<(Primitive, Primitive), Vec3>
{
    let edges1: Vec<HalfEdgeID> = mesh1.edge_iter().collect();
    let edges2: Vec<HalfEdgeID> = mesh2.edge_iter().collect();
    find_intersections_between_edge_face(mesh1, &edges1, mesh2, &edges2)
}

fn find_intersections_between_edge_face(mesh1: &Mesh, edges1: &Vec<HalfEdgeID>, mesh2: &Mesh, edges2: &Vec<HalfEdgeID>) -> HashMap<(Primitive, Primitive), Vec3>
{
    let mut intersections: HashMap<(Primitive, Primitive), Vec3> = HashMap::new();
    for edge1 in edges1
    {
        for face_id2 in mesh2.face_iter()
        {
            let (p0, p1) = mesh1.edge_positions(*edge1);
            if let Some(intersection) = mesh2.face_line_piece_intersection(face_id2, p0, p1)
            {
                match intersection {
                    Intersection::Point {primitive: primitive2, point} => {
                        if let Some(Intersection::Point {primitive: primitive1, ..}) = mesh1.edge_point_intersection(*edge1, &point)
                        {
                            intersections.insert((primitive1, primitive2), point);
                        }
                        else { unreachable!() }
                    },
                    Intersection::LinePiece {primitive0: primitive20, primitive1: primitive21, point0, point1} => {
                        if let Some(Intersection::Point {primitive: primitive1, ..}) = mesh1.edge_point_intersection(*edge1, &point0)
                        {
                            intersections.insert((primitive1, primitive20), point0);
                        }
                        else { unreachable!() }

                        if let Some(Intersection::Point {primitive: primitive1, ..}) = mesh1.edge_point_intersection(*edge1, &point1)
                        {
                            intersections.insert((primitive1, primitive21), point1);
                        }
                        else { unreachable!() }
                    }
                }
            }
        }
    }
    for edge2 in edges2
    {
        for face_id1 in mesh1.face_iter()
        {
            let (p0, p1) = mesh2.edge_positions(*edge2);
            if let Some(intersection) = mesh1.face_line_piece_intersection(face_id1, p0, p1)
            {
                match intersection {
                    Intersection::Point {primitive: primitive1, point} => {
                        if let Some(Intersection::Point {primitive: primitive2, ..}) = mesh2.edge_point_intersection(*edge2, &point)
                        {
                            intersections.insert((primitive1, primitive2), point);
                        }
                        else { unreachable!() }
                    },
                    Intersection::LinePiece {primitive0: primitive10, primitive1: primitive11, point0, point1} => {
                        if let Some(Intersection::Point {primitive: primitive2, ..}) = mesh2.edge_point_intersection(*edge2, &point0)
                        {
                            intersections.insert((primitive10, primitive2), point0);
                        }
                        else { unreachable!() }

                        if let Some(Intersection::Point {primitive: primitive2, ..}) = mesh2.edge_point_intersection(*edge2, &point1)
                        {
                            intersections.insert((primitive11, primitive2), point1);
                        }
                        else { unreachable!() }
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

    #[test]
    fn test_finding_edge_edge_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let mesh2 = create_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 5);

        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 0.25));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 0.75));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 1.25));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 1.75));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 2.25));
    }

    #[test]
    fn test_finding_face_edge_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, -0.5, 0.0,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 2);
    }

    #[test]
    fn test_finding_face_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, 0.0, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_finding_edge_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, 0.0, 0.25,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_finding_vertex_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![1.0, 0.0, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_split_edges()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);
        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections).unwrap();

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
    fn test_split_faces()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_shifted_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 8);

        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections).unwrap();

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
    fn test_split_face_two_times()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_positions(positions1).with_indices(indices1).build().unwrap();
        let area1 = mesh1.face_area(mesh1.face_iter().next().unwrap());

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.2, -0.2, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new().with_positions(positions2).with_indices(indices2).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 2);

        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections).unwrap();

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
    fn test_split_edge_two_times()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_positions(positions1).with_indices(indices1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.0, -0.2, 0.5,  0.0, -0.2, 1.5,  0.0, 1.5, 0.0];
        let mut mesh2 = MeshBuilder::new().with_positions(positions2).with_indices(indices2).build().unwrap();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 2);

        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections).unwrap();

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
    fn test_face_face_splitting()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_positions(positions1).with_indices(indices1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.2, -0.2, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new().with_positions(positions2).with_indices(indices2).build().unwrap();

        mesh1.split_primitives_at_intersection(&mut mesh2);

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh2.no_vertices(), 5);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_simple_simple_splitting()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_shifted_simple_mesh_y_z();

        mesh1.split_primitives_at_intersection( &mut mesh2);

        assert_eq!(mesh1.no_vertices(), 14);
        assert_eq!(mesh2.no_vertices(), 14);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    #[test]
    fn test_box_box_splitting()
    {
        let mut mesh1 = MeshBuilder::new().cube().build().unwrap();
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        for vertex_id in mesh2.vertex_iter() {
            mesh2.move_vertex_by(vertex_id, vec3(0.5, 0.5, 0.5));
        }
        mesh1.split_primitives_at_intersection(&mut mesh2);

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();
    }

    fn create_simple_mesh_x_z() -> Mesh
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5,  0.0, 0.0, 2.0,  1.0, 0.0, 2.5];
        MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap()
    }

    fn create_simple_mesh_y_z() -> Mesh
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.5, -0.5, 0.0,  0.5, -0.5, 1.0,  0.5, 0.5, 0.5,  0.5, 0.5, 1.5,  0.5, -0.5, 2.0,  0.5, 0.5, 2.5];
        MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap()
    }

    fn create_shifted_simple_mesh_y_z() -> Mesh
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.5, -0.5, -0.2,  0.5, -0.5, 0.8,  0.5, 0.5, 0.3,  0.5, 0.5, 1.3,  0.5, -0.5, 1.8,  0.5, 0.5, 2.3];
        MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap()
    }
}