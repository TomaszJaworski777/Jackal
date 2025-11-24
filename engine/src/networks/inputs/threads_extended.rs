use chess::{Attacks, Bitboard, Piece, Side, Square};

const KING_ALIGN_OFFSET: usize = 768 * 3;
const PAWN_ATTACKS_OFFSET: usize = KING_ALIGN_OFFSET + 128 * 3 * 2;
const KNIGHT_ATTACKS_OFFSET: usize = PAWN_ATTACKS_OFFSET + 96 * 2 * 2 * 4;
const BISHOP_ATTACKS_OFFSET: usize = KNIGHT_ATTACKS_OFFSET + 128 * 8 * 2 * 7;
const ROOK_ATTACKS_OFFSET: usize = BISHOP_ATTACKS_OFFSET + 128 * 4 * 2 * 2 * 5;
const QUEEN_ATTACKS_OFFSET: usize = ROOK_ATTACKS_OFFSET + 128 * 4 * 2 * 2 * 5;
const KING_ATTACK_OFFSETS: usize = QUEEN_ATTACKS_OFFSET + 128 * 8 * 2 * 2 * 6;
const SIZE: usize = KING_ATTACK_OFFSETS + 128 * 2 * 6;

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

        if board.piece_mask_for_side(Piece::ROOK, Side::WHITE).ls1b_square().file() > 3 {
            board.mirror();
        }

        let rings = [Attacks::get_king_attacks(board.king_square(Side::WHITE)), Attacks::get_king_attacks(board.king_square(Side::BLACK))];

        let occ = board.occupancy();

        let mut base_side_offset = 0;
        let mut align_side_offset = 0;
        let mut pawn_attack_offset = 0;
        let mut piece_attack_offset = 0;

        for piece_color in [Side::WHITE, Side::BLACK] {

            let occ_stm = board.occupancy_for_side(piece_color);
            let occ_nstm = board.occupancy_for_side(piece_color.flipped());

            let enemy_king = board.king_square(piece_color.flipped());

            for piece_idx in u8::from(Piece::PAWN)..=u8::from(Piece::KING) {
                let piece = Piece::from(piece_idx);
                let base_feat_idx = base_side_offset + 64 * (u8::from(piece) - u8::from(Piece::PAWN)) as usize;
                let align_feat_idx = KING_ALIGN_OFFSET + align_side_offset + 64 * (u8::from(piece) - u8::from(Piece::BISHOP)) as usize;

                board.piece_mask_for_side(piece, piece_color).map(|square| {
                    let all_attackers = board.all_attackers_to_square(occ, square);
                    let attackers = (all_attackers & occ_nstm).pop_count();
                    let defenders = (all_attackers & occ_stm).pop_count();

                    let base = base_feat_idx + usize::from(square);

                    let mut feat = 768 * if attackers > defenders {
                        0
                    } else if attackers < defenders {
                        2
                    } else {
                        1
                    };

                    // if diag_stm.get_bit(square) {
                    //     feat += 768 * 6;
                    // }

                    // if ortho_stm.get_bit(square) {
                    //     feat += 768 * 6 * 2;
                    // }

                    feat += base;

                    process_input(feat);

                    let ring = rings[usize::from(piece_color.flipped())];
                    let is_ring_xrayed = if piece == Piece::BISHOP {
                        (BISHOP_ATTACKS[usize::from(square)] & ring).is_not_empty()
                    } else if piece == Piece::ROOK {
                        (ROOK_ATTACKS[usize::from(square)] & ring).is_not_empty()
                    } else if piece == Piece::QUEEN {
                        ((BISHOP_ATTACKS[usize::from(square)] | ROOK_ATTACKS[usize::from(square)]) & ring).is_not_empty()
                    } else {
                        false
                    };
                    
                    let king_bb = Bitboard::from_square(enemy_king);
                    let is_king_xrayed = if piece == Piece::BISHOP {
                        (BISHOP_ATTACKS[usize::from(square)] & king_bb).is_not_empty()
                    } else if piece == Piece::ROOK {
                        (ROOK_ATTACKS[usize::from(square)] & king_bb).is_not_empty()
                    } else if piece == Piece::QUEEN {
                        ((BISHOP_ATTACKS[usize::from(square)] | ROOK_ATTACKS[usize::from(square)]) & king_bb).is_not_empty()
                    } else {
                        false
                    };

                    if piece_idx >= u8::from(Piece::BISHOP) && piece_idx <= u8::from(Piece::QUEEN) && (is_king_xrayed || is_ring_xrayed) {
                        let mut feat = align_feat_idx + usize::from(square);
                        
                        if is_king_xrayed {
                            feat += 384
                        }

                        process_input(feat);
                    }

                    match piece {
                        Piece::PAWN => map_pawn_attacks(&board, square, piece_color, pawn_attack_offset, &mut process_input),
                        Piece::KNIGHT => map_knight_attacks(&board, square, piece_color, piece_attack_offset, &mut process_input),
                        Piece::BISHOP => map_bishop_attacks(&board, square, piece_color, piece_attack_offset, &mut process_input),
                        Piece::ROOK => map_rook_attacks(&board, square, piece_color, piece_attack_offset, &mut process_input),
                        Piece::QUEEN => map_queen_attacks(&board, square, piece_color, piece_attack_offset, &mut process_input),
                        Piece::KING => map_king_attacks(&board, square, piece_color, piece_attack_offset, &mut process_input),
                        _ => unreachable!()
                    };
                });
            }

            base_side_offset += 384;
            align_side_offset += 192;
            pawn_attack_offset += 48;
            piece_attack_offset += 64;
            //(diag_stm, ortho_stm, diag_nstm, ortho_nstm) = (diag_nstm, ortho_nstm, diag_stm, ortho_stm);
        }
    }
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
    let bishop_position = (square.get_rank() as i32, square.file() as i32);

    let mut rank = bishop_position.0 + 1;
    let mut file = bishop_position.1 + 1;
    while rank <= 7 && file <= 7 {
        result |= 1 << Square::from_coords(rank as u8, file as u8).get_value();
        rank += 1;
        file += 1;
    }

    rank = bishop_position.0 - 1;
    file = bishop_position.1 + 1;
    while rank >= 0 && file <= 7 {
        result |= 1 << Square::from_coords(rank as u8, file as u8).get_value();
        rank -= 1;
        file += 1;
    }

    rank = bishop_position.0 - 1;
    file = bishop_position.1 - 1;
    while rank >= 0 && file >= 0 {
        result |= 1 << Square::from_coords(rank as u8, file as u8).get_value();
        rank -= 1;
        file -= 1;
    }

    rank = bishop_position.0 + 1;
    file = bishop_position.1 - 1;
    while rank <= 7 && file >= 0 {
        result |= 1 << Square::from_coords(rank as u8, file as u8).get_value();
        rank += 1;
        file -= 1;
    }

    Bitboard::from_value(result)
}

const fn mask_rook_attacks(square: Square) -> Bitboard {
    let mut result: u64 = 0;
    let rook_position = (square.get_rank() as i32, square.file() as i32);

    let mut rank = rook_position.0 + 1;
    let mut file = rook_position.1;
    while rank <= 7 {
        result |= 1 << Square::from_coords(rank as u8, file as u8).get_value();
        rank += 1;
    }

    rank = rook_position.0 - 1;
    file = rook_position.1;
    while rank >= 0 {
        result |= 1 << Square::from_coords(rank as u8, file as u8).get_value();
        rank -= 1;
    }

    rank = rook_position.0;
    file = rook_position.1 + 1;
    while file <= 7 {
        result |= 1 << Square::from_coords(rank as u8, file as u8).get_value();
        file += 1;
    }

    rank = rook_position.0;
    file = rook_position.1 - 1;
    while file >= 0 {
        result |= 1 << Square::from_coords(rank as u8, file as u8).get_value();
        file -= 1;
    }

    Bitboard::from_value(result)
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
    lut[(center + 7) as usize] = 0;
    lut[(center + 9) as usize] = 1;
    lut
};

const PAWN_BLACK_LUT: [u8; 32] = {
    let mut lut = [255; 32];
    let center = 16isize;
    lut[(center - 9) as usize] = 0;
    lut[(center - 7) as usize] = 1;
    lut
};

#[inline(always)]
pub fn get_knight_dir(src: usize, dest: usize) -> usize {
    let diff = (dest as isize) - (src as isize);
    let dir = KNIGHT_OFFSETS[(diff + 64) as usize];
    debug_assert!(dir != 255, "Invalid Knight Move: {} -> {}", src, dest);
    dir as usize
}

#[inline(always)]
pub fn get_pawn_dir(src: usize, dest: usize, is_stm: bool) -> usize {
    let diff = (dest as isize) - (src as isize);
    let dir = if is_stm {
        PAWN_WHITE_LUT[(diff + 16) as usize]
    } else {
        PAWN_BLACK_LUT[(diff + 16) as usize]
    };
    
    debug_assert!(dir != 255, "Invalid Pawn Capture: {} -> {}", src, dest);
    dir as usize
}

#[inline(always)]
pub fn get_rook_dir(src: usize, dest: usize) -> usize {
    let (r_src, f_src) = (src / 8, src % 8);
    let (r_dst, f_dst) = (dest / 8, dest % 8);

    if f_src == f_dst {
        if r_dst > r_src { 0 } else { 2 }
    } else {
        if f_dst > f_src { 1 } else { 3 }
    }
}

#[inline(always)]
pub fn get_bishop_dir(src: usize, dest: usize) -> usize {
    let (r_src, f_src) = (src / 8, src % 8);
    let (r_dst, f_dst) = (dest / 8, dest % 8);

    if r_dst > r_src {
        if f_dst > f_src { 0 } else { 3 }
    } else {
        if f_dst > f_src { 1 } else { 2 }
    }
}

#[inline(always)]
pub fn get_queen_dir(src: usize, dest: usize) -> usize {
    let (r_src, f_src) = (src / 8, src % 8);
    let (r_dst, f_dst) = (dest / 8, dest % 8);

    if f_src == f_dst {
        return if r_dst > r_src { 0 } else { 4 };
    }
    if r_src == r_dst {
        return if f_dst > f_src { 2 } else { 6 };
    }
    
    if r_dst > r_src {
        return if f_dst > f_src { 1 } else { 7 };
    } else {
        return if f_dst > f_src { 3 } else { 5 };
    }
}

fn map_pawn_attacks<F: FnMut(usize)>(board: &chess::ChessBoard, square: Square, piece_color: Side, color_offset: usize, f: &mut F) {
    let attacks = Attacks::get_pawn_attacks(square, piece_color);
    let ring = Attacks::get_king_attacks(board.king_square(piece_color.flipped()));

    let target_types = [
        (Piece::PAWN, 0),
        (Piece::KNIGHT, 1),
        (Piece::ROOK, 2),
    ];

    for (p_type, type_idx) in target_types {
        let targets = attacks & board.piece_mask(p_type);
        targets.map(|target_sq| {
            let target_color = board.color_on_square(target_sq);
            
            if p_type == Piece::PAWN && usize::from(target_sq) < usize::from(square) && target_color == piece_color {
                return;
            }

            let target_color_idx = usize::from(target_color != piece_color);
            let dir = get_pawn_dir(square.into(), target_sq.into(), piece_color == Side::WHITE);
            
            let idx = PAWN_ATTACKS_OFFSET
                + type_idx * 384
                + target_color_idx * 192
                + dir * 96
                + (usize::from(square) - 8 + color_offset);
                
            f(idx);
        });
    }

    let ring_hits = attacks & ring & !board.occupancy();
    ring_hits.map(|target_sq| {
        let dir = get_pawn_dir(square.into(), target_sq.into(), piece_color == Side::WHITE);
        let idx = PAWN_ATTACKS_OFFSET
            + 3 * 384
            + 1 * 192
            + dir * 96
            + (usize::from(square) - 8 + color_offset);

        f(idx);
    });
}

fn map_knight_attacks<F: FnMut(usize)>(board: &chess::ChessBoard, square: Square, piece_color: Side, color_offset: usize, f: &mut F) {
    let attacks = Attacks::get_knight_attacks(square);
    let ring = Attacks::get_king_attacks(board.king_square(piece_color.flipped()));
    
    let target_types = [
        (Piece::PAWN, 0), (Piece::KNIGHT, 1), (Piece::BISHOP, 2),
        (Piece::ROOK, 3), (Piece::QUEEN, 4), (Piece::KING, 5)
    ];

    for (p_type, type_idx) in target_types {
        let targets = attacks & board.piece_mask(p_type);
        targets.map(|target_sq| {
            let target_color = board.color_on_square(target_sq);

            if p_type == Piece::KNIGHT && usize::from(target_sq) < usize::from(square) && target_color == piece_color {
                return;
            }

            let target_color_idx = usize::from(target_color != piece_color);
            let dir = get_knight_dir(square.into(), target_sq.into());

            let idx = KNIGHT_ATTACKS_OFFSET
                + type_idx * 2048
                + target_color_idx * 1024
                + dir * 128
                + (usize::from(square) + color_offset);
                
            f(idx);
        });
    }

    let ring_hits = attacks & ring;
    ring_hits.map(|target_sq| {
        let dir = get_knight_dir(square.into(), target_sq.into());
        let idx = KNIGHT_ATTACKS_OFFSET
            + 6 * 2048
            + 1 * 1024
            + dir * 128
            + (usize::from(square) + color_offset);

        f(idx);
    });
}

fn map_bishop_attacks<F: FnMut(usize)>(board: &chess::ChessBoard, square: Square, piece_color: Side, color_offset: usize, f: &mut F) {
    let occ = board.occupancy();
    let attacks = Attacks::get_bishop_attacks(square, occ);
    let direct_hits = attacks & occ;
    let xray_occ = occ ^ direct_hits;
    let xray_attacks = Attacks::get_bishop_attacks(square, xray_occ) & xray_occ;

    let target_types = [
        (Piece::PAWN, 0), (Piece::KNIGHT, 1), (Piece::BISHOP, 2),
        (Piece::ROOK, 3), (Piece::KING, 4)
    ];

    for (p_type, type_idx) in target_types {
        let p_mask = board.piece_mask(p_type);
        
        let targets = direct_hits & p_mask;
        targets.map(|target_sq| {
            let target_color = board.color_on_square(target_sq);

            if p_type == Piece::BISHOP && usize::from(target_sq) < usize::from(square) && target_color == piece_color {
                return;
            }

            let target_color_idx = usize::from(target_color != piece_color);
            let dir = get_bishop_dir(square.into(), target_sq.into());

            let idx = BISHOP_ATTACKS_OFFSET
                + type_idx * 2048
                + target_color_idx * 1024
                + dir * 128
                + (usize::from(square) + color_offset);
                
            f(idx);
        });

        let targets_xr = xray_attacks & p_mask;
        targets_xr.map(|target_sq| {
            let target_color = board.color_on_square(target_sq);

            if p_type == Piece::BISHOP && usize::from(target_sq) < usize::from(square) && target_color == piece_color {
                return;
            }

            let target_color_idx = usize::from(target_color != piece_color);
            let dir = get_bishop_dir(square.into(), target_sq.into());

            let idx = BISHOP_ATTACKS_OFFSET
                + type_idx * 2048
                + target_color_idx * 1024
                + 512
                + dir * 128
                + (usize::from(square) + color_offset);
            f(idx);
        });
    }
}

fn map_rook_attacks<F: FnMut(usize)>(board: &chess::ChessBoard, square: Square, piece_color: Side, color_offset: usize, f: &mut F) {
    let occ = board.occupancy();
    let attacks = Attacks::get_rook_attacks(square, occ);
    let direct_hits = attacks & occ;
    let xray_occ = occ ^ direct_hits;
    let xray_attacks = Attacks::get_rook_attacks(square, xray_occ) & xray_occ;

    let target_types = [
        (Piece::PAWN, 0), (Piece::KNIGHT, 1), (Piece::BISHOP, 2),
        (Piece::ROOK, 3), (Piece::KING, 4)
    ];

    for (p_type, type_idx) in target_types {
        let p_mask = board.piece_mask(p_type);
        
        let targets = direct_hits & p_mask;
        targets.map(|target_sq| {
            let target_color = board.color_on_square(target_sq);

            if p_type == Piece::ROOK && usize::from(target_sq) < usize::from(square) && target_color == piece_color {
                return;
            }

            let target_color_idx = usize::from(target_color != piece_color);
            let dir = get_rook_dir(square.into(), target_sq.into());

            let idx = ROOK_ATTACKS_OFFSET
                + type_idx * 2048
                + target_color_idx * 1024
                + dir * 128
                + (usize::from(square) + color_offset);
                
            f(idx);
        });

        let targets_xr = xray_attacks & p_mask;
        targets_xr.map(|target_sq| {
            let target_color = board.color_on_square(target_sq);

            if p_type == Piece::ROOK && usize::from(target_sq) < usize::from(square) && target_color == piece_color {
                return;
            }

            let target_color_idx = usize::from(target_color != piece_color);
            let dir = get_rook_dir(square.into(), target_sq.into());

            let idx = ROOK_ATTACKS_OFFSET
                + type_idx * 2048
                + target_color_idx * 1024
                + 512 
                + dir * 128
                + (usize::from(square) + color_offset);

            f(idx);
        });
    }
}

fn map_queen_attacks<F: FnMut(usize)>(board: &chess::ChessBoard, square: Square, piece_color: Side, color_offset: usize, f: &mut F) {
    let occ = board.occupancy();
    let attacks = Attacks::get_rook_attacks(square, occ) | Attacks::get_bishop_attacks(square, occ);
    let direct_hits = attacks & occ;
    let xray_occ = occ ^ direct_hits;
    let xray_attacks = (Attacks::get_rook_attacks(square, xray_occ) | Attacks::get_bishop_attacks(square, xray_occ)) & xray_occ;

    let target_types = [
        (Piece::PAWN, 0), (Piece::KNIGHT, 1), (Piece::BISHOP, 2),
        (Piece::ROOK, 3), (Piece::QUEEN, 4), (Piece::KING, 5)
    ];

    for (p_type, type_idx) in target_types {
        let p_mask = board.piece_mask(p_type);
        
        let targets = direct_hits & p_mask;
        targets.map(|target_sq| {
            let target_color = board.color_on_square(target_sq);

            if p_type == Piece::QUEEN && usize::from(target_sq) < usize::from(square) && target_color == piece_color {
                return;
            }

            let target_color_idx = usize::from(target_color != piece_color);
            let dir = get_queen_dir(square.into(), target_sq.into());

            let idx = QUEEN_ATTACKS_OFFSET
                + type_idx * 4096
                + target_color_idx * 2048
                + dir * 128
                + (usize::from(square) + color_offset);

            f(idx);
        });

        let targets_xr = xray_attacks & p_mask;
        targets_xr.map(|target_sq| {
            let target_color = board.color_on_square(target_sq);

            if p_type == Piece::QUEEN && usize::from(target_sq) < usize::from(square) && target_color == piece_color {
                return;
            }

            let target_color_idx = usize::from(target_color != piece_color);
            let dir = get_queen_dir(square.into(), target_sq.into());

            let idx = QUEEN_ATTACKS_OFFSET
                + type_idx * 4096
                + target_color_idx * 2048
                + 1024
                + dir * 128
                + (usize::from(square) + color_offset);

            f(idx);
        });
    }
}

fn map_king_attacks<F: FnMut(usize)>(board: &chess::ChessBoard, square: Square, piece_color: Side, color_offset: usize, f: &mut F) {
    let attacks = Attacks::get_king_attacks(square);
    let ring = Attacks::get_king_attacks(board.king_square(piece_color.flipped()));
    
    let target_types = [
        (Piece::PAWN, 0), (Piece::KNIGHT, 1), (Piece::BISHOP, 2),
        (Piece::ROOK, 3), (Piece::KING, 4)
    ];

    for (p_type, type_idx) in target_types {
        let targets = attacks & board.piece_mask_for_side(p_type, piece_color);
        if targets.is_not_empty() {
            let idx = KING_ATTACK_OFFSETS
                + type_idx * 256
                + (usize::from(square) + color_offset);
                
            f(idx);
        }

        let targets = attacks & board.piece_mask_for_side(p_type, piece_color.flipped());
        if targets.is_not_empty() {
            let idx = KING_ATTACK_OFFSETS
                + type_idx * 256
                + 128
                + (usize::from(square) + color_offset);
                
            f(idx);
        }
    }

    if (attacks & ring).is_not_empty() {
        let idx = KING_ATTACK_OFFSETS
            + 5 * 256
            + 128
            + (usize::from(square) + color_offset);

        f(idx);
    }
}