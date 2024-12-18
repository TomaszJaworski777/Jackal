use bytemuck::{Pod, Zeroable};

use crate::spear::{Bitboard, ChessBoard, Move, Side};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PolicyPacked {
    board: [Bitboard; 4],
    side_to_move: Side,
    move_count: u8,
    moves: [PolicyMoveData; PolicyPacked::MAX_MOVE_COUNT],
}

#[allow(unused)]
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Default)]
pub struct PolicyMoveData {
    pub mv: Move,
    pub visits: u16,
}

impl Default for PolicyPacked {
    fn default() -> Self {
        Self {
            board: [Bitboard::default(); 4],
            side_to_move: Side::default(),
            move_count: 0,
            moves: [PolicyMoveData::default(); PolicyPacked::MAX_MOVE_COUNT],
        }
    }
}

#[allow(unused)]
impl PolicyPacked {
    pub const MAX_MOVE_COUNT: usize = 101;

    pub fn from_board(board: &ChessBoard) -> Self {
        Self {
            board: board_to_compressed(board),
            side_to_move: board.side_to_move(),
            move_count: 0,
            moves: [PolicyMoveData::default(); PolicyPacked::MAX_MOVE_COUNT],
        }
    }

    #[inline]
    pub fn get_board(&self) -> &[Bitboard; 4] {
        &self.board
    }

    #[inline]
    pub fn get_side_to_move(&self) -> Side {
        self.side_to_move
    }

    #[inline]
    pub fn move_count(&self) -> u8 {
        self.move_count
    }

    #[inline]
    pub fn moves(&self) -> &[PolicyMoveData; PolicyPacked::MAX_MOVE_COUNT] {
        &self.moves
    }

    #[inline]
    pub fn push_move(&mut self, mv: Move, visits: u16) {
        self.moves[self.move_count() as usize] = PolicyMoveData { mv, visits };
        self.move_count += 1;
    }
}

fn board_to_compressed(board: &ChessBoard) -> [Bitboard; 4] {
    let mut result = [Bitboard::FULL; 4];

    board.get_occupancy().map(|square| {
        result[0].pop_bit(square);
        result[1].pop_bit(square);
        result[2].pop_bit(square);
        result[3].pop_bit(square);
        let piece = board.get_piece_on_square(square);
        let color = board.get_piece_color_on_square(square);
        for (bit_index, result_bit) in result.iter_mut().enumerate().take(3usize) {
            if (piece.get_raw() & 1 << bit_index) > 0 {
                result_bit.set_bit(square);
            }
        }
        if color == Side::BLACK {
            result[3].set_bit(square);
        }
    });

    result
}

unsafe impl Zeroable for PolicyPacked {}
unsafe impl Pod for PolicyPacked {}

unsafe impl Zeroable for PolicyMoveData {}
unsafe impl Pod for PolicyMoveData {}