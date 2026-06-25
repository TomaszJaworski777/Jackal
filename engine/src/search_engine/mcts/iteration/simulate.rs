use chess::ChessPosition;

use crate::{
    BaseValueNetwork, GameState, SearchEngine, Stage1ValueNetwork, WDLScore, search_engine::{engine_options::EngineOptions, tree::NodeIndex},
};

impl SearchEngine {
    pub(super) fn simulate(
        &self,
        node_idx: NodeIndex,
        position: &ChessPosition,
        depth: f64,
        parent_score: WDLScore,
    ) -> WDLScore {
        if self.tree()[node_idx].visits() == 0 {
            let state = self.get_node_state(position);
            self.tree().set_state(node_idx, state);
        }

        let stm = depth as i32 % 2 == 0;

        if self.tree[node_idx].state() == GameState::Ongoing {
            if let Some(entry) = self.tree().hash_table().get(position.board().hash()) {
                entry
            } else {
                self.get_position_score(
                    position,
                    self.tree()[node_idx].state(),
                    depth,
                    stm,
                    parent_score,
                )
            }
        } else {
            self.get_position_score(
                position,
                self.tree()[node_idx].state(),
                depth,
                stm,
                parent_score,
            )
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

        if history_repetitions >= 3
            || search_repetitions >= 2
            || history_repetitions + search_repetitions >= 3
        {
            return true;
        }

        false
    }

    fn get_position_score(
        &self,
        position: &ChessPosition,
        node_state: GameState,
        depth: f64,
        stm: bool,
        parent_score: WDLScore,
    ) -> WDLScore {
        let mut score = match node_state {
            GameState::Draw => return WDLScore::DRAW,
            GameState::Loss(_) => return WDLScore::LOSE,
            GameState::Win(_) => return WDLScore::WIN,
            _ => {
                let score = if stm && position.board().phase() > 8 {
                    if hash(u64::from(position.board().hash())) < stage1_prob(parent_score.win_chance(), self.options()) {
                        Stage1ValueNetwork.forward(position.board())
                    } else {
                        BaseValueNetwork.forward(position.board())
                    }
                } else {
                    BaseValueNetwork.forward(position.board())
                };

                score
            }
        };

        #[cfg(not(feature = "datagen"))]
        {
            //score.apply_material_scaling(position.board(), self.options());
            score.apply_draw_pessimism(position.board(), self.options());
        }

        score.apply_50mr_and_draw_scaling(position.board().half_moves(), depth, self.options());

        let is_stm = self.root_position().board().side() == position.board().side();
        let sign = if is_stm { 1 } else { -1 };

        if (score.single() - 0.5).abs() < 0.4 {
            score.apply_contempt(self.options().contempt() * sign);
        }

        score
    }
}

fn stage1_prob(win_chance: f64, options: &EngineOptions) -> f64 {
    if win_chance <= options.value_stage_low_bound() || win_chance >= options.value_stage_high_bound() {
        return 0.0;
    }

    let up_end   = options.value_stage_low_bound() + options.value_stage_left_ramp();
    let down_beg = options.value_stage_high_bound() - options.value_stage_right_ramp();

    let bump = if win_chance < up_end {
        let t = (win_chance - options.value_stage_low_bound()) / options.value_stage_left_ramp();
        0.5 * (1.0 - (std::f64::consts::PI * t).cos())
    } else if win_chance > down_beg {
        let t = (win_chance - down_beg) / options.value_stage_right_ramp();
        0.5 * (1.0 + (std::f64::consts::PI * t).cos())
    } else {
        1.0
    };

    options.value_stage_pmax() * bump
}

fn hash(h: u64) -> f64 {
    let mut x = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30; x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27; x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 12) as f64 / (1u64 << 52) as f64
}