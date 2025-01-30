#![feature(test)]
#![feature(unbounded_shifts)]
#![feature(portable_simd)]

extern crate test;
use wasm_bindgen::prelude::*;

mod bitboards;
mod consts;
mod gamestate;
mod letterbox;
mod moves;
mod rays;

#[wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub unsafe fn log(s: &str);
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
            gamestate::Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        b.iter(|| board.calc_legal_moves())
    }

    #[test]
    fn it_panics() {
        let x = "asdf";
        panic!("aaaaah! {x}");
    }
}
