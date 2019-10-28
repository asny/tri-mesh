//! Defines unique id's for a vertex, half-edge and face.

use std::fmt;

///
/// An unique ID for a vertex
///
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct VertexID
{
    val: u32
}

impl VertexID {
    pub(crate) fn new(val: u32) -> VertexID
    {
        VertexID {val}
    }
    pub(crate) fn get(&self) -> u32
    {
        self.val
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
    val: u32
}

impl HalfEdgeID {
    pub(crate) fn new(val: u32) -> HalfEdgeID
    {
        HalfEdgeID {val}
    }
    pub(crate) fn get(&self) -> u32
    {
        self.val
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
    val: u32
}

impl FaceID {
    pub(crate) fn new(val: u32) -> FaceID
    {
        FaceID {val}
    }
    pub(crate) fn get(&self) -> u32
    {
        self.val
    }
}

impl fmt::Display for FaceID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.val)
    }
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