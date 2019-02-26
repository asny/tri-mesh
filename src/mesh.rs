//!
//! Module containing the [Mesh](crate::mesh::Mesh) definition and functionality.
//!

pub mod math {
    //!
    //! Linear algebra types for vector calculations. Basically re-export the [cgmath](https://crates.io/crates/cgmath) library.
    //!

    pub use cgmath;
    pub use cgmath::*;

    /// Vector with three elements.
    pub type Vec3 = Vector3<f32>;
    /// Vector with four elements.
    pub type Vec4 = Vector4<f32>;

    /// 3x3 matrix.
    pub type Mat3 = Matrix3<f32>;
    /// 4x4 matrix.
    pub type Mat4 = Matrix4<f32>;
}

pub mod ids;
pub mod traversal;
pub mod iterators;
pub mod connectivity;
pub mod vertex_measures;
pub mod edge_measures;
pub mod face_measures;
pub mod bounding_box;
pub mod edit;
pub mod quality;
pub mod orientation;
pub mod transformations;
pub mod intersection;
pub mod merging_and_splitting;
pub mod export;
pub mod connected_components;
pub mod validity;

mod connectivity_info;

use crate::mesh::connectivity_info::ConnectivityInfo;
use std::collections::HashMap;
use crate::mesh::ids::*;
use crate::mesh::math::*;

/// Mesh errors.
#[derive(Debug)]
pub enum Error {
    /// Returned from a Mesh method when applying the method with the given configuration is not valid.
    ActionWillResultInInvalidMesh {
        /// Error reason.
        message: String
    },
    /// Returned from a Mesh method when applying a method will produce a non-manifold mesh.
    ActionWillResultInNonManifoldMesh {
        /// Error reason.
        message: String
    },
    /// Returned from [is_valid](crate::mesh::Mesh::is_valid) method when the mesh has ended up in an invalid state.
    MeshIsInvalid {
        /// Error reason.
        message: String
    }
}

///
/// Represents a triangle mesh. Use the [Mesh builder](crate::mesh_builder::MeshBuilder) to construct a new mesh.
///
/// ## Functionality:
/// - [Traversal](#traversal)
/// - [Iterators](#iterators)
/// - [Connectivity functionality](#connectivity-functionality)
/// - [Vertex measures](#vertex-measures)
/// - [Edge measures](#edge-measures)
/// - [Face measures](#face-measures)
/// - [Bounding box functionality](#bounding-box-functionality)
/// - [Edit functionality](#edit-functionality)
/// - [Quality functionality](#quality-functionality)
/// - [Orientation functionality](#orientation-functionality)
/// - [Transformations](#transformations)
/// - [Intersection](#intersection)
/// - [Merging & splitting](#merging--splitting)
/// - [Export functionality](#export-functionality)
/// - [Connected components](#connected-components)
/// - [Validity](#validity)
///
#[derive(Debug)]
pub struct Mesh {
    positions: HashMap<VertexID, Vec3>,
    connectivity_info: ConnectivityInfo
}

impl Mesh
{
    pub(crate) fn new(indices: Vec<u32>, positions: Vec<f32>) -> Mesh
    {
        let no_vertices = positions.len()/3;
        let no_faces = indices.len()/3;
        let mut mesh = Mesh { connectivity_info: ConnectivityInfo::new(no_vertices, no_faces), positions: HashMap::new()};

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

            let face_id = mesh.connectivity_info.create_face(v0, v1, v2);

            for twin_id in mesh.halfedge_iter() {
                walker.as_halfedge_walker(twin_id);
                if walker.twin_id().is_none() && walker.face_id().unwrap() != face_id {
                    let vertex_id0 = walker.vertex_id().unwrap();
                    let vertex_id1 = walker.as_previous().vertex_id().unwrap();

                    if vertex_id0 == v0 && vertex_id1 == v1 || vertex_id0 == v1 && vertex_id1 == v0 {
                        let halfedge_id = mesh.walker_from_face(face_id).halfedge_id().unwrap();
                        mesh.connectivity_info.set_halfedge_twin(halfedge_id, twin_id);
                    }
                    if vertex_id0 == v1 && vertex_id1 == v2 || vertex_id0 == v2 && vertex_id1 == v1 {
                        let halfedge_id = mesh.walker_from_face(face_id).as_next().halfedge_id().unwrap();
                        mesh.connectivity_info.set_halfedge_twin(halfedge_id, twin_id);
                    }
                    if vertex_id0 == v2 && vertex_id1 == v0 || vertex_id0 == v0 && vertex_id1 == v2 {
                        let halfedge_id = mesh.walker_from_face(face_id).as_previous().halfedge_id().unwrap();
                        mesh.connectivity_info.set_halfedge_twin(halfedge_id, twin_id);
                    }
                }
            }
        }

        mesh.create_boundary_edges();

        mesh
    }

    fn new_internal(positions: HashMap<VertexID, Vec3>, connectivity_info: ConnectivityInfo) -> Mesh
    {
        Mesh {positions, connectivity_info}
    }

    /// Returns the number of vertices in the mesh.
    pub fn no_vertices(&self) -> usize
    {
        self.connectivity_info.no_vertices()
    }

    /// Returns the number of edges in the mesh.
    pub fn no_edges(&self) -> usize
    {
        self.connectivity_info.no_halfedges()/2
    }

    /// Returns the number of half-edges in the mesh.
    pub fn no_halfedges(&self) -> usize
    {
        self.connectivity_info.no_halfedges()
    }

    /// Returns the number of faces in the mesh.
    pub fn no_faces(&self) -> usize
    {
        self.connectivity_info.no_faces()
    }

    fn create_vertex(&mut self, position: Vec3) -> VertexID
    {
        let id = self.connectivity_info.new_vertex();
        self.positions.insert(id, position);
        id
    }

    fn create_boundary_edges(&mut self)
    {
        let mut walker = self.walker();
        for halfedge_id in self.halfedge_iter()
        {
            walker.as_halfedge_walker(halfedge_id);
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
        Mesh::new_internal(self.positions.clone(), self.connectivity_info.clone())
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
    use crate::MeshBuilder;

    #[test]
    fn test_one_face_connectivity() {
        let mesh = Mesh::new(vec![0, 1, 2], vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);

        let f1 = mesh.face_iter().next().unwrap();
        let v1 = mesh.walker_from_face(f1).vertex_id().unwrap();
        let v2 = mesh.walker_from_face(f1).as_next().vertex_id().unwrap();
        let v3 = mesh.walker_from_face(f1).as_previous().vertex_id().unwrap();

        let t1 = mesh.walker_from_vertex(v1).vertex_id();
        assert_eq!(t1, Some(v2));

        let t2 = mesh.walker_from_vertex(v1).as_twin().vertex_id();
        assert_eq!(t2, Some(v1));

        let t3 = mesh.walker_from_vertex(v2).as_next().as_next().vertex_id();
        assert_eq!(t3, Some(v2));

        let t4 = mesh.walker_from_face(f1).as_twin().face_id();
        assert!(t4.is_none());

        let t5 = mesh.walker_from_face(f1).as_twin().next_id();
        assert!(t5.is_none());

        let t6 = mesh.walker_from_face(f1).as_previous().as_previous().as_twin().as_twin().face_id();
        assert_eq!(t6, Some(f1));

        let t7 = mesh.walker_from_vertex(v2).as_next().as_next().next_id();
        assert_eq!(t7, mesh.walker_from_vertex(v2).halfedge_id());

        let t8 = mesh.walker_from_vertex(v3).face_id();
        assert_eq!(t8, Some(f1));

        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_three_face_connectivity() {
        let mesh = MeshBuilder::new().subdivided_triangle().build().unwrap();
        let mut id = None;
        for vertex_id in mesh.vertex_iter() {
            let mut round = true;
            for halfedge_id in mesh.vertex_halfedge_iter(vertex_id) {
                if mesh.walker_from_halfedge(halfedge_id).face_id().is_none() { round = false; break; }
            }
            if round { id = Some(vertex_id); break; }
        }
        let mut walker = mesh.walker_from_vertex(id.unwrap());
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
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_extreme_coordinates()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  0, 2, 3,  0, 3, 1];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0];
        let mesh = MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap();

        let (min_coordinates, max_coordinates) = mesh.extreme_coordinates();

        assert_eq!(min_coordinates, vec3(-1.0, 0.0, -0.5));
        assert_eq!(max_coordinates, vec3(1.0, 0.0, 1.0));
    }
}