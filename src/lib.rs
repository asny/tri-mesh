//!
//! A triangle mesh data structure including basic operations.
//!
//! # Features
//!
//! - The main struct [Mesh](crate::mesh::Mesh) implements the half-edge mesh data structure for easy and efficient traversal
//! - Half-edge [walker](crate::mesh::traversal::Walker) to traverse the mesh
//! - [Iterators](mesh/struct.Mesh.html#iterators) over primitives (vertices, half-edges, edges, faces)
//! - Convenient [connectivity](mesh/struct.Mesh.html#connectivity) functionality (e.g. vertices of a face, edge between two vertices)
//! - Measures on [vertices](mesh/struct.Mesh.html#vertex-measures), [edges](mesh/struct.Mesh.html#edge-measures) and [faces](mesh/struct.Mesh.html#face-measures) (e.g. position of vertex, area of face)
//! - [Bounding box](mesh/struct.Mesh.html#bounding-box) functionality (e.g. constructing the axis aligned bounding box)
//! - [Edit](mesh/struct.Mesh.html#edit) functionality (e.g. split edge, collapse edge, flip edge)
//! - [Quality](mesh/struct.Mesh.html#quality) functionality (e.g. flip edges recursively to improve triangle quality, collapse small faces)
//! - [Orientation](mesh/struct.Mesh.html#orientation) functionality (e.g. flip orientation of all faces)
//! - [Transformations](mesh/struct.Mesh.html#transformations) affecting the vertex positions (e.g. moving a single vertex or rotate the entire mesh)
//! - [Intersection](mesh/struct.Mesh.html#intersection) functionality (e.g. face/ray intersection, edge/point intersection)
//! - [Merge](mesh/struct.Mesh.html#merge) used for merging of entire meshes (e.g. append one mesh to another or merge overlapping primitives in a mesh)
//! - [Split](mesh/struct.Mesh.html#split) functionality (e.g. clone a subset of a mesh or split two meshes at their intersection)
//! - [Export](mesh/struct.Mesh.html#export) functionality (methods for extracting raw float buffers which can be used for visualisation)
//! - And more..
//!
//! All functionality is implemented as methods on the [Mesh](crate::mesh::Mesh) struct, so take a look at that rather long list of methods for a complete overview.
//! Also, to construct a new mesh, use the [Mesh builder](crate::mesh_builder::MeshBuilder).
//!

#![warn(missing_docs)]

pub mod mesh;
pub mod mesh_builder;
pub mod prelude;

pub use crate::mesh_builder::MeshBuilder;
