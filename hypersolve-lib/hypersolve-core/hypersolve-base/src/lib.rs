#[macro_use]
mod macros;
mod groups;
mod math;
mod orientation;
mod permutation;
mod puzzle;

pub use groups::*;
pub use math::*;
pub use orientation::*;
pub use permutation::*;
pub use puzzle::*;

pub const N_K4_COORD_STATES: u32 = 4_u32.pow(15);
pub const N_C3_COORD_STATES: u32 = 3_u32.pow(14);
pub const N_IO_COORD_STATES: u16 = unsafe { crate::n_choose_k(15, 7) };
pub const N_I_COORD_STATES: u16 = crate::factorial(8) as u16;
pub const N_O_COORD_STATES: u16 = crate::factorial(7) as u16;
