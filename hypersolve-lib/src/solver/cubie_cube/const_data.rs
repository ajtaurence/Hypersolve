use super::*;

const_data!(pub(crate) HYPERSOLVE_TWISTS: [crate::puzzle::Twist; Phase1::N_MOVES] =  gen_hypersolve_twists());
const_data!(pub(crate) PERM_MOVE_TABLE: [Permutation; Phase1::N_MOVES] =  gen_perm_move_table());
const_data!(pub(crate) A4_MOVE_TABLE: [Orientation<crate::common::groups::A4>; Phase1::N_MOVES] =  gen_a4_move_table());
