#![feature(test)]
#![feature(unbounded_shifts)]
#![feature(portable_simd)]

extern crate test;
use wasm_bindgen::prelude::*;

mod bitboards;
mod consts;
mod gamestate;
mod moves;
mod rays;
mod letterbox;

#[derive(serde::Serialize)]
pub enum PieceEnum {
    Empty = 0,
    WhitePawn = 1,
    WhiteKnight = 2,
    WhiteBishop = 3,
    WhiteRook = 4,
    WhiteQueen = 5,
    WhiteKing = 6,
    BlackPawn = -1,
    BlackKnight = -2,
    BlackBishop = -3,
    BlackRook = -4,
    BlackQueen = -5,
    BlackKing = -6,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

pub fn showasm(i: &mut i32) {
    while *i > 0 {
        *i -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn legal_moves(b: &mut Bencher) {
        let mut board =
// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
            gamestate::Game::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        b.iter(|| board.calc_legal_moves())
    }

    #[test]
    fn enums_piece() {
        println!(
            "{}",
            serde_json::to_string_pretty(&(PieceEnum::WhitePawn as isize)).unwrap()
        );
    }
    #[test]
    fn it_panics() {
        let x = "asdf";
        panic!("aaaaah! {x}");
    }
}
