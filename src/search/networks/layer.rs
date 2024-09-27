use spear::{ChessBoard, Piece, Side};

use super::Accumulator;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NetworkLayer<const INPUTS: usize, const OUTPUTS: usize> {
    weights: [Accumulator<OUTPUTS>; INPUTS],
    biases: Accumulator<OUTPUTS>,
}

impl<const INPUTS: usize, const OUTPUTS: usize> Default for NetworkLayer<INPUTS, OUTPUTS> {
    fn default() -> Self {
        Self {
            weights: [Accumulator::default(); INPUTS],
            biases: Accumulator::default(),
        }
    }
}

impl<const INPUTS: usize, const OUTPUTS: usize> NetworkLayer<INPUTS, OUTPUTS> {
    #[inline]
    pub fn weights(&self) -> &[Accumulator<OUTPUTS>; INPUTS] {
        &self.weights
    }

    #[inline]
    pub fn biases(&self) -> &Accumulator<OUTPUTS> {
        &self.biases
    }

    pub fn forward(&self, inputs: &Accumulator<INPUTS>) -> Accumulator<OUTPUTS> {
        let mut result = self.biases;

        for (neuron, weights) in inputs.values().iter().zip(self.weights.iter()) {
            let activated = neuron.clamp(0.0, 1.0).powi(2);
            result.madd(activated, weights);
        }

        result
    }

    pub fn map_value_inputs<F: FnMut(usize), const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
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
