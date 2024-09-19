pub struct ParamsProcessor;
impl ParamsProcessor {
    //Processes arguments passed when executing the engine and returns true,
    //if engine should be terminated after execution of those params
    #[allow(unused_assignments)]
    pub fn execute(args: Vec<String>) {
        for (_, arg) in args.clone().into_iter().enumerate() {
            match arg.as_str() {
                _ => continue
            }
        }
    }
}
