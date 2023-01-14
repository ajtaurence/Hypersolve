use std::ops::Mul;
use std::usize;

use num_traits::FromPrimitive;

use crate::groups::{A4Elem, C3Elem, K4Elem};
use crate::init::get_move_axes;
use crate::{phase1::N_PHASE1_MOVES, puzzle::AxisEnum};

pub const MOVE_AXIS: [AxisEnum; N_PHASE1_MOVES as usize] = get_move_axes();

#[repr(u8)]
pub enum PhaseEnum {
    Phase1,
    Phase2,
    Phase3,
}

pub fn n_choose_k(n: u8, k: u8) -> u16 {
    (FACTORIAL[n as usize] / (FACTORIAL[k as usize] * FACTORIAL[(n - k) as usize])) as u16
}

pub fn cycle_length(permutation: &[u8], start: u8, visited: &mut [bool]) -> u8 {
    let mut count = 1;
    visited[start as usize] = true;
    let mut index = permutation[start as usize];
    visited[index as usize] = true;

    while index != start {
        index = permutation[index as usize];
        visited[index as usize] = true;
        count += 1;
    }

    count
}

pub fn permutation_parity(permutation: &[u8]) -> bool {
    let length = permutation.len();
    let mut visited = vec![false; length];

    let mut even_parity = true;

    for i in 0..length {
        if !visited[i] {
            even_parity ^= cycle_length(permutation, i as u8, &mut visited) % 2 == 0;
        }
    }

    even_parity
}

// Returns an array of 0's where the I sticker is there and 8's where the O sticker is there
pub fn coord_to_io_permutation(coord: u16) -> [bool; 15] {
    // it is common that the coord is 0 so skip the calculation in this case
    if coord == 0 {
        let mut result = [false; 15];
        for i in 8..15 {
            result[i] = true;
        }
        return result;
    }

    let mut coord = 6434 - coord;

    let mut combination: [u8; 7] = [0; 7];

    for i in (0..7).rev() {
        let mut index = 8 + i;

        while n_choose_k(index, i + 1) > coord {
            index -= 1;
        }

        coord -= n_choose_k(index, i + 1);
        combination[i as usize] = index as u8;
    }

    let mut array: [bool; 15] = [false; 15];
    for i in 0..7 {
        array[combination[i] as usize] = true;
    }

    array
}

pub fn coord_to_i_permutation(coord: u16) -> [u8; 8] {
    // it is common that the coord is 0 so skip the calculation in this case
    if coord == 0 {
        return [0, 1, 2, 3, 4, 5, 6, 7];
    }

    let (parity, mut coord) = ((coord / 20160) == 0, coord % 20160);

    let mut permutation = [0 as u8; 8];

    for i in (0..8).rev() {
        let left = if i != 0 && i != 1 {
            (coord / (FACTORIAL[i] / 2) as u16) as u8
        } else {
            coord as u8
        };

        let mut j = 0;
        while 7 - left - permutation[i + 1..].iter().filter(|&&x| x > j).count() as u8 != j {
            j += 1;
        }

        permutation[i] = j;
        coord -= left as u16 * (FACTORIAL[i] / 2) as u16;
    }

    if permutation_parity(&permutation) == parity {
        permutation.swap(0, 1);
    }

    permutation
}

pub fn coord_to_o_permutation(coord: u16) -> [u8; 7] {
    // it is common that the coord is 0 so skip the calculation in this case
    if coord == 0 {
        return [0, 1, 2, 3, 4, 5, 6];
    }

    let (parity, mut coord) = ((coord / 2520) == 0, coord % 2520);

    let mut permutation = [0 as u8; 7];

    for i in (0..7).rev() {
        let left = if i != 0 && i != 1 {
            (coord / (FACTORIAL[i] / 2) as u16) as u8
        } else {
            coord as u8
        };

        let mut j = 0;
        while 6 - left - permutation[i + 1..].iter().filter(|&&x| x > j).count() as u8 != j {
            j += 1;
        }

        permutation[i] = j;
        coord -= left as u16 * (FACTORIAL[i] / 2) as u16;
    }

    if permutation_parity(&permutation) == parity {
        permutation.swap(0, 1);
    }

    permutation
}
