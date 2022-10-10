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

pub use three_d_asset::{Indices, Positions, TriMesh as RawMesh};

///
/// Represents a triangle mesh. Use [RawMesh] to construct a new mesh.
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
/// - [Connected components](#connected-components)
/// - [Validity](#validity)
///
#[derive(Debug)]
pub struct Mesh {
    connectivity_info: ConnectivityInfo,
}

impl Mesh {
    pub fn new(input: &RawMesh) -> Self {
        let no_vertices = input.vertex_count();
        let no_faces = input.triangle_count();
        let indices = input
            .indices
            .to_u32()
            .unwrap_or((0..no_faces as u32 * 3).collect::<Vec<_>>());
        let positions = input.positions.to_f64();
        let mesh = Mesh {
            connectivity_info: ConnectivityInfo::new(no_vertices, no_faces),
        };

        // Create vertices
        for i in 0..no_vertices {
            mesh.connectivity_info.new_vertex(positions[i]);
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

    pub fn to_raw(&self) -> RawMesh {
        let vertices: Vec<VertexID> = self.vertex_iter().collect();
        let mut indices = Vec::with_capacity(self.no_faces() * 3);
        for face_id in self.face_iter() {
            for halfedge_id in self.face_halfedge_iter(face_id) {
                let vertex_id = self.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                let index = vertices.iter().position(|v| v == &vertex_id).unwrap();
                indices.push(index as u32);
            }
        }
        RawMesh {
            indices: Indices::U32(indices),
            positions: Positions::F64(
                self.vertex_iter()
                    .map(|vertex_id| self.vertex_position(vertex_id))
                    .collect::<Vec<_>>(),
            ),
            normals: Some(
                self.vertex_iter()
                    .map(|vertex_id| self.vertex_normal(vertex_id).cast::<f32>().unwrap())
                    .collect::<Vec<_>>(),
            ),
            ..Default::default()
        }
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
        Mesh {
            connectivity_info: self.connectivity_info.clone(),
        }
    }
}

impl std::fmt::Display for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "**** Connectivity: ****")?;
        writeln!(f, "{}", self.connectivity_info)?;
        Ok(())
    }
}

impl From<RawMesh> for Mesh {
    fn from(mesh: RawMesh) -> Self {
        Self::new(&mesh)
    }
}

impl From<&RawMesh> for Mesh {
    fn from(mesh: &RawMesh) -> Self {
        Self::new(mesh)
    }
}

impl From<Mesh> for RawMesh {
    fn from(mesh: Mesh) -> Self {
        mesh.to_raw()
    }
}

impl From<&Mesh> for RawMesh {
    fn from(mesh: &Mesh) -> Self {
        mesh.to_raw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_obj() {
        let source = b"o Cube
        v 1.000000 -1.000000 -1.000000
        v 1.000000 -1.000000 1.000000
        v -1.000000 -1.000000 1.000000
        v -1.000000 -1.000000 -1.000000
        v 1.000000 1.000000 -1.000000
        v 0.999999 1.000000 1.000001
        v -1.000000 1.000000 1.000000
        v -1.000000 1.000000 -1.000000
        f 1 2 3
        f 1 3 4
        f 5 8 7
        f 5 7 6
        f 1 5 6
        f 1 6 2
        f 2 6 7
        f 2 7 3
        f 3 7 8
        f 3 8 4
        f 5 1 4
        f 5 4 8"
            .to_vec();
        let mut raw_assets = three_d_asset::io::RawAssets::new();
        raw_assets.insert("cube.obj", source);
        let mut model: three_d_asset::Model = raw_assets.deserialize(".obj").unwrap();
        let mesh: Mesh = model.geometries.remove(0).into();
        assert_eq!(mesh.no_faces(), 12);
        assert_eq!(mesh.no_vertices(), 8);
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_indexed_export() {
        let mesh: Mesh = RawMesh::cylinder(16).into();
        let m: RawMesh = (&mesh).into();
        m.validate().unwrap();

        assert_eq!(m.triangle_count(), mesh.no_faces());
        assert_eq!(m.vertex_count(), mesh.no_vertices());

        let positions = m.positions.to_f64();
        let normals = m.normals.as_ref().unwrap();
        m.for_each_triangle(|i0, i1, i2| {
            let id0 = unsafe { VertexID::new(i0 as u32) };
            let id1 = unsafe { VertexID::new(i1 as u32) };
            let id2 = unsafe { VertexID::new(i2 as u32) };
            assert!(positions[i0].distance(mesh.vertex_position(id0)) < 0.001);
            assert!(positions[i1].distance(mesh.vertex_position(id1)) < 0.001);
            assert!(positions[i2].distance(mesh.vertex_position(id2)) < 0.001);
            assert!(normals[i0].distance(mesh.vertex_normal(id0).cast::<f32>().unwrap()) < 0.001);
            assert!(normals[i1].distance(mesh.vertex_normal(id1).cast::<f32>().unwrap()) < 0.001);
            assert!(normals[i2].distance(mesh.vertex_normal(id2).cast::<f32>().unwrap()) < 0.001);
        });
    }

    #[test]
    fn test_one_face_connectivity() {
        let mesh: Mesh = crate::test_utility::triangle();

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
    fn test_with_cube() {
        let mut mesh: Mesh = RawMesh::cube().into();
        mesh.merge_overlapping_primitives().unwrap();
        assert_eq!(mesh.no_faces(), 12);
        assert_eq!(mesh.no_vertices(), 8);
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_three_face_connectivity() {
        let mesh = crate::test_utility::subdivided_triangle();
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
        let mesh: Mesh = RawMesh {
            positions: Positions::F64(vec![
                vec3(0.0, 0.0, 0.0),
                vec3(1.0, 0.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 1.0),
                vec3(1.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(1.0, 0.0, 1.0),
                vec3(0.0, 1.0, 1.0),
            ]),
            ..Default::default()
        }
        .into();

        assert_eq!(9, mesh.no_vertices());
        assert_eq!(3, mesh.no_faces());
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_extreme_coordinates() {
        let mesh: Mesh = RawMesh::sphere(4).into();

        let (min_coordinates, max_coordinates) = mesh.extreme_coordinates();

        assert_eq!(min_coordinates, vec3(-1.0, -1.0, -1.0));
        assert_eq!(max_coordinates, vec3(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_is_closed_when_not_closed() {
        let mesh = crate::test_utility::subdivided_triangle();
        assert!(!mesh.is_closed());
    }

    #[test]
    fn test_is_closed_when_closed() {
        let mesh: Mesh = RawMesh::sphere(4).into();
        assert!(mesh.is_closed());
    }
}
