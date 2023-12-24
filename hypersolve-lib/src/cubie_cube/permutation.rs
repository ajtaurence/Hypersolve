use itertools::Itertools;

use crate::math;
use crate::{
    node_cube::{N_IO_COORD_STATES, N_I_COORD_STATES, N_O_COORD_STATES},
    piece_cube::puzzle::PieceCube,
};

// TODO: Many of the functions here can be optimized especially with uninitialized memory

/// Describes the permutation of pieces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Permutation(pub [u8; 15]);

impl Default for Permutation {
    fn default() -> Self {
        Self::solved()
    }
}

impl From<PieceCube> for Permutation {
    fn from(cube: PieceCube) -> Self {
        // this gets the inverse of the map we want
        let inverse_map = cube
            .reposition()
            .pieces_except_last()
            .map(|piece| piece.current_location().index() as u8);

        // invert it to get the map we want
        Permutation(inverse_map).invert()
    }
}

impl Permutation {
    pub fn into_inner(self) -> [u8; 15] {
        self.0
    }

    /// Permutes this permutation by another
    pub fn permute(self, permutation: Permutation) -> Permutation {
        let mut map = [0; 15];

        for (i, &index) in permutation.into_inner().iter().enumerate() {
            map[i] = self.into_inner()[index as usize];
        }

        Permutation(map)
    }

    /// Returns the inverse of this permutation
    pub fn inverse(&self) -> Self {
        let mut inverse_map = [0; 15];

        for i in 0..15 {
            inverse_map[self.into_inner()[i] as usize] = i as u8;
        }

        Permutation(inverse_map)
    }

    /// Converts this permutation to its inverse
    pub fn invert(mut self) -> Self {
        self = self.inverse();
        self
    }

    /// Returns the solved permutation (identity)
    pub fn solved() -> Permutation {
        Permutation([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14])
    }

    /// Returns a number representing the IO separation state
    pub fn io_coord(self) -> u16 {
        // Find the indices of `self.map` representing pieces on I
        let indices: [u8; 7] = self
            .into_inner()
            .into_iter()
            .enumerate()
            .filter(|(_, piece)| *piece > 7)
            .map(|(i, _)| i as u8)
            .collect_vec()
            .try_into()
            .unwrap();

        (N_IO_COORD_STATES - 1)
            - (0..7)
                .map(|i| math::n_choose_k(indices[i], i as u8 + 1))
                .sum::<u16>()
    }

    /// Returns a number representing the permutation state of the I pieces
    pub fn i_coord(self) -> u16 {
        let i_map: [u8; 8] = self
            .into_inner()
            .into_iter()
            .filter(|val| *val < 8)
            .collect_vec()
            .try_into()
            .unwrap();

        let mut i_coord =
            if crate::groups::Permutation::from_array_unchecked(i_map.map(|i| i as usize))
                .parity()
                .is_odd()
            {
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
        let o_map: [u8; 7] = self
            .into_inner()
            .into_iter()
            .filter(|val| *val >= 8)
            .map(|val| val % 8)
            .collect_vec()
            .try_into()
            .unwrap();

        let mut o_coord =
            if crate::groups::Permutation::from_array_unchecked(o_map.map(|i| i as usize))
                .parity()
                .is_odd()
            {
                N_O_COORD_STATES / 2
            } else {
                0
            };

        for i in 2..7 {
            o_coord += o_map[0..i]
                .iter()
                .filter(|&&index| index > o_map[i])
                .count() as u16
                * (math::factorial(i as u8) / 2) as u16
        }

        o_coord
    }

    /// Returns a permutation from its coordinates
    pub fn from_coords(io_coord: u16, i_coord: u16, o_coord: u16) -> Permutation {
        let mut map: [u8; 15] = [0; 15];

        let io_array = Self::coord_to_io_permutation(io_coord);

        let i_array = Self::coord_to_i_permutation(i_coord);
        let o_array = Self::coord_to_o_permutation(o_coord);

        // Construct the total permutation array
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

        // Fix the total permutation parity by swapping the parity index of the I coordinate if needed
        if crate::groups::Permutation::<15>::from_array(map.map(|i| i as usize))
            .parity()
            .is_odd()
        {
            Permutation::from_coords(
                io_coord,
                (i_coord + N_I_COORD_STATES / 2) % N_I_COORD_STATES,
                o_coord,
            )
        } else {
            Permutation(map)
        }
    }

    /// Returns the IO permutation from a coordinate
    fn coord_to_io_permutation(coord: u16) -> [bool; 15] {
        // It is common that the coord is 0 so skip the calculation in this case
        if coord == 0 {
            let mut result = [false; 15];
            for value in result.iter_mut().skip(8) {
                *value = true;
            }
            return result;
        }

        let mut coord = N_IO_COORD_STATES - 1 - coord;

        let mut array: [bool; 15] = [false; 15];

        for i in (0..7).rev() {
            let mut index = 8 + i;

            while math::n_choose_k(index, i + 1) > coord {
                index -= 1;
            }

            coord -= math::n_choose_k(index, i + 1);
            array[index as usize] = true;
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

        let mut permutation = [0_u8; 8];

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

        if crate::groups::Permutation::from_array_unchecked(permutation.map(|i| i as usize))
            .parity()
            .is_odd()
            != is_odd
        {
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

        let mut permutation = [0_u8; 7];

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

        if crate::groups::Permutation::from_array_unchecked(permutation.map(|i| i as usize))
            .parity()
            .is_odd()
            != is_odd
        {
            permutation.swap(0, 1);
        }

        permutation
    }
}
