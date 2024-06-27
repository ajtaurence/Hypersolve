use const_for::const_for;
use static_assertions::const_assert;

use super::*;
use crate::{groups::*, Cube};

/// Describes the orientation of pieces
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct Orientation<T>([T; 15]);

impl<T> const_gen::CompileConst for Orientation<T>
where
    T: Copy + const_gen::CompileConst,
{
    fn const_type() -> String {
        format!("Orientation<{}>", T::const_type())
    }
    fn const_val(&self) -> String {
        format!("Orientation::from_array({})", self.0.const_val())
    }
}

impl<T> Orientation<T>
where
    T: Copy,
{
    pub const fn from_array(array: [T; 15]) -> Self {
        Orientation(array)
    }
}

impl Orientation<A4> {
    pub const SOLVED: Self = Self([A4::IDENTITY; 15]);

    pub fn from_cube(cube: Cube) -> Self {
        let pieces = cube.reposition().pieces_except_last();

        Orientation(const_arr!([A4; 15], |i| pieces[i].to_a4()))
            .permute(&Permutation::from_cube(cube))
    }

    /// # Safety
    /// `k4` must be less than `N_K4_COORD_STATES` and `c3` must be less than `N_C3_COORD_STATES`
    pub const unsafe fn from_k4_c3_coords(k4: u32, c3: u32) -> Self {
        debug_assert!(k4 < N_K4_COORD_STATES && c3 < N_C3_COORD_STATES);

        let k4_orientation = Orientation::from_k4_coord(k4);
        let c3_orientation = Orientation::from_c3_coord(c3);

        Self(const_arr!([A4; 15], |i| {
            A4::from_k4_c3(k4_orientation.0[i], c3_orientation.0[i])
        }))
    }

    pub const fn permute(&self, permutation: &Permutation) -> Self {
        Self(permutation.0.permute(&self.0))
    }

    pub const fn apply_orientation(mut self, action: &Orientation<A4>) -> Self {
        const_for!(i in 0..self.0.len() => {
            self.0[i] = A4::group_mul(action.0[i], self.0[i])
        });

        self
    }

    pub const fn k4_coord(&self) -> u32 {
        let mut result = 0;
        const_for!(i in 0..self.0.len() => {
            result += (self.0[i].to_k4() as u32) * u32::pow(4, i as u32);
        });

        result
    }

    /// Returns the index into the C3 move table
    pub const fn c3_coord(&self) -> u32 {
        let mut result: u32 = 0;

        const_for!(i in 0..14 => {
            result += self.0[i].to_c3() as u32 * u32::pow(3, i as u32);
        });

        result
    }

    pub const fn to_k4(&self) -> Orientation<K4> {
        Orientation(const_arr!([K4; 15], |i| self.0[i].to_k4()))
    }

    pub const fn to_c3(&self) -> Orientation<C3> {
        Orientation(const_arr!([C3; 15], |i| self.0[i].to_c3()))
    }
}

impl Orientation<K4> {
    pub fn from_cube(cube: Cube) -> Self {
        let pieces = cube.reposition().pieces_except_last();

        Orientation(const_arr!([K4; 15], |i| pieces[i].to_a4().to_k4()))
            .permute(&Permutation::from_cube(cube))
    }

    pub const fn k4_coord(&self) -> u32 {
        let mut result = 0;
        const_for!(i in 0..self.0.len() => {
            result += (self.0[i] as u32) * u32::pow(4, i as u32);
        });

        result
    }

    pub const fn permute(&self, permutation: &Permutation) -> Self {
        Self(permutation.0.permute(&self.0))
    }

    pub fn apply_orientation(mut self, action: &Orientation<A4>) -> Self {
        const_for!(i in 0..self.0.len() => {
            self.0[i] = crate::a4_k4_group_mul(action.0[i], self.0[i])
        });

        self
    }

    /// Returns an orientation state from a K4 pruning table index
    ///
    /// # Safety
    /// `k4_coord` must be less than `N_K4_COORD_STATES`
    pub const unsafe fn from_k4_coord(k4_coord: u32) -> Self {
        debug_assert!(k4_coord < N_K4_COORD_STATES);

        let mut coord = k4_coord;
        Self(const_arr!([K4; 15], {
            // SAFTEY: anything % 4 is less than 4 so it is a valid discriminant
            let res = unsafe { K4::from_repr_unchecked((coord % 4) as u8) };
            coord /= 4;

            res
        }))
    }

    pub const fn to_a4(&self) -> Orientation<A4> {
        Orientation(const_arr!([A4; 15], |i| self.0[i].to_a4()))
    }
}

impl Orientation<C3> {
    pub fn from_cube(cube: Cube) -> Self {
        let pieces = cube.reposition().pieces_except_last();

        Orientation(const_arr!([C3; 15], |i| pieces[i].to_a4().to_c3()))
            .permute(Permutation::from_cube(cube))
    }

    pub const fn permute(&self, permutation: Permutation) -> Self {
        Self(permutation.0.permute(&self.0))
    }

    /// Returns an orientation state from a C3 move table index
    ///
    /// # Safety
    /// `c3_coord` must be less than `N_C3_COORD_STATES`
    pub const unsafe fn from_c3_coord(c3_coord: u32) -> Orientation<C3> {
        debug_assert!(c3_coord < N_C3_COORD_STATES);

        // This is safe because we overwrite every value of the array
        // And dropping the uninitialized values when new ones are assigned does nothing
        const_assert!(!std::mem::needs_drop::<C3>());
        #[allow(invalid_value, clippy::uninit_assumed_init)]
        let mut result = unsafe { std::mem::MaybeUninit::<[C3; 15]>::uninit().assume_init() };

        let mut sum = 0;
        let mut coord = c3_coord;

        let mut i = 0;
        while i < 14 {
            let discriminant = coord % 3;
            sum += discriminant;
            result[i] = C3::from_repr_unchecked(discriminant as u8);
            coord /= 3;
            i += 1;
        }

        // fix parity of the last piece
        result[14] = C3::from_repr_unchecked((-(sum as i32)).rem_euclid(3) as u8);

        Self(result)
    }

    /// Returns the index into the C3 move table
    pub const fn c3_coord(&self) -> u32 {
        let mut result: u32 = 0;

        const_for!(i in 0..14 => {
            result += self.0[i] as u32 * u32::pow(3, i as u32);
        });

        result
    }

    pub const fn to_a4(&self) -> Orientation<A4> {
        Orientation(const_arr!([A4; 15], |i| self.0[i].to_a4()))
    }
}
