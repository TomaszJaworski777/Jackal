use spear::ChessPosition;

use crate::{
    search::{networks::ValueNetwork, Score},
    GameState,
};

#[allow(non_upper_case_globals)]
pub const ValueNetwork: ValueNetwork = unsafe {
    std::mem::transmute(*include_bytes!(
        "../../../resources/networks/value_007.network"
    ))
};

pub struct SearchHelpers;
impl SearchHelpers {
    #[inline]
    pub fn get_node_score<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        current_position: &mut ChessPosition,
        state: GameState,
    ) -> Score {
        match state {
            GameState::Drawn => Score::DRAW,
            GameState::Lost(_) => Score::LOSE,
            GameState::Won(_) => Score::WIN,
            GameState::Unresolved => Score::from(sigmoid(
                ValueNetwork.forward::<STM_WHITE, NSTM_WHITE>(current_position.board()),
            )),
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
}

#[inline]
fn sigmoid(input: f32) -> f32 {
    1.0 / (1.0 + (-input).exp())
}
