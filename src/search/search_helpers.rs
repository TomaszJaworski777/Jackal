use spear::ChessPosition;

use super::tree::GameState;

pub struct SearchHelpers;
impl SearchHelpers {
    #[inline]
    pub fn get_node_score<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        current_position: &mut ChessPosition,
        state: GameState,
    ) -> f32 {
        match state {
            GameState::Drawn => 0.5,
            GameState::Lost(_) => 0.0,
            GameState::Won(_) => 1.0,
            GameState::Unresolved => (current_position.board().get_key().get_raw() % 256) as f32 / 255.0,
        }
    }

    pub fn get_position_state<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        position: &ChessPosition,
    ) -> GameState {
        if position.is_repetition()
            || position.board().is_insufficient_material()
            || position.board().half_move_counter() >= 100
        {
            return GameState::Drawn;
        }

        let mut move_count = 0;
        position.board().map_moves::<_, STM_WHITE, NSTM_WHITE>(|_| {
            move_count += 1;
        });

        if move_count == 0 {
            if position.board().is_in_check::<STM_WHITE, NSTM_WHITE>() {
                return GameState::Lost(0);
            } else {
                return GameState::Drawn;
            }
        }

        GameState::Unresolved
    }

    #[inline]
    pub fn score_into_cp(score: f32) -> i32 {
        (-400.0 * (1.0 / score.clamp(0.0, 1.0) - 1.0).ln()) as i32
    }
}