use crate::{base_structures::CastleRights, board::ChessBoard, Piece, Side, Square, FEN};

impl From<&FEN> for ChessBoard {
    fn from(value: &FEN) -> Self {
        let mut board = Self::default();

        for (rank_index, rank) in value.board.clone().into_iter().enumerate() {
            let mut index = 0;
            let mut file = 0;
            while file < 8 {
                let square = Square::from_coords((7 - rank_index) as u8, file);
                let piece_char = rank.as_bytes()[index] as char;
                if piece_char.is_numeric() {
                    file += piece_char as u8 - b'0';
                    index += 1;
                    continue;
                }

                let side = Side::from(piece_char as u8 > 90);

                match piece_char.to_ascii_lowercase() {
                    'p' => board.set_piece_on_square(square, Piece::PAWN, side),
                    'n' => board.set_piece_on_square(square, Piece::KNIGHT, side),
                    'b' => board.set_piece_on_square(square, Piece::BISHOP, side),
                    'r' => board.set_piece_on_square(square, Piece::ROOK, side),
                    'q' => board.set_piece_on_square(square, Piece::QUEEN, side),
                    'k' => board.set_piece_on_square(square, Piece::KING, side),
                    _ => println!("Incorrect character in fen string!"),
                }

                index += 1;
                file += 1;
            }
        }

        board.side = Side::from(value.side_to_move == "b");

        if board.is_square_attacked(
            board.king_square(board.side.flipped()),
            board.side.flipped(),
        ) {
            println!("Tried to parse illegal position. Defaulting to starting position instead.");
            return Self::default();
        }

        let kings = [
            board.king_square(Side::WHITE),
            board.king_square(Side::BLACK),
        ];
        let mut rooks = [Square::NULL; 4];
        let mut rights = 0u8;

        for char in value.castle_rights.chars() {
            if char == '-' {
                break;
            }

            let side = if char.is_ascii_uppercase() {
                Side::WHITE
            } else {
                Side::BLACK
            };
            let king_square = board.king_square(side);
            let file = char.to_ascii_uppercase() as u8 - b'A';
            let index = 2 * u8::from(side) + if file < king_square.file() { 0 } else { 1 };
            rights |= 0b1000 >> index;
            rooks[usize::from(index)] = Square::from_coords(king_square.get_rank(), file);
        }

        board.castle_rights = CastleRights::create_base(rooks, kings);
        board.castle_rights.set_rights(rights);

        if value.en_passant_square != "-" {
            board.en_passant_square = Square::from(value.en_passant_square.clone())
        }

        board.half_moves = value.half_move_counter.parse::<u8>().unwrap_or(0);

        board
    }
}

impl From<&ChessBoard> for FEN {
    fn from(value: &ChessBoard) -> Self {
        let mut fen = String::new();

        // Piece placement
        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                let square = Square::from_coords(rank, file);
                let piece = value.piece_on_square(square);
                let side = value.color_on_square(square);
                if piece != Piece::NONE {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    let piece_char = char::from(piece);
                    if side == Side::WHITE {
                        fen.push(piece_char.to_uppercase().next().unwrap());
                    } else {
                        fen.push(piece_char);
                    }
                } else {
                    empty_count += 1;
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        // Side to move
        fen.push(' ');
        fen.push(if value.side() == Side::WHITE {
            'w'
        } else {
            'b'
        });

        // Castling rights
        fen.push(' ');
        fen.push_str(String::from(*value.castle_rights()).as_str());

        // En passant target square
        fen.push(' ');
        if value.en_passant_square() == Square::NULL {
            fen.push('-');
        } else {
            fen.push_str(String::from(value.en_passant_square()).as_str());
        }

        // Halfmove clock and fullmove number
        fen.push(' ');
        fen.push_str(&value.half_moves().to_string());
        fen.push(' ');
        fen.push_str(&(1).to_string());

        FEN::from(fen)
    }
}
