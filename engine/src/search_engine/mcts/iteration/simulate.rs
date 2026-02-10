use chess::ChessPosition;

use crate::{BaseValueNetwork, GameState, SearchEngine, Stage1ValueNetwork, Stage2ValueNetwork, WDLScore, search_engine::tree::NodeIndex};

impl SearchEngine {
    pub(super) fn simulate(&self, node_idx: NodeIndex, position: &ChessPosition, depth: f64, parent_score: WDLScore) -> WDLScore {
        if self.tree()[node_idx].visits() == 0 {
            let state = self.get_node_state(position);
            self.tree().set_state(node_idx, state);
        }

        let stm = depth as i32 % 2 == 0;

        if self.tree[node_idx].state() == GameState::Ongoing {
            if let Some(entry) = self.tree().hash_table().get(position.board().hash()) {
                entry
            } else {
                self.get_position_score(position, self.tree()[node_idx].state(), depth, stm, parent_score)
            }
        } else {
            self.get_position_score(position, self.tree()[node_idx].state(), depth, stm, parent_score)
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

    fn get_position_score(&self, position: &ChessPosition, node_state: GameState, depth: f64, stm: bool, mut parent_score: WDLScore) -> WDLScore {
        let mut score = match node_state {
            GameState::Draw => WDLScore::DRAW,
            GameState::Loss(_) => WDLScore::LOSE,
            GameState::Win(_) => WDLScore::WIN,
            _ => {
                if stm && position.board().phase() > 8 {
                    return if parent_score.win_chance() > 0.9 {
                        BaseValueNetwork.forward(position.board())
                    } else if parent_score.win_chance() > 0.575 {
                        Stage2ValueNetwork.forward(position.board())
                    } else {
                        Stage1ValueNetwork.forward(position.board())
                    }
                }

                if !stm {
                    parent_score = parent_score.reversed();
                }

                return if parent_score.win_chance() > 0.85 && position.board().phase() > 8 {
                    Stage1ValueNetwork.forward(position.board())
                } else {
                    BaseValueNetwork.forward(position.board())
                };
            }
        };

        #[cfg(not(feature = "datagen"))]
        score.apply_material_scaling(position.board(), self.options());
        score.apply_50mr_and_draw_scaling(position.board().half_moves(), depth, self.options());
        
        let is_stm = self.root_position().board().side() == position.board().side();
        let sign = if is_stm { 1 } else { -1};
        
        score.apply_contempt(self.options().contempt() * sign);
        
        score
    }
}