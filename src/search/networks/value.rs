use spear::{ChessBoard, Piece, Side};

pub struct ValueNetwork {
    l0: NetworkLayer<768, 64>,
    l1: NetworkLayer<64, 1>
}

impl ValueNetwork {
    pub fn forward<const STM_WHITE: bool, const NSTM_WHITE: bool>(&self, board: &ChessBoard) -> f32 {
        let mut l0_out = self.l0.biases;

        self.l0.map_value_inputs::<_, STM_WHITE, NSTM_WHITE>(board, |weight_index| {
            for (i, weight) in l0_out.vals.iter_mut().zip(&self.l0.weights[weight_index].vals) {
                *i += *weight;
            }
        });

        self.l1.forward(&l0_out).values()[0]
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NetworkLayer<const INPUTS: usize, const OUTPUTS: usize> {
    weights: [Accumulator<OUTPUTS>; INPUTS],
    biases: Accumulator<OUTPUTS>,
}

impl<const INPUTS: usize, const OUTPUTS: usize> Default for NetworkLayer<INPUTS, OUTPUTS> {
    fn default() -> Self {
        Self { weights: [Accumulator::default(); INPUTS], biases: Accumulator::default() }
    }
}

impl<const INPUTS: usize, const OUTPUTS: usize> NetworkLayer<INPUTS, OUTPUTS> {
    pub fn forward(&self, inputs: &Accumulator<INPUTS>) -> Accumulator<OUTPUTS> {
        let mut result = self.biases;

        for (neuron, weights) in inputs.vals.iter().zip(self.weights.iter()) {
            let activated = neuron.clamp(0.0, 1.0).powi(2);
            result.madd(activated, weights);
        }

        result
    }

    pub fn map_value_inputs<F: FnMut(usize), const STM_WHITE: bool, const NSTM_WHITE: bool>(&self, board: &ChessBoard, mut method: F) {
        let flip = board.side_to_move() == Side::BLACK;

        for piece in Piece::PAWN.get_raw()..=Piece::KING.get_raw() {
            let piece_index = 64 * (piece - Piece::PAWN.get_raw()) as usize;

            let mut stm_bitboard = board.get_piece_mask_for_side::<STM_WHITE>(Piece::from_raw(piece));
            let mut nstm_bitboard = board.get_piece_mask_for_side::<NSTM_WHITE>(Piece::from_raw(piece));

            if flip {
                stm_bitboard = stm_bitboard.flip();
                nstm_bitboard = nstm_bitboard.flip();
            }

            stm_bitboard.map(|square| {
                method(piece_index + (square.get_raw() as usize))
            });

            nstm_bitboard.map(|square| {
                method(384 + piece_index + (square.get_raw() as usize))
            });
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Accumulator<const HIDDEN: usize> {
    vals: [f32; HIDDEN],
}

impl<const SIZE: usize> Default for Accumulator<SIZE> {
    fn default() -> Self {
        Self { vals: [0.0; SIZE] }
    }
}

impl<const HIDDEN: usize> Accumulator<HIDDEN> {
    fn madd(&mut self, mul: f32, other: &Self) {
        for (i, &j) in self.vals.iter_mut().zip(other.vals.iter()) {
            *i += mul * j;
        }
    }

    pub fn values(&self) -> [f32; HIDDEN] {
        self.vals
    }
}