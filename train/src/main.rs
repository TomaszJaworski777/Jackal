use std::env;

use bullet_convert::{BulletConverter, DataConvertionMode};
use value::ValueTrainer;

mod bullet_convert;
mod bullet_convert_display;
mod value;

fn main() {
    let args: Vec<String> = env::args().collect();
    for arg in &args {
        match arg.as_str() {
            "convert" => convert(&args),
            "value" => ValueTrainer::execute(),
            _ => continue,
        }
    }
}

fn convert(args: &Vec<String>) {
    let mut mode = DataConvertionMode::Full;
    let mut input_path = "./value_data.bin";
    let mut output_path = "./bullet_data.bin";

    let mut cmd = String::new();
    for arg in args {
        match arg.as_str() {
            "full" => mode = DataConvertionMode::Full,
            "nodraws" => mode = DataConvertionMode::NoDraws,
            "input" | "output" => cmd = arg.clone(),
            _ => {
                match cmd.as_str() {
                    "input" => input_path = arg.as_str(),
                    "output" => output_path = arg.as_str(),
                    _ => continue,
                };
            }
        }
    }

    BulletConverter::convert(input_path, output_path, mode);
}
