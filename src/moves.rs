use crate::bitboards::*;
use crate::consts::*;
// use crate::pieces::*;
use crate::rays::*;

pub fn gen_atk_moves(color: bool, bitboards: &BitBoards) -> [u64; 64] {
    let (friends, foes, pawn_attacks): (u64, u64, [u64; 64]) = if color {
        (bitboards.whites, bitboards.blacks, WHITE_PAWN_ATTACKS)
    } else {
        (bitboards.blacks, bitboards.whites, BLACK_PAWN_ATTACKS)
    };

    let mut moves = [0; 64];
    let occupancy = friends ^ (foes & !bitboards.kings);

    // 1. No need to intersect with actual targets or occupied squares:
    //      Every square seen by an enemy piece counts as "attacked"
    //      If an enemy piece "attacks" another enemy piece, it means that piece is "defended"
    // 2. Attacking pawns don't move forward!
    // 3. Sliding pieces "don't see" the enemy king: the whole ray is hot!

    diagonal_attacks(
        friends & (bitboards.queens ^ bitboards.bishops),
        0,
        occupancy,
        &mut moves,
    );
    rankfile_attacks(
        friends & (bitboards.queens ^ bitboards.rooks),
        0,
        occupancy,
        &mut moves,
    );
    knight_attacks(friends & bitboards.knights, 0, &mut moves);

    let pawns = bitboards.pawns & friends;
    apply!(pawns, i -> moves[i] |= pawn_attacks[i]);
    let k = bsf(bitboards.kings & friends) as usize;
    moves[k] |= KING_MOVES[k];

    moves
}

pub fn gen_all_moves(
    color: bool,
    bitboards: &BitBoards,
    enpassant: Option<usize>,
    castling: u64,
) -> [u64; 64] {
    let friends: u64;
    let foes: u64;
    let consts: Consts;
    let pawn_moves: fn(u64, u64, u64, &mut [u64]);
    if color {
        friends = bitboards.whites;
        foes = bitboards.blacks;
        consts = WHITE_CONSTS;
        pawn_moves = white_pawn_moves;
    } else {
        friends = bitboards.blacks;
        foes = bitboards.whites;
        consts = BLACK_CONSTS;
        pawn_moves = black_pawn_moves;
    };

    let mut moves = [0; 64];
    let free_squares = !(friends ^ foes);
    let occupancy = !free_squares;

    let foes_attacks = gen_atk_moves(!color, bitboards);
    let attacked = foes_attacks.iter().fold(0, |acc, e| acc | *e);

    let king = friends & bitboards.kings;
    let k = bsf(king) as usize;

    let not_friends_or_attacked = !(friends | attacked);
    moves[k] |= KING_MOVES[k] & not_friends_or_attacked;

    let mut check_mask = 0;
    let mut in_check = false;
    if attacked & king != 0 {
        in_check = true;
        let mut checker: u64 = 0;
        let mut ch = 0;
        apply!(foes, i -> {
            if foes_attacks[i] & king != 0 {
                checker |= 1 << i;
                ch = i;
            }
        });

        // It's double check!
        if checker.count_ones() > 1 {
            return moves;
        };

        // pawns and knights can only be captured
        check_mask |= checker;

        if checker & (bitboards.queens ^ bitboards.bishops) != 0 {
            check_mask |= DIAGONALS_INTERSECT[ch][k];
        } // queens need both checks
        if checker & (bitboards.queens ^ bitboards.rooks) != 0 {
            check_mask |= RANKFILES_INTERSECT[ch][k];
        };
    } else {
        // castling masks don't include the king (poor choice?), so other checks are subordinated
        // to "not in check" status.
        if castling & consts.ks_castle & not_friends_or_attacked == consts.ks_castle
            && friends & consts.ks_rook != 0
        {
            moves[k] |= consts.ks_castle_k;
        }
        if castling & consts.qs_castle & not_friends_or_attacked == consts.qs_castle
            && friends & consts.qs_rook != 0
        {
            moves[k] |= consts.qs_castle_k;
        }
    }

    pawn_moves(
        friends & bitboards.pawns,
        foes | enpassant.map(|i| 1 << i).unwrap_or(0),
        free_squares,
        &mut moves,
    );

    diagonal_attacks(
        friends & (bitboards.queens ^ bitboards.bishops),
        friends,
        occupancy,
        &mut moves,
    );
    rankfile_attacks(
        friends & (bitboards.queens ^ bitboards.rooks),
        friends,
        occupancy,
        &mut moves,
    );
    knight_attacks(friends & bitboards.knights, friends, &mut moves);

    diagonal_pins(
        foes & (bitboards.queens ^ bitboards.bishops) & RAYS[k].diagonals,
        friends,
        foes,
        k,
        &mut moves,
    );
    rankfile_pins(
        foes & (bitboards.queens ^ bitboards.rooks) & RAYS[k].rankfiles,
        friends,
        foes,
        k,
        &mut moves,
    );
    if in_check {
        // Check mask restraints pieces moves to interposing or capture.
        // King moves are not affected, and only depend on `attacked` squares
        let pieces = friends & !king;
        apply!(pieces, i -> moves[i] &= check_mask);
    }
    moves
}

#[inline(always)]
pub fn white_pawn_moves(pawns: u64, targets: u64, free_squares: u64, moves: &mut [u64]) {
    let pawn_advances = {
        let mut moves = 0;
        moves |= pawns >> 8 & free_squares;
        moves |= (moves & WHITE_CONSTS.third_rank) >> 8 & free_squares;
        moves
    };
    apply!(pawns, i -> moves[i] |= (pawn_advances & WHITE_PAWN_ADVANCES[i]) ^ (WHITE_PAWN_ATTACKS[i] & targets));
}

#[inline(always)]
pub fn black_pawn_moves(pawns: u64, targets: u64, free_squares: u64, moves: &mut [u64]) {
    let pawn_advances = {
        let mut moves = 0;
        moves |= pawns << 8 & free_squares;
        moves |= (moves & BLACK_CONSTS.third_rank) << 8 & free_squares;
        moves
    };
    apply!(pawns, i -> moves[i] |= (pawn_advances & BLACK_PAWN_ADVANCES[i]) ^ (BLACK_PAWN_ATTACKS[i] & targets));
}

#[inline(always)]
fn knight_attacks(pieces: u64, friends: u64, moves: &mut [u64; 64]) {
    apply!(pieces, i -> moves[i] |= KNIGHT_MOVES[i] & !friends);
}

#[inline(always)]
pub fn line_attack(ray: &Ray, occ: u64) -> u64 {
    let low = ms1b(ray.negative & occ | 1);
    let high = ray.positive & occ;
    ray.line & ((high.wrapping_sub(low)) ^ high) // when there are no positive rays
                                                 // (H File, 1st Rank) -> high = 0!
}

#[inline(always)]
pub fn sliding_attacks<const R: usize>(square: usize, occ: u64) -> u64 {
    let rays = RAYS[square];
    line_attack(&rays[R], occ) ^ line_attack(&rays[R + 1], occ)
}

#[inline(always)]
fn diagonal_attacks(pieces: u64, friends: u64, occupancy: u64, moves: &mut [u64; 64]) {
    apply!(
        pieces,
        i ->
        moves[i] |= sliding_attacks::<{ Rays::DIAGONALS }>(i, occupancy) & !friends
    );
}

#[inline(always)]
fn rankfile_attacks(pieces: u64, friends: u64, occupancy: u64, moves: &mut [u64; 64]) {
    apply!(
        pieces,
        i ->
        moves[i] |= sliding_attacks::<{ Rays::RANKFILES }>(i, occupancy) & !friends
    );
}

#[inline(always)]
fn pin_mask(xray: u64, friends: u64, foes: u64, moves: &mut [u64; 64]) {
    let maybe_pins = friends & xray;
    if xray & foes == 0 && maybe_pins.count_ones() == 1 {
        let idx = bsf(maybe_pins);
        moves[idx as usize] &= xray;
    }
}

#[inline(always)]
fn diagonal_pins(pieces: u64, friends: u64, foes: u64, k: usize, moves: &mut [u64; 64]) {
    apply!(
        pieces,
        i ->
        pin_mask(DIAGONALS_INTERSECT[i][k], friends, foes, moves)
    );
}

#[inline(always)]
fn rankfile_pins(pieces: u64, friends: u64, foes: u64, k: usize, moves: &mut [u64; 64]) {
    apply!(
        pieces,
        i ->
        pin_mask(RANKFILES_INTERSECT[i][k], friends, foes, moves)
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_shift(b: &mut Bencher) {
        b.iter(|| shift::<N, ONES>(1 << 63));
    }
}
