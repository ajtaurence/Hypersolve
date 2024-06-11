use super::*;

/// Permutation of elements in word form ("is replaced by" format). Forms the group symmetric group `SN`
///
/// Multiplication takes the state on the right and acts on it with the state on the left.
///
/// # Warning
/// Permutation of 0 elements may not function as expected for all cases and may even cause panics.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Permutation<const N: usize>([u8; N]);

impl<const N: usize> std::fmt::Display for Permutation<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<const N: usize> From<Permutation<N>> for crate::common::Vector<u8, N> {
    fn from(value: Permutation<N>) -> Self {
        crate::common::Vector(value.0)
    }
}

impl<const N: usize> Default for Permutation<N> {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl<const N: usize> std::ops::Mul for Permutation<N> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        rhs.apply(self)
    }
}

impl<const N: usize> std::ops::Index<usize> for Permutation<N> {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        let result = &self.0[index];

        unsafe { assert_unchecked!((*result as usize) < N) };

        result
    }
}

impl<const N: usize> Identity for Permutation<N> {
    const IDENTITY: Self = Self::IDENTITY;
}

impl<const N: usize> BinaryOp for Permutation<N> {
    fn binary_op(a: Self, b: Self) -> Self {
        b.apply(a)
    }
}

impl<const N: usize> TryFrom<[u8; N]> for Permutation<N> {
    type Error = String;
    fn try_from(value: [u8; N]) -> Result<Self, Self::Error> {
        let result = Permutation(value);
        if result.is_valid() {
            Ok(result)
        } else {
            Err(format!(
                "Array {:?} contains repeated or skipped indices. Did you start at 0?",
                value
            ))
        }
    }
}

impl<const N: usize> Permutation<N> {
    /// Identity permutation
    pub(crate) const IDENTITY: Self = {
        let mut result = [0; N];
        let mut i = 0;
        while i < N as u8 {
            result[i as usize] = i;
            i += 1;
        }
        Permutation(result)
    };

    /// Converts the permutation into its array representation
    pub(crate) const fn into_array(self) -> [u8; N] {
        self.0
    }

    /// Creates a permutation from the given array
    ///
    /// # Panics
    ///
    /// Panics if the array is not a valid permutation
    pub(crate) fn from_array(array: [u8; N]) -> Self {
        array.try_into().expect("array is not a valid permutation")
    }

    /// Creates a permutation from the given array without checking for validity
    pub(crate) const fn from_array_unchecked(array: [u8; N]) -> Self {
        Permutation(array)
    }

    /// Returns whether the permutation is valid
    fn is_valid(&self) -> bool {
        let mut sorted_array = self.into_array();
        sorted_array.sort_unstable();

        sorted_array == Self::IDENTITY.into_array()
    }

    /// Applies the given permutation to self: `other * self`
    pub(crate) const fn apply(self, other: Self) -> Self {
        let mut result = [0; N];
        let mut i = 0;
        while i < N {
            result[i] = self.0[other.0[i] as usize];
            i += 1;
        }
        Permutation(result)
    }

    /// Returns the inverse of this permutation: `self^-1`
    pub(crate) const fn inverse(&self) -> Self {
        let mut result = [0; N];
        let mut i = 0;
        while i < N as u8 {
            result[self.0[i as usize] as usize] = i;
            i += 1;
        }
        Permutation(result)
    }

    /// Returns the parity of the permutation
    pub(crate) const fn parity(&self) -> crate::common::Parity {
        let mut visited = [false; N];
        let mut cycles = 0;

        let mut i = 0;
        while i < N {
            if visited[i] {
                i += 1;
                continue;
            }

            let mut current_index = self.0[i];

            visited[i] = true;
            while current_index != i as u8 {
                unsafe { assert_unchecked!((current_index as usize) < N) };

                visited[current_index as usize] = true;
                current_index = self.0[current_index as usize];
            }
            cycles += 1;
            i += 1;
        }
        match (N - cycles) % 2 {
            0 => super::super::Parity::Even,
            1 => super::super::Parity::Odd,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    // Returns a the permutation with entries at a and b swapped
    pub(crate) fn swap(mut self, a: usize, b: usize) -> Self {
        self.0.swap(a, b);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permutation_from_array() {
        Permutation::from_array([1, 2, 0, 5, 6, 3, 4]);
    }

    #[test]
    fn permutation_identity() {
        assert_eq!(Permutation::<6>::IDENTITY, Permutation([0, 1, 2, 3, 4, 5]));
    }

    #[test]
    fn permutation_parity() {
        use crate::common::Parity;
        assert_eq!(Permutation([0, 1, 2, 3, 4, 5]).parity(), Parity::Even);
        assert_eq!(Permutation([1, 0, 2, 3, 4, 5]).parity(), Parity::Odd);
        assert_eq!(Permutation([0, 1, 2, 5, 3, 4]).parity(), Parity::Even);
        assert_eq!(Permutation([0, 2, 3, 4, 5, 1]).parity(), Parity::Even);
        assert_eq!(Permutation([1, 0, 2, 5, 3, 4]).parity(), Parity::Odd);
    }

    #[test]
    fn permutation_swap() {
        assert_eq!(
            Permutation([0, 1, 2, 3, 4, 5]).swap(0, 2),
            Permutation([2, 1, 0, 3, 4, 5])
        );
        assert_eq!(
            Permutation([3, 1, 5, 0, 4, 2]).swap(0, 2),
            Permutation([5, 1, 3, 0, 4, 2])
        );
    }

    #[test]
    fn permutation_inverse() {
        assert_eq!(
            Permutation([0, 1, 2, 3, 4, 5]).inverse(),
            Permutation([0, 1, 2, 3, 4, 5])
        );
        assert_eq!(
            Permutation([1, 0, 2, 3, 4, 5]).inverse(),
            Permutation([1, 0, 2, 3, 4, 5])
        );
        assert_eq!(
            Permutation([0, 1, 2, 5, 3, 4]).inverse(),
            Permutation([0, 1, 2, 4, 5, 3])
        );
    }
}
