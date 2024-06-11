use super::*;

/// An index representing a specific cube state
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

/// Errors for converting an integer into a cube index
#[derive(Debug, thiserror::Error)]
pub enum CubeIndexError {
    #[error("index must be less than `3,357,894,533,384,932,272,635,904,000` but it is `{0}`")]
    InvalidIndex(u128),
}

impl TryFrom<u128> for CubeIndex {
    type Error = CubeIndexError;
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        if value < N_CUBE_STATES {
            Ok(CubeIndex(value))
        } else {
            Err(CubeIndexError::InvalidIndex(value))
        }
    }
}
