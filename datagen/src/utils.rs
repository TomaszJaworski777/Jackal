use spear::{ChessPosition, Move, FEN};
use rand::Rng;

pub struct DataGenUtils;
impl DataGenUtils {
    pub fn get_random_position() -> ChessPosition {
        let mut result = ChessPosition::from_fen(&FEN::start_position());
        Self::get_random_position_internal::<true, false>(
            &mut result,
            rand::thread_rng().gen_range(7..=8),
        );
        result
    }

    fn get_random_position_internal<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        position: &mut ChessPosition,
        depth: u8,
    ) {
        let mut move_list: Vec<Move> = Vec::new();
        position
            .board()
            .map_moves::<_, STM_WHITE, NSTM_WHITE>(|mv| move_list.push(mv));

        if move_list.len() == 0 {
            *position = Self::get_random_position();
            return;
        }

        position.make_move::<STM_WHITE, NSTM_WHITE>(
            move_list[rand::thread_rng().gen_range(0..move_list.len())],
        );

        if depth > 0 {
            Self::get_random_position_internal::<NSTM_WHITE, STM_WHITE>(position, depth - 1);
        } else {
            let mut has_moves = false;
            position
                .board()
                .map_moves::<_, NSTM_WHITE, STM_WHITE>(|_| has_moves = true);

            if !has_moves {
                *position = Self::get_random_position()
            }
        }
    }
}
