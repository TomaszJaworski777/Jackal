use std::env;

use bullet_convert::BulletConverter;
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
    let mut input_path = "./value_data.bin";
    let mut output_path = "./bullet_data.bin";

    let mut cmd = String::new();
    for arg in args {
        match arg.as_str() {
            "-i" | "-o" => cmd = arg.clone(),
            _ => {
                match cmd.as_str() {
                    "-i" => input_path = arg.as_str(),
                    "-o" => output_path = arg.as_str(),
                    _ => continue,
                };
            }
        }
    }

    BulletConverter::convert(input_path, output_path);
}
