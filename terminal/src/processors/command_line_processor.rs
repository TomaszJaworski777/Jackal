use engine::SearchEngine;

#[allow(clippy::ptr_arg)]
pub fn process_command_line_args(args: &Vec<String>, search_engine: &mut SearchEngine) -> bool {
    let mut commmand_processed = false;

    for (idx, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "bench" => {
                let depth = if idx >= args.len() - 1 {
                    None
                } else {
                    args[idx + 1].parse::<u64>().ok()
                };

                let (result, duration) = search_engine.bench(depth);
                let nps = result as f64 / duration.as_secs_f64();
                println!("Bench: {result} nodes {:.0} nps", nps);
                commmand_processed = true;
            }
            _ => continue,
        }
    }

    commmand_processed
}
