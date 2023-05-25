//! Cube representation based on a permutation of all pieces and an orientation
//! for each piece.

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut, Mul, Range},
};

use num_traits::FromPrimitive;

use crate::{
    common::Axis,
    groups::{Identity, A4, C3, K4},
    math,
    node_cube::node::{MOVE_AXIS, N_IO_COORD_STATES, N_I_COORD_STATES, N_O_COORD_STATES},
    phases::{Phase, Phase1},
    piece_cube::{puzzle::PieceCube, twist::Twist},
};

const_data!(pub HYPERSOLVE_TWISTS: [Twist; Phase1::N_MOVES] =  gen_hypersolve_twists());
const_data!(pub PERM_MOVE_TABLE: [Permutation; Phase1::N_MOVES] =  gen_perm_move_table());
const_data!(pub A4_MOVE_TABLE: [Orientation<A4>; Phase1::N_MOVES] =  gen_a4_move_table());

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_hypersolve_twists() {
    let _ = &*HYPERSOLVE_TWISTS;
}

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_perm_move_table() {
    let _ = &*PERM_MOVE_TABLE;
}

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_a4_move_table() {
    let _ = &*A4_MOVE_TABLE;
}

/// Calculates the twists that hypersolve uses to solve the cube
#[cfg(feature = "gen-const-data")]
fn gen_hypersolve_twists() -> Box<[Twist; Phase1::N_MOVES]> {
    use crate::piece_cube::pieces::PieceLocation;
    use itertools::Itertools;
    // Generate twist which dont affect LDBO (index 15) and perform unique actions on a cube
    let twists = Twist::iter_all_twists()
        .filter(|&twist| !PieceLocation::from_index(15).is_affected_by_twist(twist))
        .unique_by(|&twist| PieceCube::solved().twist(twist))
        .collect_vec();

    // Order the twists by the phase
    let justphase1twists = twists
        .clone()
        .into_iter()
        .filter(|&twist| Orientation::<A4>::from(PieceCube::solved().twist(twist)).k4_coord() != 0);

    let justphase2twists = twists.clone().into_iter().filter(|&twist| {
        let cube = PieceCube::solved().twist(twist);
        Orientation::<A4>::from(cube).k4_coord() == 0
            && (Permutation::from(cube).io_coord() != 0
                || Orientation::<C3>::from(cube).c3_coord() != 0)
    });

    let justphase3twists = twists.into_iter().filter(|&twist| {
        let cube = PieceCube::solved().twist(twist);
        Orientation::<A4>::from(cube).k4_coord() == 0
            && Permutation::from(cube).io_coord() == 0
            && Orientation::<C3>::from(cube).c3_coord() == 0
    });

    // Combine all twists into a list in order of phase
    justphase3twists
        .chain(justphase2twists)
        .chain(justphase1twists)
        .collect_vec()
        .try_into()
        .unwrap()
}

/// Calculates the permutation move table using piece_cube
#[cfg(feature = "gen-const-data")]
fn gen_perm_move_table() -> Box<[Permutation; Phase1::N_MOVES]> {
    use itertools::Itertools;
    HYPERSOLVE_TWISTS
        .iter()
        .map(|&twist| PieceCube::solved().twist(twist).into())
        .collect_vec()
        .try_into()
        .unwrap()
}

/// Calculates the A4 orientation move table using piece_cube
#[cfg(feature = "gen-const-data")]
fn gen_a4_move_table() -> Box<[Orientation<A4>; Phase1::N_MOVES]> {
    use itertools::Itertools;
    HYPERSOLVE_TWISTS
        .iter()
        .map(|&twist| PieceCube::solved().twist(twist).into())
        .collect_vec()
        .try_into()
        .unwrap()
}

/// Hypersolve move index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move(pub u8);

impl Deref for Move {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Move {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Move {
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn axis(&self) -> Axis {
        MOVE_AXIS[self.as_usize()]
    }
}

pub struct MoveIterator {
    moves: Range<Move>,
    current_move: Move,
}

impl MoveIterator {
    pub const fn new(moves: Range<Move>) -> Self {
        Self {
            moves,
            current_move: Move(0),
        }
    }
}

impl Iterator for MoveIterator {
    type Item = Move;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_move == self.moves.end {
            return None;
        } else {
            let result = self.current_move;
            self.current_move = Move(self.current_move.as_u8() + 1);
            return Some(result);
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.moves.end.as_usize() - self.current_move.as_usize();
        (remaining, Some(remaining))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Permutation {
    pub map: [u8; 15],
}

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
        Permutation { map: inverse_map }.invert()
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

    /// Returns the inverse of this permutation
    pub fn inverse(&self) -> Self {
        let mut inverse_map = [0; 15];

        for i in 0..15 {
            inverse_map[self.map[i] as usize] = i as u8;
        }

        Permutation { map: inverse_map }
    }

    /// Converts this permutation to its inverse
    pub fn invert(mut self) -> Self {
        self = self.inverse();
        self
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
        let mut o_map: [u8; 7] = [0; 7];

        for i in 8..15 {
            o_map[i - 8] = self.map[i] % 8;
        }

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

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct Orientation<T> {
    pub state: [T; 15],
}

impl<T: Identity> Identity for Orientation<T>
where
    T: Identity,
{
    const IDENTITY: Self = Orientation {
        state: [T::IDENTITY; 15],
    };
}

impl<T: Identity + Debug + Copy + PartialEq + From<A4>> From<PieceCube> for Orientation<T> {
    fn from(cube: PieceCube) -> Self {
        Orientation {
            state: cube
                .reposition()
                .pieces_except_last()
                .map(|piece| A4::from(piece).into()),
        }
        .permute(Permutation::from(cube))
    }
}

impl<T: Identity + Copy + PartialEq> Orientation<T> {
    pub fn is_solved(self) -> bool {
        self == Self::solved()
    }

    pub fn solved() -> Orientation<T> {
        Orientation {
            state: [T::IDENTITY; 15],
        }
    }

    pub fn permute(self, permutation: Permutation) -> Orientation<T> {
        let mut result = [T::IDENTITY; 15];

        for (i, &index) in permutation.map.iter().enumerate() {
            result[i] = self.state[index as usize];
        }

        Orientation { state: result }
    }

    pub fn apply_orientation<U>(self, action: Orientation<U>) -> Orientation<U::Output>
    where
        U: Copy,
        U: Mul<T>,
        U::Output: Identity + Copy,
    {
        let mut result = [U::Output::IDENTITY; 15];

        for i in 0..15 {
            result[i] = action.state[i] * self.state[i];
        }

        Orientation { state: result }
    }
}

macro_rules! impl_convert_orientation {
    (($T:ty) -> ($U:ty)) => {
        impl From<Orientation<$U>> for Orientation<$T> {
            fn from(value: Orientation<$U>) -> Self {
                let mut result = [<$T>::IDENTITY; 15];

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
            .sum()
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

impl<T: Copy> Orientation<T>
where
    C3: From<T>,
{
    /// Returns the index into the C3 move table
    pub fn c3_coord(self) -> u32 {
        let mut result: u32 = 0;

        for i in 0..14 {
            result += C3::from(self.state[i]) as u32 * u32::pow(3, i as u32);
        }

        result
    }
}

impl Orientation<C3> {
    /// Returns an orientation state from a C3 move table index
    pub fn from_c3_coord(c3_coord: u32) -> Orientation<C3> {
        let mut result = [C3::IDENTITY; 15];

        let mut coord = c3_coord;

        for i in 0..14 {
            result[i] = C3::from_u32(coord % 3).unwrap();
            coord /= 3;
        }

        // fix parity of the last piece
        let sum = result[0..14].iter().map(|&value| value as i32).sum::<i32>();
        result[14] = C3::from_i32((-sum).rem_euclid(3)).unwrap();

        Orientation { state: result }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CubieCube {
    pub orientation: Orientation<A4>,
    pub permutation: Permutation,
}

impl From<PieceCube> for CubieCube {
    fn from(cube: PieceCube) -> Self {
        CubieCube {
            orientation: cube.into(),
            permutation: cube.into(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubie_cube_twists() {
        for i in 0..HYPERSOLVE_TWISTS.len() {
            let cubiecube = CubieCube::from(PieceCube::solved()).apply_move(i);
            let piececube = PieceCube::solved().twist(HYPERSOLVE_TWISTS[i]);

            assert!(cubiecube == CubieCube::from(piececube))
        }
    }

    #[test]
    fn test_phase2_pruning_table_with_cubie_cube() {
        use crate::node_cube::node::{Node, Phase2Node};
        use crate::phases::Phase2;
        use itertools::Itertools;
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Phase2Node::goal().get_index());

        let mut one_move_states = Vec::new();

        (0..Phase2::N_MOVES).into_iter().for_each(|move_index| {
            let cube = CubieCube::solved().apply_move(move_index);
            let phase2_node = Phase2Node::from(cube);
            one_move_states.push(cube);
            set.insert(phase2_node.get_index());
        });

        one_move_states
            .into_iter()
            .cartesian_product(0..Phase2::N_MOVES)
            .for_each(|(cube, move_index)| {
                let cube = cube.apply_move(move_index);
                let phase2_node = Phase2Node::from(cube);
                set.insert(phase2_node.get_index());
            });

        // Should have found 152 nodes
        assert_eq!(set.len(), 152)
    }
}
