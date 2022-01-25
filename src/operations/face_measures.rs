//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;

/// # Face measures
impl Mesh {
    /// Returns the positions of the face vertices.
    pub fn face_positions(&self, face_id: FaceID) -> (Vec3, Vec3, Vec3) {
        let vertices = self.ordered_face_vertices(face_id);
        (
            self.vertex_position(vertices.0),
            self.vertex_position(vertices.1),
            self.vertex_position(vertices.2),
        )
    }

    /// Returns the unnormalized normal of the face.
    pub fn face_direction(&self, face_id: FaceID) -> Vec3 {
        let mut walker = self.walker_from_face(face_id);
        let p0 = self.vertex_position(walker.vertex_id().unwrap());
        walker.as_next();
        let v0 = self.vertex_position(walker.vertex_id().unwrap()) - p0;
        walker.as_next();
        let v1 = self.vertex_position(walker.vertex_id().unwrap()) - p0;

        v0.cross(v1)
    }

    /// Returns the normal of the face.
    pub fn face_normal(&self, face_id: FaceID) -> Vec3 {
        self.face_direction(face_id).normalize()
    }

    /// Returns the area of the face.
    pub fn face_area(&self, face_id: FaceID) -> f64 {
        0.5 * self.face_direction(face_id).magnitude()
    }

    /// Returns the center of the face given as the average of its vertex positions.
    pub fn face_center(&self, face_id: FaceID) -> Vec3 {
        let mut walker = self.walker_from_face(face_id);
        let p0 = self.vertex_position(walker.vertex_id().unwrap());
        walker.as_next();
        let p1 = self.vertex_position(walker.vertex_id().unwrap());
        walker.as_next();
        let p2 = self.vertex_position(walker.vertex_id().unwrap());

        (p0 + p1 + p2) / 3.0
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_face_area() {
        let mesh = MeshBuilder::new().triangle().build().unwrap();
        let face_id = mesh.face_iter().next().unwrap();
        assert_eq!(9.0, mesh.face_area(face_id));
    }

    #[test]
    fn test_face_normal() {
        let mesh = MeshBuilder::new().triangle().build().unwrap();
        let face_id = mesh.face_iter().next().unwrap();
        let computed_normal = mesh.face_normal(face_id);
        assert_eq!(0.0, computed_normal.x);
        assert_eq!(0.0, computed_normal.y);
        assert_eq!(1.0, computed_normal.z);
    }

    #[test]
    fn test_face_center() {
        let mesh = MeshBuilder::new().triangle().build().unwrap();
        let face_id = mesh.face_iter().next().unwrap();
        let center = mesh.face_center(face_id);
        assert_eq!(0.0, center.x);
        assert_eq!(0.0, center.y);
        assert_eq!(0.0, center.z);
    }
}
