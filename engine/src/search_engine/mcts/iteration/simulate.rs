use chess::ChessPosition;

use crate::{search_engine::tree::NodeIndex, GameState, SearchEngine, ValueNetwork, WDLScore};

impl SearchEngine {
    pub(super) fn simulate(&self, node_idx: NodeIndex, position: &ChessPosition, depth: f64) -> WDLScore {
        if self.tree()[node_idx].visits() == 0 {
            let state = self.get_node_state(position);
            self.tree().set_state(node_idx, state);
        }

        let is_stm = self.root_position().board().side() == position.board().side();

        if self.tree[node_idx].state() == GameState::Ongoing {
            if let Some(entry) = self.tree().hash_table().get(position.board().hash()) {
                entry
            } else {
                self.get_position_score(position, node_idx, is_stm, depth)
            }
        } else {
            self.get_position_score(position, node_idx, is_stm, depth)
        }
    }

    fn get_node_state(&self, position: &ChessPosition) -> GameState {
        let mut possible_moves = 0;
        position.board().map_legal_moves(|_| possible_moves += 1);

        if possible_moves == 0 {
            if position.board().is_in_check() {
                GameState::Loss(0)
            } else {
                GameState::Draw
            }
        } else if self.is_draw(position) {
            GameState::Draw
        } else {
            GameState::Ongoing
        }
    }

    fn is_draw(&self, position: &ChessPosition) -> bool {
        if position.board().half_moves() >= 100 || position.board().is_insufficient_material() {
            return true;
        }

        let key = position.board().hash();
        let history_repetitions = self.root_position().history().get_repetitions(key);
        let search_repetitions = position.history().get_repetitions(key) - history_repetitions;

        if history_repetitions >= 3 || search_repetitions >= 2 || history_repetitions + search_repetitions >= 3 {
            return true;
        }

        false
    }

    fn get_position_score(&self, position: &ChessPosition, node_idx: NodeIndex, is_stm: bool, depth: f64) -> WDLScore {
        let mut score = match self.tree()[node_idx].state() {
            GameState::Draw => WDLScore::DRAW,
            GameState::Loss(_) => WDLScore::LOSE,
            GameState::Win(_) => WDLScore::WIN,
            _ => ValueNetwork.forward(position.board())
        };

        score.apply_material_scaling(position.board(), self.options());
        score.apply_50mr(position.board().half_moves(), depth, self.options());
        
        let mut draw_chance= score.draw_chance();
        let mut win_lose_delta = score.win_chance() - score.lose_chance();
        
        let sign = if is_stm { 1.0 } else { -1.0 };
        
        if position.board().phase() > 8 {
            self.contempt().rescale(&mut win_lose_delta, &mut draw_chance, sign, false, self.options());
        }
        
        let new_win_chance = (1.0 + win_lose_delta - draw_chance) / 2.0;
        
        let mut result = WDLScore::new(new_win_chance, draw_chance);
        self.tree()[node_idx].set_base_score(result);
        self.subtree_bias().apply_bias(&mut result, position, self.options());
        result
    }
}