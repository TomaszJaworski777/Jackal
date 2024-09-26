use bullet::{
    inputs, loader, lr, optimiser, outputs, wdl, LocalSettings, Loss, TrainerBuilder,
    TrainingSchedule,
};

pub struct ValueTrainer;
impl ValueTrainer {
    pub fn execute() {
        let mut trainer = TrainerBuilder::default()
            .optimiser(optimiser::AdamW)
            .single_perspective()
            .input(inputs::Chess768::default())
            .output_buckets(outputs::Single)
            .feature_transformer(64)
            .activate(bullet::Activation::SCReLU)
            .add_layer(1)
            .build();

        let schedule = TrainingSchedule {
            net_id: "value_005".to_string(),
            eval_scale: 400.0,
            ft_regularisation: 0.0,
            batch_size: 16_384,
            batches_per_superbatch: 6104,
            start_superbatch: 31,
            end_superbatch: 40,
            wdl_scheduler: wdl::ConstantWDL { value: 1.0 },
            lr_scheduler: lr::StepLR {
                start: 0.001,
                gamma: 0.1,
                step: 10,
            },
            loss_function: Loss::SigmoidMSE,
            save_rate: 5,
            optimiser_settings: optimiser::AdamWParams {
                decay: 0.01,
                beta1: 0.9,
                beta2: 0.999,
                min_weight: -1.98,
                max_weight: 1.98,
            },
        };

        let settings = LocalSettings {
            threads: 7,
            test_set: None,
            output_directory: "checkpoints",
            batch_queue_size: 512,
        };

        let data_loader = loader::DirectSequentialDataLoader::new(&["./bullet_data_shuffled.bin"]);

        trainer.load_from_checkpoint("checkpoints/value_005-30");
        trainer.run(&schedule, &settings, &data_loader);
    }
}
