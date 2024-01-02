#[macro_use]
pub(crate) mod macros;
pub(super) mod groups;
pub(super) mod math;

#[macro_use]
mod vector;
mod parity;

pub(super) use parity::Parity;
pub(super) use vector::{Vector, Vector3, Vector4};
