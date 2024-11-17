use crate::{bench::Bench, SearchEngine};

pub struct ParamsProcessor;
impl ParamsProcessor {
    #[allow(unused_assignments)]
    pub fn execute(args: Vec<String>, search_engine: &mut SearchEngine) {
        let mut bench = false;
        let mut bench_depth = 0;
        let mut options = false;
        let mut option_name = String::new();

        for arg in args {
            match arg.as_str() {
                "--options" => options = true,
                "bench" => {
                    bench_depth = 5;
                    options = false;
                    bench = true;
                }
                _ => {
                    if bench {
                        bench_depth = arg.parse::<u32>().unwrap_or(5);
                        bench = false;
                    }

                    if options {
                        if option_name.is_empty() {
                            option_name = arg;
                            continue;
                        }

                        search_engine.engine_options_mut().set(&option_name, &arg);

                        if option_name == "Hash" {
                            let hash_size = search_engine.engine_options().hash();
                            let hash_percentage = search_engine.engine_options().hash_percentage();
                            search_engine.tree_mut().resize_tree(hash_size, hash_percentage)
                        }

                        option_name.clear();
                    }
                }
            }
        }

        if bench_depth > 0 {
            Self::bench(bench_depth, search_engine);
        }
    }

    fn bench(depth: u32, search_engine: &SearchEngine) {
        Bench::run(depth, search_engine);
        std::process::exit(0)
    }
}
