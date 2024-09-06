use crate::utils::clear_terminal_screen;

pub struct MiscCommandsProcessor;
impl MiscCommandsProcessor {
        
    //Handles commands that are not UCI related and returns true, if engine should be terminated
    pub fn execute( command: &str, args: &[String] ) -> bool {
        match command {
            "exit" | "quit" => return true,
            "clean" | "clear" | "cls" | "cln" => clear_terminal_screen(),
            _ => return false
        }

        false
    }
}