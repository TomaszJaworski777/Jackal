use spear::{ChessBoard, Piece, Side};

use super::NetworkLayer;

#[allow(non_upper_case_globals)]
pub static ValueNetwork: ValueNetwork = unsafe {
    std::mem::transmute(*include_bytes!(
        "../../../resources/networks/v440cos1024td004.network"
    ))
};

#[repr(C)]
pub struct ValueNetwork {
    l1: NetworkLayer<{ 768 * 4 }, 1024>,
    l2: NetworkLayer<1024, 1>,
}

impl ValueNetwork {
    pub fn forward<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
        board: &ChessBoard,
    ) -> f32 {
        let mut l1_out = *self.l1.biases();

        Self::map_value_inputs::<_, STM_WHITE, NSTM_WHITE>(board, |weight_index| {
            for (i, weight) in l1_out
                .values_mut()
                .iter_mut()
                .zip(&self.l1.weights()[weight_index].vals)
            {
                *i += *weight;
            }
        });

        let out = self.l2.forward(&l1_out);
        out.values()[0]
    }

    fn map_value_inputs<F: FnMut(usize), const STM_WHITE: bool, const NSTM_WHITE: bool>(
        board: &ChessBoard,
        mut method: F,
    ) {
        let horizontal_mirror = if board.get_king_square::<STM_WHITE>().get_file() > 3 {
            7
        } else {
            0
        };
        
        let flip = board.side_to_move() == Side::BLACK;

        let mut threats = board.generate_attack_map::<STM_WHITE, NSTM_WHITE>();
        let mut defences = board.generate_attack_map::<NSTM_WHITE, STM_WHITE>();

        if flip {
            threats = threats.flip();
            defences = defences.flip();
        }

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

            stm_bitboard.map(|square| {
                let mut feat = piece_index + (square.get_raw() as usize ^ horizontal_mirror);

                if threats.get_bit(square) {
                    feat += 768;
                }
    
                if defences.get_bit(square) {
                    feat += 768 * 2;
                }

                method(feat)
            });

            nstm_bitboard.map(|square| {
                let mut feat = 384 + piece_index + (square.get_raw() as usize ^ horizontal_mirror);

                if threats.get_bit(square) {
                    feat += 768;
                }
    
                if defences.get_bit(square) {
                    feat += 768 * 2;
                }

                method(feat)
            });
        }
    }
}
