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
fn has_legal_moves_matches_movegen() {
    fn check(board: &ChessBoard, depth: u8) {
        let mut moves = Vec::new();
        board.map_legal_moves(|mv| moves.push(mv));

        assert_eq!(board.has_legal_moves(), !moves.is_empty());

        if depth == 0 {
            return;
        }

        for mv in moves {
            let mut next = *board;
            next.make_move_no_mask(mv);
            check(&next, depth - 1);
        }
    }

    let file = File::open("./tests/standard.epd").unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let fen = FEN::from(line.split(';').next().unwrap());
        check(&ChessBoard::from(&fen), 2);
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
