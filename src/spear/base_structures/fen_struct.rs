use std::fmt::{Display, Formatter, Result};

use super::Square;

#[derive(Clone, PartialEq, Default)]
pub struct FEN {
    pub(crate) board: [String; 8],
    pub(crate) side_to_move: String,
    pub(crate) castle_rights: String,
    pub(crate) en_passant_square: String,
    pub(crate) half_move_counter: String,
    pub(crate) full_move_counter: String,
}

impl FEN {
    pub fn start_position() -> Self {
        Self::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    #[allow(unused)]
    pub fn kiwipete_position() -> Self {
        Self::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
    }

    pub fn from_string(fen_string: String) -> Self {
        Self::from_str(fen_string.as_str())
    }

    pub fn from_str(fen_string: &str) -> Self {
        let fen_parts: Vec<&str> = fen_string.split_whitespace().collect();
        let mut result: Self = Self::default();
        let board_parts: Vec<&str> = fen_parts[0].split('/').collect();
        for (index, part) in board_parts.into_iter().enumerate() {
            result.board[index] = part.to_string()
        }

        result.side_to_move = fen_parts[1].to_string();
        result.castle_rights = fen_parts[2].to_string();
        result.en_passant_square = fen_parts[3].to_string();

        result.half_move_counter = if fen_parts.len() > 4 {
            fen_parts[4]
        } else {
            "0"
        }
        .to_string();
        result.full_move_counter = if fen_parts.len() > 5 {
            fen_parts[5]
        } else {
            "1"
        }
        .to_string();

        result
    }

    #[allow(unused)]
    pub fn validate_fen(fen_string: &str) -> bool {
        let fen_parts: Vec<&str> = fen_string.split_whitespace().collect();

        if fen_parts.len() < 4 {
            return false;
        }

        let board_parts: Vec<&str> = fen_parts[0].split('/').collect();
        if board_parts.len() != 8 {
            return false;
        }

        if fen_parts[1] != "w" && fen_parts[1] != "b" {
            return false;
        }

        let castle_rights_contain = fen_parts[2].contains('-')
            || fen_parts[2].contains('K')
            || fen_parts[2].contains('Q')
            || fen_parts[2].contains('k')
            || fen_parts[2].contains('q');
        if fen_parts[2].len() > 4 || !castle_rights_contain {
            return false;
        }

        let square_validate = if fen_parts[3] != "-" {
            Square::from_string(fen_parts[3]).get_raw() < 63
        } else {
            true
        };
        if fen_parts[3].len() > 2 || !square_validate {
            return false;
        }

        if fen_parts.len() > 4 && fen_parts[4].parse::<u8>().is_err() {
            return false;
        }

        if fen_parts.len() > 5 && fen_parts[5].parse::<u16>().is_err() {
            return false;
        }

        true
    }
}

impl Display for FEN {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(
            formatter,
            "{}/{}/{}/{}/{}/{}/{}/{} {} {} {} {} {}",
            self.board[0],
            self.board[1],
            self.board[2],
            self.board[3],
            self.board[4],
            self.board[5],
            self.board[6],
            self.board[7],
            self.side_to_move,
            self.castle_rights,
            self.en_passant_square,
            self.half_move_counter,
            self.full_move_counter
        )
    }
}
