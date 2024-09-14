use crate::spear::{attacks::Attacks, Bitboard, ChessBoard, Move, MoveFlag, Piece};

use super::MoveGen;

impl MoveGen {
    pub const KNIGHT: i8 = 0;
    pub const BISHOP: i8 = 1;
    pub const ROOK: i8 = 2;

    pub fn generate_piece_moves<
        F: FnMut(Move),
        const CAPTURE_ONLY: bool,
        const PIECE_TYPE: i8,
        const STM_WHITE: bool,
    >(
        board: &ChessBoard,
        push_map: Bitboard,
        capture_map: Bitboard,
        diagonal_pins: Bitboard,
        ortographic_pins: Bitboard,
        method: &mut F,
    ) {
        let pieces = match PIECE_TYPE {
            MoveGen::KNIGHT => {
                board.get_piece_mask_for_side::<STM_WHITE>(Piece::KNIGHT)
                    & !diagonal_pins
                    & !ortographic_pins
            }
            MoveGen::BISHOP => {
                (board.get_piece_mask_for_side::<STM_WHITE>(Piece::BISHOP)
                    | board.get_piece_mask_for_side::<STM_WHITE>(Piece::QUEEN))
                    & !ortographic_pins
            }
            MoveGen::ROOK => {
                (board.get_piece_mask_for_side::<STM_WHITE>(Piece::ROOK)
                    | board.get_piece_mask_for_side::<STM_WHITE>(Piece::QUEEN))
                    & !diagonal_pins
            }
            _ => unreachable!(),
        };

        let pinned_pieces = match PIECE_TYPE {
            MoveGen::KNIGHT => Bitboard::EMPTY,
            MoveGen::BISHOP => pieces & diagonal_pins,
            MoveGen::ROOK => pieces & ortographic_pins,
            _ => unreachable!(),
        };

        let not_pinned_pieces = pieces & !pinned_pieces;

        not_pinned_pieces.map(|piece_square| {
            let attacks = match PIECE_TYPE {
                MoveGen::KNIGHT => Attacks::get_knight_attacks_for_square(piece_square),
                MoveGen::BISHOP => {
                    Attacks::get_bishop_attacks_for_square(piece_square, board.get_occupancy())
                }
                MoveGen::ROOK => {
                    Attacks::get_rook_attacks_for_square(piece_square, board.get_occupancy())
                }
                _ => unreachable!(),
            };

            (attacks & capture_map).map(|to_square| {
                method(Move::from_squares(
                    piece_square,
                    to_square,
                    MoveFlag::CAPTURE,
                ))
            });

            if CAPTURE_ONLY {
                return;
            }

            (attacks & push_map).map(|to_square| {
                method(Move::from_squares(
                    piece_square,
                    to_square,
                    MoveFlag::QUIET_MOVE,
                ))
            });
        });

        pinned_pieces.map(|piece_square| {
            let attacks = match PIECE_TYPE {
                MoveGen::KNIGHT => Bitboard::EMPTY,
                MoveGen::BISHOP => {
                    Attacks::get_bishop_attacks_for_square(piece_square, board.get_occupancy())
                        & diagonal_pins
                }
                MoveGen::ROOK => {
                    Attacks::get_rook_attacks_for_square(piece_square, board.get_occupancy())
                        & ortographic_pins
                }
                _ => unreachable!(),
            };

            (attacks & capture_map).map(|to_square| {
                method(Move::from_squares(
                    piece_square,
                    to_square,
                    MoveFlag::CAPTURE,
                ))
            });

            if CAPTURE_ONLY {
                return;
            }

            (attacks & push_map).map(|to_square| {
                method(Move::from_squares(
                    piece_square,
                    to_square,
                    MoveFlag::QUIET_MOVE,
                ))
            });
        });
    }
}
