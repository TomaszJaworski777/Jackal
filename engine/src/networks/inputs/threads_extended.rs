use chess::{Attacks, Bitboard, Piece, Side, Square};

const KING_ALIGN_OFFSET: usize = 768 * 2 * 2;
const PAWN_ATTACKS_OFFSET: usize = KING_ALIGN_OFFSET + 128 * 3 * 2;
const KNIGHT_ATTACKS_OFFSET: usize = PAWN_ATTACKS_OFFSET + 96 * 2 * 2 * 4;
const BISHOP_ATTACKS_OFFSET: usize = KNIGHT_ATTACKS_OFFSET + 128 * 8 * 2 * 7;
const ROOK_ATTACKS_OFFSET: usize = BISHOP_ATTACKS_OFFSET + 128 * 4 * 2 * 2 * 5;
const QUEEN_ATTACKS_OFFSET: usize = ROOK_ATTACKS_OFFSET + 128 * 4 * 2 * 2 * 5;
const KING_ATTACK_OFFSETS: usize = QUEEN_ATTACKS_OFFSET + 128 * 8 * 2 * 2 * 6;
const SIZE: usize = KING_ATTACK_OFFSETS + 128 * 2 * 5;

pub struct ThreatsExtended;

#[allow(unused)]
impl ThreatsExtended {
    pub const fn input_size() -> usize {
        SIZE
    }

    pub fn map_inputs<F: FnMut(usize)>(board: &chess::ChessBoard, mut process_input: F) {
        let mut board = *board;

        if board.side() == Side::BLACK {
            board.flip();
        }

        if board.king_square(Side::WHITE).file() > 3 {
            board.mirror();
        }

        let bb_pawn = board.piece_mask(Piece::PAWN);
        let bb_knight = board.piece_mask(Piece::KNIGHT);
        let bb_bishop = board.piece_mask(Piece::BISHOP);
        let bb_rook = board.piece_mask(Piece::ROOK);
        let bb_queen = board.piece_mask(Piece::QUEEN);
        let bb_king = board.piece_mask(Piece::KING);

        let mut piece_map = [Piece::KING; 64];
        bb_pawn.map(|sq| piece_map[usize::from(sq)] = Piece::PAWN);
        bb_knight.map(|sq| piece_map[usize::from(sq)] = Piece::KNIGHT);
        bb_bishop.map(|sq| piece_map[usize::from(sq)] = Piece::BISHOP);
        bb_rook.map(|sq| piece_map[usize::from(sq)] = Piece::ROOK);
        bb_queen.map(|sq| piece_map[usize::from(sq)] = Piece::QUEEN);

        let (mut diag_stm, mut ortho_stm) = board.generate_pin_masks(Side::WHITE);
        let (mut diag_nstm, mut ortho_nstm) = board.generate_pin_masks(Side::BLACK);

        let mut base_side_offset = 0;
        let mut align_side_offset = 0;
        let mut pawn_attack_offset = 0;
        let mut piece_attack_offset = 0;

        let occ = board.occupancy();

        for piece_color in [Side::WHITE, Side::BLACK] {
            let occ_nstm = board.occupancy_for_side(piece_color.flipped());

            let enemy_king = board.king_square(piece_color.flipped());
            let enemy_ring = Attacks::get_king_attacks(enemy_king);

            for piece_idx in u8::from(Piece::PAWN)..=u8::from(Piece::KING) {
                let piece = Piece::from(piece_idx);
                let base_feat_idx = base_side_offset + 64 * (piece_idx - u8::from(Piece::PAWN)) as usize;
                let align_feat_idx = KING_ALIGN_OFFSET + align_side_offset + 64 * (piece_idx - u8::from(Piece::BISHOP)) as usize;

                board.piece_mask_for_side(piece, piece_color).map(|square| {
                    let sq_idx = usize::from(square);
                    let mut feat = base_feat_idx + sq_idx;

                    if diag_stm.get_bit(square) { feat += 768; }
                    if ortho_stm.get_bit(square) { feat += 768 * 2; }

                    process_input(feat);

                    if piece_idx >= 2 && piece_idx <= 4 {
                        let (b_attacks, r_attacks) = match piece {
                            Piece::BISHOP => (BISHOP_ATTACKS[sq_idx], Bitboard::EMPTY),
                            Piece::ROOK => (Bitboard::EMPTY, ROOK_ATTACKS[sq_idx]),
                            Piece::QUEEN => (BISHOP_ATTACKS[sq_idx], ROOK_ATTACKS[sq_idx]),
                            _ => (Bitboard::EMPTY, Bitboard::EMPTY)
                        };
                        
                        let attacks = b_attacks | r_attacks;
                        let enemy_king_bb = Bitboard::from(enemy_king);

                        if (attacks & enemy_king_bb).is_not_empty() {
                            process_input(align_feat_idx + sq_idx + 384);
                        } else if (attacks & enemy_ring).is_not_empty() {
                            process_input(align_feat_idx + sq_idx);
                        }
                    }

                    match piece {
                        Piece::PAWN => {
                            let attacks = Attacks::get_pawn_attacks(square, piece_color);
                            let valid_targets = attacks & (bb_pawn | bb_knight | bb_rook);
                            
                            valid_targets.map(|t_sq| {
                                let t_idx = usize::from(t_sq);
                                let is_enemy = occ_nstm.get_bit(t_sq);
                                let t_piece = piece_map[t_idx];
                                
                                let type_code = match t_piece { Piece::PAWN => 0, Piece::KNIGHT => 1, _ => 2 };
                                let dir = get_pawn_dir(sq_idx, t_idx, piece_color == Side::WHITE);
                                
                                process_input(PAWN_ATTACKS_OFFSET 
                                    + type_code * 384 
                                    + (is_enemy as usize) * 192 
                                    + dir * 96 
                                    + (sq_idx - 8 + pawn_attack_offset));
                            });

                            let ring_hits = attacks & enemy_ring;
                            ring_hits.map(|t_sq| {
                                let dir = get_pawn_dir(sq_idx, usize::from(t_sq), piece_color == Side::WHITE);
                                process_input(PAWN_ATTACKS_OFFSET + 1344 + dir * 96 + (sq_idx - 8 + pawn_attack_offset));
                            });
                        },
                        Piece::KNIGHT => {
                            let attacks = Attacks::get_knight_attacks(square);
                            let valid_targets = attacks & occ;
                            valid_targets.map(|t_sq| {
                                let t_idx = usize::from(t_sq);
                                let is_enemy = occ_nstm.get_bit(t_sq);
                                let t_piece = piece_map[t_idx];

                                if t_piece == piece && t_idx < sq_idx { return; }

                                let dir = get_knight_dir(sq_idx, t_idx);

                                process_input(KNIGHT_ATTACKS_OFFSET 
                                    + (u8::from(t_piece) as usize) * 2048 
                                    + (is_enemy as usize) * 1024 
                                    + dir * 128 
                                    + (sq_idx + piece_attack_offset));
                            });

                            let ring_hits = attacks & enemy_ring;
                            ring_hits.map(|t_sq| {
                                let dir = get_knight_dir(sq_idx, usize::from(t_sq));
                                process_input(KNIGHT_ATTACKS_OFFSET + 13312 + dir * 128 + (sq_idx + piece_attack_offset));
                            });
                        },
                        Piece::KING => {
                            let attacks = Attacks::get_king_attacks(square);
                            let valid_targets = attacks & occ & !bb_queen & !bb_king;
                            valid_targets.map(|t_sq| {
                                let t_idx = usize::from(t_sq);
                                let t_piece = piece_map[t_idx];
                                let is_enemy = occ_nstm.get_bit(t_sq);

                                process_input(KING_ATTACK_OFFSETS 
                                    + (u8::from(t_piece) as usize) * 256 
                                    + (is_enemy as usize) * 128 
                                    + (sq_idx + piece_attack_offset));
                            });
                            
                            if (attacks & enemy_ring).is_not_empty() {
                                process_input(KING_ATTACK_OFFSETS + 1152 + (sq_idx + piece_attack_offset));
                            }
                        },
                        _ => {
                            let (base_offset, compass_mode) = match piece {
                                Piece::BISHOP => (BISHOP_ATTACKS_OFFSET, 1),
                                Piece::ROOK => (ROOK_ATTACKS_OFFSET, 0),
                                _ => (QUEEN_ATTACKS_OFFSET, 2),
                            };
                            
                            let target_mask = if piece == Piece::QUEEN { Bitboard::FULL } else { !bb_queen };

                            let attacks_bb = match piece {
                                Piece::BISHOP => Attacks::get_bishop_attacks(square, occ),
                                Piece::ROOK =>  Attacks::get_rook_attacks(square, occ),
                                _ =>  Attacks::get_bishop_attacks(square, occ) |  Attacks::get_rook_attacks(square, occ),
                            };
                            
                            let direct_hits = attacks_bb & occ;
                            let valid_direct = direct_hits & target_mask;
                            valid_direct.map(|t_sq| {
                                let t_idx = usize::from(t_sq);
                                let is_enemy = occ_nstm.get_bit(t_sq);
                                let t_piece = piece_map[t_idx];
                                
                                if t_piece == piece && t_idx < sq_idx { return; }
                                
                                let type_idx = if piece != Piece::QUEEN && t_piece == Piece::KING { 4 } else { u8::from(t_piece) as usize };
                                let dir = get_slider_dir(sq_idx, t_idx, compass_mode);
                                let type_stride = if piece == Piece::QUEEN { 4096 } else { 2048 };
                                let color_stride = if piece == Piece::QUEEN { 2048 } else { 1024 };

                                process_input(base_offset 
                                    + type_idx * type_stride 
                                    + (is_enemy as usize) * color_stride 
                                    + dir * 128 
                                    + (sq_idx + piece_attack_offset));
                            });

                            let xray_occ = occ ^ direct_hits;
                            let xray_attacks_bb = match piece {
                                Piece::BISHOP => Attacks::get_bishop_attacks(square, xray_occ),
                                Piece::ROOK => Attacks::get_rook_attacks(square, xray_occ),
                                _ => Attacks::get_bishop_attacks(square, xray_occ) | Attacks::get_rook_attacks(square, xray_occ),
                            };
                            
                            let valid_xray = xray_attacks_bb & xray_occ & target_mask;
                            valid_xray.map(|t_sq| {
                                let t_idx = usize::from(t_sq);
                                let is_enemy = occ_nstm.get_bit(t_sq);
                                let t_piece = piece_map[t_idx];

                                if t_piece == piece && t_idx < sq_idx { return; }

                                let type_idx = if piece != Piece::QUEEN && t_piece == Piece::KING { 4 } else { u8::from(t_piece) as usize };

                                let dir = get_slider_dir(sq_idx, t_idx, compass_mode);
                                let type_stride = if piece == Piece::QUEEN { 4096 } else { 2048 };
                                let color_stride = if piece == Piece::QUEEN { 2048 } else { 1024 };
                                let xray_offset = if piece == Piece::QUEEN { 1024 } else { 512 };

                                process_input(base_offset 
                                    + type_idx * type_stride 
                                    + (is_enemy as usize) * color_stride 
                                    + xray_offset
                                    + dir * 128 
                                    + (sq_idx + piece_attack_offset));
                            });
                        }
                    };
                });
            }

            base_side_offset += 384;
            align_side_offset += 192;
            pawn_attack_offset += 48;
            piece_attack_offset += 64;

            (diag_stm, ortho_stm, diag_nstm, ortho_nstm) = (diag_nstm, ortho_nstm, diag_stm, ortho_stm);
        }
    }
}

const KNIGHT_OFFSETS: [u8; 128] = {
    let mut lut = [255; 128];
    let center = 64isize;
    lut[(center + 17) as usize] = 0;
    lut[(center + 10) as usize] = 1;
    lut[(center -  6) as usize] = 2;
    lut[(center - 15) as usize] = 3;
    lut[(center - 17) as usize] = 4;
    lut[(center - 10) as usize] = 5;
    lut[(center +  6) as usize] = 6;
    lut[(center + 15) as usize] = 7;
    lut
};

const PAWN_WHITE_LUT: [u8; 32] = {
    let mut lut = [255; 32];
    let center = 16isize;
    lut[(center + 7) as usize] = 0; lut[(center + 9) as usize] = 1;
    lut
};

const PAWN_BLACK_LUT: [u8; 32] = {
    let mut lut = [255; 32];
    let center = 16isize;
    lut[(center - 9) as usize] = 0; lut[(center - 7) as usize] = 1;
    lut
};

const SLIDER_DIR_LUT: [u8; 4096] = {
    let mut lut = [0; 4096];
    let mut src = 0;
    while src < 64 {
        let mut dst = 0;
        while dst < 64 {
            let (r1, f1) = (src / 8, src % 8);
            let (r2, f2) = (dst / 8, dst % 8);
            let val = if f1 == f2 { if r2 > r1 { 0 } else { 4 } } 
            else if r1 == r2 { if f2 > f1 { 2 } else { 6 } } 
            else if r2 > r1 { if f2 > f1 { 1 } else { 7 } } 
            else { if f2 > f1 { 3 } else { 5 } };
            lut[src * 64 + dst] = val;
            dst += 1;
        }
        src += 1;
    }
    lut
};

#[inline(always)]
pub fn get_knight_dir(src: usize, dest: usize) -> usize {
    KNIGHT_OFFSETS[(((dest as isize) - (src as isize)) + 64) as usize] as usize
}

#[inline(always)]
pub fn get_pawn_dir(src: usize, dest: usize, is_stm: bool) -> usize {
    let diff = (dest as isize) - (src as isize);
    if is_stm {
        PAWN_WHITE_LUT[(diff + 16) as usize] as usize
    } else {
        PAWN_BLACK_LUT[(diff + 16) as usize] as usize
    }
}

#[inline(always)]
fn get_slider_dir(src: usize, dest: usize, mode: u8) -> usize {
    let val = SLIDER_DIR_LUT[(src << 6) | dest] as usize;
    if mode == 2 { val } else { val >> 1 }
}

const BISHOP_ATTACKS: [Bitboard; 64] = {
    let mut result = [Bitboard::EMPTY; 64];
    let mut square_index = 0u8;
    while square_index < 64 {
        result[square_index as usize] = mask_bishop_attacks(Square::from_value(square_index));
        square_index += 1;
    }
    result
};

const ROOK_ATTACKS: [Bitboard; 64] = {
    let mut result = [Bitboard::EMPTY; 64];
    let mut square_index = 0u8;
    while square_index < 64 {
        result[square_index as usize] = mask_rook_attacks(Square::from_value(square_index));
        square_index += 1;
    }
    result
};

const fn mask_bishop_attacks(square: Square) -> Bitboard {
    let mut result: u64 = 0;
    let rank = square.get_rank() as i8;
    let file = square.file() as i8;
    
    let mut r; let mut f;

    r = rank + 1; f = file + 1;
    while r <= 7 && f <= 7 { result |= 1 << (r * 8 + f); r += 1; f += 1; }

    r = rank - 1; f = file + 1;
    while r >= 0 && f <= 7 { result |= 1 << (r * 8 + f); r -= 1; f += 1; }

    r = rank - 1; f = file - 1;
    while r >= 0 && f >= 0 { result |= 1 << (r * 8 + f); r -= 1; f -= 1; }

    r = rank + 1; f = file - 1;
    while r <= 7 && f >= 0 { result |= 1 << (r * 8 + f); r += 1; f -= 1; }

    Bitboard::from_value(result)
}

const fn mask_rook_attacks(square: Square) -> Bitboard {
    let mut result: u64 = 0;
    let rank = square.get_rank() as i8;
    let file = square.file() as i8;

    let mut i;
    i = rank + 1; while i <= 7 { result |= 1 << (i * 8 + file); i += 1; }
    i = rank - 1; while i >= 0 { result |= 1 << (i * 8 + file); i -= 1; }
    i = file + 1; while i <= 7 { result |= 1 << (rank * 8 + i); i += 1; }
    i = file - 1; while i >= 0 { result |= 1 << (rank * 8 + i); i -= 1; }

    Bitboard::from_value(result)
}