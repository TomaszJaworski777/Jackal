use chess::{Piece, Side};

//-----------------------------------------------
// This input code was written based on Monty 768 inputs logic.
//-----------------------------------------------

pub struct Standard768;
#[allow(unused)]
impl Standard768 {
    pub const fn input_size() -> usize {
        768
    }

    pub fn map_inputs<F: FnMut(usize)>(board: &chess::ChessBoard, mut process_input: F) {
        let flip = board.side() == Side::BLACK;

        for piece_idx in u8::from(Piece::PAWN)..=u8::from(Piece::KING) {
            let feat_idx = 64 * (piece_idx - u8::from(Piece::PAWN)) as usize;

            let mut stm_bitboard = board.piece_mask_for_side(Piece::from(piece_idx), board.side());
            let mut nstm_bitboard = board.piece_mask_for_side(Piece::from(piece_idx), board.side().flipped());

            if flip {
                stm_bitboard = stm_bitboard.flip();
                nstm_bitboard = nstm_bitboard.flip();
            }

            stm_bitboard.map(|square| {
                let feat = feat_idx + usize::from(square);
                process_input(feat)
            });

            nstm_bitboard.map(|square| {
                let feat = 384 + feat_idx + usize::from(square);
                process_input(feat)
            });
        }
    }
}