use super::OptionTrait;

pub struct CheckBool {
    value: bool,
    default: bool,
}

impl CheckBool {
    pub fn new(value: bool) -> Self {
        Self {
            value,
            default: value
        }
    }

    pub fn set_value(&mut self, new_value: bool) {
        self.value = new_value;
    }

    pub fn get(&self) -> bool {
        self.value
    }
}

impl OptionTrait for CheckBool {
    type ValueType = bool;

    fn set(&mut self, new_value: &str) {
        if let Ok(parsed_value) = new_value.parse::<bool>() {
            self.set_value(parsed_value);
        } else {
            println!("Invalid value for option.");
        }
    }

    fn get(&self) -> bool {
        self.get()
    }

    fn print(&self, name: &str) {
        println!(
            "option name {} type check default {:?}",
            name, self.default
        );
    }
}