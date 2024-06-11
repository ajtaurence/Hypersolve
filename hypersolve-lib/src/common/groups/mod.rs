mod a4;
mod c3;
mod k4;
mod permutation;

pub(crate) use a4::*;
pub(crate) use c3::*;
pub(crate) use k4::*;
pub(crate) use permutation::*;

/// A trait for types with an identity element
pub(crate) trait Identity: PartialEq + Sized {
    /// The identity element `I`
    const IDENTITY: Self;
}

/// A trait for types with a binary operation
pub(crate) trait BinaryOp: Clone + PartialEq {
    /// A binary operation on group elements satisfing associativity and closure
    ///
    /// Performs the group operation `a * b` where `a` acts on `b`
    fn binary_op(a: Self, b: Self) -> Self;
}
