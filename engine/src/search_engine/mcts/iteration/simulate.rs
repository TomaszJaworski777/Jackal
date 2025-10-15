use chess::ChessPosition;

use crate::{search_engine::tree::NodeIndex, GameState, SearchEngine, ValueNetwork, WDLScore};

impl SearchEngine {
    pub(super) fn simulate(&self, node_idx: NodeIndex, position: &ChessPosition, depth: f64) -> WDLScore {
        let node = &self.tree[node_idx];

        if node.visits() == 0 {
            let state = self.get_node_state(position);
            self.tree().set_state(node_idx, state);
        }

        let is_stm = self.root_position().board().side() == position.board().side();

        if node.state() == GameState::Ongoing {
            if let Some(entry) = self.tree().hash_table().get(position.board().hash()) {
                entry
            } else {
                self.get_position_score(position, node.state(), is_stm, depth)
            }
        } else {
            self.get_position_score(position, node.state(), is_stm, depth)
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

    fn get_position_score(&self, position: &ChessPosition, node_state: GameState, is_stm: bool, depth: f64) -> WDLScore {
        match node_state {
            GameState::Draw => return WDLScore::DRAW,
            GameState::Loss(_) => return WDLScore::LOSE,
            GameState::Win(_) => return WDLScore::WIN,
            _ => ()
        };

        let mut score = ValueNetwork.forward(position.board());

        score.apply_material_scaling(position.board(), self.options());
        score.apply_50mr(position.board().half_moves(), depth, self.options());
        
        let mut draw_chance= score.draw_chance();
        let mut win_lose_delta = score.win_chance() - score.lose_chance();
        
        let sign = if is_stm { 1.0 } else { -1.0 };
        
        if position.board().phase() > 8 {
            self.contempt().rescale(&mut win_lose_delta, &mut draw_chance, sign, false, self.options());
        }
        
        let mut new_win_chance = (1.0 + win_lose_delta - draw_chance) / 2.0;
        
        if self.syzygy().is_syzygy_available(position) && draw_chance < 0.8 {
            if let Some(dtz) = self.syzygy().probe_dtz(position) {
                const SCALE: f64 = 20.0;
                const CAP:   f64 = 0.2;

                let u = (-(dtz as f64)) / (SCALE + (dtz.abs() as f64));
                let mut delta_single = (CAP * u).clamp(-CAP, CAP);

                let mut l = 1.0 - draw_chance - new_win_chance;

                if delta_single > 0.0 {
                    // Increase single: prefer stealing from LOSE first (1:1), then DRAW (0.5:1)
                    let take_l = delta_single.min(l);
                    new_win_chance += take_l; 
                    l -= take_l;
                    delta_single -= take_l;

                    if delta_single > 0.0 && draw_chance > 0.0 {
                        let take_d = (2.0 * delta_single).min(draw_chance); // 0.5 per unit to single
                        new_win_chance += take_d; draw_chance -= take_d;
                        delta_single -= 0.5 * take_d;
                    }
                } else if delta_single < 0.0 {
                    // Decrease single: prefer moving WIN -> DRAW (0.5:1), then WIN -> LOSE (1:1)
                    let mut need = -delta_single;

                    if new_win_chance > 0.0 {
                        let to_d = (2.0 * need).min(new_win_chance);
                        new_win_chance -= to_d; draw_chance += to_d;
                        need -= 0.5 * to_d;
                    }
                    if need > 0.0 && new_win_chance > 0.0 {
                        let to_l = need.min(new_win_chance);
                        new_win_chance -= to_l; l += to_l;
                    }
                }

                // Tidy clamps
                new_win_chance = new_win_chance.clamp(0.0, 1.0);
                draw_chance = draw_chance.clamp(0.0, 1.0 - new_win_chance);
            }
        }

        WDLScore::new(new_win_chance, draw_chance)
    }
}