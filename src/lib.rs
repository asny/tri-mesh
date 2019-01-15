//!
//! A triangle mesh data structure including basic operations.
//!
//! Why another triangle mesh data structure crate you might ask.
//! Well, if you want a more feature complete crate than [half_edge_mesh](https://crates.io/crates/half_edge_mesh) and a less generic crate than [plexus](https://crates.io/crates/plexus),
//! then `tri-mesh` is probably something for you!
//!
//! ## Features
//! - The main struct [Mesh](crate::mesh::Mesh) implements the half-edge mesh data structure for easy and efficient traversal
//! - [Iterators](mesh/struct.Mesh.html#iterators) over primitives (vertices, half-edges, faces)
//! - Half-edge [walker](mesh/struct.Mesh.html#traversal) to traverse the mesh
//! - Convenient [connectivity](mesh/struct.Mesh.html#connectivity-functionality) functionality (e.g. vertices of a face, edge between two vertices)
//! - Measures on [vertices](mesh/struct.Mesh.html#vertex-measures), [edges](mesh/struct.Mesh.html#edge-measures) and [faces](mesh/struct.Mesh.html#face-measures) (e.g. position of vertex, area of face)
//! - [Edit](mesh/struct.Mesh.html#edit-functionality) functionality (e.g. split edge, collapse edge, flip edge)
//! - [Orientation](mesh/struct.Mesh.html#orientation-functionality) functionality (e.g. flip orientation of all faces)
//! - [Quality](mesh/struct.Mesh.html#quality-functionality) functionality (e.g. flip edges recursively to improve triangle quality, collapse small faces)
//! - And more! Most functionality is implemented as methods on the [Mesh](crate::mesh::Mesh) struct, so take a look at that rather long list of functions for a complete overview.
//!
//! To construct a new mesh, use the [Mesh builder](crate::mesh_builder::MeshBuilder).
//!
//! For more advanced mesh algorithms, take a look at the [geo-proc](https://github.com/asny/geo-proc) crate.
//!

#![warn(missing_docs)]

pub mod mesh;
pub mod mesh_builder;
pub mod prelude;

pub use crate::mesh_builder::MeshBuilder as MeshBuilder;
