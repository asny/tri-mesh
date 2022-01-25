use crate::mesh::Mesh;
use crate::TriMeshResult;

///
/// Parses the .obj file and returns a [Mesh].
/// If the .obj file contains multiple objects, all objects are added to the mesh, but they will not be connected.
///
/// # Examples
///
/// ```no_run
/// # use tri_mesh::*;
/// # fn main() -> tri_mesh::TriMeshResult<()> {
///     let obj_source = std::fs::read_to_string("foo.obj").expect("Something went wrong reading the file");
///     let mesh = parse_from_obj(obj_source)?;
/// #    Ok(())
/// # }
/// ```
pub fn parse_from_obj(source: String) -> TriMeshResult<Mesh> {
    parse_from_named_obj(source, "")
}

///
/// Parses the .obj file and returns a [Mesh].
/// Only the object with the given name is extracted from the file.
///
/// # Examples
///
/// ```no_run
/// # use tri_mesh::*;
/// # fn main() -> tri_mesh::TriMeshResult<()> {
///     let obj_source = std::fs::read_to_string("foo.obj").expect("Something went wrong reading the file");
///     let mesh = parse_from_named_obj(obj_source, "my_object")?;
/// #    Ok(())
/// # }
/// ```
pub fn parse_from_named_obj(source: String, object_name: &str) -> TriMeshResult<Mesh> {
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

///
/// Parses the [Mesh] into a text string that follows the .obj file format and which can then be saved into a file.
///
/// # Examples
///
/// ```no_run
/// # use tri_mesh::*;
/// # fn main() -> tri_mesh::TriMeshResult<()> {
/// # let mesh = MeshBuilder::new().cube().build()?;
/// // Write the mesh data to a string
/// let obj_source = parse_to_obj(&mesh);
///
/// // Write the string to an .obj file
/// std::fs::write("foo.obj", obj_source)?;
/// # Ok(())
/// # }
/// ```
pub fn parse_to_obj(mesh: &Mesh) -> String {
    let mut output = String::from("o object\n");

    let positions = mesh.positions_buffer();
    for i in 0..mesh.no_vertices() {
        output = format!(
            "{}v {} {} {}\n",
            output,
            positions[i * 3],
            positions[i * 3 + 1],
            positions[i * 3 + 2]
        );
    }

    let normals = mesh.normals_buffer();
    for i in 0..mesh.no_vertices() {
        output = format!(
            "{}vn {} {} {}\n",
            output,
            normals[i * 3],
            normals[i * 3 + 1],
            normals[i * 3 + 2]
        );
    }

    let indices = mesh.indices_buffer();
    for i in 0..mesh.no_faces() {
        let mut face = String::new();
        for j in 0..3 {
            let index = indices[i * 3 + j] + 1;
            face = format!("{} {}//{}", face, index, index);
        }
        output = format!("{}f{}\n", output, face);
    }
    output
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

        let mesh = parse_from_obj(source).unwrap();

        assert_eq!(mesh.no_faces(), 12);
        assert_eq!(mesh.no_vertices(), 8);

        mesh.is_valid().unwrap();
    }
}
