use bullet::{
    default::inputs::{self, SparseInputType}, game::formats::bulletformat::ChessBoard, nn::optimiser::AdamW, policy::{
        loader::PolicyDataLoader,
        move_maps::{self, MoveBucket},
        PolicyLocalSettings, PolicyTrainerBuilder, PolicyTrainingSchedule,
    }, trainer::{
        save::{Layout, QuantTarget, SavedFormat},
        schedule::{lr, TrainingSteps},
    }, Shape
};
use jackal::{Bitboard, Piece, Square};

const HL_SIZE: usize = 1024;
const QA: i16 = 255;
const QB: i16 = 64;

pub struct PolicyTrainer;
impl PolicyTrainer {
    pub fn execute() {
    let inputs = ThreatsDefencesMirroredInputs;
    let transform = move_maps::HorizontalMirror;
    let buckets = move_maps::GoodSEEBuckets(-108);

    let num_inputs = inputs.num_inputs();
    let num_outputs = buckets.num_buckets() * move_maps::UNIQUE_CHESS_MOVES;

    let l1_shape = Shape::new(num_outputs, HL_SIZE / 2);

    let save_format = [
        SavedFormat::new("l0w", QuantTarget::I16(QA), Layout::Normal),
        SavedFormat::new("l0b", QuantTarget::I16(QA), Layout::Normal),
        SavedFormat::new("l1w", QuantTarget::I16(QB), Layout::Transposed(l1_shape)),
        SavedFormat::new("l1b", QuantTarget::I16(QA * QB), Layout::Normal),
    ];

    let mut trainer = PolicyTrainerBuilder::default()
        .single_perspective()
        .inputs(inputs)
        .optimiser(AdamW)
        .move_mapper(transform, buckets)
        .save_format(&save_format)
        .build(|builder, stm| {
            let l0 = builder.new_affine("l0", num_inputs, HL_SIZE);
            let l1 = builder.new_affine("l1", HL_SIZE / 2, num_outputs);

            let out = l0.forward(stm).crelu();
            let out = out.pairwise_mul();
            l1.forward(out)
        });

    let schedule = PolicyTrainingSchedule {
        net_id: "policy_007-tdp1024see_150",
        lr_scheduler: lr::ExponentialDecayLR { initial_lr: 0.001, final_lr: 0.00001, final_superbatch: 150 },
        steps: TrainingSteps {
            batch_size: 16_384,
            batches_per_superbatch: 6104,
            start_superbatch: 1,
            end_superbatch: 150,
        },
        save_rate: 10,
    };

    let settings = PolicyLocalSettings { data_prep_threads: 6, output_directory: "policy_checkpoints", batch_queue_size: 64 };

    let data_loader = PolicyDataLoader::new("conv_policy_data.bin", 48000);

    trainer.run(&schedule, &settings, &data_loader);

    trainer.display_eval("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
}
}

#[derive(Clone, Copy, Default)]
pub struct ThreatsDefencesMirroredInputs;
impl inputs::SparseInputType for ThreatsDefencesMirroredInputs {
    type RequiredDataType = ChessBoard;

    fn max_active(&self) -> usize {
        32
    }

    fn num_inputs(&self) -> usize {
        768 * 4
    }

    fn map_features<F: FnMut(usize, usize)>(&self, pos: &Self::RequiredDataType, mut f: F) {
        let mut bb = [Bitboard::EMPTY; 8];

        for (pc, sq) in pos.into_iter() {
            let square = Square::from_raw(sq);
            bb[usize::from(pc & 8 > 0)].set_bit(square);
            bb[usize::from(2 + (pc & 7))].set_bit(square);
        }

        let mut board = jackal::ChessBoard::default();
        for idx in 2..8 {
            let piece = Piece::from_raw(idx as u8 - 2);
            for side in 0..=1 {
                (bb[idx] & bb[side]).map(|square| {
                    if side == 0 {
                        board.set_piece_on_square::<true>(square, piece);
                    } else {
                        board.set_piece_on_square::<false>(square, piece);
                    }
                });
            }
        }

        let threats = board.generate_attack_map::<true, false>();
        let defences = board.generate_attack_map::<false, true>();

        let flip = if pos.our_ksq() % 8 > 3 { 7 } else { 0 };

        for (piece, square) in pos.into_iter() {
            let piece_index = usize::from(piece & 7);
            let square = usize::from(square);
            let side = usize::from(piece & 8 > 0);
            let mut input = (side * 384) + (64 * piece_index) + (square ^ flip);

            if threats.get_bit(Square::from_raw(square as u8)) {
                input += 768;
            }

            if defences.get_bit(Square::from_raw(square as u8)) {
                input += 768 * 2;
            }

            f(input, input)
        }
    }

    fn shorthand(&self) -> String {
        format!("{}", self.num_inputs())
    }

    fn description(&self) -> String {
        "Threat & Defences inputs".to_string()
    }
}