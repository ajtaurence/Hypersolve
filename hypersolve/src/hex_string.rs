use sha2::Digest;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use super::*;

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum HexStringError<const N: usize> {
    #[error("invalid character '{c}' at position {index}, valid characters are: 0...9, a...f")]
    InvalidHexCharacter { c: char, index: usize },
    #[error("invalid string length, expected {} characters", 2 * N)]
    InvalidStringLength,
}

impl<const N: usize> From<hex::FromHexError> for HexStringError<N> {
    fn from(value: hex::FromHexError) -> Self {
        match value {
            hex::FromHexError::InvalidHexCharacter { c, index } => {
                Self::InvalidHexCharacter { c, index }
            }
            hex::FromHexError::InvalidStringLength => Self::InvalidStringLength,
            hex::FromHexError::OddLength => Self::InvalidStringLength,
        }
    }
}

/// An N byte hexadecimal string
#[derive(Debug, Clone)]
pub struct HexString<const N: usize>(String);

impl<const N: usize> Display for HexString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const N: usize> HexString<N> {
    pub fn from_bytes(bytes: [u8; N]) -> Self {
        HexString(hex::encode(bytes))
    }

    pub fn bytes(&self) -> [u8; N] {
        hex::decode(&self.0).unwrap().try_into().unwrap()
    }

    pub fn get_random() -> Self {
        let mut bytes = [0_u8; N];
        getrandom::getrandom(&mut bytes).expect("unable to generate random key");
        Self::from_bytes(bytes)
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();

        hasher.update(self.bytes());

        hasher.finalize().into()
    }

    pub fn to_cube_index(&self) -> CubeIndex {
        CubeIndex::try_from(
            u128::from_le_bytes(self.hash()[..16].try_into().unwrap()) % N_CUBE_STATES,
        )
        .unwrap()
    }
}

impl<const N: usize> FromStr for HexString<N> {
    type Err = HexStringError<N>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;

        if bytes.len() != N {
            return Err(HexStringError::InvalidStringLength);
        }

        Ok(HexString::from_bytes(bytes.try_into().unwrap()))
    }
}
