use super::*;

use crate::{
    groups::{Identity, A4, C3, K4},
    piece_cube::puzzle::PieceCube,
};
use num_traits::FromPrimitive;

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

impl<T: Identity + std::fmt::Debug + Copy + PartialEq + From<A4>> From<PieceCube>
    for Orientation<T>
{
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

        for (i, &index) in permutation.into_inner().iter().enumerate() {
            result[i] = self.state[index as usize];
        }

        Orientation { state: result }
    }

    pub fn apply_orientation<U>(self, action: Orientation<U>) -> Orientation<U::Output>
    where
        U: Copy,
        U: std::ops::Mul<T>,
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
