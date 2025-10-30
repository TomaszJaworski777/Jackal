use bullet::game::{formats::bulletformat::ChessBoard, inputs::SparseInputType};
use chess::{Bitboard, Piece, Side, Square};

#[derive(Clone, Copy, Debug, Default)]
pub struct PolicyInputs;
impl SparseInputType for PolicyInputs {
    type RequiredDataType = ChessBoard;

    fn num_inputs(&self) -> usize {
        768 * 4
    }

    fn max_active(&self) -> usize {
        32
    }

    fn map_features<F: FnMut(usize, usize)>(&self, pos: &Self::RequiredDataType, mut f: F) {
        let mut bb = [Bitboard::EMPTY; 8];

        for (pc, sq) in pos.into_iter() {
            let square = Square::from(sq);
            bb[usize::from(pc & 8 > 0)].set_bit(square);
            bb[usize::from(2 + (pc & 7))].set_bit(square);
        }

        let mut board = chess::ChessBoard::default();
        for idx in 2..8 {
            let piece = Piece::from(idx as u8 - 2);
            for side in 0..=1 {
                (bb[idx] & bb[side]).map(|square| {
                    board.set_piece_on_square(square, piece, Side::from(side as u8))
                });
            }
        }

        let flip = board.side() == Side::BLACK;
        let horizontal_mirror = if board.king_square(board.side()).get_file() > 3 {
            7
        } else {
            0
        };

        let mut threats = board.generate_attack_map(board.side().flipped());
        let mut defences = board.generate_attack_map(board.side());

        if flip {
            threats = threats.flip();
            defences = defences.flip();
        }

        for piece_idx in u8::from(Piece::PAWN)..=u8::from(Piece::KING) {
            let piece_input_index = 64 * (piece_idx - u8::from(Piece::PAWN)) as usize;
            
            let piece = Piece::from(piece_idx);
            let mut stm_bitboard = board.piece_mask_for_side(piece, board.side());
            let mut nstm_bitboard = board.piece_mask_for_side(piece, board.side().flipped());

            if flip {
                stm_bitboard = stm_bitboard.flip();
                nstm_bitboard = nstm_bitboard.flip();
            }

            stm_bitboard.map(|square| {
                let mut feat = piece_input_index + (usize::from(square) ^ horizontal_mirror);

                if threats.get_bit(square) {
                    feat += 768;
                }

                if defences.get_bit(square) {
                    feat += 768 * 2;
                }

                f(feat, feat)
            });

            nstm_bitboard.map(|square| {
                let mut feat = 384 + piece_input_index + (usize::from(square) ^ horizontal_mirror);

                if threats.get_bit(square) {
                    feat += 768;
                }

                if defences.get_bit(square) {
                    feat += 768 * 2;
                }

                f(feat, feat)
            });
        }
    }

    fn shorthand(&self) -> String {
        "768x4".to_string()
    }

    fn description(&self) -> String {
        "Default psqt chess inputs".to_string()
    }
}