//! See [Mesh](crate::mesh::Mesh).

use std::fmt;

///
/// An unique ID for a vertex
///
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct VertexID
{
    val: usize
}

impl VertexID {
    pub(crate) fn new(val: usize) -> VertexID
    {
        VertexID {val}
    }
}

impl fmt::Display for VertexID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

///
/// An unique ID for a halfedge
///
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct HalfEdgeID
{
    val: usize
}

impl HalfEdgeID {
    pub(crate) fn new(val: usize) -> HalfEdgeID
    {
        HalfEdgeID {val}
    }
}

impl fmt::Display for HalfEdgeID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

///
/// An unique ID for a face
///
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct FaceID
{
    val: usize
}

impl FaceID {
    pub(crate) fn new(val: usize) -> FaceID
    {
        FaceID {val}
    }
}

impl fmt::Display for FaceID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

///
/// An enum describing the types of primitives.
///
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Primitive {
    /// Vertex
    Vertex(VertexID),
    /// Edge described as a pair of vertices
    Edge((VertexID, VertexID)),
    /// Face
    Face(FaceID)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_equality()
    {
        let v0 = VertexID::new(0);
        let v1 = VertexID::new(1);
        let v1_ = VertexID::new(1);

        assert!(v0 != v1);
        assert!(v1 == v1_);
    }
}