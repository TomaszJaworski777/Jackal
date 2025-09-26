use crate::{
    attacks::Rays,
    base_structures::CastleRights,
    move_gen::generate_moves::{MoveGen, WHITE},
    Attacks, Bitboard, ChessBoard, Move, MoveFlag, Side, Square,
};

impl MoveGen {
    pub fn generate_king_moves<F: FnMut(Move), const COLOR: u8, const CAPTURE_ONLY: bool>(
        board: &ChessBoard,
        attack_map: Bitboard,
        king_square: Square,
        apply_move: &mut F,
    ) {
        let move_mask = Attacks::get_king_attacks(king_square) & !attack_map;

        (move_mask & board.occupancy_for_side(Side::from(COLOR).flipped()))
            .map(|square| apply_move(Move::from_squares(king_square, square, MoveFlag::CAPTURE)));

        if CAPTURE_ONLY {
            return;
        }

        (move_mask & !board.occupancy()).map(|square| {
            apply_move(Move::from_squares(
                king_square,
                square,
                MoveFlag::QUIET_MOVE,
            ))
        });
    }

    pub fn generate_castle_moves<F: FnMut(Move), const COLOR: u8>(
        board: &ChessBoard,
        attack_map: Bitboard,
        king_square: Square,
        rook_pins: Bitboard,
        apply_move: &mut F,
    ) {
        let validate_castle =
            |rook_square: Square, king_destination: Square, rook_destination: Square| -> bool {
                let castle_path = Rays::get_ray(rook_square, rook_destination)
                    | Rays::get_ray(king_square, king_destination);
                let occupancy =
                    board.occupancy() ^ Bitboard::from(rook_square) ^ Bitboard::from(king_square);
                return (Rays::get_ray(king_square, king_destination) & attack_map).is_empty()
                    && (castle_path & occupancy).is_empty()
                    && (rook_pins & Bitboard::from(rook_square)).is_empty();
            };

        if COLOR == WHITE {
            let rook_square = board.castle_rights().rook_square(1);
            if board.castle_rights().has_right(CastleRights::WHITE_KING)
                && validate_castle(rook_square, Square::G1, Square::F1)
            {
                apply_move(Move::from_squares(
                    king_square,
                    rook_square,
                    MoveFlag::KING_SIDE_CASTLE,
                ))
            }

            let rook_square = board.castle_rights().rook_square(0);
            if board.castle_rights().has_right(CastleRights::WHITE_QUEEN)
                && validate_castle(rook_square, Square::C1, Square::D1)
            {
                apply_move(Move::from_squares(
                    king_square,
                    rook_square,
                    MoveFlag::QUEEN_SIDE_CASTLE,
                ))
            }
        } else {
            let rook_square = board.castle_rights().rook_square(3);
            if board.castle_rights().has_right(CastleRights::BLACK_KING)
                && validate_castle(rook_square, Square::G8, Square::F8)
            {
                apply_move(Move::from_squares(
                    king_square,
                    rook_square,
                    MoveFlag::KING_SIDE_CASTLE,
                ))
            }

            let rook_square = board.castle_rights().rook_square(2);
            if board.castle_rights().has_right(CastleRights::BLACK_QUEEN)
                && validate_castle(rook_square, Square::C8, Square::D8)
            {
                apply_move(Move::from_squares(
                    king_square,
                    rook_square,
                    MoveFlag::QUEEN_SIDE_CASTLE,
                ))
            }
        }
    }
}
