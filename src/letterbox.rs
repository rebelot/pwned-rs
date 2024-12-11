use std::ops::BitOr;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black = 0,
    White = 8,
}

impl BitOr for Color {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}

impl std::ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Pawn   = 1,
    Knight = 2,
    Bishop = 3,
    Rook   = 4,
    Queen  = 5,
    King   = 6,
}

impl BitOr for Piece {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}

impl BitOr<Color> for Piece {
    type Output = u8;

    fn bitor(self, rhs: Color) -> Self::Output {
        self as u8 | rhs as u8
    }
}
