// Much of the code for this module was taken from or closely resembles Hyperspeedcube: https://github.com/HactarCE/Hyperspeedcube

pub mod cube_index;
mod pieces;
pub mod puzzle;
mod twist;

pub use cube_index::*;
pub use pieces::*;
pub use twist::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_location_index() {
        for i in 0..16 {
            assert_eq!(PieceLocation::from_index(i).index(), i)
        }
    }

    #[test]
    fn test_piece_current_location() {
        for i in 0..16 {
            assert_eq!(
                PieceLocation::from_index(i)
                    .solved_piece()
                    .current_location(),
                PieceLocation::from_index(i)
            )
        }

        assert_ne!(
            PieceLocation::from_index(6)
                .solved_piece()
                .current_location(),
            PieceLocation::from_index(1)
        )
    }
}
