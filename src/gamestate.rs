use js_sys::Int32Array;
use serde::ser::{SerializeTuple, Serializer};
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut, Range};
use wasm_bindgen::prelude::*;

use crate::bitboards::*;
use crate::consts::*;
use crate::moves::*;
use crate::pieces::*;

#[derive(Serialize, Deserialize)]
struct MoveResponse {
    updates: Vec<(usize, i32)>,
}

#[wasm_bindgen]
pub struct Board {
    board: [Option<Piece>; 64],
    bitboards: BitBoards,
    turn: Color,
    castling: u64,
    enpassant: Option<u8>,
    halfmove: u32,
    fullmove: u32,
    legal_moves: [u64; 64],
}

#[wasm_bindgen]
impl Board {
    #[wasm_bindgen(constructor)]
    pub fn new(fen: &str) -> Self {
        let mut fields = fen.split_whitespace();

        let position = fields.next().unwrap();
        let mut board = [None; 64];
        position.split('/').enumerate().for_each(|(row, rank)| {
            let mut col: usize = 0;
            rank.chars().for_each(|c| {
                if let Some(blanks) = c.to_digit(10) {
                    col += blanks as usize;
                } else {
                    let color = if c.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let piece = match c.to_ascii_lowercase() {
                        'p' => Some(Piece::Pawn(color)),
                        'b' => Some(Piece::Bishop(color)),
                        'n' => Some(Piece::Knight(color)),
                        'r' => Some(Piece::Rook(color)),
                        'q' => Some(Piece::Queen(color)),
                        'k' => Some(Piece::King(color)),
                        _ => None,
                    };
                    board[row * 8 + col] = piece;
                    col += 1;
                }
            })
        });
        let turn = match fields.next().unwrap() {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!(""),
        };
        let castling = fields.next().unwrap().chars().fold(0b0000, |acc, c| {
            acc | match c {
                'K' => Consts::WHITE.ks_castle,
                'Q' => Consts::WHITE.qs_castle,
                'k' => Consts::BLACK.ks_castle,
                'q' => Consts::BLACK.qs_castle,
                _ => 0,
            }
        });
        let mut enp = fields.next().unwrap().chars();
        let file = enp.next().and_then(|f| "abcdefgh".find(f));
        let rank = enp.next().map(|r| r.to_digit(10).unwrap() - 1);
        let enpassant = if let (Some(f), Some(r)) = (file, rank) {
            Some(r as u8 * 8 + f as u8)
        } else {
            None
        };

        let halfmove = fields.next().unwrap().parse().unwrap();
        let fullmove = fields.next().unwrap().parse().unwrap();

        let mut board = Board {
            board,
            bitboards: BitBoards::new(&board),
            turn,
            castling,
            enpassant,
            halfmove,
            fullmove,
            legal_moves: [0; 64],
        };

        board.legal_moves = gen_all_moves(board.turn, &board.bitboards, enpassant, castling);
        board
    }

    #[wasm_bindgen]
    pub fn turn(&mut self) {
        self.turn = match self.turn {
            Color::White => !self.turn,
            Color::Black => {
                self.fullmove += 1;
                !self.turn
            }
        }
    }
    #[wasm_bindgen]
    pub fn calc_legal_moves(&mut self) {
        self.legal_moves = gen_all_moves(self.turn, &self.bitboards, self.enpassant, self.castling);
    }

    #[wasm_bindgen]
    pub fn get_legal_moves(&mut self) -> Vec<Int32Array> {
        self.legal_moves
            .into_iter()
            .map(|mut moves| {
                let mut v = vec![];
                let mut i = 0;
                while moves != 0 {
                    if moves & 1 != 0 {
                        v.push(i);
                    }
                    i += 1;
                    moves >>= 1;
                }
                Int32Array::from(&v[..])
            })
            .collect()
    }

    #[wasm_bindgen]
    pub fn send_board(&self) -> String {
        serde_json::to_string(&self).expect("oh shit")
    }

    #[wasm_bindgen]
    pub fn input_move(&mut self, from: usize, to: usize, promotion: usize) -> String {
        let from_bb: u64 = 1 << from;
        let to_bb: u64 = 1 << to;
        if self.legal_moves[from] & to_bb == 0 {
            let response = (false, String::from("Invalid move"));
            return serde_json::to_string(&response).unwrap();
        };

        if let Some(piece) = self.board[from] {
            // *self.bitboards.get_piece_bb(piece) ^= from_bb | to_bb;
            // *self.bitboards.get_color_bb(piece.color()) ^= from_bb | to_bb;
            if let Some(capture) = self.board[to] {
                // *self.bitboards.get_piece_bb(capture) ^= to_bb;
                // *self.bitboards.get_color_bb(capture.color()) ^= to_bb;
            }
            self.board[to] = Some(piece);
            self.board[from] = None;
        }

        let mut response = MoveResponse {
            updates: vec![(from, to as i32)],
        };
        let enpassant = self.enpassant;
        self.enpassant = None;
        let consts = Consts::new(self.turn);

        if self.bitboards.pawns & from_bb != 0 {
            if let Some(enpassant) = enpassant {
                if enpassant == to as u8 {
                    let captured_pawn = (enpassant as i8 + (8 * consts.direction)) as usize;
                    // let captured_pawn_bb = 1 << captured_pawn;
                    if let Some(pawn) = self.board[captured_pawn] {
                        // self.bitboards.pawns ^= captured_pawn_bb;
                        // *self.bitboards.get_color_bb(pawn.color()) ^= captured_pawn_bb;
                        self.board[captured_pawn] = None;
                    }
                    response.updates.push((captured_pawn, -1));
                }
            } else if (to as isize - from as isize).abs() == 16 {
                self.enpassant = Some((to as i8 + 8 * consts.direction) as u8);
            } else if to_bb & consts.eighth_rank != 0 {
                self.bitboards.pawns ^= to_bb;
                self.board[to] = match promotion {
                    1 => {
                        self.bitboards.queens |= to_bb;
                        Some(Piece::Queen(self.turn))
                    }
                    2 => {
                        self.bitboards.rooks |= to_bb;
                        Some(Piece::Rook(self.turn))
                    }
                    3 => {
                        self.bitboards.bishops |= to_bb;
                        Some(Piece::Bishop(self.turn))
                    }
                    4 => {
                        self.bitboards.knights |= to_bb;
                        Some(Piece::Knight(self.turn))
                    }
                    _ => self.board[to],
                }
            }
        } else if self.bitboards.kings & from_bb != 0 {
            self.castling = 0;
            match to as i32 - from as i32 {
                2 => {
                    if let Some(piece) = self.board[to + 1] {
                        self.board[to - 1] = Some(piece);
                        self.board[to + 1] = None;
                    }
                    response.updates.push((to + 1, to as i32 - 1));
                }
                -2 => {
                    if let Some(piece) = self.board[to - 2] {
                        self.board[to + 1] = Some(piece);
                        self.board[to - 2] = None;
                    }
                    response.updates.push((to - 2, to as i32 + 1))
                }
                _ => {}
            }
        } else if consts.ks_rook & from_bb != 0 {
            self.castling ^= consts.ks_castle & self.castling;
        } else if consts.qs_rook & from_bb != 0 {
            self.castling ^= consts.qs_castle & self.castling;
        }

        self.bitboards = BitBoards::new(&self.board);
        self.turn();
        self.calc_legal_moves();
        serde_json::to_string(&(true, response)).unwrap()
    }
}

impl Index<usize> for Board {
    type Output = Option<Piece>;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.board[idx]
    }
}

impl IndexMut<Range<usize>> for Board {
    fn index_mut(&mut self, idx: Range<usize>) -> &mut Self::Output {
        &mut self.board[idx]
    }
}

impl Index<Range<usize>> for Board {
    type Output = [Option<Piece>];
    fn index(&self, idx: Range<usize>) -> &Self::Output {
        &self.board[idx]
    }
}

impl IndexMut<usize> for Board {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.board[idx]
    }
}

impl Serialize for Board {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = serializer.serialize_tuple(64)?;
        for (i, square) in self.board.iter().enumerate() {
            if let Some(piece) = square {
                ser.serialize_element(&(i, piece))?;
            }
        }
        ser.end()
    }
}
