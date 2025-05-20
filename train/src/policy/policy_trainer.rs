use bullet::{
    default::inputs::{self, SparseInputType}, game::formats::bulletformat::ChessBoard, nn::optimiser::AdamW, policy::{
        loader::PolicyDataLoader,
        move_maps::{self, MoveBucket},
        PolicyLocalSettings, PolicyTrainerBuilder, PolicyTrainingSchedule,
    }, trainer::{
        save::{Layout, QuantTarget, SavedFormat},
        schedule::{lr, TrainingSteps}, NetworkTrainer,
    }, Shape
};
use jackal::{Bitboard, Piece, Side, Square};

const HL_SIZE: usize = 512;
const QA: i16 = 255;
const QB: i16 = 64;

pub struct PolicyTrainer;
impl PolicyTrainer {
    pub fn execute() {
    let inputs = inputs::Chess768;
    let transform = move_maps::NoTransform;
    let buckets = move_maps::GoodSEEBuckets(-108);

    let num_inputs = inputs.num_inputs();
    let num_outputs = buckets.num_buckets() * move_maps::UNIQUE_CHESS_MOVES;

    let l1_shape = Shape::new(num_outputs, HL_SIZE);

    let save_format = [
        SavedFormat::new("l0w", QuantTarget::Float, Layout::Normal),
        SavedFormat::new("l0b", QuantTarget::Float, Layout::Normal),
        SavedFormat::new("l1w", QuantTarget::Float, Layout::Transposed(l1_shape)),
        SavedFormat::new("l1b", QuantTarget::Float, Layout::Normal),
    ];

    let mut trainer = PolicyTrainerBuilder::default()
        .single_perspective()
        .inputs(inputs)
        .optimiser(AdamW)
        .move_mapper(transform, buckets)
        .save_format(&save_format)
        .build(|builder, stm| {
            let l0 = builder.new_affine("l0", num_inputs, HL_SIZE);
            let l1 = builder.new_affine("l1", HL_SIZE, num_outputs);

            let out = l0.forward(stm).screlu();
            //let out = out.pairwise_mul();
            l1.forward(out)
        });

    let schedule = PolicyTrainingSchedule {
        net_id: "policy_007cos-512see_150-monty",
        lr_scheduler: lr::CosineDecayLR { initial_lr: 0.001, final_lr: 0.00001, final_superbatch: 150 },
        steps: TrainingSteps {
            batch_size: 16_384,
            batches_per_superbatch: 1024,
            start_superbatch: 1,
            end_superbatch: 150,
        },
        save_rate: 10,
    };

    let settings = PolicyLocalSettings { data_prep_threads: 6, output_directory: "policy_checkpoints", batch_queue_size: 64 };

    let data_loader = PolicyDataLoader::new("monty.binpack", 48000);

    //trainer.load_from_checkpoint("policy_checkpoints/policy_007cos-tdp1024see_100-300");

    trainer.run(&schedule, &settings, &data_loader);

    trainer.display_eval("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    trainer.display_eval("rk6/8/8/p7/P7/Q7/R7/RK6 w - - 80 200");
    trainer.display_eval("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    trainer.display_eval("8/8/p5p1/2bk1p1p/5P1P/1P3PK1/8/4B3 b - - 3 48");
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

        let horizontal_mirror = if pos.our_ksq() % 8 > 3 { 7 } else { 0 };

        let flip = board.side_to_move() == Side::BLACK;

        let mut threats = if board.side_to_move() == Side::WHITE { board.generate_attack_map::<true, false>() } else { board.generate_attack_map::<false, true>() };
        let mut defences = if board.side_to_move() == Side::WHITE { board.generate_attack_map::<false, true>() } else { board.generate_attack_map::<true, false>() };

        if flip {
            threats = threats.flip();
            defences = defences.flip();
        }

        for piece in Piece::PAWN.get_raw()..=Piece::KING.get_raw() {
            let piece_index = 64 * (piece - Piece::PAWN.get_raw()) as usize;

            let mut stm_bitboard = if board.side_to_move() == Side::WHITE { board.get_piece_mask_for_side::<true>(Piece::from_raw(piece)) } else { board.get_piece_mask_for_side::<false>(Piece::from_raw(piece)) };
            let mut nstm_bitboard = if board.side_to_move() == Side::WHITE { board.get_piece_mask_for_side::<false>(Piece::from_raw(piece)) } else { board.get_piece_mask_for_side::<true>(Piece::from_raw(piece)) };

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

                f(feat, feat)
            });

            nstm_bitboard.map(|square| {
                let mut feat = 384 + piece_index + (square.get_raw() as usize ^ horizontal_mirror);

                if threats.get_bit(square) {
                    feat += 768;
                }

                if defences.get_bit(square) {
                    feat += 768 * 2;
                }

                f(feat, feat)
            });
        }
    }

    fn shorthand(&self) -> String {
        format!("{}", self.num_inputs())
    }

    fn description(&self) -> String {
        "Threat & Defences inputs".to_string()
    }
}