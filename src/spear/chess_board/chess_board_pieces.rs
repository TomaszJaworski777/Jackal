use crate::spear::{base_structures::Side, Bitboard, ChessBoard, Piece, Square};

#[derive(Clone, Copy, Default, PartialEq, Hash)]
pub struct ChessBoardPieces {
    occupancy: [Bitboard; 2],
    pieces: [Bitboard; 6],
}
impl ChessBoardPieces {
    #[inline]
    pub fn get_occupancy_for_side<const WHITE: bool>(&self) -> Bitboard {
        self.occupancy[usize::from(!WHITE)]
    }

    #[inline]
    pub fn get_piece_mask(&self, piece: Piece) -> Bitboard {
        self.pieces[piece.get_raw() as usize]
    }

    pub fn get_piece_on_square(&self, square: Square) -> Piece {
        for piece_index in Piece::PAWN.get_raw()..=Piece::KING.get_raw() {
            if self.pieces[usize::from(piece_index)].get_bit(square) {
                return Piece::from_raw(piece_index);
            }
        }

        Piece::NONE
    }

    #[inline]
    pub fn get_piece_color_on_square(&self, square: Square) -> Side {
        if self.get_occupancy_for_side::<true>().get_bit(square) {
            Side::WHITE
        } else {
            Side::BLACK
        }
    }

    #[inline]
    pub fn set_piece_on_square<const WHITE: bool>(&mut self, square: Square, piece: Piece) {
        self.pieces[piece.get_raw() as usize].set_bit(square);
        self.occupancy[usize::from(!WHITE)].set_bit(square);
    }

    #[inline]
    pub fn remove_piece_on_square<const WHITE: bool>(&mut self, square: Square, piece: Piece) {
        self.pieces[piece.get_raw() as usize].pop_bit(square);
        self.occupancy[usize::from(!WHITE)].pop_bit(square);
    }
}

impl ChessBoard {
    #[inline]
    pub fn get_occupancy(&self) -> Bitboard {
        self.get_occupancy_for_side::<true>() | self.get_occupancy_for_side::<false>()
    }

    #[inline]
    pub fn get_occupancy_for_side<const WHITE: bool>(&self) -> Bitboard {
        self.pieces.get_occupancy_for_side::<WHITE>()
    }

    #[inline]
    pub fn get_piece_mask_for_side<const WHITE: bool>(&self, piece: Piece) -> Bitboard {
        self.get_piece_mask(piece) & self.get_occupancy_for_side::<WHITE>()
    }

    #[inline]
    pub fn get_piece_mask(&self, piece: Piece) -> Bitboard {
        self.pieces.get_piece_mask(piece)
    }

    #[inline]
    pub fn get_king_square<const WHITE: bool>(&self) -> Square {
        self.get_piece_mask_for_side::<WHITE>(Piece::KING)
            .ls1b_square()
    }

    #[inline]
    pub fn get_piece_on_square(&self, square: Square) -> Piece {
        self.pieces.get_piece_on_square(square)
    }

    #[inline]
    pub fn get_piece_color_on_square(&self, square: Square) -> Side {
        self.pieces.get_piece_color_on_square(square)
    }

    #[inline]
    pub fn set_piece_on_square<const WHITE: bool>(&mut self, square: Square, piece: Piece) {
        self.pieces.set_piece_on_square::<WHITE>(square, piece);
        self.state
            .get_key_mut()
            .update_piece_hash::<WHITE>(piece, square)
    }

    #[inline]
    pub fn remove_piece_on_square<const WHITE: bool>(&mut self, square: Square, piece: Piece) {
        self.pieces.remove_piece_on_square::<WHITE>(square, piece);
        self.state
            .get_key_mut()
            .update_piece_hash::<WHITE>(piece, square)
    }
}
