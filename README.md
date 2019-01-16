# tri-mesh

[![](http://meritbadge.herokuapp.com/tri-mesh)](https://crates.io/crates/tri-mesh)
[![Docs.rs](https://docs.rs/tri-mesh/badge.svg)](https://docs.rs/tri-mesh)
[![Build Status](https://travis-ci.org/asny/tri-mesh.svg?branch=master)](https://travis-ci.org/asny/tri-mesh)

A triangle mesh data structure including basic operations.

Why another triangle mesh data structure crate you might ask.
Well, if you want a more feature complete crate than [half_edge_mesh](https://crates.io/crates/half_edge_mesh) and a less generic crate than [plexus](https://crates.io/crates/plexus),
then `tri-mesh` is probably something for you!

```toml
[dependencies]
tri-mesh = "0.1.1"
```

## Features

- The main struct Mesh implements the half-edge mesh data structure for easy and efficient traversal
- Half-edge walker to traverse the mesh
- Iterators over primitives (vertices, half-edges, edges, faces)
- Convenient connectivity functionality (e.g. vertices of a face, edge between two vertices)
- Measures on vertices, edges and faces (e.g. position of vertex, area of face)
- Edit functionality (e.g. split edge, collapse edge, flip edge)
- Quality functionality (e.g. flip edges recursively to improve triangle quality, collapse small faces)
- Orientation functionality (e.g. flip orientation of all faces)
- Transformations affecting the vertex positions (e.g. moving a single vertex or rotate the entire mesh)
- Merging and splitting used for high level merging and splitting of entire meshes (e.g. clone a subset of a mesh or merge overlapping primitives)

Please, see the [documentation](https://docs.rs/tri-mesh) for more details.

## Usage

```rust
use tri_mesh::prelude::*;

fn main() {
    // Construct a mesh from positions and indices
    let indices: Vec<u32> = vec![0, 1, 2,  0, 2, 3,  0, 3, 1];
    let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0];
    let mesh = MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap();
    
    // Compute the extreme coordinates..
    let mut min_coordinates = vec3(std::f32::MAX, std::f32::MAX, std::f32::MAX);
    let mut max_coordinates = vec3(std::f32::MIN, std::f32::MIN, std::f32::MIN);
    for vertex_id in mesh.vertex_iter()
    {
        let position = mesh.vertex_position(vertex_id);
        for i in 0..3 {
            min_coordinates[i] = min_coordinates[i].min(position[i]);
            max_coordinates[i] = max_coordinates[i].max(position[i]);
        }
    }
    
    // .. or use the built-in method
    let (min_coordinates, max_coordinates) = mesh.extreme_coordinates();
}
```

Please, see the [documentation](https://docs.rs/tri-mesh) for more examples.
