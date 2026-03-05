use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use chess::{perft, ChessBoard, FEN};

#[test]
fn standard() {
    let file = File::open("./tests/standard.epd").unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let line = line.split(';').collect::<Vec<&str>>();
        let fen = FEN::from(line[0]);
        let target = line[line.len() - 2]
            .split_whitespace()
            .collect::<Vec<&str>>();
        let expected_result = target[1].parse::<u128>().unwrap();
        let depth = target[0].chars().collect::<Vec<char>>()[1] as u8 - b'0';
        println!("{fen}");
        let (result, _) = perft::<true, false, false>(&ChessBoard::from(&fen), Some(depth));
        assert_eq!(result, expected_result);
    }
}

#[test]
fn frc() {
    let file = File::open("./tests/fischer.epd").unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let line = line.split(';').collect::<Vec<&str>>();
        let fen = FEN::from(line[0]);
        let target = line[line.len() - 3]
            .split_whitespace()
            .collect::<Vec<&str>>();
        let expected_result = target[1].parse::<u128>().unwrap();
        let depth = target[0].chars().collect::<Vec<char>>()[1] as u8 - b'0';
        println!("{fen}");
        let (result, _) = perft::<true, false, true>(&ChessBoard::from(&fen), Some(depth));
        assert_eq!(result, expected_result);
    }
}
