use bullet::{
    default::inputs::{self, SparseInputType},
    nn::optimiser::AdamW,
    policy::{
        loader::PolicyDataLoader,
        move_maps::{self, MoveBucket},
        PolicyLocalSettings, PolicyTrainerBuilder, PolicyTrainingSchedule,
    },
    trainer::{
        save::{Layout, QuantTarget, SavedFormat},
        schedule::{lr, TrainingSteps},
    }, Shape,
};

const HL_SIZE: usize = 64;

pub struct PolicyTrainer;
impl PolicyTrainer {
    pub fn execute() {
    let inputs = inputs::Chess768;
    let transform = move_maps::HorizontalMirror;
    let buckets = move_maps::NoBuckets;

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
            l1.forward(out)
        });

    let schedule = PolicyTrainingSchedule {
        net_id: "policy_007-128_40",
        lr_scheduler: lr::ExponentialDecayLR { initial_lr: 0.001, final_lr: 0.00001, final_superbatch: 40 },
        steps: TrainingSteps {
            batch_size: 16_384,
            batches_per_superbatch: 6104,
            start_superbatch: 1,
            end_superbatch: 40,
        },
        save_rate: 10,
    };

    let settings = PolicyLocalSettings { data_prep_threads: 6, output_directory: "policy_checkpoints", batch_queue_size: 64 };

    let data_loader = PolicyDataLoader::new("conv_policy_data.bin", 48000);

    trainer.run(&schedule, &settings, &data_loader);

    trainer.display_eval("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
}
}