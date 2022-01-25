//! See [Mesh](crate::mesh::Mesh).

use crate::prelude::*;
use std::collections::{HashMap, HashSet};

/// # Quality
impl Mesh {
    /// Moves the vertices to `pos + factor * (avg_pos - pos)` where `pos` is the current position
    /// and `avg_pos` is the average position of the neighbouring vertices.
    pub fn smooth_vertices(&mut self, factor: f64) {
        let mut map = HashMap::new();
        for vertex_id in self.vertex_iter() {
            let mut avg_pos = vec3(0.0, 0.0, 0.0);
            let mut i = 0;
            for halfedge_id in self.vertex_halfedge_iter(vertex_id) {
                let vid = self.walker_from_halfedge(halfedge_id).vertex_id().unwrap();
                avg_pos = avg_pos + self.vertex_position(vid);
                i = i + 1;
            }
            avg_pos = avg_pos / i as f64;
            let p = self.vertex_position(vertex_id);
            map.insert(vertex_id, p + factor * (avg_pos - p));
        }

        for vertex_id in self.vertex_iter() {
            self.move_vertex_to(vertex_id, *map.get(&vertex_id).unwrap());
        }
    }

    /// Collapse an edge of faces which has an area smaller than `area_threshold`.
    pub fn collapse_small_faces(&mut self, area_threshold: f64) {
        let mut faces_to_test = HashSet::new();
        self.face_iter().for_each(|f| {
            faces_to_test.insert(f);
        });
        while !faces_to_test.is_empty() {
            let face_id = *faces_to_test.iter().next().unwrap();
            faces_to_test.remove(&face_id);
            if self.face_area(face_id) < area_threshold {
                let mut walker = self.walker_from_face(face_id);
                if let Some(twin_face_id) = walker.as_twin().face_id() {
                    faces_to_test.remove(&twin_face_id);
                }
                let twin_id = walker.twin_id().unwrap();
                self.collapse_edge(twin_id);
            }
        }
    }

    ///
    /// Flip all edges in the mesh
    /// * which is not on the boundary
    /// * where the flip will improve the sum of the quality of the two faces adjacent to the edge
    /// (The face quality is given as the circumscribed radius divided by the inscribed radius)
    /// * where the dot product between the normals of the adjacent faces is smaller than `flattness_threshold`
    /// (1: Completely flat, 0: 90 degrees angle between normals)
    /// * where the flip will not result in inverted triangles
    ///
    pub fn flip_edges(&mut self, flatness_threshold: f64) {
        let insert_or_remove =
            |mesh: &Mesh, to_be_flipped: &mut HashSet<HalfEdgeID>, halfedge_id: HalfEdgeID| {
                let twin_id = mesh.walker_from_halfedge(halfedge_id).twin_id().unwrap();
                let id = if halfedge_id < twin_id {
                    halfedge_id
                } else {
                    twin_id
                };
                if mesh.should_flip(id, flatness_threshold) {
                    to_be_flipped.insert(id);
                } else {
                    to_be_flipped.remove(&id);
                }
            };

        let mut to_be_flipped = HashSet::new();
        for halfedge_id in self.halfedge_iter() {
            insert_or_remove(&self, &mut to_be_flipped, halfedge_id);
        }

        while to_be_flipped.len() > 0 {
            let halfedge_id = *to_be_flipped.iter().next().unwrap();
            to_be_flipped.remove(&halfedge_id);

            if self.flip_edge(halfedge_id).is_ok() {
                let mut walker = self.walker_from_halfedge(halfedge_id);
                insert_or_remove(
                    &self,
                    &mut to_be_flipped,
                    walker.as_next().halfedge_id().unwrap(),
                );
                insert_or_remove(
                    &self,
                    &mut to_be_flipped,
                    walker.as_next().halfedge_id().unwrap(),
                );
                insert_or_remove(
                    &self,
                    &mut to_be_flipped,
                    walker.as_next().as_twin().as_next().halfedge_id().unwrap(),
                );
                insert_or_remove(
                    &self,
                    &mut to_be_flipped,
                    walker.as_next().halfedge_id().unwrap(),
                );
            }
        }
    }

    fn should_flip(&self, halfedge_id: HalfEdgeID, flatness_threshold: f64) -> bool {
        !self.is_edge_on_boundary(halfedge_id)
            && self.flatness(halfedge_id) > flatness_threshold
            && !self.flip_will_invert_triangle(halfedge_id)
            && self.flip_will_improve_quality(halfedge_id)
    }

    // 1 = Completely flat, 0 = 90 degrees angle between normals
    fn flatness(&self, haledge_id: HalfEdgeID) -> f64 {
        let mut walker = self.walker_from_halfedge(haledge_id);
        let face_id1 = walker.face_id().unwrap();
        let face_id2 = walker.as_twin().face_id().unwrap();
        self.face_normal(face_id1).dot(self.face_normal(face_id2))
    }

    fn flip_will_invert_triangle(&self, haledge_id: HalfEdgeID) -> bool {
        let mut walker = self.walker_from_halfedge(haledge_id);
        let p0 = self.vertex_position(walker.vertex_id().unwrap());
        let p2 = self.vertex_position(walker.as_next().vertex_id().unwrap());
        let p1 = self.vertex_position(walker.as_previous().as_twin().vertex_id().unwrap());
        let p3 = self.vertex_position(walker.as_next().vertex_id().unwrap());

        (p2 - p0).cross(p3 - p0).dot((p3 - p1).cross(p2 - p1)) < 0.0001
    }

    fn flip_will_improve_quality(&self, haledge_id: HalfEdgeID) -> bool {
        let mut walker = self.walker_from_halfedge(haledge_id);
        let p0 = self.vertex_position(walker.vertex_id().unwrap());
        let p2 = self.vertex_position(walker.as_next().vertex_id().unwrap());
        let p1 = self.vertex_position(walker.as_previous().as_twin().vertex_id().unwrap());
        let p3 = self.vertex_position(walker.as_next().vertex_id().unwrap());

        triangle_quality(&p0, &p2, &p1) + triangle_quality(&p0, &p1, &p3)
            > 1.1 * (triangle_quality(&p0, &p2, &p3) + triangle_quality(&p1, &p3, &p2))
    }
}

// Quality measure of 1 = good (equilateral) and >> 1 = bad (needle or flattened)
fn triangle_quality(p0: &Vec3, p1: &Vec3, p2: &Vec3) -> f64 {
    let length01 = (p0 - p1).magnitude();
    let length02 = (p0 - p2).magnitude();
    let length12 = (p1 - p2).magnitude();
    let perimiter = length01 + length02 + length12;
    let area = (p1 - p0).cross(p2 - p0).magnitude();
    let inscribed_radius = 2.0 * area / perimiter;
    let circumscribed_radius = length01 * length02 * length12 / (4.0 * area);
    circumscribed_radius / inscribed_radius
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MeshBuilder;

    #[test]
    fn test_collapse_small_faces() {
        let indices: Vec<u32> = vec![0, 2, 3, 0, 3, 1, 0, 1, 2];
        let positions: Vec<f64> = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.1, 0.1, 0.0, -0.1, -1.0, 0.0, -0.5,
        ];
        let mut mesh = Mesh::new(indices, positions);

        mesh.collapse_small_faces(0.2);
        mesh.is_valid().unwrap();
    }
}
