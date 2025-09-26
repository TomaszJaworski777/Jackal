use chess::FEN;

#[test]
fn haha() {
    let fen = FEN::from("rbbqn1kr/pp2p1pp/6n1/2pp1p2/2P4P/P7/BP1PPPP1/R1BQNNKR w HAha - 0 9");
    assert_eq!(fen.castle_rights, "HAha");

    let fen = FEN::from("rbbqn1kr/pp2p1pp/6n1/2pp1p2/2P4P/P7/BP1PPPP1/R1BQNNKR w KQkq - 0 9");
    assert_eq!(fen.castle_rights, "HAha");

    let fen = FEN::from("rrrkrrrr/pp2p1pp/8/2pp1p2/2P4P/P7/BP1PPPP1/R1BRNKRR w KQkq - 0 9");
    assert_eq!(fen.castle_rights, "HAha");

    let fen = FEN::from("rrrkrrr1/pp2p1pp/8/2pp1p2/2P4P/P7/BP1PPPP1/R1BRNKRR w KQq - 0 9");
    assert_eq!(fen.castle_rights, "HAa");
}

#[test]
fn hegb() {
    let fen = FEN::from("brnr1krr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BRRKR w KQgq - 2 9");
    assert_eq!(fen.castle_rights, "HEgb");
}

#[test]
fn hfhb() {
    let fen = FEN::from("brnr1krr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BRRKR w KFkq - 2 9");
    assert_eq!(fen.castle_rights, "HFhb");
}

#[test]
fn hchb() {
    let fen = FEN::from("brkr2rr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQRBKR1R w KQkq - 2 9");
    assert_eq!(fen.castle_rights, "HChb");
}

#[test]
fn fbda() {
    let fen = FEN::from("rrkrrrrr/pp3ppp/3pp3/2p5/5P2/P2P4/1PP1P1PP/RRRRKRRR w FBdq - 2 9");
    assert_eq!(fen.castle_rights, "FBda");
}
