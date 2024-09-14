use crate::spear::{
    base_structures::{Side, ZobristKey},
    CastleRights, ChessBoard, Piece, Square,
};

pub(super) const PHASE_VALUES: [u8; 6] = [0, 1, 1, 2, 4, 0];

#[derive(Clone, Copy, Default, PartialEq)]
pub struct ChessBoardState {
    zobrist: ZobristKey,
    half_moves: u8,
    en_passant: Square,
    side_to_move: Side,
    castle_rights: CastleRights,
    phase: u8,
}

impl ChessBoardState {
    #[inline]
    pub(super) fn get_key_mut(&mut self) -> &mut ZobristKey {
        &mut self.zobrist
    }

    #[inline]
    pub(super) fn get_side_to_move_mut(&mut self) -> &mut Side {
        &mut self.side_to_move
    }

    #[inline]
    pub(super) fn get_en_passant_mut(&mut self) -> &mut Square {
        &mut self.en_passant
    }

    #[inline]
    pub(super) fn get_castle_rights_mut(&mut self) -> &mut CastleRights {
        &mut self.castle_rights
    }

    #[inline]
    pub(super) fn get_half_move_counter_mut(&mut self) -> &mut u8 {
        &mut self.half_moves
    }

    #[inline]
    pub(super) fn get_phase_mut(&mut self) -> &mut u8 {
        &mut self.phase
    }
}

impl ChessBoard {
    #[inline]
    pub fn get_key(&self) -> ZobristKey {
        let mut key = self.state.zobrist;

        if self.en_passant_square() != Square::NULL {
            key ^= ZobristKey::get_en_passant_seed(self.en_passant_square())
        }

        key ^ ZobristKey::get_castle_rights_seed(self.castle_rights().get_raw())
            ^ (ZobristKey::get_side_to_move_seed() * self.side_to_move().get_raw() as u64)
    }

    #[inline]
    pub fn en_passant_square(&self) -> Square {
        self.state.en_passant
    }

    #[inline]
    pub fn side_to_move(&self) -> Side {
        self.state.side_to_move
    }

    #[inline]
    pub fn castle_rights(&self) -> CastleRights {
        self.state.castle_rights
    }

    #[inline]
    pub fn half_move_counter(&self) -> u8 {
        self.state.half_moves
    }

    #[inline]
    pub fn get_phase(&self) -> u8 {
        self.state.phase
    }

    #[inline]
    pub fn is_insufficient_material(&self) -> bool {
        let phase = self.get_phase();
        let bishops = self.get_piece_mask(Piece::BISHOP);
        phase <= 2
            && self.get_piece_mask(Piece::PAWN).is_empty()
            && ((phase != 2)
                || (bishops & self.get_occupancy_for_side::<true>() != bishops
                    && bishops & self.get_occupancy_for_side::<false>() != bishops
                    && (bishops & 0x55AA55AA55AA55AA == bishops
                        || bishops & 0xAA55AA55AA55AA55 == bishops)))
    }
}
