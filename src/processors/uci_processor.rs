use crate::{options::EngineOptions, search::SearchEngine};

pub struct UciProcessor;
impl UciProcessor {

    //Handles commands from UCI protocol
    pub fn execute( command: &str, args: &[String], search_engine: &mut SearchEngine ) {
        match command {
            "uci" => Self::uci_message(search_engine), 
            "isready" => println!("readyok"),
            "setoption" => Self::set_option(args, search_engine.engine_options_mut()),
            "position" => Self::position(args, search_engine),
            "ucinewgame" => search_engine.reset(),
            "go" => Self::go(args, search_engine),
            "stop" => search_engine.stop(),
            _ => return
        }
    }

    fn uci_message( search_engine: &mut SearchEngine ) {
        println!("id name Jackal v{}", env!("CARGO_PKG_VERSION"));
        println!("id author Tomasz Jaworski");
        search_engine.engine_options_mut().print();
        println!("uciok");
    }

    fn set_option( args: &[String], options: &mut EngineOptions ) {

    }

    fn position( args: &[String], search_engine: &mut SearchEngine ) {

    }

    fn go( args: &[String], search_engine: &mut SearchEngine ) {
        search_engine.search(true)
    }
}