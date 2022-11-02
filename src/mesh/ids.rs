//! Defines unique id's for a vertex, half-edge and face.

use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

/// A unique ID.
pub trait ID: Clone + Eq + Copy + Ord + Hash + Debug + Deref<Target = u32> {
    /// Constructs a new ID from an integer. Should only be used for testing or similar.
    unsafe fn new(val: u32) -> Self;
}

///
/// A unique ID for a vertex.
///
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct VertexID(u32);

impl ID for VertexID {
    unsafe fn new(val: u32) -> VertexID {
        VertexID(val)
    }
}

impl Deref for VertexID {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for VertexID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

///
/// A unique ID for a halfedge.
///
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct HalfEdgeID(u32);

impl ID for HalfEdgeID {
    unsafe fn new(val: u32) -> HalfEdgeID {
        HalfEdgeID(val)
    }
}

impl Deref for HalfEdgeID {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for HalfEdgeID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

///
/// A unique ID for a face.
///
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct FaceID(u32);

impl ID for FaceID {
    unsafe fn new(val: u32) -> FaceID {
        FaceID(val)
    }
}

impl Deref for FaceID {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for FaceID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_equality() {
        unsafe {
            let v0 = VertexID::new(0);
            let v1 = VertexID::new(1);
            let v1_ = VertexID::new(1);

            assert!(v0 != v1);
            assert!(v1 == v1_);
        }
    }
}
