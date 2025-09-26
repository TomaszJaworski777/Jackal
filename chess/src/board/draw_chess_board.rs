use utils::{PieceColors, Theme};

use crate::{board::ChessBoard, Piece, Side, Square, FEN};

impl ChessBoard {
    pub fn draw_board(&self) {
        let piece_icons: [[&str; 6]; 2] = [
            [" P", " N", " B", " R", " Q", " K"],
            [" p", " n", " b", " r", " q", " k"],
        ];

        let mut info = Vec::new();
        let fen = format!(
            "FEN: {}",
            FEN::from(self).to_string().secondary(1.0 / 18.0 + 0.15)
        );
        info.push(fen.as_str());
        let zobrist = format!(
            "Zobrist Key: {}",
            self.hash().to_string().secondary(2.0 / 18.0 + 0.15)
        );
        info.push(zobrist.as_str());

        let castle_rights = format!(
            "Castle Rights: {}",
            self.castle_rights()
                .to_string()
                .secondary(3.0 / 18.0 + 0.15)
        );
        info.push(castle_rights.as_str());
        let side_sign = format!(
            "Side To Move: {}",
            self.side().to_string().secondary(4.0 / 18.0 + 0.15)
        );
        info.push(side_sign.as_str());
        let en_passant = format!(
            "En Passant: {}",
            self.en_passant_square()
                .to_string()
                .secondary(5.0 / 18.0 + 0.15)
        );
        info.push(en_passant.as_str());
        let half_moves = format!(
            "Half Moves: {}",
            self.half_moves().to_string().secondary(6.0 / 18.0 + 0.15)
        );
        info.push(half_moves.as_str());
        let in_check = format!(
            "In Check: {}",
            self.is_in_check().to_string().secondary(7.0 / 18.0 + 0.15)
        );
        info.push(in_check.as_str());
        let phase = format!(
            "Phase: {}",
            self.phase().to_string().secondary(8.0 / 18.0 + 0.15)
        );
        info.push(phase.as_str());

        let mut result = "   -----------------\n".to_string().primary(0.15);
        for rank in 0..8 {
            result += format!("{} |", if self.side() == Side::WHITE { 7 - rank } else { rank } + 1)
                .primary((rank + 1) as f32 / 18.0 + 0.15)
                .as_str();
            for file in 0..8 {
                let square = Square::from_coords(
                    if self.side() == Side::WHITE { 7 - rank } else { rank }, 
                    if self.side() == Side::WHITE { file } else { 7 - file }
                );
                
                if square == self.en_passant_square() {
                    result += " x";
                    continue;
                }

                let piece_type = self.piece_on_square(square);
                let piece_side = self.color_on_square(square);
                if piece_type == Piece::NONE {
                    result += " .";
                } else if piece_side == Side::BLACK {
                    result += piece_icons[usize::from(Side::BLACK)][usize::from(piece_type)]
                        .black_pieces()
                        .as_str();
                } else {
                    result += piece_icons[usize::from(Side::WHITE)][usize::from(piece_type)]
                        .white_pieces()
                        .as_str();
                }
            }
            result += format!(" | {}", info[rank as usize])
                .primary((rank + 1) as f32 / 18.0 + 0.15)
                .as_str();
            result += "\n".to_string().as_str();
        }
        result += "   -----------------\n".to_string().primary(0.75).as_str();

        let mut files: Vec<&str> = vec!["A", "B", "C", "D", "E", "F", "G", "H"];
        if self.side() == Side::BLACK {
            files.reverse();
        };
        result += format!("    {}\n", files.join(" ")).primary(0.77).as_str();

        println!("{}", result);
    }
}
