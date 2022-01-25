//!
//! Contains functionality for parsing to and from a [Mesh](crate::Mesh).
//!

#[cfg(feature = "obj-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "obj-io")))]
mod obj;
#[doc(inline)]
#[cfg(feature = "obj-io")]
pub use obj::*;
