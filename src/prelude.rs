//! This module contains the most common traits used in `tri-mesh`. By
//! glob-importing this module, you can avoid the need to import each trait
//! individually, while still being selective about what types you import.

pub use crate::mesh::math::*;

pub use crate::mesh::ids::*;
pub use crate::mesh::traversal::Walker;
pub use crate::mesh::iterators::*;

pub use crate::mesh::Mesh;