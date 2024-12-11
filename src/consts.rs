use crate::bitboards::*;

pub const N: i8 = -8;
pub const NW: i8 = -9;
pub const NE: i8 = -7;
pub const NNW: i8 = -17;
pub const NNE: i8 = -15;
pub const NWW: i8 = -10;
pub const NEE: i8 = -6;
pub const W: i8 = -1;
pub const S: i8 = 8;
pub const SE: i8 = 9;
pub const SW: i8 = 7;
pub const SSE: i8 = 17;
pub const SSW: i8 = 15;
pub const SWW: i8 = 6;
pub const SEE: i8 = 10;
pub const E: i8 = 1;

pub const H_FILE: u64 = 0x8080808080808080;
pub const A_FILE: u64 = 0x0101010101010101;
pub const G_FILE: u64 = H_FILE >> 1;
pub const B_FILE: u64 = A_FILE << 1;
pub const ONES: u64 = u64::MAX;

pub const NOT_H_FILE: u64 = !H_FILE;
pub const NOT_A_FILE: u64 = !A_FILE;
pub const NOT_GH_FILE: u64 = !(H_FILE | G_FILE);
pub const NOT_AB_FILE: u64 = !(A_FILE | B_FILE);

pub const WHITE_CONSTS: Consts = Consts::WHITE;
pub const BLACK_CONSTS: Consts = Consts::BLACK;

macro_rules! const_moves {
    ($const:ident <- $func:expr; $($args:ident),*) => {
        pub const $const: [u64; 64] = {
            let mut moves = [0u64; 64];
            let mut i = 0;
            while i < 64 {
                moves[i] = $func(1 << i, $($args)*);
                i += 1;
            }
            moves
        };
    };
}

const_moves!(KNIGHT_MOVES <- knight_moves;);
const_moves!(KING_MOVES <- king_moves;);
const_moves!(WHITE_PAWN_ATTACKS <- pawn_attacks::<NW, NE>;);
const_moves!(WHITE_PAWN_ADVANCES <- pawn_advances::<N, ONES>; ONES);
const_moves!(BLACK_PAWN_ATTACKS <- pawn_attacks::<SW, SE>;);
const_moves!(BLACK_PAWN_ADVANCES <- pawn_advances::<S, ONES>; ONES);

pub struct Consts {
    pub eighth_rank: u64,
    pub third_rank: u64,
    pub ks_castle: u64,
    pub ks_castle_k: u64,
    pub ks_rook: u64,
    pub qs_castle: u64,
    pub qs_castle_k: u64,
    pub qs_rook: u64,
    pub direction: i8,
}
impl Consts {
    pub const WHITE: Consts = Consts {
        // first_rank: 0xFF00000000000000,
        third_rank: 0x0000FF0000000000,
        eighth_rank: 0x00000000000000FF,
        ks_castle: 0x6000000000000000,
        ks_castle_k: 0x4000000000000000,
        ks_rook: 0x8000000000000000,
        qs_castle: 0xE00000000000000,
        qs_castle_k: 0x400000000000000,
        qs_rook: 0x100000000000000,
        direction: 1,
    };
    pub const BLACK: Consts = Consts {
        // first_rank: 0x00000000000000FF,
        third_rank: 0x0000000000FF0000,
        eighth_rank: 0xFF00000000000000,
        ks_castle: 0x60,
        ks_castle_k: 0x40,
        ks_rook: 0x80,
        qs_castle: 0xE,
        qs_castle_k: 0x4,
        qs_rook: 1,
        direction: -1,
    };
}

#[inline(always)]
pub const fn pawn_advances<const D: i8, const R: u64>(piece: u64, free_squares: u64) -> u64 {
    let mut moves = 0;
    moves |= shift::<D, ONES>(piece) & free_squares;
    moves |= shift::<D, ONES>(moves & R) & free_squares;
    moves
}

pub const fn pawn_attacks<const L: i8, const R: i8>(piece: u64) -> u64 {
    shift::<L, NOT_A_FILE>(piece) | shift::<R, NOT_H_FILE>(piece)
}

const fn knight_moves(piece: u64) -> u64 {
    shift::<NNW, NOT_A_FILE>(piece)
        | shift::<NNE, NOT_H_FILE>(piece)
        | shift::<NWW, NOT_AB_FILE>(piece)
        | shift::<NEE, NOT_GH_FILE>(piece)
        | shift::<SSE, NOT_H_FILE>(piece)
        | shift::<SSW, NOT_A_FILE>(piece)
        | shift::<SEE, NOT_GH_FILE>(piece)
        | shift::<SWW, NOT_AB_FILE>(piece)
}

const fn king_moves(piece: u64) -> u64 {
    shift::<N, ONES>(piece)
        | shift::<S, ONES>(piece)
        | shift::<NW, NOT_A_FILE>(piece)
        | shift::<NE, NOT_H_FILE>(piece)
        | shift::<W, NOT_A_FILE>(piece)
        | shift::<SE, NOT_H_FILE>(piece)
        | shift::<SW, NOT_A_FILE>(piece)
        | shift::<E, NOT_H_FILE>(piece)
}
