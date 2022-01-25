//!
//! Contains functionality for parsing from a 3d file to a [Mesh](crate::Mesh) and back.
//!

#[cfg(feature = "obj-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "obj-io")))]
mod obj;
#[doc(inline)]
#[cfg(feature = "obj-io")]
pub use obj::*;
