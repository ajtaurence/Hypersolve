use std::ops::{Mul, Neg};

use crate::{cubiecube::Orientation, groups::A4Elem};

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum AxisEnum {
    X = 0,
    Y = 1,
    Z = 2,
    W = 3,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum StickerEnum {
    F,
    B,
    R,
    L,
    U,
    D,
    I,
    O,
}

impl StickerEnum {
    pub fn axis(&self) -> AxisEnum {
        use StickerEnum::*;
        match self {
            F | B => AxisEnum::X,
            R | L => AxisEnum::Y,
            U | D => AxisEnum::Z,
            I | O => AxisEnum::W,
        }
    }

    pub fn sign(&self) -> i8 {
        use StickerEnum::*;
        match self {
            F | R | U | I => 1,
            B | L | D | O => -1,
        }
    }
}

impl Neg for StickerEnum {
    type Output = Self;

    fn neg(self) -> Self::Output {
        use StickerEnum::*;
        match self {
            F => B,
            B => F,
            U => D,
            D => U,
            R => L,
            L => R,
            I => O,
            O => I,
        }
    }
}

pub struct Piece {
    stickers: [StickerEnum; 4],
}

impl Piece {
    pub fn a4_elem(&self) -> A4Elem {
        use AxisEnum::*;
        match [
            AxisEnum::X,
            AxisEnum::X,
            self.stickers[2].axis(),
            self.stickers[3].axis(),
        ] {
            [_, _, Z, W] => A4Elem::E,
            [_, _, W, Z] => A4Elem::U1,
            [W, Z, _, _] => A4Elem::U2,
            [Z, W, _, _] => A4Elem::U3,
            [_, Z, W, _] => A4Elem::R6,
            [_, W, Z, _] => A4Elem::R7,
            [W, _, _, Z] => A4Elem::R3,
            [Z, _, _, W] => A4Elem::R2,
            [W, _, Z, _] => A4Elem::R5,
            [Z, _, W, _] => A4Elem::R8,
            [_, Z, _, W] => A4Elem::R1,
            [_, W, _, Z] => A4Elem::R4,
            _ => {
                unreachable!("Could not convert piece to A4 group element, duplicate sticker axis?")
            }
        }
    }
}

pub struct StickerCube {
    pieces: [Piece; 16],
}

impl StickerCube {
    pub fn orientation(&self) -> Orientation<A4Elem> {
        let mut orientation = [A4Elem::E; 15];

        for i in 0..15 {
            orientation[i] = self.pieces[i].a4_elem()
        }

        Orientation { state: orientation }
    }
}
