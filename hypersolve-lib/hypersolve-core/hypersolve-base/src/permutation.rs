use const_for::const_for;

use crate::*;

/// Describes the permutation of pieces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, const_gen::CompileConst)]
pub struct Permutation(pub GenericPermutation<15>);

impl Default for Permutation {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl Permutation {
    /// The solved permutation (identity)
    pub const SOLVED: Self = Self(GenericPermutation::IDENTITY);

    /// # Safety
    /// `array` must be a valid permutation
    pub const unsafe fn from_array_unchecked(array: [u8; 15]) -> Self {
        Self(GenericPermutation::from_array_unchecked(array))
    }

    pub fn from_cube(cube: Cube) -> Self {
        // this gets the inverse of the map we want
        let pieces_except_last = cube.reposition().pieces_except_last();

        let inverse_map = const_arr!([u8; 15], |i| {
            pieces_except_last[i].current_location().index().into_u8()
        });

        // invert it to get the map we want
        // SAFTEY: Each piece is unique so this represents a valid permutation
        Permutation(unsafe { GenericPermutation::from_array_unchecked(inverse_map) }).inverse()
    }

    pub const fn as_array(&self) -> &[u8; 15] {
        self.0.as_array()
    }

    pub const fn into_array(&self) -> [u8; 15] {
        self.0.into_array()
    }

    /// Permutes this permutation by another
    pub const fn permute(&self, permutation: &Permutation) -> Self {
        Self(GenericPermutation::group_mul(&permutation.0, &self.0))
    }

    /// Returns the inverse of this permutation
    pub const fn inverse(&self) -> Self {
        Self(self.0.inverse())
    }

    /// Returns a number representing the IO separation state
    pub const fn io_coord(&self) -> u16 {
        // Find the indices of `self.map` representing pieces on I
        // SAFTEY: u8 does nothing when dropped and all values are overwritten
        #[allow(invalid_value, clippy::uninit_assumed_init)]
        let mut indices: [u8; 7] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        let mut index = 0;
        const_for!(i in 0..15 => {
            if self.as_array()[i] > 7 {
                indices[index] = i as u8;

                if index == 6 {
                    // we filled all the indices
                    break;
                } else {
                    index += 1;
                }
            }
        });

        let mut sum = N_IO_COORD_STATES - 1;
        const_for!(i in 0..7 => {
            // SAFTEY: elements of indices are always less than 16 and i+1 is always less than 7
            sum -= unsafe{n_choose_k(indices[i], i as u8 + 1)};
        });

        sum
    }

    /// Returns a number representing the permutation state of the I pieces
    pub const fn i_coord(self) -> u16 {
        // SAFTEY: u8 does nothing when dropped and all values are overwritten
        #[allow(invalid_value, clippy::uninit_assumed_init)]
        let mut i_map: [u8; 8] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        let mut index = 0;
        const_for!(i in 0..15 => {
            if self.as_array()[i] < 8 {
                i_map[index] = self.as_array()[i];

                if index == 7 {
                    // we filled the i_map
                    break;
                } else {
                    index += 1;
                }
            }
        });

        // SAFTEY: i_map is a valid permutation
        let mut i_coord = if unsafe { GenericPermutation::from_array_unchecked(i_map) }
            .parity()
            .is_odd()
        {
            N_I_COORD_STATES / 2
        } else {
            0
        };

        const_for!(i in 2..8 => {
            let mut count = 0;
            const_for!(j in 0..i => {
                if i_map[j] > i_map[i] {
                    count += 1;
                }
            });

            i_coord += count * (factorial(i as u64) / 2) as u16
        });

        i_coord
    }

    /// Returns a number representing the permutation state of the O pieces
    pub const fn o_coord(self) -> u16 {
        // SAFTEY: u8 does nothing when dropped and all values are overwritten
        #[allow(invalid_value, clippy::uninit_assumed_init)]
        let mut o_map: [u8; 7] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        let mut index = 0;
        const_for!(i in 0..15 => {
            if self.as_array()[i] >= 8 {
                o_map[index] = self.as_array()[i] % 8;

                if index == 6 {
                    // we filled the o_map
                    break;
                } else {
                    index += 1;
                }
            }
        });

        // SAFTEY: o_map is a valid permutation
        let mut o_coord = if unsafe { GenericPermutation::from_array_unchecked(o_map) }
            .parity()
            .is_odd()
        {
            N_O_COORD_STATES / 2
        } else {
            0
        };

        const_for!(i in 2..7 => {
            let mut count = 0;
            const_for!(j in 0..i => {
                if o_map[j] > o_map[i] {
                    count += 1;
                }
            });

            o_coord += count * (factorial(i as u64) / 2) as u16
        });

        o_coord
    }

    // Returns a permutation from its coordinates
    pub const fn from_coords(io_coord: u16, i_coord: u16, o_coord: u16) -> Permutation {
        let io_array = Self::coord_to_io_permutation(io_coord);

        let i_array = Self::coord_to_i_permutation(i_coord);
        let o_array = Self::coord_to_o_permutation(o_coord);

        // Construct the total permutation array
        let mut i_index: usize = 0;
        let mut o_index: usize = 0;

        let map = const_arr!([u8; 15], |i| {
            if io_array[i] {
                // SAFTEY io_array only contains 7 true values
                unsafe { assert_unchecked!(o_index < o_array.len()) }
                let res = o_array[o_index] + 8;
                o_index += 1;
                res
            } else {
                // SAFTEY io_array only contains 8 false values
                unsafe { assert_unchecked!(i_index < i_array.len()) }
                let res = i_array[i_index];
                i_index += 1;
                res
            }
        });

        // Fix the total permutation parity by swapping the parity index of the I coordinate if needed
        // SAFTEY: we just constructed a valid permutation map
        let perm = unsafe { GenericPermutation::from_array_unchecked(map) };
        if perm.parity().is_odd() {
            // TODO: Optimize this. We just throw away our work if it has the wrong parity
            Permutation::from_coords(
                io_coord,
                (i_coord + N_I_COORD_STATES / 2) % N_I_COORD_STATES,
                o_coord,
            )
        } else {
            Permutation(perm)
        }
    }

    /// Returns the IO permutation from a coordinate
    pub const fn coord_to_io_permutation(coord: u16) -> [bool; 15] {
        // It is common that the coord is 0 so skip the calculation in this case
        if coord == 0 {
            return [
                false, false, false, false, false, false, false, false, true, true, true, true,
                true, true, true,
            ];
        }

        let mut coord = N_IO_COORD_STATES - 1 - coord;

        let mut array: [bool; 15] = [false; 15];

        const_for!(i in (0..7).rev() => {
            let mut index = 8 + i;

            // SAFTEY: index is at most 14 and i+1 is at most 7
            while unsafe { n_choose_k(index, i + 1) } > coord {
                index -= 1;
            }

            // SAFTEY: index is at most 14 and i+1 is at most 7
            coord -= unsafe { n_choose_k(index, i + 1) };

            // SAFTEY: index = 8 + i and i is at most 6
            unsafe{ assert_unchecked!(index < 15)};
            array[index as usize] = true;
        });

        array
    }

    /// Returns the I permutation from a coordinate
    pub const fn coord_to_i_permutation(coord: u16) -> [u8; 8] {
        // it is common that the coord is 0 so skip the calculation in this case
        if coord == 0 {
            return GenericPermutation::<8>::IDENTITY.into_array();
        }

        let is_odd = coord >= N_I_COORD_STATES / 2;
        let mut coord = coord % (N_I_COORD_STATES / 2);

        // SAFTEY: u8 does nothing when dropped and all values are overwritten.
        // We only ever read initialized values during initialization
        #[allow(invalid_value, clippy::uninit_assumed_init)]
        let mut permutation: [u8; 8] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        const_for!(i in (0..8).rev() => {
            let left = if !matches!(i, 0 | 1) {
                let denom = (factorial(i) / 2) as u16;

                // SAFTEY: factorials are positive
                unsafe{assert_unchecked!(denom > 0)};

                (coord / denom) as u8
            } else {
                coord as u8
            };

            let mut j = 0;

            while 7
                - left
                - {
                    let mut count = 0;
                    const_for!(k in i as usize+1..permutation.len() => {
                        if permutation[k] > j {
                            count += 1;
                        }
                    });
                    count
                }
                != j
            {
                j += 1;
            }

            // SAFTEY: the for loop guarantees that i < 8
            unsafe{assert_unchecked!(i < 8)};

            permutation[i as usize] = j;
            coord -= left as u16 * (factorial(i) / 2) as u16;
        });

        // SAFTEY: I have discovered a truly marvelous proof of this, which this comment is too short to contain
        let permutation = unsafe { GenericPermutation::from_array_unchecked(permutation) };

        if permutation.parity().is_odd() != is_odd {
            permutation.const_swap(0, 1).into_array()
        } else {
            permutation.into_array()
        }
    }

    /// Returns the O permutation from a coordinate
    pub const fn coord_to_o_permutation(coord: u16) -> [u8; 7] {
        // it is common that the coord is 0 so skip the calculation in this case
        if coord == 0 {
            return GenericPermutation::IDENTITY.into_array();
        }

        let is_odd = coord >= N_O_COORD_STATES / 2;
        let mut coord = coord % (N_O_COORD_STATES / 2);

        // SAFTEY: u8 does nothing when dropped and all values are overwritten.
        // We only ever read initialized values during initialization
        #[allow(invalid_value, clippy::uninit_assumed_init)]
        let mut permutation: [u8; 7] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        const_for!(i in (0..7).rev() => {
            let left = if !matches!(i, 0 | 1) {
                let denom = (factorial(i) / 2) as u16;

                // SAFTEY: factorials are positive
                unsafe{assert_unchecked!(denom > 0)};

                (coord / denom) as u8
            } else {
                coord as u8
            };

            let mut j = 0;
            while 6
                - left
                - {
                    let mut count = 0;
                    const_for!(k in i as usize+1..permutation.len() => {
                        if permutation[k] > j {
                            count += 1;
                        }
                    });
                    count
                }
                != j
            {
                j += 1;
            }

            // SAFTEY: the for loop guarantees that i < 7
            unsafe{assert_unchecked!(i < 7)};

            permutation[i as usize] = j;
            coord -= left as u16 * (factorial(i) / 2) as u16;
        });

        let permutation = unsafe { GenericPermutation::from_array_unchecked(permutation) };

        // SAFTEY: Same as before
        if permutation.parity().is_odd() != is_odd {
            permutation.const_swap(0, 1).into_array()
        } else {
            permutation.into_array()
        }
    }
}

/// Even or odd parity
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::EnumIs)]
pub enum Parity {
    Even,
    Odd,
}

impl std::ops::Mul for Parity {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(rhs)
    }
}

impl Parity {
    pub const fn mul(self, other: Self) -> Self {
        match self {
            Self::Even => other,
            Self::Odd => other.opposite(),
        }
    }

    pub const fn product<const N: usize>(array: [Self; N]) -> Self {
        let mut prod = Self::Even;

        const_for!(i in 0..N => {
            prod = prod.mul(array[i]);
        });

        prod
    }

    /// Returns the opposite parity
    pub const fn opposite(&self) -> Self {
        match self {
            Self::Even => Self::Odd,
            Self::Odd => Self::Even,
        }
    }
}
