use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Color {
    Black,
    White,
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
impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::White => write!(f, "white"),
            Self::Black => write!(f, "black"),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Piece {
    Pawn(Color),
    King(Color),
    Queen(Color),
    Rook(Color),
    Bishop(Color),
    Knight(Color),
}

impl Serialize for Piece {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = serializer.serialize_struct("piece", 2)?;
        ser.serialize_field("kind", &self.name())?;
        ser.serialize_field("color", &self.color().to_string())?;
        ser.end()
    }
}

impl Piece {
    pub fn color(&self) -> Color {
        match self {
            Self::Pawn(color)
            | Self::King(color)
            | Self::Queen(color)
            | Self::Rook(color)
            | Self::Bishop(color)
            | Self::Knight(color) => *color,
        }
    }
    pub fn name(&self) -> String {
        match *self {
            Self::Pawn(_) => String::from("pawn"),
            Self::King(_) => String::from("king"),
            Self::Queen(_) => String::from("queen"),
            Self::Rook(_) => String::from("rook"),
            Self::Bishop(_) => String::from("bishop"),
            Self::Knight(_) => String::from("knight"),
        }
    }
}
