# tri-mesh

[![crates.io](https://img.shields.io/crates/v/tri-mesh.svg)](https://crates.io/crates/tri-mesh)
[![Docs.rs](https://docs.rs/tri-mesh/badge.svg)](https://docs.rs/tri-mesh)
[![Continuous integration](https://github.com/asny/tri-mesh/actions/workflows/rust.yml/badge.svg)](https://github.com/asny/tri-mesh/actions/workflows/rust.yml)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/asny/tri-mesh/blob/master/LICENSE)

A triangle mesh data structure including basic operations. Use it to create, edit and compute on 3D models.

## Features

- The main struct Mesh implements the half-edge mesh data structure for easy and efficient traversal
- Half-edge walker to traverse the mesh
- Iterators over primitives (vertices, half-edges, edges, faces)
- Measures on vertices, edges and faces (e.g. position of vertex, area of face)
- Edit functionality (e.g. split edge, collapse edge, flip edge)
- Quality functionality (e.g. flip edges recursively to improve triangle quality, collapse small faces)
- Transformations affecting the vertex positions (e.g. moving a single vertex or rotate the entire mesh)
- Intersection functionality (e.g. face/ray intersection, edge/point intersection)
- Merge used for merging of entire meshes (e.g. append one mesh to another or merge overlapping primitives in a mesh)
- Split functionality (e.g. clone a subset of a mesh or split two meshes at their intersection)
- And more...

Please, see the [documentation](https://docs.rs/tri-mesh) for more details.