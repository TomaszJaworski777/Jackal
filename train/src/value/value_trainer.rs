use bullet::{default::{
            formats::bulletformat::ChessBoard,
            inputs::{self, SparseInputType},
            loader,
    }, game::outputs::MaterialCount, nn::{optimiser::{self, AdamW, AdamWOptimiser, AdamWParams}, Activation, ExecutionContext, Graph, NetworkBuilder, Node, Shape}, trainer::{
        default::{outputs, Trainer}, save::{Layout, QuantTarget, SavedFormat}, schedule::{ lr, wdl, TrainingSchedule, TrainingSteps }, settings::LocalSettings
    }, value::ValueTrainerBuilder
};
use jackal::{Bitboard, Piece, Square};

const HIDDEN_SIZE: usize = 2048;
const QA: i16 = 255;
const QB: i16 = 64;

const NUM_OUTPUT_BUCKETS: usize = 4;

pub struct ValueTrainer;
impl ValueTrainer {
    pub fn execute() {
        let mut trainer = ValueTrainerBuilder::default()
            .single_perspective()
            .wdl_output()
            .optimiser(AdamW)
            .inputs(ThreatsDefencesMirroredInputs)
            .output_buckets(MaterialCount::<NUM_OUTPUT_BUCKETS>)
            .save_format(&[
                SavedFormat::id("l0w").quantise::<i16>(QA),
                SavedFormat::id("l0b").quantise::<i16>(QA),
                SavedFormat::id("l1w").quantise::<i16>(QB).transpose(),
                SavedFormat::id("l1b").quantise::<i16>(QA * QB),
            ])
            .loss_fn(|output, target| output.softmax_crossentropy_loss(target))
            .build(|builder, inputs, output_buckets| {
                // weights
                let l0 = builder.new_affine("l0", ThreatsDefencesMirroredInputs.num_inputs(), HIDDEN_SIZE);
                let l1 = builder.new_affine("l1", HIDDEN_SIZE, 3 * NUM_OUTPUT_BUCKETS);

                // inference
                let out = l0.forward(inputs).activate(Activation::SCReLU);
                l1.forward(out).select(output_buckets)
            });


        let schedule: TrainingSchedule<lr::CosineDecayLR, wdl::ConstantWDL> = TrainingSchedule {
            net_id: "v600cos2048WDL-TD-OB-007a-Q".to_string(),
            eval_scale: 400.0,
            steps: TrainingSteps {
                batch_size: 16_384,
                batches_per_superbatch: 6104,
                start_superbatch: 1,
                end_superbatch: 600,
            },
            wdl_scheduler: wdl::ConstantWDL { value: 1.0 },
            lr_scheduler: lr::CosineDecayLR {
                initial_lr: 0.001,
                final_lr: 0.00001,
                final_superbatch: 600,
            },
            save_rate: 5,
        };

        let optimiser_params = optimiser::AdamWParams {
            decay: 0.01,
            beta1: 0.9,
            beta2: 0.999,
            min_weight: -0.99,
            max_weight: 0.99,
        };

        trainer.set_optimiser_params(optimiser_params);

        let settings = LocalSettings {
            threads: 8,
            test_set: None,
            output_directory: "checkpoints",
            batch_queue_size: 512,
        };

        let data_loader =
            loader::DirectSequentialDataLoader::new(&["./shuffled_value_data.bin"]);

        //trainer.load_from_checkpoint("checkpoints/value_014_2048_wdl-600");
        trainer.run(&schedule, &settings, &data_loader);

        for fen in [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        ] {
            let raw = trainer.eval_raw_output(fen);
            let (mut w, mut d, mut l) = (raw[2], raw[1], raw[0]);
            let max = w.max(d).max(l);
            
            w = (w - max).exp();
            d = (d - max).exp();
            l = (l - max).exp();

            let sum = w + d + l;

            println!("FEN: {fen}");
            println!("EVAL: [{},{},{}]", w / sum, d / sum, l / sum);
        }
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