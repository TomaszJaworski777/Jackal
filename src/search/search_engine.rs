use std::sync::atomic::{AtomicBool, Ordering};

use spear::{ChessPosition, FEN};

use crate::options::EngineOptions;

use super::{print::NoPrint, search_limits::SearchLimits, tree::SearchTree, Mcts, SearchStats};

pub struct SearchEngine<'a> {
    position: ChessPosition,
    interruption_token: &'a AtomicBool,
    tree: &'a mut SearchTree,
    options: &'a mut EngineOptions,
    uci_initialized: bool,
}

impl<'a> SearchEngine<'a> {
    pub fn new(
        position: ChessPosition,
        interruption_token: &'a AtomicBool,
        tree: &'a mut SearchTree,
        options: &'a mut EngineOptions,
    ) -> Self {
        Self {
            position,
            interruption_token,
            tree,
            options,
            uci_initialized: false,
        }
    }

    pub fn init_uci(&mut self) {
        self.uci_initialized = true;
    }

    pub fn engine_options(&self) -> &EngineOptions {
        &self.options
    }

    pub fn engine_options_mut(&mut self) -> &mut EngineOptions {
        &mut self.options
    }

    pub fn replace_position(&mut self, position: ChessPosition) {
        self.position = position
    }

    pub fn current_position(&self) -> ChessPosition {
        self.position
    }

    pub fn reset(&mut self) {
        self.position = ChessPosition::from_fen(&FEN::start_position());
        self.tree.clear();
    }

    pub fn stop(&self) {
        self.interruption_token.store(true, Ordering::Relaxed)
    }

    pub fn search(&mut self, search_limits: &SearchLimits, print_reports: bool) {
        //pass limits as argument
        self.interruption_token.store(false, Ordering::Relaxed);
        let search_stats = SearchStats::new();
        std::thread::scope(|s| {
            s.spawn(|| {
                let mut mcts = Mcts::new(
                    self.position.clone(),
                    self.tree,
                    self.interruption_token,
                    self.options,
                    &search_stats,
                    search_limits,
                );
                let (best_move, best_score) = if print_reports {
                    if self.uci_initialized {
                        mcts.search::<NoPrint>()
                    } else {
                        mcts.search::<NoPrint>()
                    }
                } else {
                    mcts.search::<NoPrint>()
                };
            });
        });
    }
}
