
pub mod math {
    use cgmath::{Vector2, Vector3};
    pub use cgmath::prelude::*;

    pub type Vec2 = Vector2<f32>;
    pub type Vec3 = Vector3<f32>;

    pub fn vec2(x: f32, y: f32) -> Vec2
    {
        Vector2::new(x, y)
    }
    pub fn vec3(x: f32, y: f32, z: f32) -> Vec3
    {
        Vector3::new(x, y, z)
    }
}

pub mod ids;
pub mod traversal;
pub mod iterators;
pub mod merging_and_splitting;
pub mod connectivity;
pub mod edit;
pub mod quality;
pub mod vertex_measures;
pub mod edge_measures;
pub mod face_measures;
pub mod orientation;
pub mod transformations;

mod connectivity_info;

use crate::mesh::connectivity_info::ConnectivityInfo;
use std::rc::Rc;
use std::collections::HashMap;
use crate::mesh::ids::*;
use crate::mesh::math::*;

#[derive(Debug)]
pub struct Mesh {
    positions: HashMap<VertexID, Vec3>,
    connectivity_info: Rc<ConnectivityInfo>
}

impl Mesh
{
    pub(crate) fn new(indices: Vec<u32>, positions: Vec<f32>) -> Mesh
    {
        let no_vertices = positions.len()/3;
        let no_faces = indices.len()/3;
        let mut mesh = Mesh { connectivity_info: Rc::new(ConnectivityInfo::new(no_vertices, no_faces)), positions: HashMap::new()};

        // Create vertices
        for i in 0..no_vertices {
            mesh.create_vertex(vec3(positions[i*3], positions[i*3+1], positions[i*3+2]));
        }

        // Create faces and twin connectivity
        let mut walker = mesh.walker();
        for face in 0..no_faces {
            let v0 = VertexID::new(indices[face * 3] as usize);
            let v1 = VertexID::new(indices[face * 3 + 1] as usize);
            let v2 = VertexID::new(indices[face * 3 + 2] as usize);

            let face_id = mesh.connectivity_info.create_face(&v0, &v1, &v2);

            for twin_id in mesh.halfedge_iter() {
                walker.as_halfedge_walker(&twin_id);
                if walker.twin_id().is_none() && walker.face_id().unwrap() != face_id {
                    let vertex_id0 = walker.vertex_id().unwrap();
                    let vertex_id1 = walker.as_previous().vertex_id().unwrap();

                    if vertex_id0 == v0 && vertex_id1 == v1 || vertex_id0 == v1 && vertex_id1 == v0 {
                        let halfedge_id = mesh.walker_from_face(&face_id).halfedge_id().unwrap();
                        mesh.connectivity_info.set_halfedge_twin(halfedge_id, twin_id);
                    }
                    if vertex_id0 == v1 && vertex_id1 == v2 || vertex_id0 == v2 && vertex_id1 == v1 {
                        let halfedge_id = mesh.walker_from_face(&face_id).as_next().halfedge_id().unwrap();
                        mesh.connectivity_info.set_halfedge_twin(halfedge_id, twin_id);
                    }
                    if vertex_id0 == v2 && vertex_id1 == v0 || vertex_id0 == v0 && vertex_id1 == v2 {
                        let halfedge_id = mesh.walker_from_face(&face_id).as_previous().halfedge_id().unwrap();
                        mesh.connectivity_info.set_halfedge_twin(halfedge_id, twin_id);
                    }
                }
            }
        }

        mesh.create_boundary_edges();

        mesh
    }

    fn new_internal(positions: HashMap<VertexID, Vec3>, connectivity_info: Rc<ConnectivityInfo>) -> Mesh
    {
        Mesh {positions, connectivity_info}
    }

    pub fn no_vertices(&self) -> usize
    {
        self.connectivity_info.no_vertices()
    }

    pub fn no_halfedges(&self) -> usize
    {
        self.connectivity_info.no_halfedges()
    }

    pub fn no_faces(&self) -> usize
    {
        self.connectivity_info.no_faces()
    }

    pub fn indices_buffer(&self) -> Vec<u32>
    {
        let vertices: Vec<VertexID> = self.vertex_iter().collect();
        let mut indices = Vec::with_capacity(self.no_faces() * 3);
        for face_id in self.face_iter()
        {
            for walker in self.face_halfedge_iter(&face_id) {
                let vertex_id = walker.vertex_id().unwrap();
                let index = vertices.iter().position(|v| v == &vertex_id).unwrap();
                indices.push(index as u32);
            }
        }
        indices
    }

    pub fn positions_buffer(&self) -> Vec<f32>
    {
        let mut positions = Vec::with_capacity(self.no_vertices() * 3);
        for v3 in self.vertex_iter().map(|ref vertex_id| self.vertex_position(vertex_id)) {
            positions.push(v3.x); positions.push(v3.y); positions.push(v3.z);
        }
        positions
    }

    pub fn normals_buffer(&self) -> Vec<f32>
    {
        let mut normals = Vec::with_capacity(self.no_vertices() * 3);
        for vertex_id in self.vertex_iter() {
            let normal = self.vertex_normal(&vertex_id);
            normals.push(normal.x);
            normals.push(normal.y);
            normals.push(normal.z);
        }
        normals
    }

    fn create_vertex(&mut self, position: Vec3) -> VertexID
    {
        let id = self.connectivity_info.new_vertex();
        self.positions.insert(id.clone(), position);
        id
    }

    fn create_boundary_edges(&mut self)
    {
        let mut walker = self.walker();
        for halfedge_id in self.halfedge_iter()
        {
            walker.as_halfedge_walker(&halfedge_id);
            if walker.twin_id().is_none()
            {
                let boundary_halfedge_id = self.connectivity_info.new_halfedge(walker.as_previous().vertex_id(), None, None);
                self.connectivity_info.set_halfedge_twin(halfedge_id, boundary_halfedge_id);
            }
        }
    }
}

impl Clone for Mesh {
    fn clone(&self) -> Mesh {
        Mesh::new_internal(self.positions.clone(), Rc::new((*self.connectivity_info).clone()))
    }
}

impl std::fmt::Display for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "**** Connectivity: ****")?;
        writeln!(f, "{}", self.connectivity_info)?;
        writeln!(f, "**** Positions: ****")?;
        writeln!(f, "{:?}", self.positions)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utility::*;

    #[test]
    fn test_one_face_connectivity() {
        let mesh = Mesh::new(vec![0, 1, 2], vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);

        let f1 = mesh.face_iter().next().unwrap();
        let v1 = mesh.walker_from_face(&f1).vertex_id().unwrap();
        let v2 = mesh.walker_from_face(&f1).as_next().vertex_id().unwrap();
        let v3 = mesh.walker_from_face(&f1).as_previous().vertex_id().unwrap();

        let t1 = mesh.walker_from_vertex(&v1).vertex_id();
        assert_eq!(t1, Some(v2.clone()));

        let t2 = mesh.walker_from_vertex(&v1).as_twin().vertex_id();
        assert_eq!(t2, Some(v1));

        let t3 = mesh.walker_from_vertex(&v2.clone()).as_next().as_next().vertex_id();
        assert_eq!(t3, Some(v2.clone()));

        let t4 = mesh.walker_from_face(&f1.clone()).as_twin().face_id();
        assert!(t4.is_none());

        let t5 = mesh.walker_from_face(&f1.clone()).as_twin().next_id();
        assert!(t5.is_none());

        let t6 = mesh.walker_from_face(&f1.clone()).as_previous().as_previous().as_twin().as_twin().face_id();
        assert_eq!(t6, Some(f1.clone()));

        let t7 = mesh.walker_from_vertex(&v2.clone()).as_next().as_next().next_id();
        assert_eq!(t7, mesh.walker_from_vertex(&v2).halfedge_id());

        let t8 = mesh.walker_from_vertex(&v3).face_id();
        assert_eq!(t8, Some(f1));

        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_three_face_connectivity() {
        let mesh = create_three_connected_faces();
        let mut id = None;
        for vertex_id in mesh.vertex_iter() {
            let mut round = true;
            for walker in mesh.vertex_halfedge_iter(&vertex_id) {
                if walker.face_id().is_none() { round = false; break; }
            }
            if round { id = Some(vertex_id); break; }
        }
        let mut walker = mesh.walker_from_vertex(&id.unwrap());
        let start_edge = walker.halfedge_id().unwrap();
        let one_round_edge = walker.as_previous().as_twin().as_previous().as_twin().as_previous().twin_id().unwrap();
        assert_eq!(start_edge, one_round_edge);
    }

    #[test]
    fn test_new_from_positions()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
                                       0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5];

        let mesh = Mesh::new((0..9).collect(), positions);

        assert_eq!(9, mesh.no_vertices());
        assert_eq!(3, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }
}