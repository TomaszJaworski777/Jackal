use crate::datagen::{DataGen, DataGenMode};

pub struct ParamsProcessor;
impl ParamsProcessor {
    //Processes arguments passed when executing the engine and returns true,
    //if engine should be terminated after execution of those params
    #[allow(unused_assignments)]
    pub fn execute(args: Vec<String>) {
        for (index, arg) in args.clone().into_iter().enumerate() {
            match arg.as_str() {
                "gen" => Self::data_gen(args[index..].to_vec()), 
                _ => continue
            }
        }
    }

    pub fn data_gen(args: Vec<String>) {
        let mut mode = DataGenMode::Value;
        let mut threads = 1;
        let mut iter_count = 1000;
        let mut target = 1000;
        let mut path = "./data.bin";

        let mut cmd = String::new();
        for arg in &args {
            match arg.as_str() {
                "policy" => mode = DataGenMode::Policy,
                "value" => mode = DataGenMode::Value,
                "threads" | "nodes" | "path" | "target" => cmd = arg.clone(),
                _ => {
                    match cmd.as_str() {
                        "threads" => threads = arg.parse::<u8>().unwrap_or(1),
                        "nodes" => iter_count = arg.parse::<u32>().unwrap_or(1000),
                        "path" => path = arg.as_str(),
                        "target" => target = arg.parse::<u64>().unwrap_or(1000),
                        _ => continue
                    };
                }
            }
        }

        DataGen::start_gen(threads, iter_count, path, target, mode);
        std::process::exit(0);
    }
}
