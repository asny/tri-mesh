
use crate::prelude::*;
use crate::mesh::collision::utility::*;

///
/// An enum describing the types of primitives.
///
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Primitive {
    /// Vertex
    Vertex(VertexID),
    /// Edge
    Edge((VertexID, VertexID)),
    /// Face
    Face(FaceID)
}

///
/// An enum describing the types of intersections.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Intersection
{
    /// The intersection occurs at a single point
    Point {
        /// The primitive, vertex, edge or face, that is intersected
        primitive: Primitive,
        /// The point where the intersection occurs
        point: Vec3
    },
    /// The intersection occurs at a line piece interval
    LinePiece {
        /// The primitive, vertex, edge or face, that is intersected
        primitive: Primitive,
        /// The first point of the line piece where the intersection occurs
        point0: Vec3,
        /// The second point of the line piece where the intersection occurs
        point1: Vec3
    }
}

/// # Intersection
impl Mesh
{
    pub fn ray_intersection(&self, ray_start_point: &Vec3, ray_direction: &Vec3) -> Option<Intersection>
    {
        let mut current: Option<Intersection> = None;
        for face_id in self.face_iter() {
            let new_intersection = self.face_ray_intersection(face_id, ray_start_point, ray_direction);
            if let Some(Intersection::Point { point, ..}) = new_intersection
            {
                let new_point = point;
                if let Some(Intersection::Point {point, ..}) = current {
                    if point.distance2(*ray_start_point) > new_point.distance2(*ray_start_point) {
                        current = new_intersection;
                    }
                }
                else {current = new_intersection;}
            }
        }
        current
    }

    pub fn face_edge_intersection(&self, face_id: FaceID, other: &Mesh, edge: (VertexID, VertexID)) -> Option<(Intersection, Intersection)>
    {
        let p0 = other.vertex_position(edge.0);
        let p1 = other.vertex_position(edge.1);

        self.face_line_piece_intersection(face_id, &p0, &p1)
            .map(|intersection| {
                match intersection {
                    Intersection::Point {point, ..} => {
                        (intersection, other.edge_point_intersection(edge, &point).unwrap())
                    },
                    Intersection::LinePiece {..} => {
                        (intersection, Intersection::LinePiece {primitive: Primitive::Edge(edge), point0: *p0, point1: *p1})
                    }
                }
            })
    }

    pub fn face_ray_intersection(&self, face_id: FaceID, ray_start_point: &Vec3, ray_direction: &Vec3) -> Option<Intersection>
    {
        let p = self.vertex_position(self.walker_from_face(face_id).vertex_id().unwrap());
        let n = self.face_normal(face_id);

        plane_ray_intersection(ray_start_point, ray_direction, &p, &n)
            .and_then(|parameter| {
                self.face_point_intersection_when_point_in_plane(face_id, &(ray_start_point + parameter * ray_direction))
            })
    }

    pub fn face_line_piece_intersection(&self, face_id: FaceID, point0: &Vec3, point1: &Vec3) -> Option<Intersection>
    {
        let p = self.vertex_position(self.walker_from_face(face_id).vertex_id().unwrap());
        let n = self.face_normal(face_id);

        plane_line_piece_intersection(&point0, &point1, p, &n).and_then(|intersection| {
            match intersection {
                PlaneLinepieceIntersectionResult::LineInPlane => {
                    let intersection0 = self.face_point_intersection_when_point_in_plane(face_id, point0);
                    let intersection1 = self.face_point_intersection_when_point_in_plane(face_id, point1);
                    if let Some(Intersection::Point {point, ..}) = intersection0 {
                        let point0 = point;
                        if let Some(Intersection::Point {point, ..}) = intersection1 {
                            Some(Intersection::LinePiece {primitive: Primitive::Face(face_id), point0, point1: point})
                        }
                        else {
                            intersection0
                        }
                    }
                    else {
                        intersection1
                    }
                },
                PlaneLinepieceIntersectionResult::P0InPlane => {
                    self.face_point_intersection_when_point_in_plane(face_id, point0)
                },
                PlaneLinepieceIntersectionResult::P1InPlane => {
                    self.face_point_intersection_when_point_in_plane(face_id, point1)
                },
                PlaneLinepieceIntersectionResult::Intersection(point) => {
                    self.face_point_intersection_when_point_in_plane(face_id, &point)
                }
            }
        })
    }

    /// Returns the vertex primitive if the point is close, otherwise None.
    pub fn vertex_point_intersection(&self, vertex_id: VertexID, point: &Vec3) -> Option<Intersection>
    {
        let p = self.vertex_position(vertex_id);
        if (p - point).magnitude2() < SQR_MARGIN
        {
            Some(Intersection::Point {primitive: Primitive::Vertex(vertex_id), point: *point})
        }
        else {
            None
        }
    }

    // Returns the vertex or edge primitive (tests are made in that order) if the point is close to either, otherwise None.
    pub fn edge_point_intersection(&self, edge: (VertexID, VertexID), point: &Vec3) -> Option<Intersection>
    {
        self.vertex_point_intersection(edge.0, point)
            .or_else(|| { self.vertex_point_intersection(edge.1, point) })
            .or_else(|| {
                if point_line_segment_distance(point, self.vertex_position(edge.0), self.vertex_position(edge.1)) < MARGIN
                {
                    Some(Intersection::Point {primitive: Primitive::Edge(edge), point: *point})
                }
                else { None }
            })
    }

    // Returns the vertex, edge or face primitive (tests are made in that order) if the point is close to either, otherwise None.
    pub fn face_point_intersection(&self, face_id: FaceID, point: &Vec3) -> Option<Intersection>
    {
        let p = *self.vertex_position(self.walker_from_face(face_id).vertex_id().unwrap());
        let n = self.face_normal(face_id);
        let v = (point - p).normalize();
        if n.dot(v).abs() > MARGIN { return None; }

        self.face_point_intersection_when_point_in_plane(face_id, point)
    }

    // Assumes that the point lies in the plane spanned by the face
    fn face_point_intersection_when_point_in_plane(&self, face_id: FaceID, point: &Vec3) -> Option<Intersection>
    {
        let face_vertices = self.ordered_face_vertices(face_id);
        let v0 = face_vertices.0;
        let v1 = face_vertices.1;
        let v2 = face_vertices.2;

        let a = self.vertex_position(v0);
        let b = self.vertex_position(v1);
        let c = self.vertex_position(v2);

        // Test if the point is located at one of the vertices
        if (*a - *point).magnitude2() < SQR_MARGIN { return Some(Intersection::Point {primitive: Primitive::Vertex(v0), point: *point}); }
        if (*b - *point).magnitude2() < SQR_MARGIN { return Some(Intersection::Point {primitive: Primitive::Vertex(v1), point: *point}); }
        if (*c - *point).magnitude2() < SQR_MARGIN { return Some(Intersection::Point {primitive: Primitive::Vertex(v2), point: *point}); }

        // Test if the point is located at one of the edges
        if point_line_segment_distance(point, a, b) < MARGIN { return Some(Intersection::Point {primitive: Primitive::Edge((v0, v1)), point: *point}); }
        if point_line_segment_distance(point, b, c) < MARGIN { return Some(Intersection::Point {primitive: Primitive::Edge((v1, v2)), point: *point}); }
        if point_line_segment_distance(point, a, c) < MARGIN { return Some(Intersection::Point {primitive: Primitive::Edge((v0, v2)), point: *point}); }

        // Test whether the intersection point is located inside the face
        let coords = barycentric(point, a, b, c);
        if 0.0 < coords.0 && coords.0 < 1.0 && 0.0 < coords.1 && coords.1 < 1.0 && 0.0 < coords.2 && coords.2 < 1.0
        {
            return Some(Intersection::Point {primitive: Primitive::Face(face_id), point: *point});
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_point_intersection_when_point_in_plane()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
        let mut mesh = MeshBuilder::new().with_positions(positions).build().unwrap();
        mesh.scale(3.0);
        let face_id = mesh.face_iter().next().unwrap();
        let (v0, v1, v2) = mesh.ordered_face_vertices(face_id);
        let p0 = mesh.vertex_position(v0);
        let p1 = mesh.vertex_position(v1);
        let p2 = mesh.vertex_position(v2);

        let edge_midpoint = (p1 + p2) * 0.5;
        let face_midpoint = mesh.face_center(face_id);

        // Vertex intersection
        let mut result = mesh.face_point_intersection_when_point_in_plane(face_id, p0);
        assert_eq!(result, Some(Intersection::Point { primitive: Primitive::Vertex(v0), point: *p0 }));

        let dir_away_from_p0 = -(0.5 * (p1 + p2) - p0).normalize();
        let p_intersect = p0 + 0.99 * MARGIN * dir_away_from_p0;
        result = mesh.face_point_intersection_when_point_in_plane(face_id, &p_intersect);
        assert_eq!(result, Some(Intersection::Point { primitive: Primitive::Vertex(v0), point: p_intersect }));

        result = mesh.face_point_intersection_when_point_in_plane(face_id, &(p0 + 1.01 * MARGIN * dir_away_from_p0));
        assert_eq!(result, None);

        // Edge intersection
        result = mesh.face_point_intersection_when_point_in_plane(face_id, &edge_midpoint);
        assert_eq!(result, Some(Intersection::Point { primitive: Primitive::Edge((v1, v2)), point: edge_midpoint }));

        let dir_away_from_edge = vec3(0.0, 1.0, 0.0);
        let p_intersect = edge_midpoint + 0.99 * MARGIN * dir_away_from_edge;
        result = mesh.face_point_intersection_when_point_in_plane(face_id, &p_intersect);
        assert_eq!(result, Some(Intersection::Point { primitive: Primitive::Edge((v1, v2)), point: p_intersect }));

        result = mesh.face_point_intersection_when_point_in_plane(face_id, &(edge_midpoint + 1.01 * MARGIN * dir_away_from_edge));
        assert_eq!(result, None);

        // Face intersection
        result = mesh.face_point_intersection_when_point_in_plane(face_id, &face_midpoint);
        assert_eq!(result, Some(Intersection::Point { primitive: Primitive::Face(face_id), point: face_midpoint }));
    }

    #[test]
    fn test_edge_point_intersection()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
        let mut mesh = MeshBuilder::new().with_positions(positions).build().unwrap();
        mesh.scale(3.0);
        let edge_id = mesh.halfedge_iter().next().unwrap();
        let (v0, v1) = mesh.ordered_edge_vertices(edge_id);
        let p0 = mesh.vertex_position(v0);
        let p1 = mesh.vertex_position(v1);

        // Vertex intersection
        let mut result = mesh.edge_point_intersection((v0, v1), p0);
        assert_eq!(result, Some(Intersection::Point { primitive: Primitive::Vertex(v0), point: *p0 }));

        let dir_away_from_p0 = -(p1 - p0).normalize();
        let p_intersect = p0 + 0.99 * MARGIN * dir_away_from_p0;
        result = mesh.edge_point_intersection((v0, v1), &p_intersect);
        assert_eq!(result, Some(Intersection::Point { primitive: Primitive::Vertex(v0), point: p_intersect }));

        result = mesh.edge_point_intersection((v0, v1), &(p0 + 1.01 * MARGIN * dir_away_from_p0));
        assert_eq!(result, None);

        // Edge intersection
        let edge_midpoint = (p0 + p1) * 0.5;
        result = mesh.edge_point_intersection((v0, v1), &edge_midpoint);
        assert_eq!(result, Some(Intersection::Point { primitive: Primitive::Edge((v0, v1)), point: edge_midpoint }));

        let dir_away_from_edge = dir_away_from_p0.cross(vec3(1.0, 1.0, 1.0)).normalize();
        let p_intersect = edge_midpoint + 0.99 * MARGIN * dir_away_from_edge;
        result = mesh.edge_point_intersection((v0, v1), &p_intersect);
        assert_eq!(result, Some(Intersection::Point { primitive: Primitive::Edge((v0, v1)), point: p_intersect }));

        result = mesh.edge_point_intersection((v0, v1), &(edge_midpoint + 1.01 * MARGIN * dir_away_from_edge));
        assert_eq!(result, None);
    }

    #[test]
    fn test_face_edge_intersection_when_no_intersection()
    {
        let mesh1 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]).build().unwrap();
        let mesh2 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![1.0 + MARGIN, 0.0, 0.0, 3.0, 0.0, 1.0, 4.0, 0.0, 0.0]).build().unwrap();
        let face_id = mesh1.face_iter().next().unwrap();
        let edge_id = mesh2.ordered_edge_vertices(mesh2.edge_iter().next().unwrap());

        let result = mesh1.face_edge_intersection(face_id, &mesh2, edge_id);
        assert_eq!(result, None);
    }

    #[test]
    fn test_face_edge_intersection_when_face_vertex_intersects()
    {
        let mesh1 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]).build().unwrap();
        let mesh2 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 1.0, 0.0, 0.1, 0.0, 0.1, 1.0, 1.0, 0.0]).build().unwrap();
        let point = vec3(0.1, 0.0, 0.1);
        let face_id = mesh1.face_iter().next().unwrap();
        let edge_id = mesh2.edge_iter().map(|halfedge_id| mesh2.ordered_edge_vertices(halfedge_id)).find(|(v1, _)| *mesh2.vertex_position(*v1) == point).unwrap();

        let result = mesh1.face_edge_intersection(face_id, &mesh2, edge_id);
        let vertex_id = mesh2.vertex_iter().find(|v| *mesh2.vertex_position(*v) == point).unwrap();
        assert_eq!(result, Some((Intersection::Point { primitive: Primitive::Face(face_id), point }, Intersection::Point {primitive: Primitive::Vertex(vertex_id), point})));
    }

    #[test]
    fn test_face_edge_intersection_when_face_edge_intersects_at_point()
    {
        let mesh1 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]).build().unwrap();
        let mesh2 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.1, 1.0, 0.1, 0.1, -0.1, 0.1, 1.0, 1.0, 0.0]).build().unwrap();

        let face_id = mesh1.face_iter().next().unwrap();
        let edge_id = mesh2.edge_iter().map(|halfedge_id| mesh2.ordered_edge_vertices(halfedge_id)).find(|(v1, v2)| mesh2.vertex_position(*v1)[0] == 0.1 && mesh2.vertex_position(*v2)[0] == 0.1).unwrap();

        let result = mesh1.face_edge_intersection(face_id, &mesh2, edge_id);
        let point = vec3(0.1, 0.0, 0.1);
        assert_eq!(result, Some((Intersection::Point { primitive: Primitive::Face(face_id), point }, Intersection::Point { primitive: Primitive::Edge(edge_id), point})));
    }

    #[test]
    fn test_face_edge_intersection_when_face_edge_intersects_at_linepiece()
    {
        let mesh1 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]).build().unwrap();
        let mesh2 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.1, 0.0, 0.1, 0.2, 0.0, 0.2, 1.0, 1.0, 0.0]).build().unwrap();

        let face_id = mesh1.face_iter().next().unwrap();
        let edge_id = mesh2.edge_iter().map(|halfedge_id| mesh2.ordered_edge_vertices(halfedge_id)).find(|(v1, v2)| mesh2.vertex_position(*v1)[1] == 0.0 && mesh2.vertex_position(*v2)[1] == 0.0).unwrap();

        let result = mesh1.face_edge_intersection(face_id, &mesh2, edge_id);
        let point0 = vec3(0.1, 0.0, 0.1);
        let point1 = vec3(0.2, 0.0, 0.2);
        assert_eq!(result, Some((Intersection::LinePiece { primitive: Primitive::Face(face_id), point0, point1 },
                                 Intersection::LinePiece { primitive: Primitive::Edge(edge_id), point0, point1 })));
    }

    #[test]
    fn test_face_edge_intersection_when_face_edge_intersects_at_point_and_edge_is_in_plane()
    {
        let mesh1 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]).build().unwrap();
        let mesh2 = MeshBuilder::new().with_indices((0..3).collect()).with_positions(vec![0.1, 0.0, 0.1, 1.2, 0.0, 0.2, 1.0, 1.0, 0.0]).build().unwrap();

        let face_id = mesh1.face_iter().next().unwrap();
        let edge_id = mesh2.edge_iter().map(|halfedge_id| mesh2.ordered_edge_vertices(halfedge_id)).find(|(v1, v2)| mesh2.vertex_position(*v1)[1] == 0.0 && mesh2.vertex_position(*v2)[1] == 0.0).unwrap();

        let result = mesh1.face_edge_intersection(face_id, &mesh2, edge_id);
        let point = vec3(0.1, 0.0, 0.1);
        assert_eq!(result, Some((Intersection::Point { primitive: Primitive::Face(face_id), point },
                                 Intersection::Point { primitive: Primitive::Vertex(edge_id.0), point })));
    }
}

mod utility {
    use crate::prelude::*;

    pub const MARGIN: f32 = 0.00001;
    pub const SQR_MARGIN: f32 = MARGIN * MARGIN;

    #[derive(Debug, PartialEq)]
    pub enum PlaneLinepieceIntersectionResult
    {
        P0InPlane,
        P1InPlane,
        LineInPlane,
        Intersection(Vec3)
    }

    pub fn plane_line_piece_intersection(p0: &Vec3, p1: &Vec3, p: &Vec3, n: &Vec3) -> Option<PlaneLinepieceIntersectionResult>
    {
        let ap0 = *p0 - *p;
        let ap1 = *p1 - *p;

        let d0 = n.dot(ap0);
        let d1 = n.dot(ap1);

        if d0.abs() < MARGIN && d1.abs() < MARGIN { // p0 and p1 lies in the plane
            Some(PlaneLinepieceIntersectionResult::LineInPlane)
        }
        else if d0.abs() < MARGIN { // p0 lies in the plane
            Some(PlaneLinepieceIntersectionResult::P0InPlane)
        }
        else if d1.abs() < MARGIN { // p1 lies in the plane
            Some(PlaneLinepieceIntersectionResult::P1InPlane)
        }
        else if d0.signum() != d1.signum() // The edge intersects the plane
        {
            // Find intersection point:
            let p01 = *p1 - *p0;
            let t = n.dot(-ap0) / n.dot(p01);
            let point = p0 + p01 * t;
            Some(PlaneLinepieceIntersectionResult::Intersection(point))
        }
        else {
            None
        }
    }

    pub fn plane_ray_intersection(ray_start_point: &Vec3, ray_direction: &Vec3, plane_point: &Vec3, plane_normal: &Vec3) -> Option<f32>
    {
        let denom = plane_normal.dot( *ray_direction);
        if denom.abs() >= MARGIN {
            let parameter = plane_normal.dot(plane_point - ray_start_point) / denom;
            if parameter >= 0.0 { Some(parameter) }
            else {None}
        }
        else {
            None
        }
    }

    // Compute barycentric coordinates (u, v, w) for
    // point p with respect to triangle (a, b, c)
    pub fn barycentric(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3) -> (f32, f32, f32)
    {
        let v0 = b - a;
        let v1 = c - a;
        let v2 = p - a;
        let d00 = v0.dot(v0);
        let d01 = v0.dot(v1);
        let d11 = v1.dot(v1);
        let d20 = v2.dot(v0);
        let d21 = v2.dot(v1);
        let denom = d00 * d11 - d01 * d01;
        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;
        (u, v, w)
    }

    pub fn point_line_segment_distance( point: &Vec3, p0: &Vec3, p1: &Vec3 ) -> f32
    {
        let v  = p1 - p0;
        let w  = point - p0;

        let c1 = w.dot(v);
        if c1 <= 0.0 { return w.magnitude(); }

        let c2 = v.dot(v);
        if c2 <= c1 { return (point - p1).magnitude(); }

        let b = c1 / c2;
        let pb = p0 + b * v;
        (point - &pb).magnitude()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_plane_ray_intersection_no_intersection()
        {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 0.0, 0.0);
            let dir = vec3(1.0, 0.0, 0.0);

            let result = plane_ray_intersection(&p0, &dir, &p, &n);
            assert_eq!(result, None);
        }

        #[test]
        fn test_plane_ray_intersection_point_in_plane()
        {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 0.0, 1.0);
            let dir = vec3(0.0, 1.0,1.0);

            let result = plane_ray_intersection(&p0, &dir, &p, &n);
            assert_eq!(result, Some(0.0));
        }

        #[test]
        fn test_plane_ray_intersection_intersection()
        {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 1.0, 0.0);
            let dir = vec3(0.0, 0.0, 1.0);

            let result = plane_ray_intersection(&p0, &dir, &p, &n);
            assert_eq!(result, Some(1.0));
        }

        #[test]
        fn test_plane_line_piece_intersection_no_intersection()
        {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 0.0, 0.0);
            let p1 = vec3(0.0, 1.0, 1.0 - MARGIN);

            let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
            assert_eq!(result, None);
        }

        #[test]
        fn test_plane_line_piece_intersection_point_in_plane()
        {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 0.0, 0.0);
            let p1 = vec3(0.0, 1.0,1.0);

            let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
            assert_eq!(result, Some(PlaneLinepieceIntersectionResult::P1InPlane));
        }

        #[test]
        fn test_plane_line_piece_intersection_intersection()
        {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 1.0, 0.0);
            let p1 = vec3(0.0, 1.0,1.0 + MARGIN);

            let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
            assert_eq!(result, Some(PlaneLinepieceIntersectionResult::Intersection(vec3(0.0, 1.0, 1.0))));
        }

        #[test]
        fn test_plane_line_piece_intersection_line_in_plane()
        {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(-1.0, 1.0, 1.0);
            let p1 = vec3(0.0, 1.0,1.0);

            let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
            assert_eq!(result, Some(PlaneLinepieceIntersectionResult::LineInPlane));
        }
    }
}