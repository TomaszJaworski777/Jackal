use bullet::{
    format::{chess::BoardIter, ChessBoard},
    inputs::{self, InputType},
    loader, lr, operations,
    optimiser::{self, AdamWOptimiser, AdamWParams},
    outputs, wdl, Activation, ExecutionContext, Graph, GraphBuilder, LocalSettings, Node,
    QuantTarget, Shape, Trainer, TrainingSchedule, TrainingSteps,
};
use spear::{Bitboard, Piece, Square};

const HIDDEN_SIZE: usize = 1024;

pub struct ValueTrainer;
impl ValueTrainer {
    pub fn execute() {
        let mut trainer = make_trainer(HIDDEN_SIZE);

        let schedule: TrainingSchedule<lr::CosineDecayLR, wdl::ConstantWDL> = TrainingSchedule {
            net_id: "value_013_1024_wdl_finetune_10m".to_string(),
            eval_scale: 400.0,
            steps: TrainingSteps {
                batch_size: 16_384,
                batches_per_superbatch: 6104,
                start_superbatch: 1,
                end_superbatch: 25,
            },
            wdl_scheduler: wdl::ConstantWDL { value: 1.0 },
            lr_scheduler: lr::CosineDecayLR {
                initial_lr: 0.000000015,
                final_lr: 0.00000000015,
                final_superbatch: 25,
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
            loader::DirectSequentialDataLoader::new(&["./shuffled_finetune_data.bin"]);

        trainer.load_from_checkpoint("checkpoints/value_013_1024_wdl-600");
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

pub struct ThreatsDefencesMirroredInputsIter {
    board_iter: BoardIter,
    threats: Bitboard,
    defences: Bitboard,
    flip: usize,
}

impl inputs::InputType for ThreatsDefencesMirroredInputs {
    type RequiredDataType = ChessBoard;
    type FeatureIter = ThreatsDefencesMirroredInputsIter;

    fn buckets(&self) -> usize {
        1
    }

    fn max_active_inputs(&self) -> usize {
        32
    }

    fn inputs(&self) -> usize {
        768 * 4
    }

    fn feature_iter(&self, position: &Self::RequiredDataType) -> Self::FeatureIter {
        let mut bb = [Bitboard::EMPTY; 8];

        for (pc, sq) in position.into_iter() {
            let square = Square::from_raw(sq);
            bb[usize::from(pc >> 3)].set_bit(square);
            bb[usize::from(2 + (pc & 7))].set_bit(square);
        }

        let mut board = spear::ChessBoard::default();
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

        ThreatsDefencesMirroredInputsIter {
            board_iter: position.into_iter(),
            threats,
            defences,
            flip: if position.our_ksq() % 8 > 3 { 7 } else { 0 },
        }
    }
}

impl Iterator for ThreatsDefencesMirroredInputsIter {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.board_iter.next().map(|(piece, square)| {
            let piece_index = usize::from(piece & 7);
            let square = usize::from(square);
            let side = usize::from(piece & 8 > 0);
            let mut input = (side * 384) + (64 * piece_index) + (square ^ self.flip);

            if self.threats.get_bit(Square::from_raw(square as u8)) {
                input += 768;
            }

            if self.defences.get_bit(Square::from_raw(square as u8)) {
                input += 768 * 2;
            }

            (input, input)
        })
    }
}

fn make_trainer(
    l1: usize,
) -> Trainer<AdamWOptimiser, ThreatsDefencesMirroredInputs, outputs::Single> {
    let num_inputs = ThreatsDefencesMirroredInputs.size();

    let (mut graph, output_node) = build_network(num_inputs, l1);

    let sizes = [num_inputs, l1];

    for (i, &size) in sizes.iter().enumerate() {
        graph
            .get_weights_mut(&format!("l{i}w"))
            .seed_random(0.0, 1.0 / (size as f32).sqrt(), true);

        graph
            .get_weights_mut(&format!("l{i}b"))
            .seed_random(0.0, 1.0 / (size as f32).sqrt(), true);
    }

    Trainer::new(
        graph,
        output_node,
        AdamWParams::default(),
        ThreatsDefencesMirroredInputs,
        outputs::Single,
        vec![
            ("l0w".to_string(), QuantTarget::Float),
            ("l0b".to_string(), QuantTarget::Float),
            ("l1w".to_string(), QuantTarget::Float),
            ("l1b".to_string(), QuantTarget::Float),
        ],
        false,
    )
}

fn build_network(inputs: usize, l1: usize) -> (Graph, Node) {
    let mut builder = GraphBuilder::default();

    let stm = builder.create_input("stm", Shape::new(inputs, 1));
    let targets = builder.create_input("targets", Shape::new(3, 1));

    let l0w = builder.create_weights("l0w", Shape::new(l1, inputs));
    let l0b = builder.create_weights("l0b", Shape::new(l1, 1));
    let l1w = builder.create_weights("l1w", Shape::new(3, l1));
    let l1b = builder.create_weights("l1b", Shape::new(3, 1));

    let l1 = operations::affine(&mut builder, l0w, stm, l0b);
    let l1 = operations::activate(&mut builder, l1, Activation::SCReLU);
    let l2 = operations::affine(&mut builder, l1w, l1, l1b);

    operations::softmax_crossentropy_loss(&mut builder, l2, targets);
    (builder.build(ExecutionContext::default()), l2)
}
