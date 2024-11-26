use std::{fs::File, io::{BufRead, BufReader}};

use spear::PolicyPacked;

const BUFFER_SIZE: usize = 50 * 1024 * 1024 * 1024;

pub struct PolicyShuffle;
impl PolicyShuffle {
    pub fn execute(source_path: &str, output_path: &str) {

        //Open dataset
        let source_file = File::open(source_path).expect("Cannot open source file!");
        let source_metadata = source_file.metadata().expect("Cannot obtain source metadata!");
        let mut source_reader = BufReader::with_capacity(BUFFER_SIZE, source_file);

        println!("start - {}", source_metadata.len());

        while let Ok(buffer) = source_reader.fill_buf() {
            if buffer.is_empty() {
                break;
            }

            println!("x");

            let len = buffer.len();
            source_reader.consume(len);
        }
    }
}