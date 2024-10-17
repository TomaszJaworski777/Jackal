use std::env;

use policy::PolicyConvert;
use policy::PolicyTrainer;
use value::ValueConverter;
use value::ValueTrainer;

mod policy;
mod value;

fn main() {
    let args: Vec<String> = env::args().collect();
    for arg in &args {
        match arg.as_str() {
            "value-conv" => value_convert(&args),
            "policy-conv" => policy_convert(&args),
            "value" => ValueTrainer::execute(),
            "policy" => PolicyTrainer::execute(),
            _ => continue,
        }
    }
}

fn value_convert(args: &Vec<String>) {
    let mut input_path = "./value_data.bin";
    let mut output_path = "./conv_value_data.bin";

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

    ValueConverter::convert(input_path, output_path);
}

fn policy_convert(args: &Vec<String>) {
    let mut input_path = "./policy_data.bin";
    let mut output_path = "./conv_policy_data.bin";

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

    PolicyConvert::convert(input_path, output_path);
}
