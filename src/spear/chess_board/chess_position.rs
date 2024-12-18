use crate::spear::{Move, MoveHistory, FEN};

use super::ChessBoard;

#[derive(Clone, Copy, Hash)]
pub struct ChessPosition {
    board: ChessBoard,
    history: MoveHistory,
}

impl Default for ChessPosition {
    fn default() -> Self {
        Self {
            board: ChessBoard::default(),
            history: MoveHistory::new(),
        }
    }
}

impl ChessPosition {
    #[inline]
    pub fn from_fen(fen: &FEN) -> Self {
        Self {
            board: ChessBoard::from_fen(fen),
            history: MoveHistory::new(),
        }
    }

    #[inline]
    pub fn from_board(board: &ChessBoard) -> Self {
        Self {
            board: *board,
            history: MoveHistory::new(),
        }
    }

    #[inline]
    pub fn board(&self) -> &ChessBoard {
        &self.board
    }

    #[inline]
    pub fn board_mut(&mut self) -> &mut ChessBoard {
        &mut self.board
    }

    #[inline]
    pub fn make_move<const STM_WHITE: bool, const NSTM_WHITE: bool>(&mut self, mv: Move) {
        self.history.push(self.board.get_key());
        self.board.make_move::<STM_WHITE, NSTM_WHITE>(mv);

        if self.board.half_move_counter() == 0 {
            self.history.reset()
        }
    }

    #[inline]
    pub fn is_repetition(&self) -> bool {
        self.history.get_key_repetitions(self.board.get_key()) > 0
    }
}
