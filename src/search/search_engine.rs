use std::sync::atomic::AtomicBool;

use spear::{ChessPosition, ZobristKey};

pub struct SearchEngine<'a> {
    current_position: &'a ChessPosition,
    previous_board: ZobristKey,
    interruption_token: &'a AtomicBool,
    //tree
    //options
}
