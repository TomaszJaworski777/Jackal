pub struct ParamsProcessor;
impl ParamsProcessor {
    //Processes arguments passed when executing the engine and returns true,
    //if engine should be terminated after execution of those params
    #[allow(unused_assignments)]
    pub fn execute(args: Vec<String>) -> bool {
        for arg in args {
            let mut cmd = "";
            match arg.as_str() {
                "bench" => cmd = "bench",
                _ => match cmd {
                    "bench" => Self::bench(arg.parse::<u32>().unwrap_or(5)),
                    _ => continue,
                },
            }
        }
        return false;
    }

    fn bench(depth: u32) {
        println!("{depth}")
    }
}
