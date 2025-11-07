use chess::{Piece, Side};

//-----------------------------------------------
// This input code was written based on Monty 3072 inputs logic.
//-----------------------------------------------

pub struct Threats3072;
#[allow(unused)]
impl Threats3072 {
    pub const fn input_size() -> usize {
        3072
    }

    pub fn map_inputs<F: FnMut(usize)>(board: &chess::ChessBoard, mut process_input: F) {
         let horizontal_mirror = if board.king_square(board.side()).file() > 3 {
            7
        } else {
            0
        };

        let flip = board.side() == Side::BLACK;

        let mut threats = board.generate_attack_map(board.side().flipped());
        let mut defences = board.generate_attack_map(board.side());

        if flip {
            threats = threats.flip();
            defences = defences.flip();
        }

        for piece in u8::from(Piece::PAWN)..=u8::from(Piece::KING) {
            let piece_index = 64 * (piece - u8::from(Piece::PAWN)) as usize;

            let mut stm_bitboard =
                board.piece_mask_for_side(Piece::from(piece), board.side());
            let mut nstm_bitboard =
                board.piece_mask_for_side(Piece::from(piece), board.side().flipped());

            if flip {
                stm_bitboard = stm_bitboard.flip();
                nstm_bitboard = nstm_bitboard.flip();
            }

            stm_bitboard.map(|square| {
                let mut feat = piece_index + (usize::from(square) ^ horizontal_mirror);

                if threats.get_bit(square) {
                    feat += 768;
                }

                if defences.get_bit(square) {
                    feat += 768 * 2;
                }

                process_input(feat)
            });

            nstm_bitboard.map(|square| {
                let mut feat = 384 + piece_index + (usize::from(square) ^ horizontal_mirror);

                if threats.get_bit(square) {
                    feat += 768;
                }

                if defences.get_bit(square) {
                    feat += 768 * 2;
                }

                process_input(feat)
            });
        }
    }
}