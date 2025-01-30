use serde::ser::{SerializeTuple, Serializer};
use serde::Serialize;

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
//
// pub const u64x8_ones: simd::u64x8 = simd::u64x8::from_array([1; 8]);
// pub const u64x8_zeros: simd::u64x8 = simd::u64x8::from_array([0; 8]);
// const u64x8_63: simd::u64x8 = simd::u64x8::from_array([63; 8]);

// #[inline(always)]
// pub fn ms1b_simd(bb: simd::u64x8) -> simd::u64x8 {
//     u64x8_ones << (u64x8_63 - bb.leading_zeros())
// }

// #[inline(always)]
// pub fn ms1br(bb: u64) -> u64 {
//     ls1b(bb.reverse_bits())
// }

#[inline(always)]
pub const fn shift<const D: i8, const B: u64>(board: u64) -> u64 {
    if D >= 0 {
        (board & B) << D
    } else {
        (board & B) >> -D
    }
}

macro_rules! apply {
    ($pieces:expr, $idx:ident -> $expr:expr) => {
        let mut bb = $pieces;
        while bb != 0 {
            let $idx = bsf(bb) as usize;
            $expr;
            bb &= bb - 1;
        }
    };
}
pub(crate) use apply;

#[derive(Debug)]
pub struct BitBoards {
    pub pawns: u64,
    pub knights: u64,
    pub bishops: u64,
    pub rooks: u64,
    pub queens: u64,
    pub kings: u64,
    pub whites: u64,
    pub blacks: u64,
}

impl Serialize for BitBoards {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Piece<'a> {
            kind: &'a str,
            color: &'a str,
        }

        let mut ser = serializer.serialize_tuple(64)?;
        apply!(self.pawns & self.whites, i -> {
            ser.serialize_element(&(i, Piece { kind: "pawn", color: "white"}))?;
        });
        apply!(self.knights & self.whites, i -> {
            ser.serialize_element(&(i, Piece { kind: "knight", color: "white"}))?;
        });
        apply!(self.bishops & self.whites, i -> {
            ser.serialize_element(&(i, Piece { kind: "bishop", color: "white"}))?;
        });
        apply!(self.rooks & self.whites, i -> {
            ser.serialize_element(&(i, Piece { kind: "rook", color: "white"}))?;
        });
        apply!(self.queens & self.whites, i -> {
            ser.serialize_element(&(i, Piece { kind: "queen", color: "white"}))?;
        });
        apply!(self.kings & self.whites, i -> {
            ser.serialize_element(&(i, Piece { kind: "king", color: "white"}))?;
        });
        apply!(self.pawns & self.blacks, i -> {
            ser.serialize_element(&(i, Piece { kind: "pawn", color: "black"}))?;
        });
        apply!(self.knights & self.blacks, i -> {
            ser.serialize_element(&(i, Piece { kind: "knight", color: "black"}))?;
        });
        apply!(self.bishops & self.blacks, i -> {
            ser.serialize_element(&(i, Piece { kind: "bishop", color: "black"}))?;
        });
        apply!(self.rooks & self.blacks, i -> {
            ser.serialize_element(&(i, Piece { kind: "rook", color: "black"}))?;
        });
        apply!(self.queens & self.blacks, i -> {
            ser.serialize_element(&(i, Piece { kind: "queen", color: "black"}))?;
        });
        apply!(self.kings & self.blacks, i -> {
            ser.serialize_element(&(i, Piece { kind: "king", color: "black"}))?;
        });
        ser.end()
    }
}

impl BitBoards {
    pub fn from_fen(position: &str) -> Self {
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
        position.split('/').enumerate().for_each(|(rank, pieces)| {
            let mut file: usize = 0;
            pieces.chars().for_each(|c| {
                if let Some(blanks) = c.to_digit(10) {
                    file += blanks as usize;
                } else {
                    if c.is_uppercase() {
                        bitboard.whites |= 1 << (rank * 8 + file);
                    } else {
                        bitboard.blacks |= 1 << (rank * 8 + file);
                    };
                    match c.to_ascii_lowercase() {
                        'p' => bitboard.pawns |= 1 << (rank * 8 + file),
                        'n' => bitboard.knights |= 1 << (rank * 8 + file),
                        'b' => bitboard.bishops |= 1 << (rank * 8 + file),
                        'r' => bitboard.rooks |= 1 << (rank * 8 + file),
                        'q' => bitboard.queens |= 1 << (rank * 8 + file),
                        'k' => bitboard.kings |= 1 << (rank * 8 + file),
                        _ => {}
                    };
                    file += 1;
                }
            })
        });
        bitboard
    }

    pub fn get_color_bb_mut(&mut self, turn: bool) -> &mut u64 {
        if turn {
            return &mut self.whites;
        }
        &mut self.blacks
    }

    pub fn get_piece_bb_mut(&mut self, bb: u64) -> &mut u64 {
        if self.pawns & bb != 0 {
            &mut self.pawns
        } else if self.knights & bb != 0 {
            &mut self.knights
        } else if self.bishops & bb != 0 {
            &mut self.bishops
        } else if self.rooks & bb != 0 {
            &mut self.rooks
        } else if self.queens & bb != 0 {
            &mut self.queens
        } else if self.kings & bb != 0 {
            &mut self.kings
        } else {
            panic!("Given bitboard does not match any piece bitboard.");
        }
    }
}

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

    // #[bench]
    // fn ms1b_2(b: &mut Bencher) {
    //     b.iter(|| ms1br(1 << 50))
    // }
}
