use crate::bitboards::shift;
use crate::consts::*;

#[derive(Clone, Copy)]
pub struct Rays {
    pub rays: [Ray; 4],
    pub diagonals: u64,
    pub rankfiles: u64,
}

impl Rays {
    pub const fn default() -> Self {
        Self {
            rays: [Ray::default(); 4],
            diagonals: 0,
            rankfiles: 0,
        }
    }
}
impl std::ops::Index<usize> for Rays {
    type Output = Ray;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.rays[idx]
    }
}

impl Rays {
    pub const RANKFILES: usize = 0;
    pub const DIAGONALS: usize = 2;
}

#[derive(Clone, Copy)]
pub struct Ray {
    pub positive: u64,
    pub negative: u64,
    pub line: u64,
}
impl Ray {
    pub const fn default() -> Self {
        Self {
            positive: 0,
            negative: 0,
            line: 0,
        }
    }
}

pub static RAYS: [Rays; 64] = {
    let mut i = 0;
    let mut rays = [Rays::default(); 64];
    while i < 64 {
        // negatives
        rays[i].rays[0].negative = scan_ray::<N, ONES>(1 << i, ONES);
        rays[i].rays[1].negative = scan_ray::<W, NOT_A_FILE>(1 << i, ONES);
        rays[i].rays[2].negative = scan_ray::<NE, NOT_H_FILE>(1 << i, ONES);
        rays[i].rays[3].negative = scan_ray::<NW, NOT_A_FILE>(1 << i, ONES);
        // positives
        rays[i].rays[0].positive = scan_ray::<S, ONES>(1 << i, ONES);
        rays[i].rays[1].positive = scan_ray::<E, NOT_H_FILE>(1 << i, ONES);
        rays[i].rays[2].positive = scan_ray::<SW, NOT_A_FILE>(1 << i, ONES);
        rays[i].rays[3].positive = scan_ray::<SE, NOT_H_FILE>(1 << i, ONES);

        rays[i].rays[0].line = rays[i].rays[0].positive | rays[i].rays[0].negative;
        rays[i].rays[1].line = rays[i].rays[1].positive | rays[i].rays[1].negative;
        rays[i].rays[2].line = rays[i].rays[2].positive | rays[i].rays[2].negative;
        rays[i].rays[3].line = rays[i].rays[3].positive | rays[i].rays[3].negative;

        rays[i].rankfiles = rays[i].rays[0].line | rays[i].rays[1].line;
        rays[i].diagonals = rays[i].rays[2].line | rays[i].rays[3].line;

        i += 1
    }
    rays
};

pub static RANKFILES_INTERSECT: [[u64; 64]; 64] = {
    let mut intersections = [[0u64; 64]; 64];
    let mut i = 0;
    while i < 64 {
        let mut j = 0;
        while j < i {
            intersections[i][j] = ray_intersect::<{ Rays::DIAGONALS }>(i, j);
            intersections[j][i] = intersections[i][j];
            j += 1;
        }
        i += 1;
    }
    intersections
};

pub static DIAGONALS_INTERSECT: [[u64; 64]; 64] = {
    let mut intersections = [[0u64; 64]; 64];
    let mut i = 0;
    while i < 64 {
        let mut j = 0;
        while j < i {
            intersections[i][j] = ray_intersect::<{ Rays::RANKFILES }>(i, j);
            intersections[j][i] = intersections[i][j];
            j += 1;
        }
        i += 1;
    }
    intersections
};

#[inline(always)]
const fn ray_intersect<const R: usize>(sq1: usize, sq2: usize) -> u64 {
    let r1_0 = &RAYS[sq1].rays[R];
    let r1_1 = &RAYS[sq1].rays[R + 1];
    let r2_0 = &RAYS[sq2].rays[R];
    let r2_1 = &RAYS[sq2].rays[R + 1];

    let x = r1_0.positive & r2_0.negative;
    if x != 0 {
        return x;
    };

    let x = r1_0.negative & r2_0.positive;
    if x != 0 {
        return x;
    };

    let x = r1_1.negative & r2_1.positive;
    if x != 0 {
        return x;
    };
    r1_1.positive & r2_1.negative
}

#[inline]
/// Repeatedly shifts `board` bitboard by `D` bits, checking for `B` boundaries before shifting:
/// ```
///    if D >= 0 {
///        (board & B) << D // unbounded
///    } else {
///        (board & B) >> -D // unbounded
///    }
///
/// ```
/// Each bit of the `board` bitboard is shifted up to (and including!) the first **occupied square**.
/// Each shifted bit will produce the mask `(start ... 1st occupied square]`
/// * `board`: the bitboard to be shifted
/// * `free_squares`: the bitboard representing unoccupied squares
/// * `-> moves`: the bitboard of successfull shifts.
pub const fn scan_ray<const D: i8, const B: u64>(board: u64, free_squares: u64) -> u64 {
    let mut moves = 0u64;
    let mut p = board;
    while p != 0 {
        p = shift::<D, B>(p);
        moves |= p;
        p &= free_squares;
    }
    moves
}
