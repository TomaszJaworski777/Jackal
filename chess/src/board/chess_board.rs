use crate::{
    base_structures::{CastleRights, ZobristKey},
    Bitboard, Piece, Side, Square,
};

const PHASE_VALUES: [u8; 6] = [0, 1, 1, 2, 4, 0];

#[repr(C)]
#[repr(align(64))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ChessBoard {
    occupancy: [Bitboard; 2],
    pieces: [Bitboard; 6],
    hash: ZobristKey,
    phase: u8,
    pub(super) side: Side,
    pub(super) castle_rights: CastleRights,
    pub(super) en_passant_square: Square,
    pub(super) half_moves: u8,
}

impl ChessBoard {
    #[inline]
    pub fn occupancy(&self) -> Bitboard {
        self.occupancy[0] | self.occupancy[1]
    }

    #[inline]
    pub fn occupancy_for_side(&self, side: Side) -> Bitboard {
        self.occupancy[usize::from(side)]
    }

    #[inline]
    pub fn piece_mask(&self, piece: Piece) -> Bitboard {
        assert_ne!(piece, Piece::NONE);
        self.pieces[usize::from(piece)]
    }

    #[inline]
    pub fn piece_mask_for_side(&self, piece: Piece, side: Side) -> Bitboard {
        assert_ne!(piece, Piece::NONE);
        self.piece_mask(piece) & self.occupancy_for_side(side)
    }

    #[inline]
    pub fn king_square(&self, side: Side) -> Square {
        self.piece_mask_for_side(Piece::KING, side).ls1b_square()
    }

    #[inline]
    pub fn color_on_square(&self, square: Square) -> Side {
        if self.occupancy[usize::from(Side::BLACK)].get_bit(square) {
            Side::BLACK
        } else {
            Side::WHITE
        }
    }

    #[inline]
    pub fn piece_on_square(&self, square: Square) -> Piece {
        for piece in usize::from(Piece::PAWN)..=usize::from(Piece::KING) {
            if self.pieces[piece].get_bit(square) {
                return Piece::from(piece);
            }
        }

        return Piece::NONE;
    }

    #[inline]
    pub fn hash(&self) -> ZobristKey {
        let mut result = self.hash;

        if self.en_passant_square != Square::NULL {
            result.add_en_passant(self.en_passant_square);
        }

        result.add_side_to_move(self.side);
        result.add_castle_rights(&self.castle_rights);

        result
    }

    #[inline]
    pub fn phase(&self) -> u8 {
        self.phase
    }

    #[inline]
    pub fn side(&self) -> Side {
        self.side
    }

    #[inline]
    pub fn castle_rights(&self) -> &CastleRights {
        &self.castle_rights
    }

    #[inline]
    pub fn en_passant_square(&self) -> Square {
        self.en_passant_square
    }

    #[inline]
    pub fn half_moves(&self) -> u8 {
        self.half_moves
    }

    #[inline]
    pub fn set_piece_on_square(&mut self, square: Square, piece: Piece, side: Side) {
        assert_ne!(square, Square::NULL);
        assert_ne!(piece, Piece::NONE);
        self.occupancy[usize::from(side)].set_bit(square);
        self.pieces[usize::from(piece)].set_bit(square);
        self.hash.update_piece_hash(square, piece, side);
        self.phase += PHASE_VALUES[usize::from(piece)];
    }

    #[inline]
    pub fn remove_piece_on_square(&mut self, square: Square, piece: Piece, side: Side) {
        assert_ne!(square, Square::NULL);
        assert_ne!(piece, Piece::NONE);
        self.occupancy[usize::from(side)].pop_bit(square);
        self.pieces[usize::from(piece)].pop_bit(square);
        self.hash.update_piece_hash(square, piece, side);
        self.phase -= PHASE_VALUES[usize::from(piece)];
    }
}
