use spear::{ChessBoard, Piece, Side};

use super::NetworkLayer;

#[allow(non_upper_case_globals)]
pub static ValueNetwork: ValueNetwork = unsafe {
    std::mem::transmute(*include_bytes!(
        "../../../resources/networks/value_008b.network"
    ))
};

#[repr(C)]
pub struct ValueNetwork {
    l0: NetworkLayer<768, 64>,
    l1: NetworkLayer<64, 1>,
}

impl ValueNetwork {
    pub fn forward<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
        board: &ChessBoard,
    ) -> f32 {
        let mut l0_out = *self.l0.biases();

        Self::map_value_inputs::<_, STM_WHITE, NSTM_WHITE>(board, |weight_index| {
            for (i, weight) in l0_out
                .values_mut()
                .iter_mut()
                .zip(&self.l0.weights()[weight_index].vals)
            {
                *i += *weight;
            }
        });

        self.l1.forward(&l0_out).values()[0]
    }

    fn map_value_inputs<F: FnMut(usize), const STM_WHITE: bool, const NSTM_WHITE: bool>(
        board: &ChessBoard,
        mut method: F,
    ) {
        let flip = board.side_to_move() == Side::BLACK;
    
        for piece in Piece::PAWN.get_raw()..=Piece::KING.get_raw() {
            let piece_index = 64 * (piece - Piece::PAWN.get_raw()) as usize;
    
            let mut stm_bitboard =
                board.get_piece_mask_for_side::<STM_WHITE>(Piece::from_raw(piece));
            let mut nstm_bitboard =
                board.get_piece_mask_for_side::<NSTM_WHITE>(Piece::from_raw(piece));
    
            if flip {
                stm_bitboard = stm_bitboard.flip();
                nstm_bitboard = nstm_bitboard.flip();
            }
    
            stm_bitboard.map(|square| method(piece_index + (square.get_raw() as usize)));
    
            nstm_bitboard.map(|square| method(384 + piece_index + (square.get_raw() as usize)));
        }
    }
}