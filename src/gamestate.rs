use js_sys::Int32Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::bitboards::*;
use crate::consts::*;
use crate::moves::*;

#[derive(Serialize, Deserialize)]
struct MoveResponse {
    updates: Vec<(usize, i32)>,
}

#[wasm_bindgen]
pub struct Game {
    bitboards: BitBoards,
    turn: bool,
    castling: u64,
    enpassant: Option<usize>,
    halfmove: u32,
    fullmove: u32,
    legal_moves: [u64; 64],
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(fen: &str) -> Self {
        let mut fields = fen.split_whitespace();

        let position = fields.next().unwrap();
        let bitboards = BitBoards::new(position);
        let turn = match fields.next().unwrap() {
            "w" => true,
            "b" => false,
            c => panic!("Unexpected color: {c}"),
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
            Some(r as usize * 8 + f)
        } else {
            None
        };

        let halfmove = fields.next().unwrap().parse().unwrap();
        let fullmove = fields.next().unwrap().parse().unwrap();

        let mut game = Game {
            bitboards,
            turn,
            castling,
            enpassant,
            halfmove,
            fullmove,
            legal_moves: [0; 64],
        };

        game.legal_moves = gen_all_moves(game.turn, &game.bitboards, enpassant, castling);
        game
    }

    #[wasm_bindgen]
    pub fn turn(&mut self) {
        if !self.turn {
            self.fullmove += 1;
        }
        self.turn = !self.turn
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
        serde_json::to_string(&self.bitboards).expect("oh shit")
    }

    #[wasm_bindgen]
    pub fn input_move(&mut self, from: usize, to: usize, promotion: usize) -> String {
        let from_bb: u64 = 1 << from;
        let to_bb: u64 = 1 << to;
        let from_to_bb = from_bb ^ to_bb;
        if self.legal_moves[from] & to_bb == 0 {
            let response = (false, String::from("Invalid move"));
            return serde_json::to_string(&response).unwrap();
        };

        let (consts, foes) = if self.turn {
            (Consts::WHITE, self.bitboards.blacks)
        } else {
            (Consts::BLACK, self.bitboards.whites)
        };

        let mut response = MoveResponse {
            updates: vec![(from, to as i32)],
        };

        *self.bitboards.get_color_bb_mut(self.turn) ^= from_to_bb;
        if foes & to_bb != 0 {
            *self.bitboards.get_color_bb_mut(!self.turn) ^= to_bb;
            *self.bitboards.get_piece_bb_mut(to_bb) ^= to_bb;
        }

        let enpassant = self.enpassant;
        self.enpassant = None;

        if self.bitboards.pawns & from_bb != 0 {
            if let Some(enpassant) = enpassant {
                if to == enpassant {
                    let captured_pawn = enpassant + (8 * consts.direction as usize);
                    let captured_pawn_bb = 1 << captured_pawn;
                    self.bitboards.pawns ^= captured_pawn_bb;
                    *self.bitboards.get_color_bb_mut(!self.turn) ^= captured_pawn_bb;
                    response.updates.push((captured_pawn, -1));
                }
            } else if (to as i32 - from as i32).abs() == 16 {
                self.enpassant = Some((to as i32 + 8 * consts.direction as i32) as usize);
            } else if to_bb & consts.eighth_rank != 0 {
                self.bitboards.pawns ^= to_bb;
                match promotion {
                    1 => {
                        self.bitboards.queens |= to_bb;
                    }
                    2 => {
                        self.bitboards.rooks |= to_bb;
                    }
                    3 => {
                        self.bitboards.bishops |= to_bb;
                    }
                    4 => {
                        self.bitboards.knights |= to_bb;
                    }
                    _ => {}
                }
            }
        } else if self.bitboards.kings & from_bb != 0 {
            self.castling &= !(consts.ks_castle | consts.qs_castle);
            match to as i32 - from as i32 {
                2 => {
                    self.bitboards.rooks ^= consts.ks_rook | (consts.ks_rook >> 2);
                    *self.bitboards.get_color_bb_mut(self.turn) ^=
                        consts.ks_rook | (consts.ks_rook >> 2);
                    response.updates.push((to + 1, to as i32 - 1))
                }
                -2 => {
                    self.bitboards.rooks ^= consts.ks_rook | (consts.ks_rook >> 2);
                    *self.bitboards.get_color_bb_mut(self.turn) ^=
                        consts.ks_rook | (consts.ks_rook << 3);
                    response.updates.push((to - 2, to as i32 + 1))
                }
                _ => {}
            }
        } else if consts.ks_rook & from_bb != 0 {
            self.castling &= !consts.ks_castle;
        } else if consts.qs_rook & from_bb != 0 {
            self.castling &= !consts.qs_castle;
        }

        *self.bitboards.get_piece_bb_mut(from_bb) ^= from_to_bb;

        self.turn();
        self.calc_legal_moves();
        serde_json::to_string(&(true, response)).unwrap()
    }
}
