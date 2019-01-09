use crate::mesh::*;
use crate::MeshBuilder;

#[derive(Debug)]
pub enum Error {
    IsNotValid {message: String}
}

pub fn test_is_valid(mesh: &Mesh) -> Result<(), Error>
{
    for vertex_id in mesh.vertex_iter() {
        if let Some(halfedge_id) = mesh.walker_from_vertex(&vertex_id).halfedge_id()
        {
            if !mesh.halfedge_iter().any(|he_id| he_id == halfedge_id) {
                return Err(Error::IsNotValid {message: format!("Vertex {} points to an invalid halfedge {}", vertex_id, halfedge_id)});
            }
            if mesh.walker_from_vertex(&vertex_id).as_twin().vertex_id().unwrap() != vertex_id
            {
                return Err(Error::IsNotValid {message: format!("Halfedge {} pointed to by vertex {} does not start in that vertex, but instead in {}", mesh.walker_from_vertex(&vertex_id).halfedge_id().unwrap(), vertex_id, mesh.walker_from_vertex(&vertex_id).as_twin().vertex_id().unwrap())});
            }
        }
        else {
            return Err(Error::IsNotValid {message: format!("Vertex {} does not point to a halfedge", vertex_id)});
        }
    }
    for halfedge_id in mesh.halfedge_iter() {
        let walker = mesh.walker_from_halfedge(&halfedge_id);

        if let Some(twin_id) = walker.twin_id()
        {
            if !mesh.halfedge_iter().any(|he_id| he_id == twin_id) {
                return Err(Error::IsNotValid {message: format!("Halfedge {} points to an invalid twin halfedge {}", halfedge_id, twin_id)});
            }
            if mesh.walker_from_halfedge(&twin_id).twin_id().unwrap() != halfedge_id
            {
                return Err(Error::IsNotValid {message: format!("Halfedge twin pointed to by halfedge {} does not point back to halfedge", halfedge_id)});
            }
            if mesh.walker_from_halfedge(&twin_id).vertex_id() == walker.vertex_id()
            {
                return Err(Error::IsNotValid {message: format!("Invalid orientation: The halfedge {} and its twin halfedge {} points to the same vertex {}", halfedge_id, twin_id, walker.vertex_id().unwrap())});
            }
        }
        else {
            return Err(Error::IsNotValid {message: format!("Halfedge {} does not point to a twin halfedge", halfedge_id)});
        }

        if let Some(vertex_id) = walker.vertex_id()
        {
            if !mesh.vertex_iter().any(|vid| vid == vertex_id) {
                return Err(Error::IsNotValid {message: format!("Halfedge {} points to an invalid vertex {}", halfedge_id, vertex_id)});
            }
        }
        else {
            return Err(Error::IsNotValid {message: format!("Halfedge {} does not point to a vertex", halfedge_id)});
        }

        if let Some(face_id) = walker.face_id()
        {
            if !mesh.face_iter().any(|fid| fid == face_id) {
                return Err(Error::IsNotValid {message: format!("Halfedge {} points to an invalid face {}", halfedge_id, face_id)});
            }
            if walker.next_id().is_none() {
                return Err(Error::IsNotValid {message: format!("Halfedge {} points to a face but not a next halfedge", halfedge_id)});
            }
        }

        if let Some(next_id) = walker.next_id()
        {
            if !mesh.halfedge_iter().any(|he_id| he_id == next_id) {
                return Err(Error::IsNotValid {message: format!("Halfedge {} points to an invalid next halfedge {}", halfedge_id, next_id)});
            }
            if walker.face_id().is_none() {
                return Err(Error::IsNotValid {message: format!("Halfedge {} points to a next halfedge but not a face", halfedge_id)});
            }
            if mesh.walker_from_halfedge(&next_id).previous_id().unwrap() != halfedge_id
            {
                return Err(Error::IsNotValid {message: format!("Halfedge next pointed to by halfedge {} does not point back to halfedge", halfedge_id)});
            }
        }

        if mesh.edge_length(&halfedge_id) < 0.00001
        {
            return Err(Error::IsNotValid {message: format!("Length of edge {} is too small ({})", halfedge_id, mesh.edge_length(&halfedge_id))})
        }
    }
    for face_id in mesh.face_iter() {
        if let Some(halfedge_id) = mesh.walker_from_face(&face_id).halfedge_id()
        {
            if !mesh.halfedge_iter().any(|he_id| he_id == halfedge_id) {
                return Err(Error::IsNotValid {message: format!("Face {} points to an invalid halfedge {}", face_id, halfedge_id)});
            }
            if mesh.walker_from_face(&face_id).face_id().unwrap() != face_id
            {
                return Err(Error::IsNotValid {message: format!("Halfedge pointed to by face {} does not point to back to face", face_id)});
            }
        }
        else {
            return Err(Error::IsNotValid {message: format!("Face {} does not point to a halfedge", face_id)});
        }

        if mesh.face_area(&face_id) < 0.00001
        {
            return Err(Error::IsNotValid {message: format!("Area of face {} is too small ({})", face_id, mesh.face_area(&face_id))})
        }
    }

    for vertex_id1 in mesh.vertex_iter()
    {
        for vertex_id2 in mesh.vertex_iter()
        {
            if mesh.connecting_edge(&vertex_id1, &vertex_id2).is_some() != mesh.connecting_edge(&vertex_id2, &vertex_id1).is_some()
            {
                return Err(Error::IsNotValid {message: format!("Vertex {} and Vertex {} is connected one way, but not the other way", vertex_id1, vertex_id2)});
            }
            let mut found = false;
            for halfedge in mesh.vertex_halfedge_iter(&vertex_id1) {
                if halfedge.vertex_id().unwrap() == vertex_id2 {
                    if found {
                        return Err(Error::IsNotValid {message: format!("Vertex {} and Vertex {} is connected by multiple edges", vertex_id1, vertex_id2)})
                    }
                    found = true;
                }
            }
        }
    }
    Ok(())
}

pub fn create_single_face() -> Mesh
{
    let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.0];
    MeshBuilder::new().with_positions(positions).build().unwrap()
}

pub fn create_two_connected_faces() -> Mesh
{
    let indices: Vec<u32> = vec![0, 2, 3,  0, 3, 1];
    let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5];
    MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap()
}

pub fn create_three_connected_faces() -> Mesh
{
    let indices: Vec<u32> = vec![0, 2, 3,  0, 3, 1,  0, 1, 2];
    let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5];
    MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap()
}

pub fn create_unconnected_cube() -> Mesh
{
    let positions: Vec<f32> = vec![
        1.0, 1.0, -1.0,
        -1.0, 1.0, -1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0, 1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0, -1.0,

        -1.0, -1.0, -1.0,
        1.0, -1.0, -1.0,
        1.0, -1.0, 1.0,
        1.0, -1.0, 1.0,
        -1.0, -1.0, 1.0,
        -1.0, -1.0, -1.0,

        1.0, -1.0, -1.0,
        -1.0, -1.0, -1.0,
        1.0, 1.0, -1.0,
        -1.0, 1.0, -1.0,
        1.0, 1.0, -1.0,
        -1.0, -1.0, -1.0,

        -1.0, -1.0, 1.0,
        1.0, -1.0, 1.0,
        1.0, 1.0, 1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0, 1.0,
        -1.0, -1.0, 1.0,

        1.0, -1.0, -1.0,
        1.0, 1.0, -1.0,
        1.0, 1.0, 1.0,
        1.0, 1.0, 1.0,
        1.0, -1.0, 1.0,
        1.0, -1.0, -1.0,

        -1.0, 1.0, -1.0,
        -1.0, -1.0, -1.0,
        -1.0, 1.0, 1.0,
        -1.0, -1.0, 1.0,
        -1.0, 1.0, 1.0,
        -1.0, -1.0, -1.0
    ];

    MeshBuilder::new().with_positions(positions).build().unwrap()
}