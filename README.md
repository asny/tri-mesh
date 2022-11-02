# tri-mesh

[![crates.io](https://img.shields.io/crates/v/tri-mesh.svg)](https://crates.io/crates/tri-mesh)
[![Docs.rs](https://docs.rs/tri-mesh/badge.svg)](https://docs.rs/tri-mesh)
[![Continuous integration](https://github.com/asny/tri-mesh/actions/workflows/rust.yml/badge.svg)](https://github.com/asny/tri-mesh/actions/workflows/rust.yml)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/asny/tri-mesh/blob/master/LICENSE)

This crate contains an implementation of the half-edge data structure which represents a triangle mesh and is efficient for creating, editing, traversing and computing on that mesh. 
The mesh can easily be created from and exported into a format that is efficient for visualization.

This crate also contains basic functionality to safely operate on this mesh data structure and which can be used for implementing more advanced operations, for example
- Half-edge walker to traverse the mesh
- Iterators over primitives (vertices, half-edges, edges, faces)
- Edit functionality (e.g. split edge, collapse edge, flip edge)

Finally, a set of more or less advanced operations is also provided, for example
- Transformations affecting the vertex positions (e.g. moving a single vertex or rotate the entire mesh)
- Measures on vertices, edges and faces (e.g. position of vertex, area of face)
- Quality functionality (e.g. flip edges recursively to improve triangle quality, collapse small faces)
- Intersection functionality (e.g. face/ray intersection, edge/point intersection)
- Merge used for merging of entire meshes (e.g. append one mesh to another or merge overlapping primitives in a mesh)
- Split functionality (e.g. clone a subset of a mesh or split two meshes at their intersection)

Please, see the [documentation](https://docs.rs/tri-mesh) for more details.