use std::{
    f32::consts::PI,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    time::Instant,
};

use goober::{
    activation,
    layer::{DenseConnected, SparseConnected},
    FeedForwardNetwork, Matrix, OutputLayer, SparseVector, Vector,
};
use jackal::{ChessBoard, PolicyNetwork, PolicyPacked, Side, SEE};
use rand::{seq::SliceRandom, Rng};

const NAME: &'static str = "policy_007-32x32see_300";

const THREADS: usize = 6;
const SUPERBATCHES_COUNT: usize = 300;
const START_LR: f32 = 0.001;
const END_LR: f32 = 0.00001;
const WARMUP_BATCHES: usize = 200;

const BATCH_SIZE: usize = 16_384;
const BATCHES_PER_SUPERBATCH: usize = 1024;
const TRAINING_DATA_PATH: &'static str = "D:\\policy_data.bin";

pub struct PolicyTrainer;
impl PolicyTrainer {
    pub fn execute() {
        let root_dictionary = env!("CARGO_MANIFEST_DIR");
        let mut training_data_path = PathBuf::new();
        // training_data_path.push(root_dictionary);
        // training_data_path.push("..");
        training_data_path.push(TRAINING_DATA_PATH);
        let training_data_path = training_data_path.to_str().unwrap();
        let training_data = File::open(training_data_path).expect("Cannot open training data file");
        let training_metadata = training_data
            .metadata()
            .expect("Cannot obtain training metadata");

        let entry_count = training_metadata.len() as usize / std::mem::size_of::<PolicyPacked>();

        let mut policy = TrainerPolicyNet::rand_init();
        let throughput = SUPERBATCHES_COUNT * BATCHES_PER_SUPERBATCH * BATCH_SIZE;

        println!("Network Name:          {}", NAME);
        println!("Thread Count:          {}", THREADS);
        println!("Loaded Positions:      {}", entry_count);
        println!("Superbatches:          {}", SUPERBATCHES_COUNT);
        println!("Batch size:            {}", BATCH_SIZE);
        println!("Batches in superbatch: {}", BATCHES_PER_SUPERBATCH);
        println!("Start LR:              {}", START_LR);
        println!("End LR:                {}", END_LR);
        println!(
            "Epochs                 {:.2}\n",
            throughput as f64 / entry_count as f64
        );

        let mut momentum = boxed_and_zeroed::<TrainerPolicyNet>();
        let mut velocity = boxed_and_zeroed::<TrainerPolicyNet>();

        let mut running_error = 0.0;
        let mut learning_rate = START_LR;

        let mut superbatch_index = 0;
        let mut batch_index = 0;

        const BUFFER_SIZE: usize = 512;
        'training: loop {
            let training_data =
                File::open(training_data_path).expect("Cannot open training data file");

            let mut training_data_reader = BufReader::with_capacity(
                BUFFER_SIZE * BATCH_SIZE * std::mem::size_of::<PolicyPacked>(),
                training_data,
            );

            while let Ok(buffer) = training_data_reader.fill_buf() {
                if buffer.is_empty() {
                    break;
                }

                let mut superbatch: Vec<PolicyPacked> = unsafe {
                    std::slice::from_raw_parts(buffer.as_ptr().cast(), BUFFER_SIZE * BATCH_SIZE)
                }
                .to_vec();

                let mut rng = rand::thread_rng();
                superbatch.shuffle(&mut rng);

                let timer = Instant::now();

                for (idx, batch) in superbatch.chunks(BATCH_SIZE).enumerate() {
                    let mut gradient = boxed_and_zeroed::<TrainerPolicyNet>();
                    running_error += gradient_batch(&policy, &mut gradient, batch);
                    let adjustment = 1.0 / batch.len() as f32;

                    let used_lr = if superbatch_index == 0 && batch_index < WARMUP_BATCHES {
                        START_LR / (WARMUP_BATCHES - batch_index) as f32
                    } else {
                        learning_rate
                    };

                    update(
                        &mut policy,
                        &gradient,
                        adjustment,
                        used_lr,
                        &mut momentum,
                        &mut velocity,
                    );
                    batch_index += 1;
                    print!(
                        "> Superbatch {}/{} Batch {}/{}   Speed {:.0}                     \r",
                        superbatch_index + 1,
                        SUPERBATCHES_COUNT,
                        batch_index,
                        BATCHES_PER_SUPERBATCH,
                        (idx * BATCH_SIZE) as f32 / timer.elapsed().as_secs_f32()
                    );
                    let _ = std::io::stdout().flush();
                }

                if batch_index % BATCHES_PER_SUPERBATCH == 0 {
                    superbatch_index += 1;
                    batch_index = 0;
                    let superbatches_left = SUPERBATCHES_COUNT - superbatch_index;
                    let time_in_seconds = timer.elapsed().as_secs_f32()
                        * (BATCHES_PER_SUPERBATCH as f32 / BUFFER_SIZE as f32);
                    let time_left_in_seconds =
                        (superbatches_left as f32 * time_in_seconds).ceil() as usize;
                    let hh = time_left_in_seconds / 3600;
                    let mm = (time_left_in_seconds - hh * 3600) / 60;
                    let ss = time_left_in_seconds - hh * 3600 - mm * 60;

                    println!(
                        "> Superbatch {superbatch_index}/{} Running Loss {} Estimated training time: {}h {}m {}s        ",
                        SUPERBATCHES_COUNT,
                        running_error / (BATCHES_PER_SUPERBATCH * BATCH_SIZE) as f32,
                        hh, mm, ss
                    );

                    running_error = 0.0;

                    let training_percentage = superbatch_index as f32 / SUPERBATCHES_COUNT as f32;
                    let cosine_decay = 1.0 - (1.0 + (PI * training_percentage).cos()) / 2.0;
                    learning_rate = START_LR + (END_LR - START_LR) * cosine_decay;
                    println!("Dropping LR to {learning_rate}");

                    let mut export_path = PathBuf::new();
                    export_path.push(root_dictionary);
                    export_path.push("..");
                    export_path.push("policy_checkpoints");
                    export_path.push(format!("{}-{}.bin", NAME, superbatch_index));

                    policy.export(export_path.to_str().unwrap());

                    if superbatch_index == SUPERBATCHES_COUNT {
                        break 'training;
                    }
                }

                let len = buffer.len();
                training_data_reader.consume(len);
            }
        }
    }
}

fn update(
    policy: &mut TrainerPolicyNet,
    gradient: &TrainerPolicyNet,
    adjustment: f32,
    learning_rate: f32,
    momentum: &mut TrainerPolicyNet,
    velocity: &mut TrainerPolicyNet,
) {
    for (i, subnet) in policy.subnets.iter_mut().enumerate() {
        subnet.adam(
            &gradient.subnets[i],
            &mut momentum.subnets[i],
            &mut velocity.subnets[i],
            adjustment,
            learning_rate,
        );
    }
}

fn gradient_batch(
    policy: &TrainerPolicyNet,
    grad: &mut TrainerPolicyNet,
    batch: &[PolicyPacked],
) -> f32 {
    let size = (batch.len() / THREADS).max(1);
    let mut errors = vec![0.0; THREADS];

    std::thread::scope(|s| {
        batch
            .chunks(size)
            .zip(errors.iter_mut())
            .map(|(chunk, error)| {
                s.spawn(move || {
                    let mut inner_grad = boxed_and_zeroed();
                    for entry in chunk {
                        update_single_grad(entry, policy, &mut inner_grad, error);
                    }
                    inner_grad
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|p| p.join().unwrap())
            .for_each(|part| grad.add_without_explicit_lifetime(&part));
    });

    errors.iter().sum::<f32>()
}

fn update_single_grad(
    entry: &PolicyPacked,
    policy: &TrainerPolicyNet,
    grad: &mut TrainerPolicyNet,
    error: &mut f32,
) {
    let mut policies = Vec::with_capacity(entry.move_count() as usize);
    let board = ChessBoard::from_policy_pack(entry);
    let mut inputs = SparseVector::with_capacity(32);

    if board.side_to_move() == Side::WHITE {
        PolicyNetwork::map_policy_inputs::<_, true, false>(&board, |feat| inputs.push(feat));
    } else {
        PolicyNetwork::map_policy_inputs::<_, false, true>(&board, |feat| inputs.push(feat));
    }

    let vertical_flip = if entry.get_side_to_move() == Side::WHITE {
        0
    } else {
        56
    };

    let mut max = f32::NEG_INFINITY;
    let mut total = 0.0;
    let mut total_expected = 0;

    for move_data in &entry.moves()[..entry.move_count() as usize] {
        total_expected += move_data.visits;

        let see_index = if board.side_to_move() == Side::WHITE {
            usize::from(SEE::static_exchange_evaluation::<true, false>(
                &board,
                move_data.mv,
                -108,
            ))
        } else {
            usize::from(SEE::static_exchange_evaluation::<false, true>(
                &board,
                move_data.mv,
                -108,
            ))
        };

        let from_index = (move_data.mv.get_from_square().get_raw() ^ vertical_flip) as usize;
        let to_index = (move_data.mv.get_to_square().get_raw() ^ vertical_flip) as usize
            + 64
            + (see_index * 64);

        let from_out = policy.subnets[from_index].out_with_layers(&inputs);
        let to_out = policy.subnets[to_index].out_with_layers(&inputs);

        let policy_value = from_out.output_layer().dot(&to_out.output_layer());

        max = max.max(policy_value);
        policies.push((
            from_index,
            to_index,
            from_out,
            to_out,
            policy_value,
            move_data.visits as f32,
        ));
    }

    for (_, _, _, _, policy_value, expected_policy) in policies.iter_mut() {
        *policy_value = (*policy_value - max).exp();
        total += *policy_value;
        *expected_policy /= total_expected as f32;
    }

    for (from_index, to_index, from_out, to_out, policy_value, expected_value) in policies {
        let policy_value = policy_value / total;
        let error_factor = policy_value - expected_value;

        *error -= expected_value * policy_value.ln();

        policy.subnets[from_index].backprop(
            &inputs,
            &mut grad.subnets[from_index],
            error_factor * to_out.output_layer(),
            &from_out,
        );

        policy.subnets[to_index].backprop(
            &inputs,
            &mut grad.subnets[to_index],
            error_factor * from_out.output_layer(),
            &to_out,
        );
    }
}

#[repr(C)]
#[derive(Clone, Copy, FeedForwardNetwork)]
struct TrainerPolicySubnet {
    l0: SparseConnected<activation::ReLU, 768, 32>,
    l1: DenseConnected<activation::ReLU, 32, 32>,
}

impl TrainerPolicySubnet {
    pub const fn zeroed() -> Self {
        Self {
            l0: SparseConnected::zeroed(),
            l1: DenseConnected::zeroed(),
        }
    }

    pub fn from_fn<F: FnMut() -> f32>(mut f: F) -> Self {
        let weights = Matrix::from_fn(|_, _| f());
        let biases = Vector::from_fn(|_| f());

        let weights1 = Matrix::from_fn(|_, _| f());
        let biases1 = Vector::from_fn(|_| f());

        Self {
            l0: SparseConnected::from_raw(weights, biases),
            l1: DenseConnected::from_raw(weights1, biases1),
        }
    }
}

struct TrainerPolicyNet {
    pub subnets: [TrainerPolicySubnet; 192],
}

#[allow(unused)]
impl TrainerPolicyNet {
    pub const fn zeroed() -> Self {
        Self {
            subnets: [TrainerPolicySubnet::zeroed(); 192],
        }
    }

    fn add_without_explicit_lifetime(&mut self, rhs: &TrainerPolicyNet) {
        for (i, j) in self.subnets.iter_mut().zip(rhs.subnets.iter()) {
            *i += j;
        }
    }

    fn export(&self, path: &str) {
        let size = std::mem::size_of::<TrainerPolicyNet>();
        let mut file = std::fs::File::create(path).unwrap();

        unsafe {
            let slice: *const u8 = std::slice::from_ref(self).as_ptr().cast();
            let struct_bytes: &[u8] = std::slice::from_raw_parts(slice, size);
            file.write_all(struct_bytes).expect("Failed to write data!");
        }
    }

    fn rand_init() -> Box<TrainerPolicyNet> {
        let mut policy = boxed_and_zeroed::<TrainerPolicyNet>();

        let mut rng = rand::thread_rng();
        for subnet in policy.subnets.iter_mut() {
            *subnet = TrainerPolicySubnet::from_fn(|| {
                (rng.gen_range(0..u32::MAX) as f32 / u32::MAX as f32) * 0.2
            });
        }

        policy
    }
}

fn boxed_and_zeroed<T>() -> Box<T> {
    unsafe {
        let layout = std::alloc::Layout::new::<T>();
        let ptr = std::alloc::alloc_zeroed(layout);
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        Box::from_raw(ptr.cast())
    }
}
