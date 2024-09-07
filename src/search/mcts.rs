use std::sync::atomic::AtomicBool;

use spear::{ChessPosition, Move};

use super::{print::SearchPrinter, SearchTree};

pub struct Mcts<'a> {
    root_position: ChessPosition,
    tree: &'a SearchTree,
    interruption_token: &'a AtomicBool,
}

impl<'a> Mcts<'a> {
    pub fn new(
        root_position: ChessPosition,
        tree: &'a SearchTree,
        interruption_token: &'a AtomicBool,
    ) -> Self {
        //add uci options struct to the params
        Self {
            root_position,
            tree,
            interruption_token,
        }
    }

    pub fn search<PRINTER: SearchPrinter>(&self) -> (Move, f32) {
        //params like limits of the search
        (Move::NULL, 0.0)
    }
}
