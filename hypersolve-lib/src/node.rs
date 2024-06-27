use data_loading::load_or_generate_data;
use hypersolve_core::*;
use nohash_hasher::IsEnabled;
use solution_iterators::PhaseSolutionIterator;

use std::{hash::Hash, ops::RangeBounds};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use rkyv::Archive;

use crate::*;

// Include constants generated at build-time
include!(concat!(env!("OUT_DIR"), "/const_gen.rs"));

/// A trait for highly optimized computation of how certain aspects of the cube
/// are affected by twists.
pub trait Node: Copy + PartialEq
where
    Self: Sized,
{
    const N_STATES: usize;
    const GOAL: Self;
    const PRUNING_DEPTH: u8;
    type Phase: Phase;
    type Index: Into<u64> + Archive<Archived = Self::Index> + Hash + Eq + IsEnabled;
    type PruningTable: PruningTable<Self>;

    /// Returns the index of the node
    fn index(&self) -> Self::Index;

    /// Returns a node from an index
    fn from_index(index: Self::Index) -> Self;

    /// Applies the given move to the node
    fn apply_move(&self, move_index: Move<Self::Phase>) -> Self;

    /// Gets the lower bound on the number of moves requied to reach the goal node from this node
    fn get_depth_bound(&self) -> u8;

    /// Returns an iterator over the phase solutions
    fn phase_solutions<R: RangeBounds<usize>>(
        self,
        prev_moves: Vec<Move>,
        total_sol_lens: R,
    ) -> PhaseSolutionIterator<Self, R>
    where
        Move<Self::Phase>: Into<Move>,
    {
        PhaseSolutionIterator::new(prev_moves, self, total_sol_lens)
    }
}

/// A node representing a cube state in phase 1
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Phase1Node {
    orientation: Orientation<K4>,
}

impl Node for Phase1Node {
    const N_STATES: usize = N_K4_COORD_STATES as usize;
    const GOAL: Self = Phase1Node {
        orientation: unsafe { Orientation::<K4>::from_k4_coord(0) },
    };
    const PRUNING_DEPTH: u8 = 5;
    type Phase = Phase1;
    type Index = u32;
    type PruningTable = HashMapPruningTable<Self>;

    fn index(&self) -> u32 {
        self.orientation.k4_coord()
    }

    fn from_index(index: u32) -> Self {
        debug_assert!(index < Self::N_STATES as u32);

        Phase1Node {
            orientation: unsafe { Orientation::<K4>::from_k4_coord(index) },
        }
    }

    fn apply_move(&self, move_index: Move<Self::Phase>) -> Self {
        Self {
            orientation: self
                .orientation
                .permute(move_index.permutation())
                .apply_orientation(move_index.orientation()),
        }
    }

    fn get_depth_bound(&self) -> u8 {
        load_or_generate_data!(static PHASE1_PRUNING_TABLE: <Phase1Node as Node>::PruningTable = <Phase1Node as Node>::PruningTable::generate(Phase1Node::PRUNING_DEPTH), "phase1.prun");

        PHASE1_PRUNING_TABLE.get_depth_bound(*self)
    }
}

impl From<CubieCube> for Phase1Node {
    fn from(value: CubieCube) -> Self {
        Phase1Node {
            orientation: value.orientation.to_k4(),
        }
    }
}

/// A node representing a cube state in phase 2
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Phase2Node {
    pub c3_coord: u32,
    pub io_coord: u16,
}

impl Node for Phase2Node {
    const N_STATES: usize = N_C3_COORD_STATES as usize * N_IO_COORD_STATES as usize;
    const GOAL: Self = Phase2Node {
        c3_coord: 0,
        io_coord: 0,
    };
    const PRUNING_DEPTH: u8 = 6;
    type Phase = Phase2;
    type Index = u64;
    type PruningTable = HashMapPruningTable<Self>;

    fn index(&self) -> u64 {
        (self.io_coord as u64) * (N_C3_COORD_STATES as u64) + (self.c3_coord as u64)
    }

    fn from_index(index: Self::Index) -> Self {
        Phase2Node {
            c3_coord: (index % N_C3_COORD_STATES as u64) as u32,
            io_coord: (index / N_C3_COORD_STATES as u64) as u16,
        }
    }

    fn apply_move(&self, move_index: Move<Self::Phase>) -> Self {
        fn gen_c3_move_table() -> Box<[[u32; Phase2::N_MOVES]; N_C3_COORD_STATES as usize]> {
            let mut table = vec![[0_u32; Phase2::N_MOVES]; N_C3_COORD_STATES as usize];

            table.par_iter_mut().enumerate().for_each(|(i, entry)| {
                let cube = CubieCube {
                    // SAFTEY: i < N_C3_COORD_STATES
                    orientation: unsafe { Orientation::from_c3_coord(i as u32) }.to_a4(),
                    permutation: Permutation::SOLVED,
                };

                for (j, val) in entry.iter_mut().enumerate().take(Phase2::N_MOVES) {
                    *val = cube
                        .apply_move(Move::<Phase2>::from_u8(j as u8))
                        .orientation
                        .c3_coord();
                }
            });

            table.try_into().unwrap()
        }

        load_or_generate_data!(static C3_MOVE_TABLE: Box<[[u32; Phase2::N_MOVES]; N_C3_COORD_STATES as usize]> = gen_c3_move_table(), "c3.move");

        unsafe {
            assert_unchecked!(self.c3_coord < N_C3_COORD_STATES);
            assert_unchecked!(self.io_coord < N_IO_COORD_STATES);
        }

        let c3_coord = C3_MOVE_TABLE[self.c3_coord as usize][move_index.into_usize()];
        let io_coord = IO_MOVE_TABLE[self.io_coord as usize][move_index.into_usize()];
        Self { c3_coord, io_coord }
    }

    fn get_depth_bound(&self) -> u8 {
        load_or_generate_data!(static PHASE2_PRUNING_TABLE: <Phase2Node as Node>::PruningTable = <Phase2Node as Node>::PruningTable::generate(Phase2Node::PRUNING_DEPTH), "phase2.prun");

        PHASE2_PRUNING_TABLE.get_depth_bound(*self)
    }
}

impl From<CubieCube> for Phase2Node {
    fn from(value: CubieCube) -> Self {
        Phase2Node {
            c3_coord: value.orientation.c3_coord(),
            io_coord: value.permutation.io_coord(),
        }
    }
}

/// A node representing a cube state in phase 3
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Phase3Node {
    pub i_coord: u16,
    pub o_coord: u16,
}

impl Node for Phase3Node {
    const N_STATES: usize = N_I_COORD_STATES as usize * N_O_COORD_STATES as usize / 2;
    const GOAL: Self = Phase3Node {
        i_coord: 0,
        o_coord: 0,
    };
    const PRUNING_DEPTH: u8 = 21;
    type Phase = Phase3;
    type Index = u32;
    type PruningTable = ArrayPruningTable<Self>;

    fn index(&self) -> u32 {
        self.o_coord as u32 * (N_I_COORD_STATES / 2) as u32
            + (self.i_coord as u32 % (N_I_COORD_STATES / 2) as u32)
    }

    fn from_index(index: Self::Index) -> Self {
        let o_coord = (index / (N_I_COORD_STATES / 2) as u32) as u16;
        let mut i_coord = (index % (N_I_COORD_STATES / 2) as u32) as u16;

        // if O piece permutation is odd then the I piece permutation must also be odd
        if o_coord >= (N_O_COORD_STATES / 2) {
            i_coord += N_I_COORD_STATES / 2;
        };

        Phase3Node { i_coord, o_coord }
    }

    fn apply_move(&self, move_index: Move<Self::Phase>) -> Self {
        unsafe {
            assert_unchecked!(self.i_coord < N_I_COORD_STATES);
            assert_unchecked!(self.o_coord < N_O_COORD_STATES);
        }

        let i_coord = I_MOVE_TABLE[self.i_coord as usize][move_index.into_usize()];
        let o_coord = O_MOVE_TABLE[self.o_coord as usize][move_index.into_usize()];
        Self { i_coord, o_coord }
    }

    fn get_depth_bound(&self) -> u8 {
        load_or_generate_data!(static PHASE3_PRUNING_TABLE: <Phase3Node as Node>::PruningTable = <Phase3Node as Node>::PruningTable::generate(Phase3Node::PRUNING_DEPTH), "phase3.prun");

        PHASE3_PRUNING_TABLE.get_depth_bound(*self)
    }
}

impl From<CubieCube> for Phase3Node {
    fn from(value: CubieCube) -> Self {
        Phase3Node {
            i_coord: value.permutation.i_coord(),
            o_coord: value.permutation.o_coord(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orientation_to_from_c3_coord() {
        use hypersolve_core::A4;
        for i in (0..N_C3_COORD_STATES).step_by(10) {
            let orientation: Orientation<A4> = unsafe { Orientation::from_c3_coord(i) }.to_a4();
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
        for i in 0..N_I_COORD_STATES / 2 {
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
