use goober::{activation, layer::SparseConnected, FeedForwardNetwork, Matrix, SparseVector, Vector};
use spear::{ChessBoard, Move, Side};

pub struct PolicyTrainer;
impl PolicyTrainer {
    pub fn execute() {

    }
}

#[repr(C)]
#[derive(Clone, Copy, FeedForwardNetwork)]
struct TrainerPolicySubnet {
    l0: SparseConnected<activation::ReLU, 768, 16>,
}

impl TrainerPolicySubnet {
    pub const fn zeroed() -> Self {
        Self { l0: SparseConnected::zeroed() }
    }

    pub fn from_fn<F: FnMut() -> f32>(mut f: F) -> Self {
        let weights = Matrix::from_fn(|_, _| f());
        let biases = Vector::from_fn(|_| f());

        Self { l0: SparseConnected::from_raw(weights, biases) }
    }
}

struct TrainerPolicyNet {
    pub subnets: [TrainerPolicySubnet; 128],
}

#[allow(unused)]
impl TrainerPolicyNet {
    pub const fn zeroed() -> Self {
        Self {
            subnets: [TrainerPolicySubnet::zeroed(); 128],
        }
    }

    #[inline]
    pub fn evaluate(&self, board: &ChessBoard, mv: &Move, inputs: &SparseVector) -> f32 {
        let flip = if board.side_to_move() == Side::WHITE { 0 } else { 56 };

        let from_subnet = &self.subnets[usize::from(mv.get_from_square().get_raw() ^ flip)];
        let from_vec = from_subnet.out(inputs);

        let to_subnet = &self.subnets[64 + usize::from(mv.get_to_square().get_raw() ^ flip)];
        let to_vec = to_subnet.out(inputs);

        from_vec.dot(&to_vec)
    }
}