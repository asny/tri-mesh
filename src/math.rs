//!
//! Linear algebra types for vector calculations. Basically re-export the [cgmath](https://crates.io/crates/cgmath) library.
//!

use cgmath;
pub use cgmath::prelude::*;
use cgmath::{Deg, Matrix3, Matrix4, Rad, Vector3, Vector4};

/// Vector with three elements.
pub type Vec3 = Vector3<f64>;
/// Vector with four elements.
pub type Vec4 = Vector4<f64>;

/// 3x3 matrix.
pub type Mat3 = Matrix3<f64>;
/// 4x4 matrix.
pub type Mat4 = Matrix4<f64>;

/// Degrees
pub type Degrees = Deg<f64>;
/// Radians
pub type Radians = Rad<f64>;

/// Constructs a [Vec3]
pub const fn vec3(x: f64, y: f64, z: f64) -> Vec3 {
    Vector3::new(x, y, z)
}

/// Constructs a [Vec4]
pub const fn vec4(x: f64, y: f64, z: f64, w: f64) -> Vec4 {
    Vector4::new(x, y, z, w)
}

/// Constructs a [Degrees]
pub const fn degrees(v: f64) -> Degrees {
    Deg(v)
}
/// Constructs a [Radians]
pub const fn radians(v: f64) -> Radians {
    Rad(v)
}
