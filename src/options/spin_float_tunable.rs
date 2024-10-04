use super::{OptionTrait, Tunable};

#[allow(unused)]
pub struct SpinOptionFloatTunable {
    value: f32,
    default: f32,
    min: f32,
    max: f32,
    step: f32,
    r: f32,
}

#[allow(unused)]
impl SpinOptionFloatTunable {
    pub fn new(value: f32, min: f32, max: f32, step: f32, r: f32) -> Self {
        Self {
            value,
            default: value,
            min,
            max,
            step,
            r,
        }
    }

    pub fn set_value(&mut self, new_value: i32) {
        let adjusted = new_value as f32 / 100.0;
        if adjusted >= self.min && adjusted <= self.max {
            self.value = adjusted;
        } else {
            println!("Value out of range.");
        }
    }

    pub fn get(&self) -> f32 {
        self.value
    }
}

impl OptionTrait for SpinOptionFloatTunable {
    type ValueType = f32;

    fn set(&mut self, new_value: &str) {
        if let Ok(parsed_value) = new_value.parse::<i32>() {
            self.set_value(parsed_value);
        } else {
            println!("Invalid value for option.");
        }
    }

    fn get(&self) -> f32 {
        self.get()
    }

    fn print(&self, name: &str) {
        println!(
            "option name {} type spin default {:?} min {:?} max {:?}",
            name,
            (self.default * 100.0) as i32,
            (self.min * 100.0) as i32,
            (self.max * 100.0) as i32
        );
    }
}

impl Tunable for SpinOptionFloatTunable {}
