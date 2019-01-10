use crate::mesh::*;
use crate::MeshBuilder;

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