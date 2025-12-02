use chess::{Piece, Side};

pub struct Threats3072;
#[allow(unused)]
impl Threats3072 {
    pub const fn input_size() -> usize {
        768 * 4 * 3
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

        let (mut diag_stm, mut ortho_stm) = board.generate_pin_masks(board.side());
        let (mut diag_nstm, mut ortho_nstm) = board.generate_pin_masks(board.side().flipped());

        if flip {
            threats.flip_mut();
            defences.flip_mut();
        }

        // threats.draw_bitboard();
        // defences.draw_bitboard();

        let occupacy = board.occupancy();
        for side in if flip { [Side::BLACK, Side::WHITE] } else { [Side::WHITE, Side::BLACK] } {
            for piece_idx in 0..6 {
                let piece = Piece::from(piece_idx);
                let piece_index = 64 * (piece_idx - u8::from(Piece::PAWN)) as usize;
                let mask = board.piece_mask_for_side(piece, side); 
                
                mask.map(|square| {
                    let mut feat = [384, 0][usize::from(side == board.side())] + piece_index + (usize::from(square) ^ horizontal_mirror);

                    if threats.get_bit(square) {
                        //println!("Piece on {square} is attacked!");
                        feat += 768;
                    }

                    if defences.get_bit(square) {
                        //println!("Piece on {square} is defended!");
                        feat += 768 * 2;
                    }

                    if diag_stm.get_bit(square) {
                        //println!("{piece} pinned diagonally!");
                        feat += 768 * 4;
                    }

                    if ortho_stm.get_bit(square) {
                        feat += 768 * 4 * 2;
                    }
                });
            } 

            (diag_stm, ortho_stm, diag_nstm, ortho_nstm) = (diag_nstm, ortho_nstm, diag_stm, ortho_stm);
        }

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