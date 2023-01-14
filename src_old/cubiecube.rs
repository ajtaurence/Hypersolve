use std::ops::Mul;

use num_traits::FromPrimitive;

use crate::{
    groups::{A4Elem, C3Elem, Identity, K4Elem},
    init::get_move_cubiecubes,
    utils::{
        coord_to_i_permutation, coord_to_io_permutation, coord_to_o_permutation,
        permutation_parity, FACTORIAL,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct Permutation {
    pub map: [u8; 15],
}


impl Permutation {
    #[inline]
    pub fn permute(&self, permutation: &Permutation) -> Permutation {
        let mut result = [0; 15];

        for (i, &index) in permutation.map.iter().enumerate() {
            result[i] = self.map[index as usize];
        }

        Permutation { map: result }
    }

    pub fn is_solved(&self) -> bool {
        for value in self.map.iter() {
            if *value != 0 {
                return false;
            }
        }
        return true;
    }

    pub fn solved() -> Permutation {
        Permutation { map: [0; 15] }
    }

    pub fn io_coord(&self) -> u16 {
        let mut indices: [i64; 7] = [0; 7];

        let mut index = 0;
        for i in 0..15 {
            if self.map[i] > 7 {
                indices[index] = i as i64;
                index += 1;
            }
        }

        (6434
            - indices[0]
            - (-1 + indices[1]) * indices[1] / 2
            - (-2 + indices[2]) * (-1 + indices[2]) * indices[2] / 6
            - (-3 + indices[3]) * (-2 + indices[3]) * (-1 + indices[3]) * indices[3] / 24
            - (-4 + indices[4])
                * (-3 + indices[4])
                * (-2 + indices[4])
                * (-1 + indices[4])
                * indices[4]
                / 120
            - (-5 + indices[5])
                * (-4 + indices[5])
                * (-3 + indices[5])
                * (-2 + indices[5])
                * (-1 + indices[5])
                * indices[5]
                / 720
            - (-6 + indices[6])
                * (-5 + indices[6])
                * (-4 + indices[6])
                * (-3 + indices[6])
                * (-2 + indices[6])
                * (-1 + indices[6])
                * indices[6]
                / 5040)
            .try_into()
            .unwrap()
    }

    pub fn i_coord(&self) -> u16 {
        let mut i_map: [u8; 8] = [0; 8];

        for i in 0..8 {
            i_map[i] = self.map[i] % 8;
        }

        let mut i_coord = if permutation_parity(&i_map) { 20160 } else { 0 };

        for i in 2..8 {
            i_coord += i_map[0..i]
                .iter()
                .filter(|&&index| index > i_map[i])
                .count() as u16
                * (FACTORIAL[i] / 2) as u16
        }

        i_coord
    }

    pub fn o_coord(&self) -> u16 {
        let mut o_map: [u8; 7] = [0; 7];

        for i in 8..15 {
            o_map[i - 8] = self.map[i] % 8;
        }

        let mut i_coord = if permutation_parity(&o_map) { 2520 } else { 0 };

        for i in 2..7 {
            i_coord += o_map[0..i]
                .iter()
                .filter(|&&index| index > o_map[i])
                .count() as u16
                * (FACTORIAL[i] / 2) as u16
        }

        i_coord
    }

    pub fn from_coords(io_coord: u16, i_coord: u16, o_coord: u16) -> Permutation {
        let mut array: [u8; 15] = [0; 15];

        let io_array = coord_to_io_permutation(io_coord);

        let i_array = coord_to_i_permutation(i_coord);
        let o_array = coord_to_o_permutation(o_coord);

        let mut i_index: usize = 0;
        let mut o_index: usize = 0;
        for i in 0..15 {
            if io_array[i] {
                array[i] = o_array[o_index] + 8;
                o_index += 1;
            } else {
                array[i] = i_array[i_index];
                i_index += 1;
            }
        }

        Permutation { map: array }
    }
}

#[derive(PartialEq, Default, Clone, Copy)]
pub struct Orientation<T: Mul<A4Elem>> {
    pub state: [T; 15],
}

impl<T: Mul<A4Elem, Output = T> + Identity<T> + Copy + PartialEq> Orientation<T> {
    #[inline]
    pub fn permute(&self, permutation: &Permutation) -> Orientation<T> {
        let mut result = [T::identity(); 15];

        for (i, &index) in permutation.map.iter().enumerate() {
            result[i] = self.state[index as usize];
        }

        Orientation { state: result }
    }

    #[inline]
    pub fn apply_orientation(&self, action: &Orientation<A4Elem>) -> Orientation<T> {
        let mut result = [T::identity(); 15];

        for i in 0..15 {
            result[i] = self.state[i] * action.state[i];
        }

        Orientation { state: result }
    }

    #[inline]
    pub fn is_solved(&self) -> bool {
        for value in self.state.iter() {
            if *value != T::identity() {
                return false;
            }
        }
        return true;
    }

    #[inline]
    pub fn solved() -> Orientation<T> {
        Orientation {
            state: [T::identity(); 15],
        }
    }
}

impl Orientation<A4Elem> {
    pub fn to_c3(&self) -> Orientation<C3Elem> {
        let mut result = [C3Elem::identity(); 15];

        for i in 0..15 {
            result[i] = self.state[i].into();
        }

        Orientation { state: result }
    }
}

impl Orientation<K4Elem> {
    #[inline]
    pub fn to_int(&self) -> u32 {
        self.state
            .iter()
            .enumerate()
            .fold(0, |result, (i, value)| result | (*value as u32) << 2 * i)
    }

    #[inline]
    pub fn from_int(int: u32) -> Orientation<K4Elem> {
        let mut result = [K4Elem::E; 15];

        for i in 0..15 {
            result[i] = K4Elem::from_u32((int >> 2 * i) & 3).unwrap();
        }

        Orientation::<K4Elem> { state: result }
    }
}

impl Orientation<C3Elem> {
    pub fn c3_coord(&self) -> u32 {
        let mut result: u32 = 0;

        for i in 0..14 {
            result += self.state[i] as u32 * u32::pow(2, i as u32);
        }

        result
    }

    pub fn from_c3_coord(mut c3_coord: u32) -> Orientation<C3Elem> {
        let mut orientation = [C3Elem::identity(); 15];

        let mut coord = c3_coord.clone();

        for i in 0..14 {
            orientation[i] = C3Elem::from_u32(coord % 3).unwrap();
            coord /= 3;
        }

        orientation[14] = C3Elem::from_u32(
            orientation[0..14]
                .iter()
                .map(|value| *value as u32)
                .sum::<u32>()
                % 3,
        )
        .unwrap();

        Orientation { state: orientation }
    }
}

static CUBIECUBE_MOVES: [CubieCube; 92] = get_move_cubiecubes();

pub struct CubieCube {
    pub orientation: Orientation<A4Elem>,
    pub permutation: Permutation,
}

impl CubieCube {
    pub fn solved() -> CubieCube {
        CubieCube { orientation: Orientation::<A4Elem>::solved(), permutation: Permutation::solved() }
    }

    pub fn multiply(&self, other: &CubieCube) -> CubieCube {
        CubieCube {
            orientation: self
                .orientation
                .permute(&other.permutation)
                .apply_orientation(&other.orientation),
            permutation: self.permutation.permute(&other.permutation),
        }
    }

    pub fn apply_move(&self, i: usize) -> CubieCube {
        self.multiply(&CUBIECUBE_MOVES[i])
    }
}
