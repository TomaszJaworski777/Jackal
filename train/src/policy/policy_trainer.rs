use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf, time::Instant,
};

use goober::{
    activation, layer::SparseConnected, FeedForwardNetwork, Matrix, OutputLayer, SparseVector,
    Vector,
};
use rand::{seq::SliceRandom, Rng};
use spear::{Bitboard, ChessBoard, Move, PolicyPacked, Side, Square};

const NAME: &'static str = "policy_001";

const THREADS: usize = 4;
const SUPERBATCHES_COUNT: usize = 60;
const START_LR: f32 = 0.001;
const LR_DROP: usize = 25;

const BATCH_SIZE: usize = 16_384;
const BATCHES_PER_SUPERBATCH: usize = 1024;
const TRAINING_DATA_PATH: &'static str = "conv_policy_data.bin";

pub struct PolicyTrainer;
impl PolicyTrainer {
    pub fn execute() {
        let root_dictionary = env!("CARGO_MANIFEST_DIR");
        let mut training_data_path = PathBuf::new();
        training_data_path.push(root_dictionary);
        training_data_path.push("..");
        training_data_path.push(TRAINING_DATA_PATH);
        let training_data_path = training_data_path.to_str().unwrap();
        let training_data = File::open(training_data_path).expect("Cannot open training data file");
        let training_metadata = training_data
            .metadata()
            .expect("Cannot obtain training metadata");

        let entry_count = training_metadata.len() as usize / std::mem::size_of::<PolicyPacked>();

        let mut policy = TrainerPolicyNet::rand_init();
        let throughput = SUPERBATCHES_COUNT * BATCHES_PER_SUPERBATCH * BATCH_SIZE;

        println!("Network Name: {}", NAME);
        println!("Thread Count: {}", THREADS);
        println!("Loaded Positions: {}", entry_count);
        println!("Superbatches: {}", SUPERBATCHES_COUNT);
        println!("LR Drop: {}", LR_DROP);
        println!("Start LR: {}", START_LR);
        println!("Epochs {:.2}\n", throughput as f64 / entry_count as f64);

        let mut momentum = boxed_and_zeroed::<TrainerPolicyNet>();
        let mut velocity = boxed_and_zeroed::<TrainerPolicyNet>();

        let mut running_error = 0.0;
        let mut learning_rate = START_LR;

        let mut superbatch_index = 0;
        let mut batch_index = 0;

        const BUFFER_SIZE: usize = 512;
        'training: loop {
            let training_data = File::open(training_data_path).expect("Cannot open training data file");
            let mut training_data_reader = BufReader::with_capacity(BUFFER_SIZE * BATCH_SIZE * std::mem::size_of::<PolicyPacked>(), training_data);
            while let Ok(buffer) = training_data_reader.fill_buf() {
                if buffer.is_empty() {
                    break;
                }
    
                let mut superbatch: Vec<PolicyPacked> = unsafe {
                    std::slice::from_raw_parts(
                        buffer.as_ptr().cast(),
                        BUFFER_SIZE * BATCH_SIZE,
                    )
                }.to_vec();
    
                let mut rng = rand::thread_rng();
                superbatch.shuffle(&mut rng);
    
                let timer = Instant::now();
    
                for (idx, batch) in superbatch.chunks(BATCH_SIZE).enumerate() {
                    let mut gradient = boxed_and_zeroed::<TrainerPolicyNet>();
                    running_error += gradient_batch(&policy, &mut gradient, batch);
                    let adjustment = 1.0 / batch.len() as f32;
                    update(&mut policy, &gradient, adjustment, learning_rate, &mut momentum, &mut velocity);
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
                    let time_in_seconds = timer.elapsed().as_secs_f32() * (BATCHES_PER_SUPERBATCH as f32 / BUFFER_SIZE as f32);
                    let time_left_in_seconds = (superbatches_left as f32 * time_in_seconds).ceil() as usize;
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
    
                    if superbatch_index % LR_DROP == 0 {
                        learning_rate *= 0.1;
                        println!("Dropping LR to {learning_rate}");
                    } 
    
                    if superbatch_index == SUPERBATCHES_COUNT {
                        break 'training;
                    }
    
                    let mut export_path = PathBuf::new();
                    export_path.push(root_dictionary);
                    export_path.push("..");
                    export_path.push("policy_checkpoints");
                    export_path.push(format!("{}-{}.bin", NAME, superbatch_index));
    
                    policy.export(export_path.to_str().unwrap());
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
    let inputs = extract_inputs(convert_to_12_bitboards(entry.get_board()));
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
        
        let from_index = (move_data.mv.get_from_square().get_raw() ^ vertical_flip) as usize;
        let to_index = (move_data.mv.get_to_square().get_raw() ^ vertical_flip) as usize;

        let from_out = policy.subnets[from_index].out_with_layers(&inputs);
        let to_out = policy.subnets[64 + to_index].out_with_layers(&inputs);
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

        policy.subnets[64 + to_index].backprop(
            &inputs,
            &mut grad.subnets[64 + to_index],
            error_factor * from_out.output_layer(),
            &to_out,
        );
    }
}

#[repr(C)]
#[derive(Clone, Copy, FeedForwardNetwork)]
struct TrainerPolicySubnet {
    l0: SparseConnected<activation::ReLU, 768, 16>,
}

impl TrainerPolicySubnet {
    pub const fn zeroed() -> Self {
        Self {
            l0: SparseConnected::zeroed(),
        }
    }

    pub fn from_fn<F: FnMut() -> f32>(mut f: F) -> Self {
        let weights = Matrix::from_fn(|_, _| f());
        let biases = Vector::from_fn(|_| f());

        Self {
            l0: SparseConnected::from_raw(weights, biases),
        }
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

    fn evaluate(&self, board: &ChessBoard, mv: &Move, inputs: &SparseVector) -> f32 {
        let flip = if board.side_to_move() == Side::WHITE {
            0
        } else {
            56
        };

        let from_subnet = &self.subnets[usize::from(mv.get_from_square().get_raw() ^ flip)];
        let from_vec = from_subnet.out(inputs);

        let to_subnet = &self.subnets[64 + usize::from(mv.get_to_square().get_raw() ^ flip)];
        let to_vec = to_subnet.out(inputs);

        from_vec.dot(&to_vec)
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

fn convert_to_12_bitboards(board: &[Bitboard; 4]) -> [Bitboard; 12] {
    let mut result = [Bitboard::EMPTY; 12];
    for square_index in 0..64 {
        let square = Square::from_raw(square_index);
        let piece_index: usize = (if board[0].get_bit(square) { 1 } else { 0 }
            | if board[1].get_bit(square) { 2 } else { 0 }
            | if board[2].get_bit(square) { 4 } else { 0 })
            + if board[3].get_bit(square) { 6 } else { 0 };
        if piece_index == 7 || piece_index == 13 {
            continue;
        }

        if piece_index == 12 {
            board[0].draw_bitboard();
            board[1].draw_bitboard();
            board[2].draw_bitboard();
            board[3].draw_bitboard();
            println!("{square}");
        }

        result[piece_index].set_bit(square);
    }
    result
}

fn extract_inputs(board: [Bitboard; 12]) -> SparseVector {
    let mut result = SparseVector::with_capacity(32);
    for piece_index in 0..12 {
        for square in board[piece_index] {
            result.push(piece_index * 64 + square.get_raw() as usize)
        }
    }
    result
}