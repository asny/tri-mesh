//!
//! Module containing [MeshBuilder](crate::mesh_builder::MeshBuilder) which has functionality to build a new [Mesh](crate::mesh::Mesh) instance.
//!

use crate::mesh::Mesh;

/// MeshBuilder errors.
#[derive(Debug)]
pub enum Error {
    /// Returned when the positions haven't been specified before calling the build function.
    NoPositionsSpecified {
        /// Error reason.
        message: String
    }
}

///
/// `MeshBuilder` contains functionality to build a mesh from either raw data (indices, positions, normals)
/// or from simple geometric shapes (box, icosahedron, cylinder, ..) or from file source (.obj).
///
#[derive(Debug, Default)]
pub struct MeshBuilder {
    indices: Option<Vec<u32>>,
    positions: Option<Vec<f32>>
}

impl MeshBuilder {

    /// Creates a new [MeshBuilder](crate::mesh_builder::MeshBuilder) instance.
    pub fn new() -> Self
    {
        MeshBuilder {indices: None, positions: None}
    }

    ///
    /// Set the indices of each face, where the indices of face `x` is `(i0, i1, i2) = (indices[3*x], indices[3*x+1], indices[3*x+2])`.
    ///
    /// # Examples
    /// ```
    /// # use tri_mesh::mesh_builder::{MeshBuilder, Error};
    /// #
    /// # fn main() -> Result<(), Box<Error>> {
    /// let indices: Vec<u32> = vec![0, 1, 2,  0, 2, 3,  0, 3, 1];
    /// let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0];
    /// let mesh = MeshBuilder::new().with_indices(indices).with_positions(positions).build()?;
    ///
    /// assert_eq!(mesh.no_faces(), 3);
    /// assert_eq!(mesh.no_vertices(), 4);
    ///
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn with_indices(mut self, indices: Vec<u32>) -> Self
    {
        self.indices = Some(indices);
        self
    }

    ///
    /// Set the positions of each vertex, where the position of vertex `x` is `(x, y, z) = (positions[3*x], positions[3*x+1], positions[3*x+2])`;
    ///
    /// # Examples
    ///
    /// Build from positions (note: Use [merge_overlapping_primitives](crate::mesh::Mesh::merge_overlapping_primitives) if you want to merge
    /// unconnected but overlapping parts of the mesh):
    /// ```
    /// # use tri_mesh::mesh_builder::{MeshBuilder, Error};
    /// #
    /// # fn main() -> Result<(), Box<Error>> {
    /// let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
    ///                                    0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
    ///                                    0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5];
    /// let mesh = MeshBuilder::new().with_positions(positions).build()?;
    ///
    /// assert_eq!(mesh.no_faces(), 3);
    /// assert_eq!(mesh.no_vertices(), 9);
    ///
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn with_positions(mut self, positions: Vec<f32>) -> Self
    {
        self.positions = Some(positions);
        self
    }

    ///
    /// Parses the .obj file and extracts the connectivity information (indices) and positions which is used to construct a mesh.
    /// If the .obj file contains multiple objects, all objects are added to the mesh, but they will not be connected.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::mesh_builder::{MeshBuilder, Error};
    /// #
    /// # fn main() -> Result<(), Box<Error>> {
    /// let source = "o Cube
    /// v 1.000000 -1.000000 -1.000000
    /// #...
    /// # v 1.000000 -1.000000 1.000000
    /// # v -1.000000 -1.000000 1.000000
    /// # v -1.000000 -1.000000 -1.000000
    /// # v 1.000000 1.000000 -1.000000
    /// # v 0.999999 1.000000 1.000001
    /// # v -1.000000 1.000000 1.000000
    /// # v -1.000000 1.000000 -1.000000
    /// # f 1 2 3
    /// # f 1 3 4
    /// # f 5 8 7
    /// # f 5 7 6
    /// # f 1 5 6
    /// # f 1 6 2
    /// # f 2 6 7
    /// # f 2 7 3
    /// # f 3 7 8
    /// # f 3 8 4
    /// # f 5 1 4
    /// f 5 4 8".to_string();
    ///
    /// let mesh = MeshBuilder::new().with_obj(source).build()?;
    ///
    /// assert_eq!(mesh.no_faces(), 12);
    /// assert_eq!(mesh.no_vertices(), 8);
    ///
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn with_obj(mut self, source: String) -> Self
    {
        let mut positions = Vec::new();
        let mut indices = Vec::new();

        let objs = wavefront_obj::obj::parse(source).unwrap().objects;
        for obj in objs.iter()
        {
            let start_index = positions.len()/3;
            for vertex in obj.vertices.iter() {
                positions.push(vertex.x as f32);
                positions.push(vertex.y as f32);
                positions.push(vertex.z as f32);
            }

            for shape in obj.geometry.first().unwrap().shapes.iter() {
                match shape.primitive {
                    wavefront_obj::obj::Primitive::Triangle((i0, ..), (i1, ..), (i2, ..)) => {
                        indices.push((start_index + i0) as u32);
                        indices.push((start_index + i1) as u32);
                        indices.push((start_index + i2) as u32);
                    },
                    _ => {}
                }
            }
        }
        self.positions = Some(positions);
        self.indices = Some(indices);
        self
    }

    ///
    /// Builds the mesh. Returns the mesh if the definition is valid and otherwise an error.
    ///
    /// # Errors
    ///
    /// If no positions are specified, [NoPositionsSpecified](crate::mesh_builder::Error::NoPositionsSpecified) error is returned.
    ///
    pub fn build(self) -> Result<Mesh, Error>
    {
        let positions = self.positions.ok_or(
            Error::NoPositionsSpecified {message: format!("Did you forget to specify the vertex positions?")})?;
        let indices = self.indices.unwrap_or((0..positions.len() as u32/3).collect());
        Ok(Mesh::new(indices, positions))
    }

    ///
    /// Creates a cube.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::mesh_builder::{MeshBuilder, Error};
    /// #
    /// # fn main() -> Result<(), Box<Error>> {
    /// let mesh = MeshBuilder::new().cube().build()?;
    ///
    /// assert_eq!(mesh.no_faces(), 12);
    /// assert_eq!(mesh.no_vertices(), 8);
    ///
    /// #   mesh.is_valid().unwrap();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn cube(self) -> Self
    {
        self.with_positions(vec![
            1.0, -1.0, -1.0,
            1.0, -1.0, 1.0,
            -1.0, -1.0, 1.0,
            -1.0, -1.0, -1.0,
            1.0, 1.0, -1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0,
            -1.0, 1.0, -1.0
        ]).with_indices(vec![
            0, 1, 2,
            0, 2, 3,
            4, 7, 6,
            4, 6, 5,
            0, 4, 5,
            0, 5, 1,
            1, 5, 6,
            1, 6, 2,
            2, 6, 7,
            2, 7, 3,
            4, 0, 3,
            4, 3, 7]
        )
    }

    /// Creates a cube where each face is not connected to any other face.
    pub fn unconnected_cube(self) -> Self
    {
        self.with_positions(vec![
            1.0, 1.0, -1.0,
            -1.0, 1.0, -1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, -1.0,

            -1.0, -1.0, -1.0,
            1.0, -1.0, -1.0,
            1.0, -1.0, 1.0,
            1.0, -1.0, 1.0,
            -1.0, -1.0, 1.0,
            -1.0, -1.0, -1.0,

            1.0, -1.0, -1.0,
            -1.0, -1.0, -1.0,
            1.0, 1.0, -1.0,
            -1.0, 1.0, -1.0,
            1.0, 1.0, -1.0,
            -1.0, -1.0, -1.0,

            -1.0, -1.0, 1.0,
            1.0, -1.0, 1.0,
            1.0, 1.0, 1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0,

            1.0, -1.0, -1.0,
            1.0, 1.0, -1.0,
            1.0, 1.0, 1.0,
            1.0, 1.0, 1.0,
            1.0, -1.0, 1.0,
            1.0, -1.0, -1.0,

            -1.0, 1.0, -1.0,
            -1.0, -1.0, -1.0,
            -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0,
            -1.0, 1.0, 1.0,
            -1.0, -1.0, -1.0
        ])
    }

    /// Creates an icosahedron, i.e. a discretised sphere.
    pub fn icosahedron(self) -> Self
    {
        let x = 0.525731112119133606;
        let z = 0.850650808352039932;

        self.with_positions(vec!(
            -x, 0.0, z, x, 0.0, z, -x, 0.0, -z, x, 0.0, -z,
            0.0, z, x, 0.0, z, -x, 0.0, -z, x, 0.0, -z, -x,
            z, x, 0.0, -z, x, 0.0, z, -x, 0.0, -z, -x, 0.0
        )).with_indices(vec!(
            0, 1, 4, 0, 4, 9, 9, 4, 5, 4, 8, 5, 4, 1, 8,
            8, 1, 10, 8, 10, 3, 5, 8, 3, 5, 3, 2, 2, 3, 7,
            7, 3, 10, 7, 10, 6, 7, 6, 11, 11, 6, 0, 0, 6, 1,
            6, 10, 1, 9, 11, 0, 9, 2, 11, 9, 5, 2, 7, 11, 2
        ))
    }

    /// Creates a cylinder with the x-direction as axis, length 1 and radius 1.
    /// `x_subdivisions` defines the number of subdivisions in the x-direction
    /// and `angle_subdivisions` defines the number of circular subdivisions.
    pub fn cylinder(self, x_subdivisions: usize, angle_subdivisions: usize) -> Self
    {
        if x_subdivisions < 2 || angle_subdivisions < 3 { return self; }
        let mut positions = Vec::new();
        for i in 0..x_subdivisions + 1 {
            let x = i as f32 / x_subdivisions as f32;
            for j in 0..angle_subdivisions {
                let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

                positions.push(x);
                positions.push(angle.cos());
                positions.push(angle.sin());
            }
        }

        let mut indices = Vec::new();
        for i in 0..x_subdivisions as u32 {
            for j in 0..angle_subdivisions as u32 {
                indices.push(i * angle_subdivisions as u32 + j);
                indices.push(i * angle_subdivisions as u32 + (j + 1) % angle_subdivisions as u32);
                indices.push((i + 1) * angle_subdivisions as u32 + (j + 1) % angle_subdivisions as u32);

                indices.push(i * angle_subdivisions as u32 + j);
                indices.push((i + 1) * angle_subdivisions as u32 + (j + 1) % angle_subdivisions as u32);
                indices.push((i + 1) * angle_subdivisions as u32 + j);
            }
        }
        self.with_positions(positions).with_indices(indices)
    }

    /// Creates a triangle in `x = [-3, 3]`, `y = [-1, 2]` and `z = 0` which covers a square in `x = [-1, 1]`, `y = [-1, 1]` and `z = 0`.
    pub fn triangle(self) -> Self
    {
        self.with_positions(vec![-3.0, -1.0, 0.0,  3.0, -1.0, 0.0,  0.0, 2.0, 0.0])
    }

    /// Creates a square in `x = [-1, 1]`, `y = [-1, 1]` and `z = 0`.
    pub fn square(self) -> Self
    {
        self.with_indices(vec![0, 1, 2,  2, 1, 3])
            .with_positions(vec![-1.0, -1.0, 0.0,  1.0, -1.0, 0.0,  -1.0, 1.0, 0.0,  1.0, 1.0, 0.0])
    }

    /// Creates three connected triangles in `x = [-3, 3]`, `y = [-1, 2]` and `z = 0`
    /// which covers a square in `x = [-1, 1]`, `y = [-1, 1]` and `z = 0`
    /// and has a common vertex in `(0, 0, 0)`.
    pub fn subdivided_triangle(self) -> Self
    {
        self.with_indices(vec![0, 2, 3,  0, 3, 1,  0, 1, 2])
            .with_positions(vec![0.0, 0.0, 0.0,  -3.0, -1.0, 0.0,  3.0, -1.0, 0.0,  0.0, 2.0, 0.0])
    }
}
