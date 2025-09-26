use std::io::stdin;

pub struct InputWrapper {
    command_queue: Vec<String>,
}

impl InputWrapper {
    pub fn new() -> Self {
        Self {
            command_queue: Vec::new(),
        }
    }

    pub fn get_input(&mut self) -> String {
        if self.command_queue.len() > 0 {
            let command = self.command_queue.first().unwrap().clone();
            self.command_queue.remove(0);
            return command;
        }

        let mut input_command = String::new();

        if stdin().read_line(&mut input_command).is_err() {
            println!("Error reading input, please try again.");
        }

        input_command.trim().to_string()
    }

    pub fn get_input_no_queue(&mut self) -> String {
        let mut input_command = String::new();

        if stdin().read_line(&mut input_command).is_err() {
            println!("Error reading input, please try again.");
        }

        input_command.trim().to_string()
    }

    pub fn push_back(&mut self, command: String) {
        self.command_queue.push(command);
    }
}
