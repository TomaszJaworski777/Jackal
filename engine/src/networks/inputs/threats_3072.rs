use chess::{Piece, Side};

pub struct Threats3072;
#[allow(unused)]
impl Threats3072 {
    pub const fn input_size() -> usize {
        3072
    }

    pub fn map_inputs<F: FnMut(usize)>(board: &chess::ChessBoard, mut process_input: F) {
        let horizontal_mirror = if board.king_square(board.side()).file() > 3 {
            7
        } else {
            0
        };

        let flip = board.side() == Side::BLACK;

        let mut threats = board.generate_attack_map(board.side().flipped());
        let mut defences = board.generate_attack_map(board.side());

        if flip {
            threats = threats.flip();
            defences = defences.flip();
        }

        let occupacy = board.occupancy();
        occupacy.map(|square| {
            let piece = board.piece_on_square(square);
            let color = board.color_on_square(square);
            let square = square ^ if flip { 56 } else { 0 };

            let piece_index = 64 * (u8::from(piece) - u8::from(Piece::PAWN)) as usize;
            let mut feat = [384, 0][usize::from(color == board.side())] + piece_index + (usize::from(square) ^ horizontal_mirror);

            if threats.get_bit(square) {
                feat += 768;
            }

            if defences.get_bit(square) {
                feat += 768 * 2;
            }

            process_input(feat)
        });
    }
}