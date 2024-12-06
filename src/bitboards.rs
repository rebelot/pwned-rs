use std::simd::{self, num::SimdUint};

use crate::pieces::*;

#[derive(Debug)]
pub struct BitBoards {
    pub whites: u64,
    pub blacks: u64,
    pub pawns: u64,
    pub rooks: u64,
    pub bishops: u64,
    pub knights: u64,
    pub queens: u64,
    pub kings: u64,
}

#[inline(always)]
pub fn bsf(bb: u64) -> u32 {
    tzc(bb)
}

#[inline(always)]
pub fn tzc(bb: u64) -> u32 {
    bb.trailing_zeros()
}

#[inline(always)]
pub fn lzc(bb: u64) -> u32 {
    bb.leading_zeros()
}

#[inline(always)]
pub fn ls1b(bb: u64) -> u64 {
    (bb as i64 & -(bb as i64)) as u64
}

#[inline(always)]
pub fn ms1b(bb: u64) -> u64 {
    1 << (63 - lzc(bb))
}

pub const u64x8_ones: simd::u64x8 = simd::u64x8::from_array([1; 8]);
pub const u64x8_zeros: simd::u64x8 = simd::u64x8::from_array([0; 8]);
const u64x8_63: simd::u64x8 = simd::u64x8::from_array([63; 8]);

#[inline(always)]
pub fn ms1b_simd(bb: simd::u64x8) -> simd::u64x8 {
    u64x8_ones << (u64x8_63 - bb.leading_zeros())
}

#[inline(always)]
pub fn ms1br(bb: u64) -> u64 {
    ls1b(bb.reverse_bits())
}

#[inline(always)]
pub const fn shift<const D: i8, const B: u64>(board: u64) -> u64 {
    if D >= 0 {
        (board & B) << D
    } else {
        (board & B) >> -D
    }
}

impl BitBoards {
    pub fn new(board: &[Option<Piece>]) -> Self {
        let mut bitboard = BitBoards {
            whites: 0,
            blacks: 0,
            pawns: 0,
            rooks: 0,
            bishops: 0,
            knights: 0,
            queens: 0,
            kings: 0,
        };
        board.iter().enumerate().for_each(|(i, square)| {
            let mask = 1 << i;
            if let Some(piece) = square {
                match piece {
                    Piece::Pawn(_) => bitboard.pawns |= mask,
                    Piece::Rook(_) => bitboard.rooks |= mask,
                    Piece::Bishop(_) => bitboard.bishops |= mask,
                    Piece::Knight(_) => bitboard.knights |= mask,
                    Piece::King(_) => bitboard.kings |= mask,
                    Piece::Queen(_) => bitboard.queens |= mask,
                };
                match piece.color() {
                    Color::White => bitboard.whites |= mask,
                    Color::Black => bitboard.blacks |= mask,
                };
            }
        });

        bitboard
    }
    pub fn get_piece_bbs_mut(&mut self, piece: Piece) -> (&mut u64, &mut u64) {
        (
            match piece {
                Piece::Pawn(_) => &mut self.pawns,
                Piece::Rook(_) => &mut self.rooks,
                Piece::Bishop(_) => &mut self.bishops,
                Piece::Knight(_) => &mut self.knights,
                Piece::King(_) => &mut self.kings,
                Piece::Queen(_) => &mut self.queens,
            },
            match piece.color() {
                Color::White => &mut self.whites,
                Color::Black => &mut self.blacks,
            },
        )
    }

    pub fn to_string(board: u64) -> String {
        let ranks = [1, 2, 3, 4, 5, 6, 7, 8];
        let files = "abcdefgh";

        let mut squares = vec![];
        let mut b = board;
        let mut i = 0;
        while b != 0 {
            if b & 1 != 0 {
                let row = i / 8;
                let col = i % 8;
                squares.push(format!(
                    "{}{}",
                    files.chars().nth(col).unwrap(),
                    ranks[7 - row]
                ));
            }
            i += 1;
            b >>= 1;
        }

        squares.join(", ")
    }

    #[inline(always)]
    /// Apply closure `f` trait-bound to `FnMut(usize)` at the index *of each* **ON** bit in `board` bitboard
    ///
    /// * `board`: the bitboard to be searched for **ON** bits
    /// * `f`: the closure `|i: usize|` that recieves the `i`ndex of each ON bit
    pub fn apply<F: FnMut(usize)>(bb: u64, mut f: F) {
        let mut bb = bb;
        while bb != 0 {
            let idx = bsf(bb); // find next ls1b
            f(idx as usize);
            bb &= bb - 1; // clear LS1B
        }
    }
}

macro_rules! apply {
    ($pieces:ident, $idx:ident -> $expr:expr) => {
        let mut bb = $pieces;
        while bb != 0 {
            let $idx = bsf(bb) as usize;
            $expr;
            bb &= bb - 1;
        }
    };
}
pub(crate) use apply;

#[cfg(test)]
mod tests {
    use super::*;
    use std::simd::{
        cmp::{SimdOrd, SimdPartialEq, SimdPartialOrd},
        u8x4, Mask, Simd, SimdElement,
    };
    use test::Bencher;

    #[bench]
    fn find_while(b: &mut Bencher) {
        b.iter(|| {
            let mut idx = 0;
            let mut b: u64 = 1 << 63 | 1 << 50 | 1 << 25 | 1 << 5;
            let mut i = 0;
            while b != 0 {
                if b & 1 != 0 {
                    idx += i;
                }
                b >>= 1;
                i += 1;
            }
            assert_eq!(idx, 63 + 50 + 25 + 5);
        });
    }
    #[bench]
    fn find_leading(b: &mut Bencher) {
        b.iter(|| {
            let mut b: u64 = 1 << 63 | 1 << 50 | 1 << 25 | 1 << 5;
            let mut idx = 0;
            let mut sha = 0;
            while b != 0 {
                let lz = b.leading_zeros();
                idx += u64::BITS - lz - 1 - sha;
                sha += lz + 1;
                b <<= lz + 1;
            }
            assert_eq!(idx, 63 + 50 + 25 + 5);
        })
    }
    #[bench]
    fn find_trailing(b: &mut Bencher) {
        b.iter(|| {
            let mut b: u64 = 1 << 63 | 1 << 50 | 1 << 25 | 1 << 5;
            let mut idx = 0;
            let mut sha = 0;
            while b != 0 {
                let tz = b.trailing_zeros();
                idx += tz + sha;
                sha += tz + 1;
                b >>= tz + 1;
            }
            assert_eq!(idx, 63 + 50 + 25 + 5);
        })
    }
    #[bench]
    fn find_trailing_kill(b: &mut Bencher) {
        b.iter(|| {
            let mut b: u64 = 1 << 63 | 1 << 50 | 1 << 25 | 1 << 5;
            let mut idx = 0;
            while b != 0 {
                let tz = b.trailing_zeros();
                idx += tz;
                b ^= 1 << tz;
            }
            assert_eq!(idx, 63 + 50 + 25 + 5);
        })
    }
    #[test]
    fn tst() {
        let mut b: u64 = 1 << 63 | 1 << 50 | 1 << 25 | 1 << 5;
        let i = b.trailing_zeros();
        println!("{:064b}", b);
        println!("{:064b}", b ^ (1 << i));
        println!("{i} {}", (b ^ (1 << i)).trailing_zeros());
    }

    #[test]
    fn it_simd() {
        let a = u8x4::from_array([0b01000001, 0b00010001, 0b01010000, 0b00001100]);
        let mask = u8x4::splat(0b01000001_u8);
        println!("simd: {:?}", (a & mask).simd_eq(mask));
        assert_eq!(
            (a & mask).simd_eq(mask),
            Mask::from_array([true, false, false, false])
        );
    }

    #[bench]
    fn ms1b_1(b: &mut Bencher) {
        b.iter(|| ms1b(1 << 50))
    }

    #[bench]
    fn ms1b_2(b: &mut Bencher) {
        b.iter(|| ms1br(1 << 50))
    }
}
