use bullet::{game::inputs::{self, SparseInputType}, lr, nn::optimiser::AdamW, policy::{loader::PolicyDataLoader, move_maps::{self, MoveBucket}, PolicyLocalSettings, PolicyTrainerBuilder, PolicyTrainingSchedule}, trainer::save::{Layout, QuantTarget, SavedFormat}, Shape, TrainingSteps};

const HL_SIZE: usize = 128;

const END_SUPERBATCH: usize = 50;
const START_LR: f32 = 0.001;
const END_LR: f32 = 0.00001;

#[allow(unused)]
pub fn run() {
    let inputs = inputs::Chess768;
    let transform = move_maps::HorizontalMirror;
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
        net_id: "policy_128_50_111",
        lr_scheduler: lr::CosineDecayLR { initial_lr: START_LR, final_lr: END_LR, final_superbatch: END_SUPERBATCH },
        steps: TrainingSteps {
            batch_size: 16_384,
            batches_per_superbatch: 6104,
            start_superbatch: 1,
            end_superbatch: END_SUPERBATCH,
        },
        save_rate: 10,
    };

    let settings = PolicyLocalSettings { data_prep_threads: 4, output_directory: "policy_checkpoints", batch_queue_size: 64 };

    let data_loader = PolicyDataLoader::new("policy_data.bin", 48000);

    trainer.run(&schedule, &settings, &data_loader);

    trainer.display_eval("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    trainer.display_eval("rk6/8/8/p7/P7/Q7/R7/RK6 w - - 80 200");
    trainer.display_eval("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    trainer.display_eval("8/8/p5p1/2bk1p1p/5P1P/1P3PK1/8/4B3 b - - 3 48");
}