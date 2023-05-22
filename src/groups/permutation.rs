use super::*;

/// Permutation of elements in word form ("is replaced by" format). Forms the group symmetric group `SN`
///
/// Multiplication takes the state on the right and acts on it with the state on the left.
///
/// # Warning
/// Permutation of 0 elements may not function as expected for all cases and may even cause panics.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Permutation<const N: usize>([usize; N]);

impl<const N: usize> std::fmt::Display for Permutation<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<const N: usize> From<Permutation<N>> for crate::common::Vector<usize, N> {
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
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner()[index]
    }
}

impl<const N: usize> Identity for Permutation<N> {
    const IDENTITY: Self = Self::IDENTITY;
}

impl<const N: usize> BinaryOp for Permutation<N> {
    fn binary_op(a: Self, b: Self) -> Self {
        b.apply(a)
    }

    fn order(&self) -> usize {
        self.order()
    }
}

impl<const N: usize> Inverse for Permutation<N> {
    fn inverse(&self) -> Self {
        self.inverse()
    }
}

impl<const N: usize> Group for Permutation<N> {
    fn iter_elements() -> Box<dyn std::iter::Iterator<Item = Self>> {
        Box::new(Self::iter_permutations())
    }

    fn is_abelian() -> bool {
        match N {
            1 | 2 => true,
            _ => false,
        }
    }

    fn group_order() -> usize {
        crate::math::compute_factorial(N as u64) as usize
    }

    fn is_associative() -> bool {
        true
    }

    fn is_closed() -> bool {
        true
    }

    fn has_valid_inverse() -> bool {
        true
    }

    fn is_valid_group() -> bool {
        true
    }
}

impl<const N: usize> TryFrom<[usize; N]> for Permutation<N> {
    type Error = String;
    fn try_from(value: [usize; N]) -> Result<Self, Self::Error> {
        let result = Permutation(value);
        if result.is_valid() {
            return Ok(result);
        } else {
            return Err(format!(
                "Array {:?} contains repeated or skipped indices. Did you start at 0?",
                value
            ));
        }
    }
}

impl<const N: usize> Permutation<N> {
    /// Identity permutation
    pub const IDENTITY: Self = {
        let mut result = [0; N];
        let mut i = 0;
        while i < N {
            result[i] = i;
            i += 1;
        }
        Permutation(result)
    };

    /// Converts the permutation into its array representation
    pub const fn into_inner(self) -> [usize; N] {
        self.0
    }

    /// Returns a reference to the array representation
    pub const fn inner(&self) -> &[usize; N] {
        &self.0
    }

    /// Creates a permutation from the given array
    ///
    /// # Panics
    ///
    /// Panics if the array is not a valid permutation
    pub fn from_array(array: [usize; N]) -> Self {
        array.try_into().expect("Array is not a valid permutation")
    }

    /// Creates a permutation from the given array without checking for validity
    pub const fn from_array_unchecked(array: [usize; N]) -> Self {
        Permutation(array)
    }

    /// Returns an iterator over all permutations
    pub fn iter_permutations() -> impl Iterator<Item = Self> {
        use itertools::Itertools;
        (0..N)
            .into_iter()
            .permutations(N)
            .map(|p| Permutation(p.try_into().unwrap()))
    }

    /// Test whether two permutations are equal
    pub const fn is_equal(a: &Self, b: &Self) -> bool {
        let mut i = 0;
        while i < N {
            if a.0[i] != b.0[i] {
                return false;
            }
            i += 1;
        }
        return true;
    }

    /// Returns whether the permutation is valid
    fn is_valid(&self) -> bool {
        let mut sorted_array = self.into_inner();
        sorted_array.sort_unstable();

        sorted_array == Self::IDENTITY.into_inner()
    }

    /// Applies the given permutation to self: `other * self`
    pub const fn apply(self, other: Self) -> Self {
        let mut result: [usize; N] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        let mut i = 0;
        while i < N {
            result[i] = self.0[other.0[i]];
            i += 1;
        }
        Permutation(result)
    }

    /// Returns the inverse of this permutation: `self^-1`
    pub const fn inverse(&self) -> Self {
        let mut result: [usize; N] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        let mut i = 0;
        while i < N {
            result[self.0[i]] = i;
            i += 1;
        }
        Permutation(result)
    }

    /// Returns whether the permutation is its own inverse
    pub const fn is_self_inverse(&self) -> bool {
        Self::is_equal(self, &self.inverse())
    }

    /// Returns the parity of the permutation
    pub const fn parity(&self) -> crate::common::Parity {
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
            while current_index != i {
                visited[current_index] = true;
                current_index = self.0[current_index];
            }
            cycles += 1;
            i += 1;
        }
        match (N - cycles) % 2 {
            0 => crate::common::Parity::Even,
            1 => crate::common::Parity::Odd,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    /// Returns the order of the permutation. The order is the lowest non-zero power `n` for which `self^n = self`
    pub const fn order(&self) -> usize {
        use crate::math::lcm;
        let mut visited = [false; N];

        let mut cycle_lengths = [1; N];

        let mut i = 0;
        while i < N {
            if visited[i] {
                i += 1;
                continue;
            }

            let mut current_index = self.0[i];
            let mut cycle_length = 1;
            visited[i] = true;
            while current_index != i {
                visited[current_index] = true;
                current_index = self.0[current_index];
                cycle_length += 1;
            }
            cycle_lengths[i] = cycle_length;
            i += 1;
        }

        let mut acc = 1;
        let mut i = 0;
        while i < N {
            acc = lcm(acc, cycle_lengths[i]);
            i += 1;
        }

        acc
    }

    /// Returns the conjugate of this permutation by another permutation: `other * self * other^-1`
    pub const fn conjugate(self, other: Self) -> Self {
        other.inverse().apply(self).apply(other)
    }

    // Returns a the permutation with entries at a and b swapped
    pub fn swap(mut self, a: usize, b: usize) -> Self {
        let a_ptr = &mut self.0[a] as *mut usize;
        let b_ptr = &mut self.0[b] as *mut usize;
        unsafe { std::ptr::swap(a_ptr, b_ptr) };
        return self;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permutation_order() {
        assert_eq!(Permutation([0, 1, 2, 3, 4, 5]).order(), 1);
        assert_eq!(Permutation([1, 0, 2, 3, 4, 5]).order(), 2);
        assert_eq!(Permutation([0, 1, 2, 5, 3, 4]).order(), 3);
        assert_eq!(Permutation([0, 2, 3, 4, 5, 1]).order(), 5);
        assert_eq!(Permutation([1, 0, 2, 5, 3, 4]).order(), 6);
    }

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

    #[test]
    fn permutation_self_inverse() {
        assert_eq!(Permutation([0, 1, 2, 3, 4, 5]).is_self_inverse(), true);
        assert_eq!(Permutation([1, 0, 2, 3, 4, 5]).is_self_inverse(), true);
        assert_eq!(Permutation([0, 1, 2, 5, 3, 4]).is_self_inverse(), false);
    }

    #[test]
    fn permutation_pow() {
        assert_eq!(
            Permutation([1, 0, 2, 3, 4, 5]).pow(3),
            Permutation([1, 0, 2, 3, 4, 5])
        );
        assert_eq!(
            Permutation([1, 0, 2, 3, 4, 5]).pow(100),
            Permutation([0, 1, 2, 3, 4, 5])
        );
        assert_eq!(
            Permutation([1, 0, 2, 3, 4, 5]).pow(-1),
            Permutation([1, 0, 2, 3, 4, 5]).inverse()
        );
    }

    #[test]
    fn test() {
        println!("{:?}", Permutation::<3>::is_valid_group())
    }
}
