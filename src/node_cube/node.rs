use crate::{
    common::Axis,
    cubie_cube::{
        CubieCube, Move, MoveIterator, Orientation, Permutation, A4_MOVE_TABLE, PERM_MOVE_TABLE,
    },
    groups::{Identity, K4},
    math,
    phases::{Phase, Phase1, Phase2, Phase3},
};
use itertools::Itertools;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

pub const N_K4_COORD_STATES: u32 = 4_u32.pow(15);
pub const N_C3_COORD_STATES: u32 = 3_u32.pow(14);
pub const N_IO_COORD_STATES: u16 = math::n_choose_k(15, 7);
pub const N_I_COORD_STATES: u16 = math::factorial(8) as u16;
pub const N_O_COORD_STATES: u16 = math::factorial(7) as u16;

const_data!(pub IO_MOVE_TABLE: [[u16; Phase2::N_MOVES ]; N_IO_COORD_STATES as usize] = gen_io_move_table());
const_data!(pub I_MOVE_TABLE: [[u16;  Phase3::N_MOVES]; N_I_COORD_STATES as usize] = gen_i_move_table());
const_data!(pub O_MOVE_TABLE: [[u16;  Phase3::N_MOVES]; N_O_COORD_STATES as usize] = gen_o_move_table());
const_data!(pub MOVE_AXIS: [Axis;  Phase1::N_MOVES] = gen_move_axis_table());

runtime_data!("C3.MOVE", pub static C3_MOVE_TABLE: Box<[[u32; Phase2::N_MOVES as usize]; N_C3_COORD_STATES as usize]> = gen_c3_move_table());

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_io_move_table() {
    let _ = &*IO_MOVE_TABLE;
}

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_i_move_table() {
    let _ = &*I_MOVE_TABLE;
}

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_o_move_table() {
    let _ = &*O_MOVE_TABLE;
}

#[cfg(feature = "gen-const-data")]
#[test]
fn generate_move_axis_table() {
    let _ = &*MOVE_AXIS;
}

#[cfg(feature = "gen-const-data")]
fn gen_move_axis_table() -> Box<[Axis; Phase1::N_MOVES]> {
    use crate::cubie_cube::HYPERSOLVE_TWISTS;
    HYPERSOLVE_TWISTS
        .iter()
        .map(|&twist| twist.axis())
        .collect_vec()
        .try_into()
        .unwrap()
}

fn gen_c3_move_table() -> Box<[[u32; Phase2::N_MOVES]; N_C3_COORD_STATES as usize]> {
    let mut table = vec![[0_u32; Phase2::N_MOVES]; N_C3_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let cube = CubieCube {
            orientation: Orientation::from_c3_coord(i as u32).into(),
            permutation: Permutation::solved(),
        };

        for j in 0..(Phase2::N_MOVES) {
            entry[j] = cube.apply_move(j).orientation.c3_coord();
        }
    });

    table.try_into().unwrap()
}

#[cfg(feature = "gen-const-data")]
fn gen_io_move_table() -> Box<[[u16; Phase2::N_MOVES]; N_IO_COORD_STATES as usize]> {
    let mut table = vec![[0_u16; Phase2::N_MOVES]; N_IO_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(i as u16, 0, 0),
        };

        for j in 0..(Phase2::N_MOVES as usize) {
            entry[j] = cube.apply_move(j).permutation.io_coord();
        }
    });

    table.try_into().unwrap()
}

#[cfg(feature = "gen-const-data")]
fn gen_i_move_table() -> Box<[[u16; Phase3::N_MOVES]; N_I_COORD_STATES as usize]> {
    let mut table = vec![[0_u16; Phase3::N_MOVES]; N_I_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(0, i as u16, 0),
        };

        for j in 0..(Phase3::N_MOVES) {
            entry[j] = cube.apply_move(j).permutation.i_coord();
        }
    });

    table.try_into().unwrap()
}

#[cfg(feature = "gen-const-data")]
fn gen_o_move_table() -> Box<[[u16; Phase3::N_MOVES]; N_O_COORD_STATES as usize]> {
    let mut table = vec![[0_u16; Phase3::N_MOVES]; N_O_COORD_STATES as usize];

    table.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let cube = CubieCube {
            orientation: Orientation::solved(),
            permutation: Permutation::from_coords(0, 0, i as u16),
        };

        for j in 0..(Phase3::N_MOVES) {
            entry[j] = cube.apply_move(j).permutation.o_coord();
        }
    });

    table.try_into().unwrap()
}

pub struct NodeIterator<N: Node> {
    node: N,
    move_iter: MoveIterator,
}

impl<N: Node> NodeIterator<N> {
    pub const fn new(node: N) -> NodeIterator<N> {
        NodeIterator {
            node,
            move_iter: N::Phase::MOVE_ITERATOR,
        }
    }
}

impl<N: Node> Iterator for NodeIterator<N> {
    type Item = N;
    fn next(&mut self) -> Option<Self::Item> {
        self.move_iter.next().map(|m| self.node.apply_move(m))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.move_iter.size_hint()
    }
}

pub trait Node: Identity + PartialEq + Copy + From<CubieCube> {
    const N_STATES: usize;

    type Phase: Phase;

    /// Returns the index of the node
    fn get_index(&self) -> usize;

    /// Returns a node from an index
    fn from_index(index: usize, last_move: Option<Move>) -> Self;

    /// Returns a node from an index
    fn last_move(&self) -> Option<Move>;

    /// Applies the given move to the node
    fn apply_move(self, move_index: Move) -> Self;

    /// Returns a vector of the nodes connected to this node.
    /// Omits nodes obtained by redundant moves (moves with the same axis as the last move's axis).
    fn connected(&self) -> NodeIterator<Self> {
        NodeIterator::new(*self)
    }

    /// Returns a vector of the nodes connected to this node.
    /// Gives priority to the nodes obtained through moves with the same axis as the last move's axis.
    fn connected_axis_priority(&self) -> Vec<Self> {
        todo!()
    }

    /// Returns whether the node is the goal node
    fn is_goal(&self) -> bool {
        *self == Self::IDENTITY
    }

    /// Returns the goal node
    fn goal() -> Self {
        Self::IDENTITY
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Phase1Node {
    orientation: Orientation<K4>,
    last_move: Option<Move>,
}

impl PartialEq for Phase1Node {
    fn eq(&self, other: &Self) -> bool {
        self.orientation == other.orientation
    }
}

impl Phase1Node {
    pub fn apply_move(self, i: usize) -> Self {
        Phase1Node {
            orientation: self
                .orientation
                .permute(PERM_MOVE_TABLE[i])
                .apply_orientation(A4_MOVE_TABLE[i]),
            last_move: Some(Move(i as u8)),
        }
    }
}

impl Identity for Phase1Node {
    const IDENTITY: Self = Phase1Node {
        orientation: Orientation::<K4>::IDENTITY,
        last_move: None,
    };
}

impl Node for Phase1Node {
    const N_STATES: usize = N_K4_COORD_STATES as usize;
    type Phase = Phase1;

    fn get_index(&self) -> usize {
        self.orientation.k4_coord() as usize
    }

    fn from_index(index: usize, last_move: Option<Move>) -> Self {
        Phase1Node {
            orientation: Orientation::<K4>::from_k4_coord(index as u32),
            last_move,
        }
    }

    fn last_move(&self) -> Option<Move> {
        self.last_move
    }

    fn apply_move(self, move_index: Move) -> Self {
        let perm = PERM_MOVE_TABLE[move_index.as_usize()];
        let orien = A4_MOVE_TABLE[move_index.as_usize()];
        Self {
            orientation: self.orientation.permute(perm).apply_orientation(orien),
            last_move: Some(move_index),
        }
    }
}

impl From<CubieCube> for Phase1Node {
    fn from(value: CubieCube) -> Self {
        Phase1Node {
            orientation: value.orientation.into(),
            last_move: None,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Phase2Node {
    c3_coord: u32,
    io_coord: u16,
    last_move: Option<Move>,
}

impl PartialEq for Phase2Node {
    fn eq(&self, other: &Self) -> bool {
        self.c3_coord == other.c3_coord && self.io_coord == other.io_coord
    }
}

impl Identity for Phase2Node {
    const IDENTITY: Self = Phase2Node {
        c3_coord: 0,
        io_coord: 0,
        last_move: None,
    };
}

impl Node for Phase2Node {
    const N_STATES: usize = N_C3_COORD_STATES as usize * N_IO_COORD_STATES as usize;
    type Phase = Phase2;

    fn get_index(&self) -> usize {
        (self.io_coord as usize) * (N_C3_COORD_STATES as usize) + (self.c3_coord as usize)
    }

    fn from_index(index: usize, last_move: Option<Move>) -> Self {
        Phase2Node {
            c3_coord: (index % N_C3_COORD_STATES as usize) as u32,
            io_coord: (index / N_C3_COORD_STATES as usize) as u16,
            last_move,
        }
    }

    fn last_move(&self) -> Option<Move> {
        self.last_move
    }

    fn apply_move(self, move_index: Move) -> Self {
        let c3_coord = C3_MOVE_TABLE[self.c3_coord as usize][move_index.as_usize()];
        let io_coord = IO_MOVE_TABLE[self.io_coord as usize][move_index.as_usize()];
        Self {
            c3_coord,
            io_coord,
            last_move: Some(move_index),
        }
    }
}

impl From<CubieCube> for Phase2Node {
    fn from(value: CubieCube) -> Self {
        Phase2Node {
            c3_coord: value.orientation.c3_coord(),
            io_coord: value.permutation.io_coord(),
            last_move: None,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Phase3Node {
    i_coord: u16,
    o_coord: u16,
    last_move: Option<Move>,
}

impl PartialEq for Phase3Node {
    fn eq(&self, other: &Self) -> bool {
        self.i_coord == other.i_coord && self.o_coord == other.o_coord
    }
}

impl Identity for Phase3Node {
    const IDENTITY: Self = Phase3Node {
        i_coord: 0,
        o_coord: 0,
        last_move: None,
    };
}

impl Node for Phase3Node {
    const N_STATES: usize = N_I_COORD_STATES as usize * N_O_COORD_STATES as usize / 2;
    type Phase = Phase3;

    fn get_index(&self) -> usize {
        self.o_coord as usize * (N_I_COORD_STATES / 2) as usize
            + (self.i_coord % (N_I_COORD_STATES / 2)) as usize
    }

    fn from_index(index: usize, last_move: Option<Move>) -> Self {
        let o_coord = (index / (N_I_COORD_STATES / 2) as usize) as u16;
        let mut i_coord = (index % (N_I_COORD_STATES / 2) as usize) as u16;
        if o_coord >= (N_O_COORD_STATES / 2) as u16 {
            i_coord += N_I_COORD_STATES / 2;
        };

        Phase3Node {
            i_coord,
            o_coord,
            last_move,
        }
    }

    fn last_move(&self) -> Option<Move> {
        self.last_move
    }

    fn apply_move(self, move_index: Move) -> Self {
        let i_coord = I_MOVE_TABLE[self.i_coord as usize][move_index.as_usize()];
        let o_coord = O_MOVE_TABLE[self.o_coord as usize][move_index.as_usize()];
        Self {
            i_coord,
            o_coord,
            last_move: Some(move_index),
        }
    }
}

impl From<CubieCube> for Phase3Node {
    fn from(value: CubieCube) -> Self {
        Phase3Node {
            i_coord: value.permutation.i_coord(),
            o_coord: value.permutation.o_coord(),
            last_move: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orientation_to_from_c3_coord() {
        use crate::groups::A4;
        for i in (0..N_C3_COORD_STATES).into_iter().step_by(10) {
            let orientation: Orientation<A4> = Orientation::from_c3_coord(i).into();
            assert_eq!(orientation.c3_coord(), i)
        }
    }

    #[test]
    fn test_permutation_to_from_io_coord() {
        for i in 0..N_IO_COORD_STATES {
            let permutation = Permutation::from_coords(i, 0, 0);
            assert_eq!(permutation.io_coord(), i)
        }
    }

    #[test]
    fn test_permutation_to_from_i_coord() {
        for i in 0..N_I_COORD_STATES {
            let permutation = Permutation::from_coords(0, i, 0);
            assert_eq!(permutation.i_coord(), i)
        }
    }

    #[test]
    fn test_permutation_to_from_o_coord() {
        for i in 0..N_O_COORD_STATES {
            let permutation = Permutation::from_coords(0, 0, i);
            assert_eq!(permutation.o_coord(), i)
        }
    }
}
