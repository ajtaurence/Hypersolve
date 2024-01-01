use crate::cubie_cube::CubieCube;
use crate::piece_cube::puzzle::PieceCube;
use thiserror::Error;

/// An index representing a specific cube state
///
/// The index is effectively an integer and can be converted to and from `u128`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CubeIndex(pub(crate) u128);

impl CubeIndex {
    /// The solved cube index
    pub const SOLVED: Self = CubeIndex(0);
}

impl From<CubeIndex> for u128 {
    fn from(value: CubeIndex) -> Self {
        value.0
    }
}

impl TryFrom<u128> for CubeIndex {
    type Error = CubeIndexError;
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        if value < crate::N_CUBE_STATES {
            Ok(CubeIndex(value))
        } else {
            Err(CubeIndexError::InvalidIndex(value))
        }
    }
}

impl From<CubieCube> for CubeIndex {
    fn from(value: CubieCube) -> Self {
        value.get_index()
    }
}

impl From<PieceCube> for CubeIndex {
    fn from(value: PieceCube) -> Self {
        CubieCube::from(value).into()
    }
}

#[derive(Debug, Error)]
/// An error produced when trying to convert an integer into a cube index
pub enum CubeIndexError {
    #[error("index must be less than `3,357,894,533,384,932,272,635,904,000` but it is `{0}`")]
    InvalidIndex(u128),
}
