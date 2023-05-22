use crate::{
    cubie_cube::Move,
    node_cube::node::Node,
    prune::{PruningTable, PHASE2_PRUNING_TABLE},
};

pub enum YesNoMaybe {
    Yes,
    Maybe,
    No,
}

impl YesNoMaybe {
    pub fn is_no(&self) -> bool {
        use YesNoMaybe::*;
        match self {
            No => true,
            _ => false,
        }
    }

    pub fn is_yes(&self) -> bool {
        use YesNoMaybe::*;
        match self {
            Yes => true,
            _ => false,
        }
    }

    pub fn is_maybe(&self) -> bool {
        use YesNoMaybe::*;
        match self {
            Maybe => true,
            _ => false,
        }
    }

    pub fn is_certain(&self) -> bool {
        use YesNoMaybe::*;
        match self {
            Maybe => false,
            _ => true,
        }
    }
}

/// Finds the sequence to reach the goal node at the given depth
pub fn phase_search(
    node: impl Node,
    sequence: Vec<Move>,
    depth: usize,
    solution_certainty: YesNoMaybe,
) -> Option<Vec<Move>> {
    // // if we found a solution at the desired depth then return it
    // if node.is_goal() && sequence.len() == depth {
    //     return Some(sequence);
    // }

    // // get the lower bound on the distance to the solution
    // let dist = sequence.len() + PHASE2_PRUNING_TABLE.get_depth(node) as usize;

    // // if the lower bound on the distance is more than the depth then return no solution
    // if dist > depth {
    //     return None;
    // }

    todo!()
}
