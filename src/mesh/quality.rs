
use crate::mesh::Mesh;
use crate::mesh::math::*;
use crate::mesh::ids::*;
use std::collections::{HashSet, HashMap};

/// # Quality functionality
impl Mesh
{
    pub fn smooth_vertices(&mut self, factor: f32)
    {
        let mut map = HashMap::new();
        for vertex_id in self.vertex_iter() {
            let mut avg_pos = vec3(0.0, 0.0, 0.0);
            let mut i = 0;
            for walker in self.vertex_halfedge_iter(&vertex_id) {
                avg_pos = avg_pos + *self.vertex_position(&walker.vertex_id().unwrap());
                i = i + 1;
            }
            avg_pos = avg_pos / i as f32;
            let p = self.vertex_position(&vertex_id);
            map.insert(vertex_id, p + factor * (avg_pos - p));
        }

        for vertex_id in self.vertex_iter() {
            self.move_vertex_to(vertex_id, *map.get(&vertex_id).unwrap());
        }
    }

    pub fn collapse_small_faces(&mut self, area_threshold: f32)
    {
        let mut faces_to_test = HashSet::new();
        self.face_iter().for_each(|f| { faces_to_test.insert(f); } );
        while !faces_to_test.is_empty() {
            let face_id = *faces_to_test.iter().next().unwrap();
            faces_to_test.remove(&face_id);
            if self.face_area(&face_id) < area_threshold {
                let mut walker = self.walker_from_face(&face_id);
                if let Some(twin_face_id) = walker.as_twin().face_id() { faces_to_test.remove(&twin_face_id); }
                self.collapse_edge(&walker.twin_id().unwrap());
            }
        }
    }

    pub fn remove_lonely_primitives(&mut self)
    {
        for (halfedge_id, _) in self.halfedge_twins_iter() {
            self.remove_edge_if_lonely(&halfedge_id);
        }

        for vertex_id in self.vertex_iter() {
            self.remove_vertex_if_lonely(&vertex_id);
        }
    }

    pub fn flip_edges(&mut self, flatness_threshold: f32)
    {
        let insert_or_remove = |mesh: &Mesh, to_be_flipped: &mut HashSet<HalfEdgeID>, halfedge_id: HalfEdgeID| {
            let twin_id = mesh.walker_from_halfedge(&halfedge_id).twin_id().unwrap();
            let id = if halfedge_id < twin_id {halfedge_id} else {twin_id};
            if mesh.should_flip(&id, flatness_threshold) { to_be_flipped.insert(id); } else { to_be_flipped.remove(&id); }
        };

        let mut to_be_flipped = HashSet::new();
        for halfedge_id in self.halfedge_iter()
        {
            insert_or_remove(&self,&mut to_be_flipped, halfedge_id);
        }

        while to_be_flipped.len() > 0
        {
            let halfedge_id = to_be_flipped.iter().next().unwrap().clone();
            to_be_flipped.remove(&halfedge_id);

            if self.flip_edge(&halfedge_id).is_ok() {
                let mut walker = self.walker_from_halfedge(&halfedge_id);
                insert_or_remove(&self,&mut to_be_flipped, walker.as_next().halfedge_id().unwrap());
                insert_or_remove(&self,&mut to_be_flipped, walker.as_next().halfedge_id().unwrap());
                insert_or_remove(&self,&mut to_be_flipped, walker.as_next().as_twin().as_next().halfedge_id().unwrap());
                insert_or_remove(&self,&mut to_be_flipped, walker.as_next().halfedge_id().unwrap());
            }
        }
    }

    fn should_flip(&self, halfedge_id: &HalfEdgeID, flatness_threshold: f32) -> bool
    {
        !self.on_boundary(halfedge_id)
            && self.flatness(halfedge_id) > flatness_threshold
            && !self.flip_will_invert_triangle(halfedge_id)
            && self.flip_will_improve_quality(halfedge_id)
    }

    // 1 = Completely flat, 0 = 90 degrees angle between normals
    fn flatness(&self, haledge_id: &HalfEdgeID) -> f32
    {
        let mut walker = self.walker_from_halfedge(haledge_id);
        let face_id1 = walker.face_id().unwrap();
        let face_id2 = walker.as_twin().face_id().unwrap();
        self.face_normal(&face_id1).dot(self.face_normal(&face_id2))
    }

    fn flip_will_invert_triangle(&self, haledge_id: &HalfEdgeID) -> bool
    {
        let mut walker = self.walker_from_halfedge(haledge_id);
        let p0 = self.vertex_position(&walker.vertex_id().unwrap());
        let p2 = self.vertex_position(&walker.as_next().vertex_id().unwrap());
        let p1 = self.vertex_position(&walker.as_previous().as_twin().vertex_id().unwrap());
        let p3 = self.vertex_position(&walker.as_next().vertex_id().unwrap());

        (p2 - p0).cross(p3 - p0).dot((p3 - p1).cross(p2 - p1)) < 0.0001
    }

    fn flip_will_improve_quality(&self, haledge_id: &HalfEdgeID) -> bool
    {
        let mut walker = self.walker_from_halfedge(haledge_id);
        let p0 = self.vertex_position(&walker.vertex_id().unwrap());
        let p2 = self.vertex_position(&walker.as_next().vertex_id().unwrap());
        let p1 = self.vertex_position(&walker.as_previous().as_twin().vertex_id().unwrap());
        let p3 = self.vertex_position(&walker.as_next().vertex_id().unwrap());

        triangle_quality(p0, p2, p1) + triangle_quality(p0, p1, p3) >
            1.1 * (triangle_quality(p0, p2, p3) + triangle_quality(p1, p3, p2))
    }
}

// Quality measure of 1 = good (equilateral) and >> 1 = bad (needle or flattened)
fn triangle_quality(p0: &Vec3, p1: &Vec3, p2: &Vec3) -> f32
{
    let length01 = (p0-p1).magnitude();
    let length02 = (p0-p2).magnitude();
    let length12 = (p1-p2).magnitude();
    let perimiter = length01 + length02 + length12;
    let area = (p1-p0).cross(p2-p0).magnitude();
    let inscribed_radius = 2.0 * area / perimiter;
    let circumscribed_radius = length01 * length02 * length12 / (4.0 * area);
    circumscribed_radius / inscribed_radius
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utility;

    #[test]
    fn test_collapse_small_faces()
    {
        let indices: Vec<u32> = vec![0, 2, 3,  0, 3, 1,  0, 1, 2];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 0.1,  0.1, 0.0, -0.1,  -1.0, 0.0, -0.5];
        let mut mesh = Mesh::new(indices, positions);

        mesh.collapse_small_faces(0.2);
        test_utility::test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_remove_lonely_vertices()
    {
        let mut mesh = test_utility::create_three_connected_faces();
        let mut iter = mesh.face_iter();
        let face_id1 = iter.next().unwrap();
        let face_id2 = iter.next().unwrap();
        mesh.remove_face_unsafe(&face_id1);
        mesh.remove_face_unsafe(&face_id2);

        mesh.remove_lonely_primitives();

        assert_eq!(3, mesh.no_vertices());
        assert_eq!(6, mesh.no_halfedges());
        assert_eq!(1, mesh.no_faces());
        test_utility::test_is_valid(&mesh).unwrap();
    }
}