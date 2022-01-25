use crate::mesh::Mesh;
use crate::TriMeshResult;

impl Mesh {
    ///
    /// Parses the .obj file and extracts the connectivity information (indices) and positions which is used to construct a mesh when the `build` method is called.
    /// If the .obj file contains multiple objects, all objects are added to the mesh, but they will not be connected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use tri_mesh::*;
    /// # fn main() -> tri_mesh::TriMeshResult<()> {
    ///     let obj_source = std::fs::read_to_string("foo.obj").expect("Something went wrong reading the file");
    ///     let mesh = Mesh::from_obj(obj_source)?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn from_obj(source: String) -> TriMeshResult<Self> {
        Self::from_named_obj(source, "")
    }

    ///
    /// Parses the .obj file and extracts the connectivity information (indices) and positions which is used to construct a mesh when the `build` method is called.
    /// Only the object with the given name is extracted from the file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use tri_mesh::*;
    /// # fn main() -> tri_mesh::TriMeshResult<()> {
    ///     let obj_source = std::fs::read_to_string("foo.obj").expect("Something went wrong reading the file");
    ///     let mesh = Mesh::from_named_obj(obj_source, "my_object")?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn from_named_obj(source: String, object_name: &str) -> TriMeshResult<Self> {
        let objs = wavefront_obj::obj::parse(source).unwrap();
        let mut positions = Vec::new();
        let mut indices = Vec::new();

        for obj in objs.objects.iter() {
            // Objects consisting of several meshes with different materials
            if &obj.name == object_name || object_name == "" {
                let start_index = positions.len() / 3;
                obj.vertices.iter().for_each(|v| {
                    positions.push(v.x);
                    positions.push(v.y);
                    positions.push(v.z);
                });

                for mesh in obj.geometry.iter() {
                    // All meshes with different materials
                    for primitive in mesh.shapes.iter() {
                        // All triangles with same material
                        match primitive.primitive {
                            wavefront_obj::obj::Primitive::Triangle(i0, i1, i2) => {
                                indices.push((start_index + i0.0) as u32);
                                indices.push((start_index + i1.0) as u32);
                                indices.push((start_index + i2.0) as u32);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(Mesh::new(indices, positions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_obj() {
        let source = "o Cube
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
            .to_string();

        let mesh = Mesh::from_obj(source).unwrap();

        assert_eq!(mesh.no_faces(), 12);
        assert_eq!(mesh.no_vertices(), 8);

        mesh.is_valid().unwrap();
    }
}
