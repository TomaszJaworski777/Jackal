use crate::spear::{
    attacks::{Attacks, Rays},
    Bitboard, CastleRights, ChessBoard, Move, MoveFlag, Square,
};

use super::MoveGen;

impl MoveGen {
    pub fn generate_king_moves<F: FnMut(Move), const CAPTURE_ONLY: bool, const NSTM_WHITE: bool>(
        board: &ChessBoard,
        attack_map: Bitboard,
        king_square: Square,
        method: &mut F,
    ) {
        let move_mask = Attacks::get_king_attacks_for_square(king_square) & !attack_map;

        (move_mask & board.get_occupancy_for_side::<NSTM_WHITE>())
            .map(|square| method(Move::from_squares(king_square, square, MoveFlag::CAPTURE)));

        if CAPTURE_ONLY {
            return;
        }

        (move_mask & !board.get_occupancy()).map(|square| {
            method(Move::from_squares(
                king_square,
                square,
                MoveFlag::QUIET_MOVE,
            ))
        });
    }

    pub fn generate_castle_moves<F: FnMut(Move), const STM_WHITE: bool>(
        board: &ChessBoard,
        attack_map: Bitboard,
        king_square: Square,
        method: &mut F,
    ) {
        let king_side_destination = (king_square.get_bit() << 2).ls1b_square();
        let queen_side_destination = (king_square.get_bit() >> 2).ls1b_square();

        if STM_WHITE {
            let king_side_room = Rays::get_ray(
                king_square,
                Square::from_raw(CastleRights::ROOK_POSITIONS[1]).shift_right(1),
            ) & board.get_occupancy();
            let queen_side_room = Rays::get_ray(
                king_square,
                Square::from_raw(CastleRights::ROOK_POSITIONS[0]).shift_left(1),
            ) & board.get_occupancy();
            if board.castle_rights().has_right(CastleRights::WHITE_KING)
                && (Rays::get_ray(king_square, king_side_destination) & attack_map).is_empty()
                && king_side_room.is_empty()
            {
                method(Move::from_squares(
                    king_square,
                    king_side_destination,
                    MoveFlag::KING_SIDE_CASTLE,
                ))
            }
            if board.castle_rights().has_right(CastleRights::WHITE_QUEEN)
                && (Rays::get_ray(king_square, queen_side_destination) & attack_map).is_empty()
                && queen_side_room.is_empty()
            {
                method(Move::from_squares(
                    king_square,
                    queen_side_destination,
                    MoveFlag::QUEEN_SIDE_CASTLE,
                ))
            }
        } else {
            let king_side_room = Rays::get_ray(
                king_square,
                Square::from_raw(CastleRights::ROOK_POSITIONS[1])
                    .flip()
                    .shift_right(1),
            ) & board.get_occupancy();
            let queen_side_room = Rays::get_ray(
                king_square,
                Square::from_raw(CastleRights::ROOK_POSITIONS[0])
                    .flip()
                    .shift_left(1),
            ) & board.get_occupancy();
            if board.castle_rights().has_right(CastleRights::BLACK_KING)
                && (Rays::get_ray(king_square, king_side_destination) & attack_map).is_empty()
                && king_side_room.is_empty()
            {
                method(Move::from_squares(
                    king_square,
                    king_side_destination,
                    MoveFlag::KING_SIDE_CASTLE,
                ))
            }
            if board.castle_rights().has_right(CastleRights::BLACK_QUEEN)
                && (Rays::get_ray(king_square, queen_side_destination) & attack_map).is_empty()
                && queen_side_room.is_empty()
            {
                method(Move::from_squares(
                    king_square,
                    queen_side_destination,
                    MoveFlag::QUEEN_SIDE_CASTLE,
                ))
            }
        }
    }
}
