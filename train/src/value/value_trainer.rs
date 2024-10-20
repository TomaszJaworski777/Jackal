use bullet::{
    format::{chess::BoardIter, ChessBoard}, inputs, loader, lr, optimiser, outputs, wdl, LocalSettings, Loss, TrainerBuilder, TrainingSchedule, TrainingSteps
};
use spear::{Bitboard, Piece, Square};

pub struct ValueTrainer;
impl ValueTrainer {
    pub fn execute() {
        let mut trainer = TrainerBuilder::default()
            .optimiser(optimiser::AdamW)
            .single_perspective()
            .loss_fn(Loss::SigmoidMSE)
            .input(ThreatsDefencesMirroredInputs)
            .output_buckets(outputs::Single)
            .feature_transformer(64)
            .activate(bullet::Activation::SCReLU)
            .add_layer(1)
            .build();

        let schedule = TrainingSchedule {
            net_id: "value_010ft2".to_string(),
            eval_scale: 400.0,
            steps: TrainingSteps {
                batch_size: 16_384,
                batches_per_superbatch: 6104,
                start_superbatch: 1,
                end_superbatch: 1,
            },
            wdl_scheduler: wdl::ConstantWDL { value: 1.0 },
            lr_scheduler: lr::ConstantLR {
                value: 0.0000001,
            },
            save_rate: 1,
        };

        let settings = LocalSettings {
            threads: 16,
            test_set: None,
            output_directory: "checkpoints",
            batch_queue_size: 512,
        };

        let data_loader = loader::DirectSequentialDataLoader::new(&["./shuffled_finetune_data.bin"]);

        trainer.load_from_checkpoint("checkpoints/value_010-50");
        trainer.run(&schedule, &settings, &data_loader);
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
