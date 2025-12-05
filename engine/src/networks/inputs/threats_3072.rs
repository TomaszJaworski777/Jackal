use chess::{Piece, Side};

pub struct Threats3072;
#[allow(unused)]
impl Threats3072 {
    pub const fn input_size() -> usize {
        768 * 4
    }

    pub fn map_inputs<F: FnMut(usize)>(board: &chess::ChessBoard, mut process_input: F) {
        let flip = board.side() == Side::BLACK;
        let horizontal_mirror = if board.king_square(board.side()).file() > 3 {
            7
        } else {
            0
        };

        let mut threats = board.generate_attack_map(board.side().flipped());
        let mut defences = board.generate_attack_map(board.side());

        if flip {
            threats.flip_mut();
            defences.flip_mut();
        }

        for piece in (0..6u8).map(|x| Piece::from(x)) {
            let piece_idx = 64 * (u8::from(piece) - u8::from(Piece::PAWN)) as usize;

            let mut stm_bb = board.piece_mask_for_side(piece, board.side());
            let mut nstm_bb = board.piece_mask_for_side(piece, board.side().flipped());

            if flip {
                stm_bb.flip_mut();
                nstm_bb.flip_mut();
            }

            stm_bb.map(|square| {
                let mut feat = piece_idx + (usize::from(square) ^ horizontal_mirror);

                if threats.get_bit(square) {
                    feat += 768;
                }

                if defences.get_bit(square) {
                    feat += 768 * 2;
                }

                process_input(feat)
            });

            nstm_bb.map(|square| {
                let mut feat = 384 + piece_idx + (usize::from(square) ^ horizontal_mirror);

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
}