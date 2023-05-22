use super::*;

/// Cyclic group `CN`
///
/// Multiplication takes the state on the right and acts on it with the state on the left.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Cyclic<const N: usize>(usize);

impl<const N: usize> Default for Cyclic<N> {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl<const N: usize> TryFrom<usize> for Cyclic<N> {
    type Error = String;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > N {
            Err(format!(
                "Cannot convert {value} to cyclic group {N} element"
            ))
        } else {
            Ok(Cyclic::from_index_unchecked(value))
        }
    }
}

impl<const N: usize> From<Cyclic<N>> for usize {
    fn from(value: Cyclic<N>) -> Self {
        value.index()
    }
}

impl<const N: usize> Identity for Cyclic<N> {
    const IDENTITY: Self = Cyclic(0);
}

impl<const N: usize> BinaryOp for Cyclic<N> {
    fn binary_op(a: Self, b: Self) -> Self {
        Cyclic((a.index() + b.index()) % N)
    }

    fn order(&self) -> usize {
        if self.is_identity() {
            return 1;
        }
        N / crate::math::gcd(self.index(), N)
    }
}

impl<const N: usize> Inverse for Cyclic<N> {
    fn inverse(&self) -> Self {
        Cyclic((N - self.index()) % N)
    }
}

impl<const N: usize> Group for Cyclic<N> {
    fn iter_elements() -> Box<dyn std::iter::Iterator<Item = Self>> {
        Box::new(Self::iter_elements())
    }

    fn group_order() -> usize {
        N
    }

    fn has_valid_inverse() -> bool {
        true
    }

    fn is_abelian() -> bool {
        true
    }

    fn is_closed() -> bool {
        true
    }

    fn is_associative() -> bool {
        true
    }

    fn is_valid_group() -> bool {
        true
    }

    fn pow(self, n: i32) -> Self {
        Cyclic((self.index() as i64 * n as i64).rem_euclid(N as i64) as usize)
    }
}

impl<const N: usize> std::ops::Mul for Cyclic<N> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::binary_op(self, rhs)
    }
}

impl<const N: usize> Cyclic<N> {
    /// Returns the index starting from 0
    pub const fn index(self) -> usize {
        self.0
    }

    /// Returns a cyclic group element from its index starting at 0
    ///
    /// # Panics
    /// This function panics if `i > N`
    pub fn from_index(i: usize) -> Cyclic<N> {
        assert!(i < N);
        Cyclic(i)
    }

    /// Returns a cyclic group element from its index starting at 0 without
    /// checking if the index is valid
    pub fn from_index_unchecked(i: usize) -> Cyclic<N> {
        Cyclic(i)
    }

    /// Iterates over all the cyclic group elements
    pub fn iter_elements() -> impl Iterator<Item = Self> {
        Box::new((0..N).map(|i| Cyclic(i)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplication() {
        assert_eq!(Cyclic::<5>(2) * Cyclic(4), Cyclic(1));
        assert_eq!(Cyclic::<5>(0) * Cyclic(4), Cyclic(4));
        assert_eq!(Cyclic::<5>(4) * Cyclic(1), Cyclic(0));
    }

    #[test]
    fn inverse() {
        assert_eq!(Cyclic::<5>(2).inverse(), Cyclic(3));
        assert_eq!(Cyclic::<5>(1).inverse(), Cyclic(4));
        assert_eq!(Cyclic::<5>(0).inverse(), Cyclic(0));
    }

    #[test]
    fn pow() {
        assert_eq!(Cyclic::<5>(2).pow(19), Cyclic(3));
        assert_eq!(Cyclic::<5>(1).pow(0), Cyclic(0));
        assert_eq!(Cyclic::<5>(2).pow(-2), Cyclic(1));
    }
}
