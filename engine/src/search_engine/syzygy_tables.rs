use chess::{ChessPosition, Piece, Side, Square};
use shakmaty_syzygy::{AmbiguousWdl, Tablebase, Wdl};

use shakmaty::{Color, FromSetup, Piece as SPiece, Position, Role, Setup, Square as SSquare};
use core::num::NonZeroU32;

use crate::{GameState, WDLScore};

#[derive(Debug)]
pub struct SyzygyTables {
    tables: Option<Tablebase<shakmaty::Chess>>,
    piece_count: usize,
}

impl Default for SyzygyTables {
    fn default() -> Self {
        Self { 
            tables: None,
            piece_count: 0
        }
    }
}

impl SyzygyTables {
    pub fn load_table(&mut self, path: &String) -> String {
        let mut tb: Tablebase<shakmaty::Chess> = unsafe {
            Tablebase::with_mmap_filesystem()
        };
        let result = tb.add_directory(path);
        if result.is_err() {
            *self = SyzygyTables::default();
            return format!("Incorrect syzygy files under {path}!");
        }

        self.piece_count = tb.max_pieces();
        self.tables = Some(tb);

        format!("{} files of {}-man syzygy tables has been loaded succesfully.", result.unwrap(), self.piece_count)
    }

    pub fn is_syzygy_available(&self, position: &ChessPosition) -> bool {
        self.tables.is_some() && position.board().occupancy().pop_count() <= self.piece_count as u32 && u8::from(position.board().castle_rights()) == 0
    }
    
    pub fn probe_wdl(&self, position: &ChessPosition) -> Option<WDLScore> { 
        let position = position_to_shakmaty(position);
        let tables = self.tables.as_ref()?;
        
        let wdl_result = tables.probe_wdl(&position).ok()?;

        Some(wdl_to_score(wdl_result.after_zeroing()))
    }

    pub fn probe_wdl_after_zeroing(&self, position: &ChessPosition) -> Option<WDLScore> { 
        let position =position_to_shakmaty(position);
        let tables = self.tables.as_ref()?;
        match tables.probe_wdl_after_zeroing(&position) {
            Ok(wdl) => Some(wdl_to_score(wdl)),
            Err(_)  => None,
        }
    }

    pub fn probe_dtz(&self, position: &ChessPosition) -> Option<i32> { 
        let position =position_to_shakmaty(position);
        let tables = self.tables.as_ref()?;
        match tables.probe_dtz(&position) {
            Ok(maybe) => {
                let dtz = maybe.ignore_rounding();
                Some(dtz.0)
            }
            Err(_) => None,
        }
    }

    pub fn probe_outcome(&self, position: &ChessPosition) -> Option<GameState> {
        let mut position = position_to_shakmaty(position);
        let tables = self.tables.as_ref().unwrap();

        let mut plies = 0;

        loop {
            if position.is_game_over() {
                return match tables.probe_wdl_after_zeroing(&position) {
                    Ok(wdl) if wdl == Wdl::Win => Some(GameState::Win(plies)),
                    Ok(wdl) if wdl == Wdl::Loss => Some(GameState::Loss(plies)),
                    _ => Some(GameState::Draw),
                };
            }

            let wdl = match tables.probe_wdl(&position) {
                Ok(w) => w,
                Err(_) => return None,
            };

            if wdl == AmbiguousWdl::Draw {
                return Some(GameState::Draw);
            }

            let best = match tables.best_move(&position) {
                Ok(Some((best_move, _))) => best_move,
                _ => return None,
            };

            position.play_unchecked(best);
            plies = plies.saturating_add(1);

            if plies == u8::MAX {
                return None;
            }
        }
    }
}

#[inline]
pub fn position_to_shakmaty(position: &ChessPosition) -> shakmaty::Chess {
    let board = position.board();

    let mut setup = Setup::empty();

    board.occupancy().for_each(|square| {
        let piece = board.piece_on_square(square);
        let side  = board.color_on_square(square);
        let shak_square  = map_square(square);
        let shak_piece = SPiece { color: map_color(side), role: map_role(piece) };
        setup.board.set_piece_at(shak_square, shak_piece);
    });

    setup.turn = map_color(board.side());

    setup.castling_rights = shakmaty::Bitboard::EMPTY;

    let ep = board.en_passant_square();
    if ep != Square::NULL {
        setup.ep_square = Some(map_square(ep));
    }

    setup.halfmoves = board.half_moves() as u32;
    setup.fullmoves = NonZeroU32::MIN;

    shakmaty::Chess::from_setup(setup, shakmaty::CastlingMode::Standard).unwrap()
}

#[inline]
fn map_color(side: Side) -> Color {
    match side {
        Side::WHITE => Color::White,
        Side::BLACK => Color::Black,
        _ => unreachable!(),
    }
}

#[inline]
fn map_role(piece: Piece) -> Role {
    match piece {
        Piece::PAWN  => Role::Pawn,
        Piece::KNIGHT => Role::Knight,
        Piece::BISHOP => Role::Bishop,
        Piece::ROOK  => Role::Rook,
        Piece::QUEEN => Role::Queen,
        Piece::KING  => Role::King,
        _ => unreachable!(),
    }
}

#[inline]
fn map_square(sq: Square) -> SSquare {
    SSquare::new(u8::from(sq) as u32)
}

#[inline]
fn wdl_to_score(wdl: Wdl) -> WDLScore {
    match wdl {
        Wdl::Loss => WDLScore::LOSE,
        Wdl::BlessedLoss => WDLScore::DRAW,
        Wdl::Draw => WDLScore::DRAW,
        Wdl::CursedWin => WDLScore::DRAW,
        Wdl::Win => WDLScore::WIN
    }
}