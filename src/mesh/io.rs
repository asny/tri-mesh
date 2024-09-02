//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;

impl Mesh {
    ///
    /// Constructs a new [Mesh] from a [three_d_asset::TriMesh] which can either be manually constructed or loaded via the [three_d_asset::io] module.
    ///
    /// # Examples
    /// ```no_run
    /// # use tri_mesh::*;
    /// let model: three_d_asset::Model =
    ///     three_d_asset::io::load_and_deserialize("cube.obj").expect("Failed loading asset");
    /// let mesh = match &model.geometries[0].geometry {
    ///     three_d_asset::Geometry::Triangles(mesh) => Mesh::new(mesh),
    ///     _ => panic!("Geometry is not a triangle mesh")
    /// };
    /// ```
    ///
    /// ```
    /// # use tri_mesh::*;
    /// let mesh = Mesh::new(&three_d_asset::TriMesh::sphere(4));
    /// ```
    ///
    /// ```
    /// # use tri_mesh::*;
    /// let mesh = Mesh::new(&three_d_asset::TriMesh {
    ///     positions: three_d_asset::Positions::F64(vec![vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0)]),
    ///     ..Default::default()
    /// });
    /// ```
    ///
    pub fn new(input: &three_d_asset::TriMesh) -> Self {
        let no_vertices = input.vertex_count();
        let no_faces = input.triangle_count();
        let indices = input
            .indices
            .to_u32()
            .unwrap_or((0..no_faces as u32 * 3).collect::<Vec<_>>());
        let positions = input.positions.to_f64();
        let mesh = Mesh {
            connectivity_info: ConnectivityInfo::new(no_vertices, no_faces),
        };

        // Create vertices
        for i in 0..no_vertices {
            mesh.connectivity_info.new_vertex(positions[i]);
        }

        let mut twins = HashMap::<(VertexID, VertexID), HalfEdgeID>::new();
        fn sort(a: VertexID, b: VertexID) -> (VertexID, VertexID) {
            if a < b {
                (a, b)
            } else {
                (b, a)
            }
        }

        // Create faces and twin connectivity
        for face in 0..no_faces {
            let v0 = indices[face * 3];
            let v1 = indices[face * 3 + 1];
            let v2 = indices[face * 3 + 2];

            let face = mesh.connectivity_info.create_face(
                unsafe { VertexID::new(v0) },
                unsafe { VertexID::new(v1) },
                unsafe { VertexID::new(v2) },
            );

            // mark twin halfedges
            let mut walker = mesh.walker_from_face(face);
            for _ in 0..3 {
                let vertex_id = walker.vertex_id().unwrap();
                walker.as_next();
                let key = sort(vertex_id, walker.vertex_id().unwrap());
                if let Some(twin) = twins.get(&key) {
                    mesh.connectivity_info
                        .set_halfedge_twin(walker.halfedge_id().unwrap(), *twin);
                } else {
                    twins.insert(key, walker.halfedge_id().unwrap());
                }
            }
        }
        for halfedge in mesh.connectivity_info.halfedge_iterator() {
            if mesh
                .connectivity_info
                .halfedge(halfedge)
                .unwrap()
                .twin
                .is_none()
            {
                let vertex = mesh
                    .walker_from_halfedge(halfedge)
                    .as_previous()
                    .vertex_id()
                    .unwrap();
                mesh.connectivity_info.set_halfedge_twin(
                    halfedge,
                    mesh.connectivity_info
                        .new_halfedge(Some(vertex), None, None),
                );
            }
        }

        mesh
    }

    ///
    /// Exports the [Mesh] into a [three_d_asset::TriMesh] that contain the raw buffer data.
    /// The [three_d_asset::TriMesh] can then for example be visualized or saved to disk (using the [three_d_asset::io] module).
    ///
    pub fn export(&self) -> three_d_asset::TriMesh {
        use three_d_asset::{Indices, Positions, TriMesh};
        let vertices: Vec<VertexID> = self.vertex_iter().collect();
        let mut indices = Vec::with_capacity(self.no_faces() * 3);
        for face_id in self.face_iter() {
            for halfedge_id in self.face_halfedge_iter(face_id) {
                let vertex_id = self.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                let index = vertices.iter().position(|v| v == &vertex_id).unwrap();
                indices.push(index as u32);
            }
        }
        TriMesh {
            indices: Indices::U32(indices),
            positions: Positions::F64(
                self.vertex_iter()
                    .map(|vertex_id| self.vertex_position(vertex_id))
                    .collect::<Vec<_>>(),
            ),
            normals: Some(
                self.vertex_iter()
                    .map(|vertex_id| self.vertex_normal(vertex_id).cast::<f32>().unwrap())
                    .collect::<Vec<_>>(),
            ),
            ..Default::default()
        }
    }
}

impl From<three_d_asset::TriMesh> for Mesh {
    fn from(mesh: three_d_asset::TriMesh) -> Self {
        Self::new(&mesh)
    }
}

impl From<&three_d_asset::TriMesh> for Mesh {
    fn from(mesh: &three_d_asset::TriMesh) -> Self {
        Self::new(mesh)
    }
}

impl From<Mesh> for three_d_asset::TriMesh {
    fn from(mesh: Mesh) -> Self {
        mesh.export()
    }
}

impl From<&Mesh> for three_d_asset::TriMesh {
    fn from(mesh: &Mesh) -> Self {
        mesh.export()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use three_d_asset::{Positions, TriMesh};

    #[test]
    fn test_from_obj() {
        let source = b"o Cube
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
            .to_vec();
        let mut raw_assets = three_d_asset::io::RawAssets::new();
        raw_assets.insert("cube.obj", source);
        let mut model: three_d_asset::Model = raw_assets.deserialize(".obj").unwrap();
        let three_d_asset::Geometry::Triangles(m) = model.geometries.remove(0).geometry else {
            unreachable!()
        };
        let mesh: Mesh = m.into();
        assert_eq!(mesh.no_faces(), 12);
        assert_eq!(mesh.no_vertices(), 8);
        mesh.is_valid().unwrap();
    }

    #[test]
    fn test_indexed_export() {
        let mesh: Mesh = TriMesh::cylinder(16).into();
        let m: TriMesh = (&mesh).into();
        m.validate().unwrap();

        assert_eq!(m.triangle_count(), mesh.no_faces());
        assert_eq!(m.vertex_count(), mesh.no_vertices());

        let positions = m.positions.to_f64();
        let normals = m.normals.as_ref().unwrap();
        m.for_each_triangle(|i0, i1, i2| {
            let id0 = unsafe { VertexID::new(i0 as u32) };
            let id1 = unsafe { VertexID::new(i1 as u32) };
            let id2 = unsafe { VertexID::new(i2 as u32) };
            assert!(positions[i0].distance(mesh.vertex_position(id0)) < 0.001);
            assert!(positions[i1].distance(mesh.vertex_position(id1)) < 0.001);
            assert!(positions[i2].distance(mesh.vertex_position(id2)) < 0.001);
            assert!(normals[i0].distance(mesh.vertex_normal(id0).cast::<f32>().unwrap()) < 0.001);
            assert!(normals[i1].distance(mesh.vertex_normal(id1).cast::<f32>().unwrap()) < 0.001);
            assert!(normals[i2].distance(mesh.vertex_normal(id2).cast::<f32>().unwrap()) < 0.001);
        });
    }

    #[test]
    fn test_new_from_positions() {
        let mesh: Mesh = TriMesh {
            positions: Positions::F64(vec![
                vec3(0.0, 0.0, 0.0),
                vec3(1.0, 0.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 1.0),
                vec3(1.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(1.0, 0.0, 1.0),
                vec3(0.0, 1.0, 1.0),
            ]),
            ..Default::default()
        }
        .into();

        assert_eq!(9, mesh.no_vertices());
        assert_eq!(3, mesh.no_faces());
        mesh.is_valid().unwrap();
    }
}
