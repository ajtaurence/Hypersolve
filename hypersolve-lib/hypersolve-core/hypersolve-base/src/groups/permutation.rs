use const_for::const_for;
use const_gen::CompileConstArray;

use crate::*;

/// Permutation of elements in word form ("is replaced by" format). Forms the group symmetric group `SN`
///
/// The permutation can be constructed from an array of numbers 0..N
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GenericPermutation<const N: usize>([u8; N]);

impl<const N: usize> const_gen::CompileConst for GenericPermutation<N> {
    fn const_type() -> String {
        format!("GenericPermutation<{}>", N)
    }
    fn const_val(&self) -> String {
        let array = self.as_array().as_ref();
        format!(
            "unsafe{{GenericPermutation::from_array_unchecked({})}}",
            array.const_array_val()
        )
    }
}

impl<const N: usize> std::fmt::Display for GenericPermutation<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

macro_rules! assert_invariant {
    ($perm:ident, $SIZE:ident) => {
        const_for!(i in 0..N => {
            // SAFTEY: this is the invariant
            unsafe{assert_unchecked!($perm.0[i] < $SIZE as u8)};
        });
    };
}

impl<const N: usize> Default for GenericPermutation<N> {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl<const N: usize> GenericPermutation<N> {
    /// Identity permutation
    pub const IDENTITY: Self = GenericPermutation(const_arr!([u8; N], |i| i as u8));

    /// Number of permutations (N!)
    pub const N_PERMUTATIONS: u64 = {
        let n = N as u64;
        let mut prod = 1;
        const_for!(i in 2..n+1 => {
            prod *= i;
        });
        prod
    };

    /// Factorials up to (N-1)!
    const FACTORIALS: [u64; N] = const_arr!([u64; N], |i| factorial(i as u64));

    /// Creates a permutation from the given array without checking for validity
    ///
    /// # Safety
    /// It is undefined behavior if the array does not contain all numbers from 0..N
    pub const unsafe fn from_array_unchecked(array: [u8; N]) -> Self {
        debug_assert!(Self::is_valid_perm(&array));
        GenericPermutation(array)
    }

    /// Creates a permutation from the given array
    pub const fn from_array(array: [u8; N]) -> Option<Self> {
        if Self::is_valid_perm(&array) {
            // SAFTEY: we just confirmed that it is valid
            Some(unsafe { Self::from_array_unchecked(array) })
        } else {
            None
        }
    }

    /// Ensures that the permutation is valid
    const fn is_valid_perm(array: &[u8; N]) -> bool {
        let mut check = [false; N];

        const_for!(i in 0..N => {
            check[array[i] as usize] = true;
        });

        const_for!(i in 0..N => {
            if !check[i] {
                return false;
            }
        });

        true
    }

    /// Converts the permutation into its array representation
    pub const fn into_array(self) -> [u8; N] {
        assert_invariant!(self, N);
        self.0
    }

    /// Views the permutation as its array representation
    pub const fn as_array(&self) -> &[u8; N] {
        assert_invariant!(self, N);
        &self.0
    }

    /// Mutably views the permutation as its array representation
    ///
    /// # Safety
    /// The array must be a valid permutation when the returned reference is dropped
    pub unsafe fn as_array_mut(&mut self) -> &mut [u8; N] {
        assert_invariant!(self, N);
        &mut self.0
    }

    /// Returns the nth permutation. Intended to be used for constant evaluation only.
    ///
    /// # Safety
    /// It is undefined behavior `n >= factorial(N)`
    pub const unsafe fn nth_permutation_unchecked(mut n: usize) -> Self {
        debug_assert!(n < Self::N_PERMUTATIONS as usize);

        let mut result = [0_u8; N];
        let mut available = [true; N];

        let mut index;
        let mut count;
        let mut chosen;

        const_for!(i in 0..N => {
            let fac = Self::FACTORIALS[N - 1 - i] as usize;
            index = n / fac;
            n %= fac;

            count = 0;
            chosen = 0;
            const_for!(j in 0..N => {
                if available[j] {
                    if count == index {
                        chosen = j;
                        break;
                    }
                    count += 1;
                }
            });

            result[i] = chosen as u8;
            available[chosen] = false;
        });

        Self(result)
    }

    /// Returns an iterator over all permutations
    pub fn iter_permutations() -> impl DoubleEndedIterator<Item = Self> + ExactSizeIterator {
        // SAFTEY: `i`` is at most `Self::N_PERMUTATIONS - 1` which is less than `N!`
        (0..Self::N_PERMUTATIONS as usize).map(|i| unsafe { Self::nth_permutation_unchecked(i) })
    }

    /// Applies the given permutation a to b (i.e. `a * b`)
    pub const fn group_mul(a: &Self, b: &Self) -> Self {
        GenericPermutation(a.permute(b.as_array()))
    }

    /// Permutes the elements of the array by this permutation
    pub const fn permute<T>(&self, array: &[T; N]) -> [T; N]
    where
        T: Copy,
    {
        const_arr!([T; N], |i| {
            // SAFTEY: All elements in the array must be less than N so this branch is unreachable
            unsafe { assert_unchecked!(self.as_array()[i] < N as u8) };

            array[self.as_array()[i] as usize]
        })
    }

    /// Returns the inverse of this permutation: `self^-1`
    pub const fn inverse(&self) -> Self {
        // this is safe because we overwrite every value of the array and drop being called on u8 does nothing
        #[allow(clippy::uninit_assumed_init)]
        let mut result = unsafe { std::mem::MaybeUninit::<[u8; N]>::uninit().assume_init() };

        let mut i = 0;
        while i < N {
            // SAFTEY: All elements in the array must be less than N so this branch is unreachable
            unsafe { assert_unchecked!(self.as_array()[i] < N as u8) };

            result[self.as_array()[i] as usize] = i as u8;
            i += 1;
        }
        GenericPermutation(result)
    }

    /// Returns the parity of the permutation
    pub const fn parity(&self) -> Parity {
        let mut visited = [false; N];
        let mut cycles = 0;

        let mut i = 0;
        while i < N {
            if visited[i] {
                i += 1;
                continue;
            }

            let mut current_index = self.as_array()[i] as usize;
            visited[i] = true;
            while current_index != i {
                // SAFTEY: Current index is always less than N because only elements of
                // the permutation array are assigned to it. These are guaranteed to be
                // less than N
                unsafe { assert_unchecked!(current_index < N) };

                visited[current_index] = true;
                current_index = self.as_array()[current_index] as usize;
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

    /// Returns the order of the permutation. The order is the lowest non-zero power `n` for which `self^n = self`
    pub const fn order(&self) -> usize {
        use super::super::math::lcm;

        let mut visited = [false; N];

        let mut cycle_lengths = [1; N];

        let mut i = 0;
        while i < N {
            if visited[i] {
                i += 1;
                continue;
            }

            let mut current_index = self.as_array()[i] as usize;
            let mut cycle_length = 1;
            visited[i] = true;
            while current_index != i {
                visited[current_index] = true;
                current_index = self.as_array()[current_index] as usize;
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

    // Returns a the permutation with entries at a and b swapped
    pub fn swap(mut self, a: usize, b: usize) -> Self {
        // SAFTEY: Swapping two elements results in a valid permutation
        unsafe {
            self.as_array_mut().swap(a, b);
        }
        self
    }

    // Returns a the permutation with entries at a and b swapped
    pub const fn const_swap(mut self, a: usize, b: usize) -> Self {
        // SAFTEY: Swapping two elements results in a valid permutation
        let temp = self.as_array()[a];
        self.0[a] = self.as_array()[b];
        self.0[b] = temp;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permutation_order() {
        assert_eq!(GenericPermutation([0, 1, 2, 3, 4, 5]).order(), 1);
        assert_eq!(GenericPermutation([1, 0, 2, 3, 4, 5]).order(), 2);
        assert_eq!(GenericPermutation([0, 1, 2, 5, 3, 4]).order(), 3);
        assert_eq!(GenericPermutation([0, 2, 3, 4, 5, 1]).order(), 5);
        assert_eq!(GenericPermutation([1, 0, 2, 5, 3, 4]).order(), 6);
    }

    #[test]
    fn permutation_identity() {
        assert_eq!(
            GenericPermutation::<6>::IDENTITY,
            GenericPermutation([0, 1, 2, 3, 4, 5])
        );
    }

    #[test]
    fn permutation_parity() {
        use crate::Parity;
        assert_eq!(
            GenericPermutation([0, 1, 2, 3, 4, 5]).parity(),
            Parity::Even
        );
        assert_eq!(GenericPermutation([1, 0, 2, 3, 4, 5]).parity(), Parity::Odd);
        assert_eq!(
            GenericPermutation([0, 1, 2, 5, 3, 4]).parity(),
            Parity::Even
        );
        assert_eq!(
            GenericPermutation([0, 2, 3, 4, 5, 1]).parity(),
            Parity::Even
        );
        assert_eq!(GenericPermutation([1, 0, 2, 5, 3, 4]).parity(), Parity::Odd);
    }

    #[test]
    fn permutation_swap() {
        assert_eq!(
            GenericPermutation([0, 1, 2, 3, 4, 5]).swap(0, 2),
            GenericPermutation([2, 1, 0, 3, 4, 5])
        );
        assert_eq!(
            GenericPermutation([3, 1, 5, 0, 4, 2]).swap(0, 2),
            GenericPermutation([5, 1, 3, 0, 4, 2])
        );
    }

    #[test]
    fn permutation_inverse() {
        assert_eq!(
            GenericPermutation([0, 1, 2, 3, 4, 5]).inverse(),
            GenericPermutation([0, 1, 2, 3, 4, 5])
        );
        assert_eq!(
            GenericPermutation([1, 0, 2, 3, 4, 5]).inverse(),
            GenericPermutation([1, 0, 2, 3, 4, 5])
        );
        assert_eq!(
            GenericPermutation([0, 1, 2, 5, 3, 4]).inverse(),
            GenericPermutation([0, 1, 2, 4, 5, 3])
        );
    }
}
