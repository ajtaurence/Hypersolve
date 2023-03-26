// Much of the code for this modcule was taken from Hyperspeedcube: https://github.com/HactarCE/Hyperspeedcube

pub mod pieces;
pub mod puzzle;
pub mod twist;

use pieces::*;
use twist::*;

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
