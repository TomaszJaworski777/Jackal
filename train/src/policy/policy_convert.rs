use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read, Write},
    time::Instant,
};

use jackal::PolicyPacked;

use super::PolicyConvertDisplay;

pub struct PolicyConvert;
impl PolicyConvert {
    pub fn convert(input_path: &str, output_path: &str) {
        let input_file = File::open(input_path).expect("Cannot open input file");
        let input_meta = input_file.metadata().expect("Cannot obtain file metadata");
        let mut reader = BufReader::new(input_file);

        let mut output_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(output_path)
            .expect("Cannot open output file");

        let entry_size = std::mem::size_of::<PolicyPacked>();
        let policy_packed_size = std::mem::size_of::<PolicyPacked>();
        let entry_count = input_meta.len() / entry_size as u64;
        let mut buffer = vec![0u8; entry_size];

        let mut timer = Instant::now();
        let mut entries_processed = 0;
        let mut unfiltered = 0;

        while reader.read_exact(&mut buffer).is_ok() {
            if timer.elapsed().as_secs_f32() > 1.0 {
                PolicyConvertDisplay::print_report(entries_processed, entry_count, unfiltered);
                timer = Instant::now();
            }

            let position: PolicyPacked = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };
            entries_processed += 1;

            let chess_board_bytes = unsafe {
                std::slice::from_raw_parts(
                    (&position as *const PolicyPacked) as *const u8,
                    policy_packed_size,
                )
            };

            output_file
                .write_all(&chess_board_bytes)
                .expect("Couldnt write to output file");
            unfiltered += 1;
        }
    }
}
