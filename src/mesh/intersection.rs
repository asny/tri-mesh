//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::intersection::utility::*;
use crate::prelude::*;

///
/// An enum describing the types of primitives.
///
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Primitive {
    /// Vertex
    Vertex(VertexID),
    /// Edge
    Edge(HalfEdgeID),
    /// Face
    Face(FaceID),
}

///
/// An enum describing the types of intersections.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Intersection {
    /// The intersection occurs at a single point
    Point {
        /// The [Primitive] (vertex, edge or face) that is intersected
        primitive: Primitive,
        /// The point where the intersection occurs
        point: Vec3,
    },
    /// The intersection occurs at a line piece interval
    LinePiece {
        /// The [Primitive] (vertex, edge or face) that is intersected at the first end point of the line piece where the intersection occurs
        primitive0: Primitive,
        /// The [Primitive] (vertex, edge or face) that is intersected at the second end point of the line piece where the intersection occurs
        primitive1: Primitive,
        /// The first end point of the line piece where the intersection occurs
        point0: Vec3,
        /// The second end point of the line piece where the intersection occurs
        point1: Vec3,
    },
}

/// # Intersection
impl Mesh {
    ///
    /// Find the [Intersection] between any face in the mesh and the given ray.
    /// If the ray intersects multiple faces, the face closest to the starting point in the direction of the ray is returned.
    /// If no faces are intersected, None is returned.
    ///
    pub fn ray_intersection(
        &self,
        ray_start_point: &Vec3,
        ray_direction: &Vec3,
    ) -> Option<Intersection> {
        let mut current: Option<Intersection> = None;
        for face_id in self.face_iter() {
            let new_intersection =
                self.face_ray_intersection(face_id, ray_start_point, ray_direction);
            if let Some(Intersection::Point { point, .. }) = new_intersection {
                let new_point = point;
                if let Some(Intersection::Point { point, .. }) = current {
                    if point.distance2(*ray_start_point) > new_point.distance2(*ray_start_point) {
                        current = new_intersection;
                    }
                } else {
                    current = new_intersection;
                }
            }
        }
        current
    }

    ///
    /// Find the [Intersection] between the given face and ray.
    /// If the face is not intersected by the ray, None is returned.
    ///
    pub fn face_ray_intersection(
        &self,
        face_id: FaceID,
        ray_start_point: &Vec3,
        ray_direction: &Vec3,
    ) -> Option<Intersection> {
        let p = self.vertex_position(self.walker_from_face(face_id).vertex_id().unwrap());
        let n = self.face_normal(face_id);

        plane_ray_intersection(ray_start_point, ray_direction, &p, &n).and_then(|parameter| {
            self.face_point_intersection_when_point_in_plane(
                face_id,
                &(ray_start_point + parameter * ray_direction),
            )
        })
    }

    ///
    /// Find the [Intersection] between the given face and line piece.
    /// If the face is not intersected by the line piece, None is returned.
    ///
    /// Note: Intersections, where the line piece is in the plane spanned by the face, are not yet fully handled.
    ///
    pub fn face_line_piece_intersection(
        &self,
        face_id: FaceID,
        point0: &Vec3,
        point1: &Vec3,
    ) -> Option<Intersection> {
        let p = self.vertex_position(self.walker_from_face(face_id).vertex_id().unwrap());
        let n = self.face_direction(face_id);

        plane_line_piece_intersection(&point0, &point1, &p, &n).and_then(|intersection| {
            match intersection {
                PlaneLinepieceIntersectionResult::LineInPlane => {
                    let intersection0 =
                        self.face_point_intersection_when_point_in_plane(face_id, point0);
                    let intersection1 =
                        self.face_point_intersection_when_point_in_plane(face_id, point1);
                    if let Some(Intersection::Point {
                        point: p0,
                        primitive: primitive0,
                    }) = intersection0
                    {
                        if let Some(Intersection::Point {
                            point: p1,
                            primitive: primitive1,
                        }) = intersection1
                        {
                            Some(Intersection::LinePiece {
                                primitive0,
                                primitive1,
                                point0: p0,
                                point1: p1,
                            })
                        } else {
                            intersection0 // TODO: Should return a Intersection::LinePiece instead of a Intersection::Point
                        }
                    } else {
                        intersection1 // TODO: Should return a Intersection::LinePiece instead of a Intersection::Point
                    }
                    // TODO: Handle case where the line piece intersects the face, but the end points are both outside
                }
                PlaneLinepieceIntersectionResult::P0InPlane => {
                    self.face_point_intersection_when_point_in_plane(face_id, point0)
                }
                PlaneLinepieceIntersectionResult::P1InPlane => {
                    self.face_point_intersection_when_point_in_plane(face_id, point1)
                }
                PlaneLinepieceIntersectionResult::Intersection(point) => {
                    self.face_point_intersection_when_point_in_plane(face_id, &point)
                }
            }
        })
    }

    ///
    /// Find the [Intersection] between the given vertex and the point.
    /// If the vertex is not close to the point, None is returned.
    ///
    pub fn vertex_point_intersection(
        &self,
        vertex_id: VertexID,
        point: &Vec3,
    ) -> Option<Intersection> {
        let p = self.vertex_position(vertex_id);
        if (p - point).magnitude2() < SQR_MARGIN {
            Some(Intersection::Point {
                primitive: Primitive::Vertex(vertex_id),
                point: *point,
            })
        } else {
            None
        }
    }

    ///
    /// Find the [Intersection] (the primitive is either a vertex or edge) between the given edge and the point.
    /// If the edge is not close to the point, None is returned.
    ///
    pub fn edge_point_intersection(
        &self,
        halfedge_id: HalfEdgeID,
        point: &Vec3,
    ) -> Option<Intersection> {
        let (v0, v1) = self.edge_vertices(halfedge_id);
        self.vertex_point_intersection(v0, point)
            .or_else(|| self.vertex_point_intersection(v1, point))
            .or_else(|| {
                if point_line_segment_distance(
                    point,
                    &self.vertex_position(v0),
                    &self.vertex_position(v1),
                ) < MARGIN
                {
                    let twin_id = self.walker_from_halfedge(halfedge_id).twin_id().unwrap();
                    Some(Intersection::Point {
                        primitive: Primitive::Edge(if twin_id < halfedge_id {
                            twin_id
                        } else {
                            halfedge_id
                        }),
                        point: *point,
                    })
                } else {
                    None
                }
            })
    }

    ///
    /// Find the [Intersection] (the primitive is either a vertex, edge or face) between the given face and the point.
    /// If the face is not close to the point, None is returned.
    ///
    pub fn face_point_intersection(&self, face_id: FaceID, point: &Vec3) -> Option<Intersection> {
        let p = self.vertex_position(self.walker_from_face(face_id).vertex_id().unwrap());
        let n = self.face_normal(face_id);
        if n.dot(point - p).abs() > MARGIN {
            return None;
        }

        self.face_point_intersection_when_point_in_plane(face_id, point)
    }

    /// Assumes that the point lies in the plane spanned by the face
    fn face_point_intersection_when_point_in_plane(
        &self,
        face_id: FaceID,
        point: &Vec3,
    ) -> Option<Intersection> {
        // Test whether the intersection point is located at the edges or vertices of the face
        for halfedge_id in self.face_halfedge_iter(face_id) {
            if let Some(intersection) = self.edge_point_intersection(halfedge_id, point) {
                return Some(intersection);
            }
        }

        // Test whether the intersection point is located inside the face
        let (a, b, c) = self.face_positions(face_id);
        let coords = barycentric(point, &a, &b, &c);
        if 0.0 < coords.0
            && coords.0 < 1.0
            && 0.0 < coords.1
            && coords.1 < 1.0
            && 0.0 < coords.2
            && coords.2 < 1.0
        {
            Some(Intersection::Point {
                primitive: Primitive::Face(face_id),
                point: *point,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_point_intersection_when_point_in_plane() {
        let positions: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
        let mut mesh = MeshBuilder::new()
            .with_positions(positions)
            .build()
            .unwrap();
        mesh.scale(3.0);
        let face_id = mesh.face_iter().next().unwrap();
        let (v0, v1, v2) = mesh.ordered_face_vertices(face_id);
        let p0 = mesh.vertex_position(v0);
        let p1 = mesh.vertex_position(v1);
        let p2 = mesh.vertex_position(v2);

        let edge_midpoint = (p1 + p2) * 0.5;
        let face_midpoint = mesh.face_center(face_id);

        // Vertex intersection
        let mut result = mesh.face_point_intersection_when_point_in_plane(face_id, &p0);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Vertex(v0),
                point: p0
            })
        );

        let dir_away_from_p0 = -(0.5 * (p1 + p2) - p0).normalize();
        let p_intersect = p0 + 0.99 * MARGIN * dir_away_from_p0;
        result = mesh.face_point_intersection_when_point_in_plane(face_id, &p_intersect);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Vertex(v0),
                point: p_intersect
            })
        );

        result = mesh.face_point_intersection_when_point_in_plane(
            face_id,
            &(p0 + 1.01 * MARGIN * dir_away_from_p0),
        );
        assert_eq!(result, None);

        // Edge intersection
        result = mesh.face_point_intersection_when_point_in_plane(face_id, &edge_midpoint);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Edge(mesh.connecting_edge(v1, v2).unwrap()),
                point: edge_midpoint
            })
        );

        let dir_away_from_edge = vec3(0.0, 1.0, 0.0);
        let p_intersect = edge_midpoint + 0.99 * MARGIN * dir_away_from_edge;
        result = mesh.face_point_intersection_when_point_in_plane(face_id, &p_intersect);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Edge(mesh.connecting_edge(v1, v2).unwrap()),
                point: p_intersect
            })
        );

        result = mesh.face_point_intersection_when_point_in_plane(
            face_id,
            &(edge_midpoint + 1.01 * MARGIN * dir_away_from_edge),
        );
        assert_eq!(result, None);

        // Face intersection
        result = mesh.face_point_intersection_when_point_in_plane(face_id, &face_midpoint);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Face(face_id),
                point: face_midpoint
            })
        );
    }

    #[test]
    fn test_edge_point_intersection() {
        let positions: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
        let mut mesh = MeshBuilder::new()
            .with_positions(positions)
            .build()
            .unwrap();
        mesh.scale(3.0);
        let mut edge_id = mesh.halfedge_iter().next().unwrap();
        let twin_id = mesh.walker_from_halfedge(edge_id).twin_id().unwrap();
        if twin_id < edge_id {
            edge_id = twin_id;
        };
        let (v0, v1) = mesh.ordered_edge_vertices(edge_id);
        let p0 = mesh.vertex_position(v0);
        let p1 = mesh.vertex_position(v1);

        // Vertex intersection
        let mut result = mesh.edge_point_intersection(edge_id, &p0);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Vertex(v0),
                point: p0
            })
        );

        let dir_away_from_p0 = -(p1 - p0).normalize();
        let p_intersect = p0 + 0.99 * MARGIN * dir_away_from_p0;
        result = mesh.edge_point_intersection(edge_id, &p_intersect);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Vertex(v0),
                point: p_intersect
            })
        );

        result = mesh.edge_point_intersection(edge_id, &(p0 + 1.01 * MARGIN * dir_away_from_p0));
        assert_eq!(result, None);

        // Edge intersection
        let edge_midpoint = (p0 + p1) * 0.5;
        result = mesh.edge_point_intersection(edge_id, &edge_midpoint);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Edge(edge_id),
                point: edge_midpoint
            })
        );

        let dir_away_from_edge = dir_away_from_p0.cross(vec3(1.0, 1.0, 1.0)).normalize();
        let p_intersect = edge_midpoint + 0.99 * MARGIN * dir_away_from_edge;
        result = mesh.edge_point_intersection(edge_id, &p_intersect);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Edge(edge_id),
                point: p_intersect
            })
        );

        result = mesh.edge_point_intersection(
            edge_id,
            &(edge_midpoint + 1.01 * MARGIN * dir_away_from_edge),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn test_face_line_piece_intersection_when_no_intersection() {
        let mesh = MeshBuilder::new()
            .with_indices((0..3).collect())
            .with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0])
            .build()
            .unwrap();
        let face_id = mesh.face_iter().next().unwrap();
        let (p0, p1) = (vec3(1.0 + MARGIN, 0.0, 0.0), vec3(3.0, 0.0, 1.0));

        let result = mesh.face_line_piece_intersection(face_id, &p0, &p1);
        assert_eq!(result, None);
    }

    #[test]
    fn test_face_line_piece_intersection_when_face_end_point_intersects() {
        let mesh = MeshBuilder::new()
            .with_indices((0..3).collect())
            .with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0])
            .build()
            .unwrap();
        let face_id = mesh.face_iter().next().unwrap();
        let point = vec3(0.1, 0.0, 0.1);
        let (p0, p1) = (point, vec3(0.0, 1.0, 0.0));

        let result = mesh.face_line_piece_intersection(face_id, &p0, &p1);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Face(face_id),
                point
            })
        );
    }

    #[test]
    fn test_face_line_piece_intersection_when_face_line_piece_intersects_at_point() {
        let mesh = MeshBuilder::new()
            .with_indices((0..3).collect())
            .with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0])
            .build()
            .unwrap();
        let face_id = mesh.face_iter().next().unwrap();
        let point = vec3(0.1, 0.0, 0.1);
        let (p0, p1) = (vec3(0.1, 1.0, 0.1), vec3(0.1, -0.1, 0.1));

        let result = mesh.face_line_piece_intersection(face_id, &p0, &p1);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Face(face_id),
                point
            })
        );
    }

    #[test]
    fn test_face_line_piece_intersection_when_vertex_line_piece_intersects_at_point() {
        let point = vec3(0.1, 0.0, 0.1);
        let mesh = MeshBuilder::new()
            .with_indices((0..3).collect())
            .with_positions(vec![
                point.x, point.y, point.z, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0,
            ])
            .build()
            .unwrap();
        let face_id = mesh.face_iter().next().unwrap();
        let vertex_id = mesh
            .vertex_iter()
            .find(|v| mesh.vertex_position(*v) == point)
            .unwrap();
        let (p0, p1) = (vec3(0.1, 0.1, 0.1), vec3(0.1, -0.1, 0.1));

        let result = mesh.face_line_piece_intersection(face_id, &p0, &p1);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Vertex(vertex_id),
                point
            })
        );
    }

    #[test]
    fn test_face_line_piece_intersection_when_edge_line_piece_intersects_at_point() {
        let point = vec3(0.3, 0.0, 0.0);
        let mesh = MeshBuilder::new()
            .with_indices((0..3).collect())
            .with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0])
            .build()
            .unwrap();
        let face_id = mesh.face_iter().next().unwrap();
        let halfedge_id = mesh
            .face_halfedge_iter(face_id)
            .find(|e| (vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0)) == mesh.edge_positions(*e))
            .unwrap();
        let (p0, p1) = (point + vec3(0.0, 0.1, 0.0), point + vec3(0.0, -0.1, 0.0));

        let result = mesh.face_line_piece_intersection(face_id, &p0, &p1);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Edge(halfedge_id),
                point
            })
        );
    }

    #[test]
    fn test_face_line_piece_intersection_when_face_line_piece_intersects_at_linepiece() {
        let mesh = MeshBuilder::new()
            .with_indices((0..3).collect())
            .with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0])
            .build()
            .unwrap();
        let face_id = mesh.face_iter().next().unwrap();
        let (p0, p1) = (vec3(0.0, 0.0, 0.0), vec3(0.2, 0.0, 0.2));
        let vertex_id = mesh
            .vertex_iter()
            .find(|v| mesh.vertex_position(*v) == p0)
            .unwrap();

        let result = mesh.face_line_piece_intersection(face_id, &p0, &p1);
        assert_eq!(
            result,
            Some(Intersection::LinePiece {
                primitive0: Primitive::Vertex(vertex_id),
                primitive1: Primitive::Face(face_id),
                point0: p0,
                point1: p1
            })
        );
    }

    #[test]
    fn test_face_line_piece_intersection_when_face_line_piece_intersects_at_point_and_line_piece_is_in_plane(
    ) {
        let mesh = MeshBuilder::new()
            .with_indices((0..3).collect())
            .with_positions(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0])
            .build()
            .unwrap();
        let face_id = mesh.face_iter().next().unwrap();
        let point = vec3(0.1, 0.0, 0.1);
        let (p0, p1) = (point, vec3(1.2, 0.0, 0.2));

        let result = mesh.face_line_piece_intersection(face_id, &p0, &p1);
        assert_eq!(
            result,
            Some(Intersection::Point {
                primitive: Primitive::Face(face_id),
                point
            })
        );
    }
}

mod utility {
    use crate::prelude::*;

    pub const MARGIN: f64 = 0.0000001;
    pub const SQR_MARGIN: f64 = MARGIN * MARGIN;

    #[derive(Debug, PartialEq)]
    pub enum PlaneLinepieceIntersectionResult {
        P0InPlane,
        P1InPlane,
        LineInPlane,
        Intersection(Vec3),
    }

    pub fn plane_line_piece_intersection(
        p0: &Vec3,
        p1: &Vec3,
        p: &Vec3,
        n: &Vec3,
    ) -> Option<PlaneLinepieceIntersectionResult> {
        let ap0 = *p0 - *p;
        let ap1 = *p1 - *p;

        let d0 = n.dot(ap0);
        let d1 = n.dot(ap1);

        if d0.abs() < MARGIN && d1.abs() < MARGIN {
            // p0 and p1 lies in the plane
            Some(PlaneLinepieceIntersectionResult::LineInPlane)
        } else if d0.abs() < MARGIN {
            // p0 lies in the plane
            Some(PlaneLinepieceIntersectionResult::P0InPlane)
        } else if d1.abs() < MARGIN {
            // p1 lies in the plane
            Some(PlaneLinepieceIntersectionResult::P1InPlane)
        } else if d0.signum() != d1.signum()
        // The edge intersects the plane
        {
            // Find intersection point:
            let p01 = *p1 - *p0;
            let t = n.dot(-ap0) / n.dot(p01);
            let point = p0 + p01 * t;
            Some(PlaneLinepieceIntersectionResult::Intersection(point))
        } else {
            None
        }
    }

    pub fn plane_ray_intersection(
        ray_start_point: &Vec3,
        ray_direction: &Vec3,
        plane_point: &Vec3,
        plane_normal: &Vec3,
    ) -> Option<f64> {
        let denom = plane_normal.dot(*ray_direction);
        if denom.abs() >= MARGIN {
            let parameter = plane_normal.dot(plane_point - ray_start_point) / denom;
            if parameter >= 0.0 {
                Some(parameter)
            } else {
                None
            }
        } else {
            None
        }
    }

    // Compute barycentric coordinates (u, v, w) for
    // point p with respect to triangle (a, b, c)
    pub fn barycentric(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3) -> (f64, f64, f64) {
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

    pub fn point_line_segment_distance(point: &Vec3, p0: &Vec3, p1: &Vec3) -> f64 {
        let v = p1 - p0;
        let w = point - p0;

        let c1 = w.dot(v);
        if c1 <= 0.0 {
            return w.magnitude();
        }

        let c2 = v.dot(v);
        if c2 <= c1 {
            return (point - p1).magnitude();
        }

        let b = c1 / c2;
        let pb = p0 + b * v;
        (point - &pb).magnitude()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_barycentric() {
            let a = vec3(0.0, 0.0, 0.0);
            let b = vec3(1.0, 0.0, 0.0);
            let c = vec3(0.0, 0.0, 1.0);

            assert_eq!(
                barycentric(&vec3(0.0, 0.0, 0.0), &a, &b, &c),
                (1.0, 0.0, 0.0)
            );
            assert_eq!(
                barycentric(&vec3(1.0, 0.0, 0.0), &a, &b, &c),
                (0.0, 1.0, 0.0)
            );
            assert_eq!(
                barycentric(&vec3(0.0, 0.0, 1.0), &a, &b, &c),
                (0.0, 0.0, 1.0)
            );
            assert_eq!(
                barycentric(&vec3(0.5, 0.0, 0.5), &a, &b, &c),
                (0.0, 0.5, 0.5)
            );
            assert_eq!(
                barycentric(&vec3(0.25, 0.0, 0.25), &a, &b, &c),
                (0.5, 0.25, 0.25)
            );
        }

        #[test]
        fn test_point_line_segment_distance() {
            let a = vec3(0.0, 0.0, 0.0);
            let b = vec3(1.0, 0.0, 1.0);

            assert_eq!(
                point_line_segment_distance(&vec3(0.0, 0.0, 0.0), &a, &b),
                0.0
            );
            assert_eq!(
                point_line_segment_distance(&vec3(1.0, 0.0, 1.0), &a, &b),
                0.0
            );
            assert_eq!(
                point_line_segment_distance(&vec3(0.0, 0.0, 1.0), &a, &b),
                0.5 * 2.0f64.sqrt()
            );
            assert_eq!(
                point_line_segment_distance(&vec3(0.5, 0.0, 0.5), &a, &b),
                0.0
            );
            assert_eq!(
                point_line_segment_distance(&vec3(0.0, 0.0, -0.25), &a, &b),
                0.25
            );
            assert_eq!(
                point_line_segment_distance(&vec3(0.25, 0.0, 0.0), &a, &b),
                0.5 * (2.0 * 0.25f64 * 0.25f64).sqrt()
            );
        }

        #[test]
        fn test_plane_ray_intersection_no_intersection() {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 0.0, 0.0);
            let dir = vec3(1.0, 0.0, 0.0);

            let result = plane_ray_intersection(&p0, &dir, &p, &n);
            assert_eq!(result, None);
        }

        #[test]
        fn test_plane_ray_intersection_point_in_plane() {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 0.0, 1.0);
            let dir = vec3(0.0, 1.0, 1.0);

            let result = plane_ray_intersection(&p0, &dir, &p, &n);
            assert_eq!(result, Some(0.0));
        }

        #[test]
        fn test_plane_ray_intersection_intersection() {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 1.0, 0.0);
            let dir = vec3(0.0, 0.0, 1.0);

            let result = plane_ray_intersection(&p0, &dir, &p, &n);
            assert_eq!(result, Some(1.0));
        }

        #[test]
        fn test_plane_line_piece_intersection_no_intersection() {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 0.0, 0.0);
            let p1 = vec3(0.0, 1.0, 1.0 - 1.0001 * MARGIN);

            let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
            assert_eq!(result, None);
        }

        #[test]
        fn test_plane_line_piece_intersection_point_in_plane() {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 0.0, 0.0);
            let p1 = vec3(0.0, 1.0, 1.0);

            let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
            assert_eq!(result, Some(PlaneLinepieceIntersectionResult::P1InPlane));
        }

        #[test]
        fn test_plane_line_piece_intersection_intersection() {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(0.0, 1.0, 0.0);
            let p1 = vec3(0.0, 1.0, 1.0 + MARGIN);

            let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
            assert_eq!(
                result,
                Some(PlaneLinepieceIntersectionResult::Intersection(vec3(
                    0.0, 1.0, 1.0
                )))
            );
        }

        #[test]
        fn test_plane_line_piece_intersection_line_in_plane() {
            let p = vec3(1.0, 1.0, 1.0);
            let n = vec3(0.0, 0.0, -1.0);

            let p0 = vec3(-1.0, 1.0, 1.0);
            let p1 = vec3(0.0, 1.0, 1.0);

            let result = plane_line_piece_intersection(&p0, &p1, &p, &n);
            assert_eq!(result, Some(PlaneLinepieceIntersectionResult::LineInPlane));
        }
    }
}
