use spear::ChessBoard;

use super::NetworkLayer;

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

        self.l0
            .map_value_inputs::<_, STM_WHITE, NSTM_WHITE>(board, |weight_index| {
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
}
