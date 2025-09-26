use chess::ChessPosition;

use crate::{search_engine::{contempt::Contempt, engine_options::EngineOptions, tree::NodeIndex}, GameState, SearchEngine, ValueNetwork, WDLScore};

impl SearchEngine {
    pub(super) fn simulate(&self, node_idx: NodeIndex, position: &ChessPosition, depth: f64) -> WDLScore {
        if self.tree()[node_idx].visits() == 0 {
            let state = get_node_state(position, self.root_position());
            self.tree().set_state(node_idx, state);
        }

        let is_stm = self.root_position().board().side() == position.board().side();

        if self.tree[node_idx].state() == GameState::Ongoing {
            if let Some(entry) = self.tree().hash_table().get(position.board().hash()) {
                entry
            } else {
                get_position_score(position, self.tree()[node_idx].state(), self.contempt(), self.options(), is_stm, depth)
            }
        } else {
            get_position_score(position, self.tree()[node_idx].state(), self.contempt(), self.options(), is_stm, depth)
        }
    }
}

fn get_node_state(position: &ChessPosition, root_position: &ChessPosition) -> GameState {
    let mut possible_moves = 0;
    position.board().map_legal_moves(|_| possible_moves += 1);

    if possible_moves == 0 {
        if position.board().is_in_check() {
            GameState::Loss(0)
        } else {
            GameState::Draw
        }
    } else if is_draw(position, root_position) {
        GameState::Draw
    } else {
        GameState::Ongoing
    }
}

fn is_draw(position: &ChessPosition, root_position: &ChessPosition) -> bool {
    if position.board().half_moves() >= 100 || position.board().is_insufficient_material() {
        return true;
    }

    let key = position.board().hash();
    let history_repetitions = root_position.history().get_repetitions(key);
    let search_repetitions = position.history().get_repetitions(key) - history_repetitions;

    if history_repetitions >= 3 || search_repetitions >= 2 || history_repetitions + search_repetitions >= 3 {
        return true;
    }

    false
}

fn get_position_score(position: &ChessPosition, node_state: GameState, contempt: &Contempt, options: &EngineOptions, is_stm: bool, depth: f64) -> WDLScore {
    let mut score = match node_state {
        GameState::Draw => WDLScore::DRAW,
        GameState::Loss(_) => WDLScore::LOSE,
        GameState::Win(_) => WDLScore::WIN,
        _ => ValueNetwork.forward(position.board())
    };

    score.apply_50mr(position.board().half_moves(), depth, options);
    score.apply_material_scaling(position.board(), options);

    let mut draw_chance= score.draw_chance();
    let mut win_lose_delta = score.win_chance() - score.lose_chance();

    let sign = if is_stm { 1.0 } else { -1.0 };

    contempt.rescale(&mut win_lose_delta, &mut draw_chance, sign, false, options);

    let new_win_chance = (1.0 + win_lose_delta - draw_chance) / 2.0;

    WDLScore::new(new_win_chance, draw_chance)
}