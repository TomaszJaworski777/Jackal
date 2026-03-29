use std::io::stdin;

pub struct InputWrapper {
    command_queue: Vec<String>,
}

impl Default for InputWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl InputWrapper {
    pub fn new() -> Self {
        Self {
            command_queue: Vec::new(),
        }
    }

    pub fn get_input(&mut self) -> Option<String> {
        if !self.command_queue.is_empty() {
            let command = self.command_queue.first().unwrap().clone();
            self.command_queue.remove(0);
            return Some(command);
        }

        let mut input_command = String::new();

        match stdin().read_line(&mut input_command) {
            Ok(0) | Err(_) => return None,
            _ => {}
        }

        Some(input_command.trim().to_string())
    }

    pub fn get_input_no_queue(&mut self) -> Option<String> {
        let mut input_command = String::new();

        match stdin().read_line(&mut input_command) {
            Ok(0) | Err(_) => return None,
            _ => {}
        }

        Some(input_command.trim().to_string())
    }

    pub fn push_back(&mut self, command: String) {
        self.command_queue.push(command);
    }
}
