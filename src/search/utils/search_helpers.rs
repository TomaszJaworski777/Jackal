use crate::spear::ChessPosition;

use crate::{
    search::{Score, ValueNetwork},
    EngineOptions, GameState, Tree,
};

use super::contempt::{Contempt, ContemptParams};

pub struct SearchHelpers;
impl SearchHelpers {
    #[inline]
    pub fn get_node_score<const STM_WHITE: bool, const NSTM_WHITE: bool, const US: bool>(
        current_position: &mut ChessPosition,
        state: GameState,
        key: u64,
        tree: &Tree,
        material_difference: i32,
        options: &EngineOptions,
        contempt_parms: &ContemptParams,
    ) -> Score {
        let score_bonus = if material_difference != 0 {
            options.material_reduction_bonus() / 10.0
        } else {
            0.0
        };

        let score = match state {
            GameState::Drawn => Score::DRAW,
            GameState::Lost(_) => Score::LOSE,
            GameState::Won(_) => Score::WIN,
            GameState::Unresolved => {
                if let Some(score) = tree.hash_table().probe(key) {
                    Score::new(score.win_chance() + score_bonus, score.draw_chance())
                } else {
                    let (win_chance, draw_chance, _) = ValueNetwork.forward::<STM_WHITE, NSTM_WHITE>(current_position.board());
                    let score = Score::new(win_chance, draw_chance);

                    tree.hash_table().store(key, score);

                    Score::new(score.win_chance() + score_bonus, score.draw_chance())
                }
            }
        };

        let (w, mut d, l) = (score.win_chance(), score.draw_chance(), score.lose_chance());
        
        let mut v = w - l;
        
        Contempt::wdl_rescale::<US>(&mut v, &mut d, options, contempt_parms);

        let half_move_scalar = current_position.board().half_move_counter() as f32 / options.move_count_scale();

        let mut w_new = (1.0 + v - d) / 2.0;
        let mut d_new = d;
        
        if US {
            let virtual_l = 1.0 - w_new - d;
            let sw = w_new * half_move_scalar;
            let sl = virtual_l * half_move_scalar;

            w_new = w_new - sw;
            d_new = d + sw + sl;
        }

        Score::new(w_new, d_new)
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
