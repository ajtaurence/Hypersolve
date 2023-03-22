//! Cube representation based on a permutation of all pieces and an orientation
//! for each piece.

use std::ops::Mul;

use bytemuck::{Pod, Zeroable};
use num_traits::FromPrimitive;

use crate::{
    groups::{Identity, A4, C3, K4},
    math,
    node::{N_IO_COORD_STATES, N_I_COORD_STATES, N_O_COORD_STATES},
    permutations,
};

const_lookup_table!(PERM_MOVE_TABLE: &'static [Permutation] = ("perm.move", gen_perm_move_table));
const_lookup_table!(A4_MOVE_TABLE: &'static [Orientation<A4>] = ("a4.move", gen_a4_move_table));

/// Calculates the permutation move table using matrixcube
fn gen_perm_move_table() -> &'static [Permutation] {
    todo!()
}

/// Calculates the A4 orientation move table using matrixcube
fn gen_a4_move_table() -> &'static [Orientation<A4>] {
    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permutation {
    map: [u8; 15],
}

unsafe impl Zeroable for Permutation {
    fn zeroed() -> Self {
        Permutation { map: [0; 15] }
    }
}
unsafe impl Pod for Permutation {}

impl Default for Permutation {
    fn default() -> Self {
        Self::solved()
    }
}

impl Permutation {
    /// Permutes this permutation by another
    pub fn permute(self, permutation: Permutation) -> Permutation {
        let mut map = [0; 15];

        for (i, &index) in permutation.map.iter().enumerate() {
            map[i] = self.map[index as usize];
        }

        Permutation { map }
    }

    /// Returns whether the permutation is solved
    pub fn is_solved(self) -> bool {
        self == Self::solved()
    }

    /// Returns the solved permutation (identity)
    pub fn solved() -> Permutation {
        Permutation {
            map: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
        }
    }

    /// Returns a number representing the IO separation state
    pub fn io_coord(self) -> u16 {
        let mut indices: [u8; 7] = [0; 7];

        // Find the indices of `self.map` representing pieces on I
        let mut index = 0;
        for i in 0..15 {
            if self.map[i] > 7 {
                indices[index] = i as u8;
                index += 1;
            }
        }

        (N_IO_COORD_STATES - 1)
            - (0..7)
                .map(|i| math::n_choose_k(indices[i], i as u8 + 1))
                .sum::<u16>()
    }

    /// Returns a number representing the permutation state of the I pieces
    pub fn i_coord(self) -> u16 {
        let mut i_map: [u8; 8] = [0; 8];

        for i in 0..8 {
            i_map[i] = self.map[i] % 8;
        }

        let mut i_coord = if permutations::is_odd(i_map) {
            N_I_COORD_STATES / 2
        } else {
            0
        };

        for i in 2..8 {
            i_coord += i_map[0..i]
                .iter()
                .filter(|&&index| index > i_map[i])
                .count() as u16
                * (math::factorial(i as u8) / 2) as u16
        }

        i_coord
    }

    /// Returns a number representing the permutation state of the O pieces
    pub fn o_coord(self) -> u16 {
        let mut o_map: [u8; 7] = [0; 7];

        for i in 8..15 {
            o_map[i - 8] = self.map[i] % 8;
        }

        let mut i_coord = if permutations::is_odd(o_map) {
            N_O_COORD_STATES / 2
        } else {
            0
        };

        for i in 2..7 {
            i_coord += o_map[0..i]
                .iter()
                .filter(|&&index| index > o_map[i])
                .count() as u16
                * (math::factorial(i as u8) / 2) as u16
        }

        i_coord
    }

    /// Returns a permutation from its coordinates
    pub fn from_coords(io_coord: u16, i_coord: u16, o_coord: u16) -> Permutation {
        let mut map: [u8; 15] = [0; 15];

        let io_array = Self::coord_to_io_permutation(io_coord);

        let i_array = Self::coord_to_i_permutation(i_coord);
        let o_array = Self::coord_to_o_permutation(o_coord);

        let mut i_index: usize = 0;
        let mut o_index: usize = 0;
        for i in 0..15 {
            if io_array[i] {
                map[i] = o_array[o_index] + 8;
                o_index += 1;
            } else {
                map[i] = i_array[i_index];
                i_index += 1;
            }
        }

        Permutation { map }
    }

    /// Returns the IO permutation from a coordinate
    fn coord_to_io_permutation(coord: u16) -> [bool; 15] {
        // It is common that the coord is 0 so skip the calculation in this case
        if coord == 0 {
            let mut result = [false; 15];
            for i in 8..15 {
                result[i] = true;
            }
            return result;
        }

        let mut coord = N_IO_COORD_STATES - coord;

        let mut combination: [u8; 7] = [0; 7];

        for i in (0..7).rev() {
            let mut index = 8 + i;

            while math::n_choose_k(index, i + 1) > coord {
                index -= 1;
            }

            coord -= math::n_choose_k(index, i + 1);
            combination[i as usize] = index as u8;
        }

        let mut array: [bool; 15] = [false; 15];
        for i in 0..7 {
            array[combination[i] as usize] = true;
        }

        array
    }

    /// Returns the I permutation from a coordinate
    pub fn coord_to_i_permutation(coord: u16) -> [u8; 8] {
        // it is common that the coord is 0 so skip the calculation in this case
        if coord == 0 {
            return [0, 1, 2, 3, 4, 5, 6, 7];
        }

        let is_odd = coord >= N_I_COORD_STATES / 2;
        let mut coord = coord % (N_I_COORD_STATES / 2);

        let mut permutation = [0 as u8; 8];

        for i in (0..8).rev() {
            let left = if i != 0 && i != 1 {
                (coord / (math::factorial(i) / 2) as u16) as u8
            } else {
                coord as u8
            };

            let mut j = 0;
            while 7
                - left
                - permutation[i as usize + 1..]
                    .iter()
                    .filter(|&&x| x > j)
                    .count() as u8
                != j
            {
                j += 1;
            }

            permutation[i as usize] = j;
            coord -= left as u16 * (math::factorial(i) / 2) as u16;
        }

        if permutations::is_odd(permutation) != is_odd {
            permutation.swap(0, 1);
        }

        permutation
    }

    /// Returns the O permutation from a coordinate
    pub fn coord_to_o_permutation(coord: u16) -> [u8; 7] {
        // it is common that the coord is 0 so skip the calculation in this case
        if coord == 0 {
            return [0, 1, 2, 3, 4, 5, 6];
        }

        let is_odd = coord >= N_O_COORD_STATES / 2;
        let mut coord = coord % (N_O_COORD_STATES / 2);

        let mut permutation = [0 as u8; 7];

        for i in (0..7).rev() {
            let left = if i != 0 && i != 1 {
                (coord / (math::factorial(i) / 2) as u16) as u8
            } else {
                coord as u8
            };

            let mut j = 0;
            while 6
                - left
                - permutation[i as usize + 1..]
                    .iter()
                    .filter(|&&x| x > j)
                    .count() as u8
                != j
            {
                j += 1;
            }

            permutation[i as usize] = j;
            coord -= left as u16 * (math::factorial(i) / 2) as u16;
        }

        if permutations::is_odd(permutation) != is_odd {
            permutation.swap(0, 1);
        }

        permutation
    }
}

#[derive(PartialEq, Default, Clone, Copy)]
pub struct Orientation<T> {
    pub state: [T; 15],
}

unsafe impl<T: Identity + Copy + PartialEq> Zeroable for Orientation<T> {
    fn zeroed() -> Self {
        Orientation::solved()
    }
}
unsafe impl<T: PartialEq + Identity + Copy + 'static> Pod for Orientation<T> {}

impl<T: Identity + Copy + PartialEq> Orientation<T> {
    pub fn is_solved(self) -> bool {
        self == Self::solved()
    }

    pub fn solved() -> Orientation<T> {
        Orientation {
            state: [T::identity(); 15],
        }
    }

    pub fn permute(self, permutation: Permutation) -> Orientation<T> {
        let mut result = [T::identity(); 15];

        for (i, &index) in permutation.map.iter().enumerate() {
            result[i] = self.state[index as usize];
        }

        Orientation { state: result }
    }

    pub fn apply_orientation<U>(self, action: Orientation<U>) -> Orientation<T::Output>
    where
        U: Copy,
        T: Mul<U>,
        T::Output: Identity + Copy,
    {
        let mut result = [T::Output::identity(); 15];

        for i in 0..15 {
            result[i] = self.state[i] * action.state[i];
        }

        Orientation { state: result }
    }
}

macro_rules! impl_convert_orientation {
    (($T:ty) -> ($U:ty)) => {
        impl From<Orientation<$U>> for Orientation<$T> {
            fn from(value: Orientation<$U>) -> Self {
                let mut result = [<$T>::identity(); 15];

                for i in 0..15 {
                    result[i] = value.state[i].into();
                }

                Orientation { state: result }
            }
        }
    };
}

impl_convert_orientation!((A4) -> (K4));
impl_convert_orientation!((A4) -> (C3));
impl_convert_orientation!((K4) -> (A4));
impl_convert_orientation!((C3) -> (A4));

impl<T: Into<K4>> Orientation<T> {
    /// Returns the index into K4 the pruning table
    pub fn k4_coord(self) -> u32 {
        self.state
            .into_iter()
            .enumerate()
            .map(|(i, value)| (Into::<K4>::into(value) as u32) << (2 * i))
            .product()
    }
}

impl Orientation<K4> {
    /// Returns an orientation state from a K4 pruning table index
    pub fn from_k4_coord(int: u32) -> Orientation<K4> {
        let mut result = [K4::E; 15];

        for i in 0..15 {
            result[i] = K4::from_u32((int >> (2 * i)) & 3).unwrap();
        }

        Orientation::<K4> { state: result }
    }
}

impl<T: Into<C3>> Orientation<T> {
    /// Returns the index into the C3 move table
    pub fn c3_coord(self) -> u32 {
        let mut result: u32 = 0;

        for i in 0..14 {
            result += Into::<C3>::into(self.state[i]) as u32 * u32::pow(2, i as u32);
        }

        result
    }
}

impl Orientation<C3> {
    /// Returns an orientation state from a C3 move table index
    pub fn from_c3_coord(mut c3_coord: u32) -> Orientation<C3> {
        let mut result = [C3::identity(); 15];

        let mut coord = c3_coord.clone();

        for i in 0..14 {
            result[i] = C3::from_u32(coord % 3).unwrap();
            coord /= 3;
        }

        result[14] =
            C3::from_u32(result[0..14].iter().map(|&value| value as u32).sum::<u32>() % 3).unwrap();

        Orientation { state: result }
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct CubieCube {
    pub orientation: Orientation<A4>,
    pub permutation: Permutation,
}

impl Mul for CubieCube {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        CubieCube {
            orientation: self
                .orientation
                .permute(rhs.permutation)
                .apply_orientation(rhs.orientation),
            permutation: self.permutation.permute(rhs.permutation),
        }
    }
}

impl CubieCube {
    /// Returns the solved state
    pub fn solved() -> CubieCube {
        CubieCube::default()
    }

    /// Applies the given move to the cubiecube
    pub fn apply_move(self, i: usize) -> CubieCube {
        CubieCube {
            orientation: self
                .orientation
                .permute(PERM_MOVE_TABLE[i])
                .apply_orientation(A4_MOVE_TABLE[i]),
            permutation: self.permutation.permute(PERM_MOVE_TABLE[i]),
        }
    }
}
