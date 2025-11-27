use chess::{Attacks, Piece, Side, Square};

pub struct ThreatsExtended;

impl ThreatsExtended {
    pub const INPUT_SIZE: usize = ValueOffsets::END * 2 + 768;
    const THREATS_OFFSET: usize = ValueOffsets::END * 2;

    pub fn map_inputs<F: FnMut(usize)>(board: &chess::ChessBoard, mut process_input: F) {
        let mut board = *board;

        if board.side() == Side::BLACK {
            board.flip();
        }

        if board.king_square(Side::WHITE).file() > 3 {
            board.mirror();
        }

        let occ = board.occupancy();

        let mut piece_map = [13usize; 64]; 
        for side in [Side::WHITE, Side::BLACK] {
            let base = 6 * usize::from(side);
            for piece_idx in 0..6 {
                let mask = board.piece_mask_for_side(Piece::from(piece_idx as u8), side);
                mask.map(|sq| piece_map[usize::from(sq)] = base + piece_idx)
            }
        }

        for side in [Side::WHITE, Side::BLACK] {
            let side_idx = usize::from(side);
            let side_offset = ValueOffsets::END * side_idx; 
            
            let exist_base = Self::THREATS_OFFSET + (side_idx * 384); 

            let enemy_occ = board.occupancy_for_side(side.flipped());

            for piece_idx in 0..6 {
                let piece = Piece::from(piece_idx as u8);
                let mask = board.piece_mask_for_side(piece, side);

                mask.map(|src| {
                    let sq_idx = usize::from(src);
                    process_input(exist_base + (piece_idx * 64) + sq_idx);

                    let attacks_bb = match piece {
                        Piece::PAWN => Attacks::get_pawn_attacks(src, side),
                        Piece::KNIGHT => Attacks::get_knight_attacks(src),
                        Piece::BISHOP => Attacks::get_bishop_attacks(src, occ),
                        Piece::ROOK => Attacks::get_rook_attacks(src, occ),
                        Piece::QUEEN => Attacks::get_bishop_attacks(src, occ) 
                                      | Attacks::get_rook_attacks(src, occ),
                        Piece::KING => Attacks::get_king_attacks(src),
                        _ => unreachable!(),
                    } & occ;

                    attacks_bb.map(|dest| {
                        let is_enemy = enemy_occ.get_bit(dest);
                        let target_type = piece_map[usize::from(dest)];

                        if let Some(idx) = map_threat(piece, src, dest, target_type, is_enemy) {
                            process_input(side_offset + idx);
                        }
                    });
                });
            }
        }
    }
}

#[inline(always)]
fn map_threat(
    piece: Piece,
    src: Square,
    dest: Square,
    target: usize,
    enemy: bool,
) -> Option<usize> {
    let src = usize::from(src);
    let dest = usize::from(dest);

    match piece {
        Piece::PAWN => map_pawn(src, dest, target, enemy),
        Piece::KNIGHT => map_knight(src, dest, target),
        Piece::BISHOP => map_bishop(src, dest, target),
        Piece::ROOK => map_rook(src, dest, target),
        Piece::QUEEN => map_queen(src, dest, target),
        Piece::KING => map_king(src, dest, target),
        _ => None,
    }
}

#[inline(always)]
fn below(src: usize, dest: usize, table: &[u64; 64]) -> usize {
    (table[src] & ((1u64 << dest) - 1)).count_ones() as usize
}

#[inline(always)]
fn target_is(target: usize, piece: Piece) -> bool {
    target % 6 == usize::from(piece)
}

fn map_pawn(src: usize, dest: usize, target: usize, enemy: bool) -> Option<usize> {
    const MAP: [usize; 12] = gen_map(&[Piece::PAWN, Piece::KNIGHT, Piece::ROOK]); 
    
    if MAP[target] == usize::MAX || (enemy && dest > src && target_is(target, Piece::PAWN)) {
        None
    } else {
        let up = usize::from(dest > src);
        let diff = dest.abs_diff(src);
        let id = if diff == [9, 7][up] { 0 } else { 1 };
        let attack = 2 * (src % 8) + id - 1;
        
        let threat = ValueOffsets::PAWN 
            + MAP[target] * ValueIndices::PAWN 
            + (src / 8 - 1) * 14
            + attack;

        Some(threat)
    }
}

fn map_knight(src: usize, dest: usize, target: usize) -> Option<usize> {
    if dest > src && target_is(target, Piece::KNIGHT) {
        None
    } else {
        let idx = ValueIndices::KNIGHT[src] + below(src, dest, &PseudoAttacks::KNIGHT);
        let threat = ValueOffsets::KNIGHT + target * ValueIndices::KNIGHT[64] + idx;
        Some(threat)
    }
}

fn map_bishop(src: usize, dest: usize, target: usize) -> Option<usize> {
    const MAP: [usize; 12] = gen_map(&[
        Piece::PAWN,
        Piece::KNIGHT,
        Piece::BISHOP,
        Piece::ROOK,
        Piece::KING
    ]);

    if MAP[target] == usize::MAX || (dest > src && target_is(target, Piece::BISHOP)) {
        None
    } else {
        let idx = ValueIndices::BISHOP[src] + below(src, dest, &PseudoAttacks::BISHOP);
        let threat = ValueOffsets::BISHOP + MAP[target] * ValueIndices::BISHOP[64] + idx;
        Some(threat)
    }
}

fn map_rook(src: usize, dest: usize, target: usize) -> Option<usize> {
    const MAP: [usize; 12] = gen_map(&[
        Piece::PAWN,
        Piece::KNIGHT,
        Piece::BISHOP,
        Piece::ROOK,
        Piece::KING
    ]);

    if MAP[target] == usize::MAX || (dest > src && target_is(target, Piece::ROOK)) {
        None
    } else {
        let idx = ValueIndices::ROOK[src] + below(src, dest, &PseudoAttacks::ROOK);
        let threat = ValueOffsets::ROOK + MAP[target] * ValueIndices::ROOK[64] + idx;
        Some(threat)
    }
}

fn map_queen(src: usize, dest: usize, target: usize) -> Option<usize> {
    if dest > src && target_is(target, Piece::QUEEN) {
        None
    } else {
        let idx = ValueIndices::QUEEN[src] + below(src, dest, &PseudoAttacks::QUEEN);
        let threat = ValueOffsets::QUEEN + target * ValueIndices::QUEEN[64] + idx;
        Some(threat)
    }
}

fn map_king(src: usize, dest: usize, target: usize) -> Option<usize> {
    const MAP: [usize; 12] = gen_map(&[Piece::PAWN, Piece::KNIGHT, Piece::BISHOP, Piece::ROOK]);

    if MAP[target] == usize::MAX {
        None
    } else {
        let idx = ValueIndices::KING[src] + below(src, dest, &PseudoAttacks::KING);
        let threat = ValueOffsets::KING + MAP[target] * ValueIndices::KING[64] + idx;
        Some(threat)
    }
}

const fn gen_map(pieces: &[Piece]) -> [usize; 12] {
    let mut res = [usize::MAX; 12];
    let mut i = 0;
    while i < pieces.len() {
        let p = pieces[i].value();
        res[p] = i;
        res[p + 6] = i + pieces.len();
        i += 1;
    }
    res
}

pub struct ValueOffsets;
impl ValueOffsets {
    pub const PAWN: usize = 0;
    pub const KNIGHT: usize = Self::PAWN + 6 * ValueIndices::PAWN;
    pub const BISHOP: usize = Self::KNIGHT + 12 * ValueIndices::KNIGHT[64];
    pub const ROOK: usize = Self::BISHOP + 10 * ValueIndices::BISHOP[64];
    pub const QUEEN: usize = Self::ROOK + 10 * ValueIndices::ROOK[64];
    pub const KING: usize = Self::QUEEN + 12 * ValueIndices::QUEEN[64];
    pub const END: usize = Self::KING + 8 * ValueIndices::KING[64];
}

pub struct ValueIndices;
impl ValueIndices {
    pub const PAWN: usize = 84;
    pub const KNIGHT: [usize; 65] = compute_indices(PseudoAttacks::KNIGHT);
    pub const BISHOP: [usize; 65] = compute_indices(PseudoAttacks::BISHOP);
    pub const ROOK: [usize; 65] = compute_indices(PseudoAttacks::ROOK);
    pub const QUEEN: [usize; 65] = compute_indices(PseudoAttacks::QUEEN);
    pub const KING: [usize; 65] = compute_indices(PseudoAttacks::KING);
}

const fn compute_indices(attacks: [u64; 64]) -> [usize; 65] {
    let mut indices = [0; 65];
    let mut acc = 0;
    let mut i = 0;
    while i < 64 {
        indices[i] = acc;
        acc += attacks[i].count_ones() as usize;
        i += 1;
    }
    indices[64] = acc;
    indices
}

struct PseudoAttacks;
impl PseudoAttacks {
    const KNIGHT: [u64; 64] = {
        let mut t = [0; 64];
        let mut i = 0;
        while i < 64 {
            let n = 1u64 << i;
            let h1 = ((n >> 1) & 0x7f7f_7f7f_7f7f_7f7f) | ((n << 1) & 0xfefe_fefe_fefe_fefe);
            let h2 = ((n >> 2) & 0x3f3f_3f3f_3f3f_3f3f) | ((n << 2) & 0xfcfc_fcfc_fcfc_fcfc);
            t[i] = (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8);
            i += 1;
        }
        t
    };

    const BISHOP: [u64; 64] = {
        let mut t = [0; 64];
        let mut i = 0;
        while i < 64 { t[i] = mask_bishop(i as u8); i += 1; }
        t
    };

    const ROOK: [u64; 64] = {
        let mut t = [0; 64];
        let mut i = 0;
        while i < 64 { t[i] = mask_rook(i as u8); i += 1; }
        t
    };

    const QUEEN: [u64; 64] = {
        let mut t = [0; 64];
        let mut i = 0;
        while i < 64 { t[i] = Self::BISHOP[i] | Self::ROOK[i]; i += 1; }
        t
    };

    const KING: [u64; 64] = {
        let mut t = [0; 64];
        let mut i = 0;
        while i < 64 {
            let mut k = 1u64 << i;
            k |= (k << 8) | (k >> 8);
            k |= ((k & !0x0101_0101_0101_0101) >> 1) | ((k & !0x8080_8080_8080_8080) << 1);
            t[i] = k ^ (1u64 << i);
            i += 1;
        }
        t
    };
}

const fn mask_bishop(sq: u8) -> u64 {
    let mut res: u64 = 0;
    let r = (sq / 8) as i8; let f = (sq % 8) as i8;
    let mut i = 1;
    while r + i <= 7 && f + i <= 7 { res |= 1 << ((r + i) * 8 + f + i); i += 1; }
    i = 1; while r - i >= 0 && f + i <= 7 { res |= 1 << ((r - i) * 8 + f + i); i += 1; }
    i = 1; while r - i >= 0 && f - i >= 0 { res |= 1 << ((r - i) * 8 + f - i); i += 1; }
    i = 1; while r + i <= 7 && f - i >= 0 { res |= 1 << ((r + i) * 8 + f - i); i += 1; }
    res
}

const fn mask_rook(sq: u8) -> u64 {
    let mut res: u64 = 0;
    let r = (sq / 8) as i8; let f = (sq % 8) as i8;
    let mut i = 1;
    while r + i <= 7 { res |= 1 << ((r + i) * 8 + f); i += 1; }
    i = 1; while r - i >= 0 { res |= 1 << ((r - i) * 8 + f); i += 1; }
    i = 1; while f + i <= 7 { res |= 1 << (r * 8 + f + i); i += 1; }
    i = 1; while f - i >= 0 { res |= 1 << (r * 8 + f - i); i += 1; }
    res
}