mod a4;
mod c3;
mod k4;
mod permutation;

pub use a4::*;
pub use c3::*;
pub use k4::*;
pub use permutation::*;

/// A trait for types with an identity element
pub trait Identity: PartialEq + Sized {
    /// The identity element `I`
    const IDENTITY: Self;

    /// Returns whether the element is the identity
    fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }
}

/// A trait for types with inverse elements
pub trait Inverse: PartialEq + Sized {
    /// Gets the inverse of a group element such that
    /// `I = a * a^-1 = a^-1 * a`
    fn inverse(&self) -> Self;

    /// Returns whether the element is its own inverse
    fn is_self_inverse(&self) -> bool {
        *self == self.inverse()
    }
}

/// A trait for types with a binary operation
pub trait BinaryOp: Clone + PartialEq {
    /// A binary operation on group elements satisfing associativity and closure
    ///
    /// Performs the group operation `a * b` where `a` acts on `b`
    fn binary_op(a: Self, b: Self) -> Self;

    /// Applies the given element to self, i.e. performs `b * self`
    fn apply(self, b: Self) -> Self {
        Self::binary_op(b, self)
    }

    /// Returns the order of this element. The order is the lowest non-zero power `n` for which `a^n = a`.
    ///
    /// # Warning
    /// Default implementation computes order by repeated multiplication and can be slow for large orders.
    /// It is recommended to replace this implementation.
    fn order(&self) -> usize {
        let mut element = self.clone().apply(self.clone());
        let mut count = 1;
        while element != *self {
            element = element.apply(self.clone());
            count += 1;
        }
        count
    }
}

/// Defining the group trait for a type means that instances of this type are group elements.
///
/// A group `G` is defined by a binary operation on group elements satisfying closure and associativity.
/// The group must contain an identity element and each element must have an inverse.
///
/// # Warning
/// For large groups it is a good idea to replace many of the default implementations since they are brute
/// force and may take a while to compute. There is likely a faster way to calculate things for specific
/// groups.
pub trait Group: Identity + BinaryOp + Inverse {
    /// An iterator over all elements in the group
    fn iter_elements() -> Box<dyn std::iter::Iterator<Item = Self>>;

    /// Returns the conjugate of this element by another element: `b * self * b^-1`
    fn conjugate(self, b: Self) -> Self {
        b.inverse().apply(self).apply(b)
    }

    /// Calculates `self^n` using binary exponentiation
    fn pow(self, n: i32) -> Self {
        if n == 0 {
            return Self::IDENTITY;
        }

        let elem = if n > 0 { self } else { self.inverse() };
        let n = n.abs();

        let half = elem.clone().pow(n / 2);
        let result = half.clone().apply(half);
        if n % 2 == 1 {
            result.apply(elem)
        } else {
            result
        }
    }

    /// Returns whether the group is closed under the binary operation
    fn is_closed() -> bool {
        use itertools::Itertools;
        for elem1 in Self::iter_elements() {
            for elem2 in Self::iter_elements() {
                if !Self::iter_elements().contains(&elem1.clone().apply(elem2)) {
                    return false;
                }
            }
        }
        true
    }

    /// Returns whether the binary operation is associative
    fn is_associative() -> bool {
        for elem1 in Self::iter_elements() {
            for elem2 in Self::iter_elements() {
                for elem3 in Self::iter_elements() {
                    if Self::binary_op(Self::binary_op(elem1.clone(), elem2.clone()), elem3.clone())
                        != Self::binary_op(elem1.clone(), Self::binary_op(elem2.clone(), elem3))
                    {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Returns whether every element's inverse is valid
    fn has_valid_inverse() -> bool {
        for elem in Self::iter_elements() {
            if (elem.clone().apply(elem.inverse()) != Self::IDENTITY)
                || (elem.inverse().apply(elem) != Self::IDENTITY)
            {
                return false;
            }
        }
        true
    }

    /// Returns whether this group is a valid group satisfying
    ///
    /// 1. Closure
    /// 1. Associativity
    /// 1. Identity element
    /// 1. Inverse element
    fn is_valid_group() -> bool {
        Self::is_associative() && Self::is_closed() && Self::has_valid_inverse()
    }

    /// Returns the size of the group
    fn group_order() -> usize {
        Self::iter_elements().count()
    }

    /// Returns whether the group is abelian, i.e. `a * b = b * a ∀ a,b ∈ G`
    fn is_abelian() -> bool {
        for (i, elem1) in Self::iter_elements().enumerate() {
            for (j, elem2) in Self::iter_elements().enumerate() {
                if j >= i {
                    break;
                }
                if elem1.clone().apply(elem2.clone()) != elem2.apply(elem1.clone()) {
                    return false;
                }
            }
        }
        true
    }
}
