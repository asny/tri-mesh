# tri-mesh

[![](http://meritbadge.herokuapp.com/tri-mesh)](https://crates.io/crates/tri-mesh)
[![Docs.rs](https://docs.rs/tri-mesh/badge.svg)](https://docs.rs/tri-mesh)
[![Build Status](https://travis-ci.org/asny/tri-mesh.svg?branch=master)](https://travis-ci.org/asny/tri-mesh)

A triangle mesh data structure including basic operations.

Why another triangle mesh data structure crate you might ask.
Well, if you want a more feature complete crate than [half_edge_mesh](https://crates.io/crates/half_edge_mesh) and a less generic crate than [plexus](https://crates.io/crates/plexus),
then `tri-mesh` is probably something for you!

## Examples

- [Morph tool](https://asny.github.io/morph-web/) (and [source code](https://github.com/asny/tri-mesh/tree/master/examples/morph.rs))
- [Stitch tool](https://asny.github.io/stitch-web/) (and [source code](https://github.com/asny/tri-mesh/tree/master/examples/stitch.rs))

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

## Usage
Add the following to your `Cargo.toml`:
```toml
[dependencies]
tri-mesh = "0.2.0"
```

### I have a mesh without normals, how can I use `tri-mesh` to compute them?

```rust
use tri_mesh::prelude::*;

fn main() {
    // Construct a mesh from indices and positions buffers.
    let indices: Vec<u32> = vec![0, 1, 2,  0, 2, 3,  0, 3, 1];
    let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0];
    let mesh = MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap();
    
    // Get the indices, positions and normal buffers
    let indices_out = mesh.indices_buffer();
    let positions_out = mesh.positions_buffer();
    let normals_out = mesh.normals_buffer();
}
```

### I need the bounding box of my mesh, how can I get that?

```rust
use tri_mesh::prelude::*;

fn main() {
    // Construct any mesh, this time, we will construct a simple icosahedron
    let mesh = MeshBuilder::new().icosahedron().build().unwrap();
    
    // Compute the extreme coordinates which defines the axis aligned bounding box..
    let (min_coordinates, max_coordinates) = mesh.extreme_coordinates();
    
    // .. or construct an actual mesh representing the axis aligned bounding box
    let aabb = mesh.axis_aligned_bounding_box();
    
    // Export the bounding box to an obj file
    std::fs::write("foo.obj", mesh.parse_as_obj()).unwrap();
}
```

### How can I use `tri-mesh` to compute my own very special curvature measure?

```rust
use tri_mesh::prelude::*;

fn main() {
    // Construct any mesh, for simplicity, let's use a cube mesh
    let mesh = MeshBuilder::new().cube().build().unwrap();
    
    let mut curvature_measure = 0.0;
    // Let's say that the curvature measure is a sum of a curvature measure for each vertex
    // which means we need to visit all vertices
    for vertex_id in mesh.vertex_iter()
    {
        // Let's say that to compute the curvature of one vertex we need to visit the neighbouring faces
        // We will do that by iterating the half-edges pointing away from the vertex ..
        let mut curvature_measure_vertex = 0.0;
        for halfedge_id in mesh.vertex_halfedge_iter(vertex_id) {
            // .. and then create a walker from that halfedge and then get the face pointed to by that walker
            if let Some(face_id) = mesh.walker_from_halfedge(halfedge_id).face_id() {
                // Finally, insert the code for computing your special vertex curvature measure right here!
                // curvature_measure_vertex += ??; 
            }
        }
        curvature_measure += curvature_measure_vertex;
    }
}
```
