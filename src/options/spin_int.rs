use super::OptionTrait;

pub struct SpinOptionInt {
    value: i32,
    default: i32,
    min: i32,
    max: i32,
}

impl SpinOptionInt {
    pub fn new(value: i32, min: i32, max: i32) -> Self {
        Self {
            value,
            default: value,
            min,
            max,
        }
    }

    pub fn set_value(&mut self, new_value: i32) {
        if new_value >= self.min && new_value <= self.max {
            self.value = new_value;
        } else {
            println!("Value out of range.");
        }
    }

    pub fn get(&self) -> i32 {
        self.value
    }
}

impl OptionTrait for SpinOptionInt {
    type ValueType = i32;

    fn set(&mut self, new_value: &str) {
        if let Ok(parsed_value) = new_value.parse::<i32>() {
            self.set_value(parsed_value);
        } else {
            println!("Invalid value for option.");
        }
    }

    fn get(&self) -> i32 {
        self.get()
    }

    fn print(&self, name: &str) {
        println!(
            "option name {} type spin default {:?} min {:?} max {:?}",
            name, self.default, self.min, self.max
        );
    }
}
