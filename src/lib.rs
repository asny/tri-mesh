//!
//! A triangle mesh data structure including basic operations.
//!
//! # Features
//!
//! - The main struct [Mesh](crate::mesh::Mesh) implements the half-edge mesh data structure for easy and efficient traversal
//! - Half-edge [walker](crate::mesh::traversal::Walker) to traverse the mesh
//! - [Iterators](mesh/struct.Mesh.html#iterators) over primitives (vertices, half-edges, edges, faces)
//! - Convenient [connectivity](mesh/struct.Mesh.html#connectivity-functionality) functionality (e.g. vertices of a face, edge between two vertices)
//! - Measures on [vertices](mesh/struct.Mesh.html#vertex-measures), [edges](mesh/struct.Mesh.html#edge-measures) and [faces](mesh/struct.Mesh.html#face-measures) (e.g. position of vertex, area of face)
//! - [Edit](mesh/struct.Mesh.html#edit-functionality) functionality (e.g. split edge, collapse edge, flip edge)
//! - [Quality](mesh/struct.Mesh.html#quality-functionality) functionality (e.g. flip edges recursively to improve triangle quality, collapse small faces)
//! - [Orientation](mesh/struct.Mesh.html#orientation-functionality) functionality (e.g. flip orientation of all faces)
//! - [Transformations](mesh/struct.Mesh.html#transformations) affecting the vertex positions (e.g. moving a single vertex or rotate the entire mesh)
//! - [Merging and splitting](mesh/struct.Mesh.html#merging--splitting) used for high level merging and splitting of entire meshes (e.g. clone a subset of a mesh or merge overlapping primitives)
//!
//! All functionality is implemented as methods on the [Mesh](crate::mesh::Mesh) struct, so take a look at that rather long list of methods for a complete overview.
//! Also, to construct a new mesh, use the [Mesh builder](crate::mesh_builder::MeshBuilder).
//!
//! For more advanced mesh algorithms, take a look at the [geo-proc](https://github.com/asny/geo-proc) crate.
//!

#![warn(missing_docs)]

pub mod mesh;
pub mod mesh_builder;
pub mod prelude;

pub use crate::mesh_builder::MeshBuilder as MeshBuilder;
