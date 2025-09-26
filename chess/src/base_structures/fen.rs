use std::{
    char,
    fmt::{Display, Formatter, Result},
};

use crate::Bitboard;

use super::Square;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FEN {
    pub(crate) board: [String; 8],
    pub(crate) side_to_move: String,
    pub castle_rights: String,
    pub(crate) en_passant_square: String,
    pub(crate) half_move_counter: String,
    pub(crate) full_move_counter: String,
}

impl FEN {
    pub fn start_position() -> Self {
        Self::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1")
    }

    pub fn kiwipete_position() -> Self {
        Self::from("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w HAha - 0 1")
    }

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

        let valid_en_passant = if fen_parts[3] != "-" {
            (Bitboard::from(Square::from(fen_parts[3])) & Bitboard::RANK_3.or(Bitboard::RANK_6))
                .is_not_empty()
        } else {
            true
        };

        if fen_parts[3].len() > 2 || !valid_en_passant {
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

impl From<String> for FEN {
    fn from(value: String) -> Self {
        FEN::from(value.as_str())
    }
}

impl From<&str> for FEN {
    fn from(value: &str) -> Self {
        let mut result: Self = Self::default();
        let fen_parts: Vec<&str> = value.trim().split_whitespace().collect();

        let board_parts: Vec<&str> = fen_parts[0].split('/').collect();
        for (index, part) in board_parts.into_iter().enumerate() {
            result.board[index] = part.to_string()
        }

        result.side_to_move = fen_parts[1].to_string();
        result.castle_rights = normalize_castle_rights(&result, fen_parts[2]);

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
}

impl From<FEN> for String {
    fn from(value: FEN) -> Self {
        format!(
            "{}/{}/{}/{}/{}/{}/{}/{} {} {} {} {} {}",
            value.board[0],
            value.board[1],
            value.board[2],
            value.board[3],
            value.board[4],
            value.board[5],
            value.board[6],
            value.board[7],
            value.side_to_move,
            value.castle_rights,
            value.en_passant_square,
            value.half_move_counter,
            value.full_move_counter
        )
    }
}

impl Display for FEN {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", String::from(self.clone()))
    }
}

fn normalize_castle_rights(fen: &FEN, rights: &str) -> String {
    //Helper method to find occurance of the character in the fen string
    let find_files = |rank_str: &String, target: char| -> Vec<u8> {
        let mut result = Vec::new();
        let mut file_idx = 0;
        for char in rank_str.chars() {
            if char.is_digit(10) {
                file_idx += char as u8 - b'0'
            } else {
                if char == target {
                    result.push(file_idx);
                }
                file_idx += 1;
            }
        }

        result
    };

    if rights == "-" {
        return String::from("-");
    }

    let white_king = find_files(&fen.board[7], 'K');
    let black_king = find_files(&fen.board[0], 'k');

    let white_king_file = if white_king.is_empty() {
        255
    } else {
        white_king[0]
    };
    let black_king_file = if black_king.is_empty() {
        255
    } else {
        black_king[0]
    };

    let white_rooks = find_files(&fen.board[7], 'R');
    let black_rooks = find_files(&fen.board[0], 'r');

    let mut result = String::new();

    //Convert different castle rights format into unified 'HAha'
    for char in rights.chars() {
        match char {
            'K' => {
                if white_king_file >= 8 {
                    continue;
                }

                let file = white_rooks
                    .iter()
                    .rev()
                    .find(|&element| *element > white_king_file);
                if let Some(file) = file {
                    result.push(char::from(b'A' + file));
                }
            }
            'Q' => {
                if white_king_file >= 8 {
                    continue;
                }

                let file = white_rooks
                    .iter()
                    .find(|&element| *element < white_king_file);
                if let Some(file) = file {
                    result.push(char::from(b'A' + file));
                }
            }
            'k' => {
                if black_king_file >= 8 {
                    continue;
                }

                let file = black_rooks
                    .iter()
                    .rev()
                    .find(|&element| *element > black_king_file);
                if let Some(file) = file {
                    result.push(char::from(b'a' + file));
                }
            }
            'q' => {
                if black_king_file >= 8 {
                    continue;
                }

                let file = black_rooks
                    .iter()
                    .find(|&element| *element < black_king_file);
                if let Some(file) = file {
                    result.push(char::from(b'a' + file));
                }
            }
            'A'..='H' | 'a'..='h' => {
                result.push(char);
            }
            _ => continue,
        }
    }

    if result.is_empty() {
        String::from("-")
    } else {
        result
    }
}
