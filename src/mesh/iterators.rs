//! Defines iterator types for easy iterating mesh primitives. See [Mesh](crate::mesh::Mesh) for more information.

use crate::mesh::connectivity_info::ConnectivityInfo;
use crate::mesh::*;

/// An iterator over the vertices. See [here](../struct.Mesh.html#method.vertex_iter) for more information.
pub type VertexIter = Box<dyn Iterator<Item = VertexID>>;

/// An iterator over the half-edges. See [here](../struct.Mesh.html#method.halfedge_iter) for more information.
pub type HalfEdgeIter = Box<dyn Iterator<Item = HalfEdgeID>>;

/// An iterator over the faces. See [here](../struct.Mesh.html#method.face_iter) for more information.
pub type FaceIter = Box<dyn Iterator<Item = FaceID>>;

/// An iterator over the half-edges starting in a given vertex. See [here](../struct.Mesh.html#method.vertex_halfedge_iter) for more information.
pub struct VertexHalfedgeIter<'a> {
    walker: Walker<'a>,
    start: HalfEdgeID,
    is_done: bool,
}

impl<'a> VertexHalfedgeIter<'a> {
    fn new(vertex_id: VertexID, connectivity_info: &'a ConnectivityInfo) -> VertexHalfedgeIter<'a> {
        let walker = Walker::new(connectivity_info).into_vertex_halfedge_walker(vertex_id);
        let start = walker.halfedge_id().unwrap();
        VertexHalfedgeIter {
            walker,
            start,
            is_done: false,
        }
    }
}

impl<'a> Iterator for VertexHalfedgeIter<'a> {
    type Item = HalfEdgeID;

    fn next(&mut self) -> Option<HalfEdgeID> {
        if self.is_done {
            return None;
        }
        let curr = self.walker.halfedge_id().unwrap();

        match self.walker.face_id() {
            Some(_) => {
                self.walker.as_previous().as_twin();
            }
            None => {
                // In the case there are holes in the one-ring
                self.walker.as_twin();
                while let Some(_) = self.walker.face_id() {
                    self.walker.as_next().as_twin();
                }
                self.walker.as_twin();
            }
        }
        self.is_done = self.walker.halfedge_id().unwrap() == self.start;
        Some(curr)
    }
}

/// An iterator over the three half-edges in a face. See [here](../struct.Mesh.html#method.face_halfedge_iter) for more information.
pub struct FaceHalfedgeIter<'a> {
    walker: Walker<'a>,
    count: usize,
}

impl<'a> FaceHalfedgeIter<'a> {
    fn new(face_id: FaceID, connectivity_info: &'a ConnectivityInfo) -> FaceHalfedgeIter<'a> {
        FaceHalfedgeIter {
            walker: Walker::new(connectivity_info).into_face_halfedge_walker(face_id),
            count: 0,
        }
    }
}

impl<'a> Iterator for FaceHalfedgeIter<'a> {
    type Item = HalfEdgeID;

    fn next(&mut self) -> Option<HalfEdgeID> {
        if self.count == 3 {
            return None;
        }
        self.walker.as_next();
        self.count += 1;
        Some(self.walker.halfedge_id().unwrap())
    }
}

/// An iterator over the edges. See [here](../struct.Mesh.html#method.edge_iter) for more information.
pub struct EdgeIter<'a> {
    walker: Walker<'a>,
    iter: HalfEdgeIter,
}

impl<'a> EdgeIter<'a> {
    fn new(connectivity_info: &'a ConnectivityInfo) -> EdgeIter<'a> {
        EdgeIter {
            walker: Walker::new(connectivity_info),
            iter: connectivity_info.halfedge_iterator(),
        }
    }
}

impl<'a> Iterator for EdgeIter<'a> {
    type Item = HalfEdgeID;

    fn next(&mut self) -> Option<HalfEdgeID> {
        if let Some(next_id) = self.iter.next() {
            if self.walker.as_halfedge_walker(next_id).twin_id().unwrap() < next_id {
                self.next()
            } else {
                Some(next_id)
            }
        } else {
            None
        }
    }
}

/// # Iterators
impl Mesh {
    ///
    /// Iterator over the vertex ids.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::*;
    /// # let mesh: Mesh = TriMesh::sphere(4).into();
    /// let mut sum_vertex_positions = Vec3::zero();
    /// for vertex_id in mesh.vertex_iter() {
    ///     sum_vertex_positions += mesh.vertex_position(vertex_id);
    /// }
    /// ```
    ///
    pub fn vertex_iter(&self) -> VertexIter {
        self.connectivity_info.vertex_iterator()
    }

    ///
    /// Iterator over the half-edge ids.
    ///
    /// **Note:** Each edge is visited two times, one for each half-edge. If you want to visit the edges only once, then use `edge_iter` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::*;
    /// # let mesh: Mesh = TriMesh::sphere(4).into();
    /// let mut halfedge_length_average = 0.0;
    /// let mut i = 0;
    /// for halfedge_id in mesh.halfedge_iter() {
    ///     halfedge_length_average += mesh.edge_length(halfedge_id);
    ///     i += 1;
    /// }
    /// halfedge_length_average /= i as f64;
    /// ```
    ///
    pub fn halfedge_iter(&self) -> HalfEdgeIter {
        self.connectivity_info.halfedge_iterator()
    }

    ///
    /// Iterator over the edges given as a half-edge id.
    ///
    /// **Note:** Each edge is visited once. If you want to visit both half-edges of an edge, then use `halfedge_iter` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::*;
    /// # let mesh: Mesh = TriMesh::sphere(4).into();
    /// let mut edge_length_average = 0.0;
    /// let mut i = 0;
    /// for halfedge_id in mesh.edge_iter() {
    ///     edge_length_average += mesh.edge_length(halfedge_id);
    ///     i += 1;
    /// }
    /// edge_length_average /= i as f64;
    /// ```
    ///
    pub fn edge_iter(&self) -> EdgeIter {
        EdgeIter::new(&self.connectivity_info)
    }

    ///
    /// Iterator over the face ids.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::*;
    /// # let mesh: Mesh = TriMesh::sphere(4).into();
    /// let mut sum_face_area = 0.0;
    /// for face_id in mesh.face_iter() {
    ///     sum_face_area += mesh.face_area(face_id);
    /// }
    /// ```
    ///
    pub fn face_iter(&self) -> FaceIter {
        self.connectivity_info.face_iterator()
    }

    ///
    /// Iterator over the half-edges which starts in the given vertex, ie. the one-ring.
    ///
    /// **Note:** If the given vertex is the only connection between two or more separate sets of faces,
    /// then this iterator will only iterate the half-edges in one of the sets.
    /// If the vertex is on the boundary, all half-edges are visited.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::*;
    /// # let mesh: Mesh = TriMesh::sphere(4).into();
    /// # let vertex_id = mesh.vertex_iter().next().unwrap();
    /// let mut one_ring_average_position = Vec3::zero();
    /// let mut i = 0;
    /// for halfedge_id in mesh.vertex_halfedge_iter(vertex_id) {
    ///     let walker = mesh.walker_from_halfedge(halfedge_id);
    ///     one_ring_average_position += mesh.vertex_position(walker.vertex_id().unwrap());
    ///     i = i+1;
    /// }
    /// one_ring_average_position /= i as f64;
    /// ```
    ///
    pub fn vertex_halfedge_iter(&self, vertex_id: VertexID) -> VertexHalfedgeIter {
        VertexHalfedgeIter::new(vertex_id, &self.connectivity_info)
    }

    ///
    /// Iterator over the three half-edges connected to the given face.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tri_mesh::*;
    /// # let mesh: Mesh = TriMesh::sphere(4).into();
    /// # let face_id = mesh.face_iter().next().unwrap();
    /// let mut face_circumference = 0.0f64;
    /// for halfedge_id in mesh.face_halfedge_iter(face_id) {
    ///     face_circumference += mesh.edge_length(halfedge_id);
    /// }
    /// ```
    ///
    pub fn face_halfedge_iter(&self, face_id: FaceID) -> FaceHalfedgeIter {
        FaceHalfedgeIter::new(face_id, &self.connectivity_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_iterator() {
        let mesh = crate::test_utility::subdivided_triangle();

        let mut i = 0;
        for _ in mesh.vertex_iter() {
            i = i + 1;
        }
        assert_eq!(4, i);

        // Test that two iterations return the same result
        let vec: Vec<VertexID> = mesh.vertex_iter().collect();
        i = 0;
        for vertex_id in mesh.vertex_iter() {
            assert_eq!(vertex_id, vec[i]);
            i = i + 1;
        }
    }

    #[test]
    fn test_halfedge_iterator() {
        let mesh = crate::test_utility::subdivided_triangle();

        let mut i = 0;
        for _ in mesh.halfedge_iter() {
            i = i + 1;
        }
        assert_eq!(12, i);

        // Test that two iterations return the same result
        let vec: Vec<HalfEdgeID> = mesh.halfedge_iter().collect();
        i = 0;
        for halfedge_id in mesh.halfedge_iter() {
            assert_eq!(halfedge_id, vec[i]);
            i = i + 1;
        }
    }

    #[test]
    fn test_edge_iterator() {
        let mesh = crate::test_utility::subdivided_triangle();

        let mut i = 0;
        for _ in mesh.edge_iter() {
            i = i + 1;
        }
        assert_eq!(6, i);

        // Test that two iterations return the same result
        let vec: Vec<HalfEdgeID> = mesh.edge_iter().collect();
        i = 0;
        for halfedge_id in mesh.edge_iter() {
            assert_eq!(halfedge_id, vec[i]);
            i = i + 1;
        }

        // Test that the twin is not returned
        for halfedge_id in mesh.edge_iter() {
            let twin_id = mesh.walker_from_halfedge(halfedge_id).twin_id().unwrap();
            assert!(halfedge_id < twin_id);
            assert!(vec.iter().find(|edge_id| *edge_id == &twin_id).is_none());
        }
    }

    #[test]
    fn test_face_iterator() {
        let mesh = crate::test_utility::subdivided_triangle();

        let mut i = 0;
        for _ in mesh.face_iter() {
            i = i + 1;
        }
        assert_eq!(3, i);

        // Test that two iterations return the same result
        let vec: Vec<FaceID> = mesh.face_iter().collect();
        i = 0;
        for face_id in mesh.face_iter() {
            assert_eq!(face_id, vec[i]);
            i = i + 1;
        }
    }

    #[test]
    fn test_vertex_halfedge_iterator() {
        let mesh = crate::test_utility::subdivided_triangle();

        let mut i = 0;
        let vertex_id = mesh.vertex_iter().last().unwrap();
        for halfedge_id in mesh.vertex_halfedge_iter(vertex_id) {
            assert!(mesh.walker_from_halfedge(halfedge_id).vertex_id().is_some());
            i = i + 1;
        }
        assert_eq!(i, 3, "All edges of a one-ring are not visited");
    }

    #[test]
    fn test_vertex_halfedge_iterator_with_holes() {
        let mesh: Mesh = TriMesh {
            indices: Indices::U8(vec![0, 2, 3, 0, 4, 1, 0, 1, 2]),
            positions: Positions::F64(vec![vec3(0.0, 0.0, 0.0); 5]),
            ..Default::default()
        }
        .into();

        let mut i = 0;
        for halfedge_id in mesh.vertex_halfedge_iter(unsafe { VertexID::new(0) }) {
            assert!(mesh.walker_from_halfedge(halfedge_id).vertex_id().is_some());
            i = i + 1;
        }
        assert_eq!(i, 4, "All edges of a one-ring are not visited");
    }

    #[test]
    fn test_face_halfedge_iterator() {
        let mesh = crate::test_utility::triangle();
        let mut i = 0;
        for halfedge_id in mesh.face_halfedge_iter(unsafe { FaceID::new(0) }) {
            let walker = mesh.walker_from_halfedge(halfedge_id);
            assert!(walker.halfedge_id().is_some());
            assert!(walker.face_id().is_some());
            i = i + 1;
        }
        assert_eq!(i, 3, "All edges of a face are not visited");
    }
}
