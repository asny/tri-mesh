//!
//! Module containing the [Mesh](crate::mesh::Mesh) definition and functionality.
//!

pub use crate::math::*;

mod ids;
#[doc(inline)]
pub use ids::*;

mod iterators;
#[doc(inline)]
pub use iterators::*;

mod traversal;
#[doc(inline)]
pub use traversal::*;

mod edit;
#[doc(inline)]
pub use edit::*;

mod validity;
#[doc(inline)]
pub use validity::*;

mod orientation;
#[doc(inline)]
pub use orientation::*;

mod cleanup;
#[doc(inline)]
pub use cleanup::*;

mod append;
#[doc(inline)]
pub use append::*;

mod connectivity_info;

use crate::mesh::connectivity_info::ConnectivityInfo;
use std::collections::HashMap;

use thiserror::Error;
///
/// Error when performing a mesh operation
///
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum MeshError {
    #[error("configuration is not valid: {0}")]
    ActionWillResultInInvalidMesh(String),
    #[error("action will produce a non-manifold mesh: {0}")]
    ActionWillResultInNonManifoldMesh(String),
    #[error("the mesh has ended up in an invalid state: {0}")]
    MeshIsInvalid(String),
}

///
/// Represents a triangle mesh. Use the [Mesh builder](crate::mesh_builder::MeshBuilder) to construct a new mesh.
///
/// ## Functionality:
/// - [Traversal](#traversal)
/// - [Iterators](#iterators)
/// - [Connectivity](#connectivity)
/// - [Vertex measures](#vertex-measures)
/// - [Edge measures](#edge-measures)
/// - [Face measures](#face-measures)
/// - [Bounding box](#bounding-box)
/// - [Edit](#edit)
/// - [Quality](#quality)
/// - [Orientation](#orientation)
/// - [Transformations](#transformations)
/// - [Intersection](#intersection)
/// - [Merge](#merge)
/// - [Split](#split)
/// - [Export](#export)
/// - [Connected components](#connected-components)
/// - [Validity](#validity)
///
#[derive(Debug)]
pub struct Mesh {
    connectivity_info: ConnectivityInfo,
}

impl Mesh {
    /// Constructs a new mesh. For more options to construct a mesh, see [MeshBuilder](crate::MeshBuilder) or the [io](crate::io) module.
    pub fn new(indices: Vec<u32>, positions: Vec<f64>) -> Mesh {
        let no_vertices = positions.len() / 3;
        let no_faces = indices.len() / 3;
        let mesh = Mesh {
            connectivity_info: ConnectivityInfo::new(no_vertices, no_faces),
        };

        // Create vertices
        for i in 0..no_vertices {
            mesh.connectivity_info.new_vertex(vec3(
                positions[i * 3],
                positions[i * 3 + 1],
                positions[i * 3 + 2],
            ));
        }

        let mut twins = HashMap::<(VertexID, VertexID), HalfEdgeID>::new();
        fn sort(a: VertexID, b: VertexID) -> (VertexID, VertexID) {
            if a < b {
                (a, b)
            } else {
                (b, a)
            }
        }

        // Create faces and twin connectivity
        for face in 0..no_faces {
            let v0 = indices[face * 3];
            let v1 = indices[face * 3 + 1];
            let v2 = indices[face * 3 + 2];

            let face = mesh.connectivity_info.create_face(
                unsafe { VertexID::new(v0) },
                unsafe { VertexID::new(v1) },
                unsafe { VertexID::new(v2) },
            );

            // mark twin halfedges
            let mut walker = mesh.walker_from_face(face);
            for _ in 0..3 {
                let vertex_id = walker.vertex_id().unwrap();
                walker.as_next();
                let key = sort(vertex_id, walker.vertex_id().unwrap());
                if let Some(twin) = twins.get(&key) {
                    mesh.connectivity_info
                        .set_halfedge_twin(walker.halfedge_id().unwrap(), *twin);
                } else {
                    twins.insert(key, walker.halfedge_id().unwrap());
                }
            }
        }
        for halfedge in mesh.connectivity_info.halfedge_iterator() {
            if mesh
                .connectivity_info
                .halfedge(halfedge)
                .unwrap()
                .twin
                .is_none()
            {
                let vertex = mesh
                    .walker_from_halfedge(halfedge)
                    .as_previous()
                    .vertex_id()
                    .unwrap();
                mesh.connectivity_info.set_halfedge_twin(
                    halfedge,
                    mesh.connectivity_info
                        .new_halfedge(Some(vertex), None, None),
                );
            }
        }

        mesh
    }

    fn new_internal(connectivity_info: ConnectivityInfo) -> Mesh {
        Mesh { connectivity_info }
    }

    /// Returns the vertex position.
    pub fn vertex_position(&self, vertex_id: VertexID) -> Vec3 {
        self.connectivity_info.position(vertex_id)
    }

    /// Moves the vertex to the specified position.
    pub fn set_vertex_position(&mut self, vertex_id: VertexID, value: Vec3) {
        self.connectivity_info.set_position(vertex_id, value);
    }

    /// Returns the number of vertices in the mesh.
    pub fn no_vertices(&self) -> usize {
        self.connectivity_info.no_vertices()
    }

    /// Returns the number of edges in the mesh.
    pub fn no_edges(&self) -> usize {
        self.connectivity_info.no_halfedges() / 2
    }

    /// Returns the number of half-edges in the mesh.
    pub fn no_halfedges(&self) -> usize {
        self.connectivity_info.no_halfedges()
    }

    /// Returns the number of faces in the mesh.
    pub fn no_faces(&self) -> usize {
        self.connectivity_info.no_faces()
    }

    /// Returns whether or not the mesh is closed, ie. contains no holes.
    pub fn is_closed(&self) -> bool {
        for halfedge_id in self.edge_iter() {
            if self.is_edge_on_boundary(halfedge_id) {
                return false;
            }
        }
        true
    }
}

impl Clone for Mesh {
    fn clone(&self) -> Mesh {
        Mesh::new_internal(self.connectivity_info.clone())
    }
}

impl std::fmt::Display for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "**** Connectivity: ****")?;
        writeln!(f, "{}", self.connectivity_info)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MeshBuilder;

    #[test]
    fn test_one_face_connectivity() {
        let mesh = Mesh::new(
            vec![0, 1, 2],
            vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        );

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

        let t6 = mesh
            .walker_from_face(f1)
            .as_previous()
            .as_previous()
            .as_twin()
            .as_twin()
            .face_id();
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
                if mesh.walker_from_halfedge(halfedge_id).face_id().is_none() {
                    round = false;
                    break;
                }
            }
            if round {
                id = Some(vertex_id);
                break;
            }
        }
        let mut walker = mesh.walker_from_vertex(id.unwrap());
        let start_edge = walker.halfedge_id().unwrap();
        let one_round_edge = walker
            .as_previous()
            .as_twin()
            .as_previous()
            .as_twin()
            .as_previous()
            .twin_id()
            .unwrap();
        assert_eq!(start_edge, one_round_edge);
    }

    #[test]
    fn test_new_from_positions() {
        let positions: Vec<f64> = vec![
            0.0, 0.0, 0.0, 1.0, 0.0, -0.5, -1.0, 0.0, -0.5, 0.0, 0.0, 0.0, -1.0, 0.0, -0.5, 0.0,
            0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, -0.5,
        ];

        let mesh = Mesh::new((0..9).collect(), positions);

        assert_eq!(9, mesh.no_vertices());
        assert_eq!(3, mesh.no_faces());
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_extreme_coordinates() {
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3, 0, 3, 1];
        let positions: Vec<f64> = vec![
            0.0, 0.0, 0.0, 1.0, 0.0, -0.5, -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
        ];
        let mesh = MeshBuilder::new()
            .with_indices(indices)
            .with_positions(positions)
            .build()
            .unwrap();

        let (min_coordinates, max_coordinates) = mesh.extreme_coordinates();

        assert_eq!(min_coordinates, vec3(-1.0, 0.0, -0.5));
        assert_eq!(max_coordinates, vec3(1.0, 0.0, 1.0));
    }

    #[test]
    fn test_is_closed_when_not_closed() {
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3, 0, 3, 1];
        let positions: Vec<f64> = vec![
            0.0, 0.0, 0.0, 1.0, 0.0, -0.5, -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
        ];
        let mesh = MeshBuilder::new()
            .with_indices(indices)
            .with_positions(positions)
            .build()
            .unwrap();
        assert!(!mesh.is_closed());
    }

    #[test]
    fn test_is_closed_when_closed() {
        let mesh = MeshBuilder::new().cube().build().unwrap();
        assert!(mesh.is_closed());
    }
}
