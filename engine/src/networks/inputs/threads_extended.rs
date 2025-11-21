use chess::{Bitboard, ChessBoard, Piece, Rays, Side};

const BASE_INPUTS: usize = 18816;

pub struct ThreatsExtended;
#[allow(unused)]
impl ThreatsExtended {
    pub const fn input_size() -> usize {
        BASE_INPUTS * 2
    }

    pub fn map_inputs<F: FnMut(usize)>(board: &chess::ChessBoard, mut process_input: F) {
        let (diag, ortho) = board.generate_pin_masks(board.side());
        let defender_pin_mask = diag | ortho;

        let (diag, ortho) = board.generate_pin_masks(board.side().flipped());
        let attack_pin_mask = diag | ortho;

        let horizontal_mirror = if board.king_square(board.side()).file() > 3 {
            7
        } else {
            0
        };

        let flip = board.side() == Side::BLACK;

        let occ = board.occupancy();
        occ.map(|square| {
            let piece = board.piece_on_square(square);
            let color = board.color_on_square(square);

            let attack_pin_mask = attack_pin_mask & !Rays::get_ray(square, board.king_square(board.side().flipped()));

            let all_attackers = board.all_attackers_to_square(occ, square);
            let attackers = all_attackers & board.occupancy_for_side(board.side().flipped()) & !attack_pin_mask;
            let defenders = all_attackers & board.occupancy_for_side(board.side()) & !defender_pin_mask;

            let (attacker, defender) = attacker_defender(&board, attackers, defenders);

            let piece_index = 64 * (u8::from(piece) - u8::from(Piece::PAWN)) as usize;
            let mut feat = [384, 0][usize::from(color == board.side())] + piece_index + (usize::from(square) ^ horizontal_mirror ^ if flip { 56 } else { 0 });

            if attacker != Piece::NONE {
                feat += 768 * (usize::from(attacker) + 1)
            }

            if defender != Piece::NONE {
                feat += 768 * 7 * (usize::from(defender) + 1)
            }

            process_input(feat)
        });
    }
}

fn attacker_defender(board: &ChessBoard, attackers: Bitboard, defenders: Bitboard) -> (Piece, Piece) {
    let bb_pawn = board.piece_mask(Piece::PAWN);
    let bb_knight = board.piece_mask(Piece::KNIGHT);
    let bb_bishop = board.piece_mask(Piece::BISHOP);
    let bb_rook = board.piece_mask(Piece::ROOK);
    let bb_queen = board.piece_mask(Piece::QUEEN);
    let bb_king = board.piece_mask(Piece::KING);

    let attacker = if (attackers & bb_pawn).is_not_empty() {
        Piece::PAWN
    } else if (attackers & bb_knight).is_not_empty() {
        Piece::KNIGHT
    } else if (attackers & bb_bishop).is_not_empty() {
        Piece::BISHOP
    } else if (attackers & bb_rook).is_not_empty() {
        Piece::ROOK
    } else if (attackers & bb_queen).is_not_empty() {
        Piece::QUEEN
    } else if (attackers & bb_king).is_not_empty() {
        Piece::KING
    } else {
        Piece::NONE
    };

    let defender = if (defenders & bb_pawn).is_not_empty() {
        Piece::PAWN
    } else if (defenders & bb_knight).is_not_empty() {
        Piece::KNIGHT
    } else if (defenders & bb_bishop).is_not_empty() {
        Piece::BISHOP
    } else if (defenders & bb_rook).is_not_empty() {
        Piece::ROOK
    } else if (defenders & bb_queen).is_not_empty() {
        Piece::QUEEN
    } else if (defenders & bb_king).is_not_empty() {
        Piece::KING
    } else {
        Piece::NONE
    };

    (attacker, defender)
}