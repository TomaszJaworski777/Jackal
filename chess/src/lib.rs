mod attacks;
mod base_structures;
mod board;
mod move_gen;
mod magics;

use std::time::Duration;
use std::time::Instant;

pub use attacks::Attacks;
pub use attacks::Rays;
pub use base_structures::Bitboard;
pub use base_structures::Move;
pub use base_structures::MoveFlag;
pub use base_structures::Piece;
pub use base_structures::Side;
pub use base_structures::Square;
pub use base_structures::FEN;
pub use base_structures::ZobristKey;
pub use board::ChessBoard;
pub use board::ChessPosition;
pub use magics::get_bishop_attacks;
pub use magics::get_rook_attacks;

pub const DEFAULT_PERFT_DEPTH: u8 = 5;

pub fn perft<const BULK: bool, const SPLIT: bool, const CHESS_960: bool>(
    board: &ChessBoard,
    depth: Option<u8>,
) -> (u128, Duration) {
    let timer = Instant::now();
    let mask = board.castle_rights().get_castle_mask();
    let result = if board.side() == Side::WHITE {
        perft_internal_white::<BULK, SPLIT, CHESS_960>(
            board,
            depth.unwrap_or(DEFAULT_PERFT_DEPTH),
            &mask,
        )
    } else {
        perft_internal_black::<BULK, SPLIT, CHESS_960>(
            board,
            depth.unwrap_or(DEFAULT_PERFT_DEPTH),
            &mask,
        )
    };
    let duration = timer.elapsed();

    (result, duration)
}

fn perft_internal_white<const BULK: bool, const SPLIT: bool, const CHESS_960: bool>(
    board: &ChessBoard,
    depth: u8,
    mask: &[u8; 64],
) -> u128 {
    let mut node_count = 0u128;

    if BULK && depth == 1 {
        return board.count_legal_moves::<0>() as u128;
    }

    if !BULK && depth == 0 {
        return 1;
    }

    board.map_legal_moves_templated::<_, 0>(|mv| {
        let mut board_copy = *board;
        board_copy.make_move_templated::<0>(mv, mask);
        let result = perft_internal_black::<BULK, false, CHESS_960>(&board_copy, depth - 1, mask);
        node_count += result;

        if SPLIT {
            println!("  {} - {result}", mv.to_string(CHESS_960))
        }
    });

    node_count
}

fn perft_internal_black<const BULK: bool, const SPLIT: bool, const CHESS_960: bool>(
    board: &ChessBoard,
    depth: u8,
    mask: &[u8; 64],
) -> u128 {
    let mut node_count = 0u128;

    if BULK && depth == 1 {
        return board.count_legal_moves::<1>() as u128;
    }

    if !BULK && depth == 0 {
        return 1;
    }

    board.map_legal_moves_templated::<_, 1>(|mv| {
        let mut board_copy = *board;
        board_copy.make_move_templated::<1>(mv, mask);
        let result = perft_internal_white::<BULK, false, CHESS_960>(&board_copy, depth - 1, mask);
        node_count += result;

        if SPLIT {
            println!("  {} - {result}", mv.to_string(CHESS_960))
        }
    });

    node_count
}
