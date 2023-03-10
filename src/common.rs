use std::ops::{Mul, Neg};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
    W,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Sign {
    #[default]
    Pos = 1,
    Neg = -1,
}

impl Mul for Sign {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self == rhs {
            Sign::Pos
        } else {
            Sign::Neg
        }
    }
}

impl Neg for Sign {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Sign::Pos => Sign::Neg,
            Sign::Neg => Sign::Pos,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Facet {
    pub axis: Axis,
    pub sign: Sign,
}

impl Facet {
    const R: Facet = Facet::new(Axis::X, Sign::Pos);
    const L: Facet = Facet::new(Axis::X, Sign::Neg);
    const U: Facet = Facet::new(Axis::Y, Sign::Pos);
    const D: Facet = Facet::new(Axis::Y, Sign::Neg);
    const F: Facet = Facet::new(Axis::Z, Sign::Pos);
    const B: Facet = Facet::new(Axis::Z, Sign::Neg);
    // Normally I is negative and O is positive, but we switch them
    // here so that the OLDB piece stays fixed in our solution.
    const I: Facet = Facet::new(Axis::W, Sign::Pos);
    const O: Facet = Facet::new(Axis::W, Sign::Neg);

    const fn new(axis: Axis, sign: Sign) -> Self {
        Self { axis, sign }
    }

    const fn unit_vec4(self) -> [i8; 4] {
        let mut ret = [0; 4];
        ret[self.axis as usize] = self.sign as i8;
        ret
    }

    const fn basis_facets(self) -> [Facet; 3] {
        let w = match self.sign {
            Sign::Pos => Facet::O,
            Sign::Neg => Facet::I,
        };

        [
            if self.axis == Axis::X { w } else { Facet::R },
            if self.axis == Axis::Y { w } else { Facet::U },
            if self.axis == Axis::Z { w } else { Facet::F },
        ]
    }
}

impl Mul<Sign> for Facet {
    type Output = Self;

    fn mul(self, rhs: Sign) -> Self::Output {
        self
    }
}

impl Neg for Facet {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            axis: self.axis,
            sign: -self.sign,
        }
    }
}

/// Matrix representing a piece position
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Matrix([Facet; 4]);

impl Default for Matrix {
    fn default() -> Self {
        use Axis::*;

        let sign = Sign::Pos;
        Self([X, Y, Z, W].map(|axis| Facet { axis, sign }))
    }
}

impl Mul for Matrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self([0, 1, 2, 3].map(|i| {
            let Facet { axis, sign } = rhs.0[i as usize];
            self.0[axis as usize] * sign
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_mul() {
        assert_eq!(
            Matrix([Facet::R, Facet::U, Facet::F, Facet::I])
                * Matrix([Facet::R, Facet::U, Facet::F, Facet::I]),
            Matrix([Facet::R, Facet::U, Facet::F, Facet::I]),
        )
    }
}
