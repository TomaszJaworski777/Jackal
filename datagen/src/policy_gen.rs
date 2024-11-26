use super::{display::Printer, utils::DataGenUtils};
use crossbeam_queue::SegQueue;
use jackal::{EngineOptions, GameState, Mcts, NoPrint, SearchLimits, SearchStats, Tree};
use spear::{ChessPosition, Move, PolicyPacked, Side};
use std::sync::atomic::AtomicBool;

pub struct PolicyGen;
impl PolicyGen {
    pub fn start_game_loop(
        save_queue: &SegQueue<Vec<u8>>,
        iter_count: u32,
        printer: &Printer,
        interruption_token: &AtomicBool,
    ) {
        let options = EngineOptions::new();
        let mut tree = Tree::new(options.hash(), options.hash_percentage() / 10.0);
        let mut limits = SearchLimits::new();
        limits.add_iters(iter_count);

        while !interruption_token.load(std::sync::atomic::Ordering::Relaxed) {
            let mut position = DataGenUtils::get_random_position();
            tree.clear();

            let mut packed_positions: Vec<PolicyPacked> = Vec::new();
            let mut state = GameState::Unresolved;
            let mut previous_position = *position.board();

            while state == GameState::Unresolved {
                tree.reuse_tree(&previous_position, position.board());
                previous_position = *position.board();

                let search_stats = SearchStats::new();
                let search_interruption_token = AtomicBool::new(false);

                let mcts = Mcts::new(
                    position,
                    &tree,
                    &search_interruption_token,
                    &options,
                    &search_stats,
                    &limits,
                );

                let (best_move, _) = mcts.search::<NoPrint>();
                let mut packed_position = PolicyPacked::from_board(position.board());

                let move_count = tree[tree.root_index()].actions().len();
                if move_count <= PolicyPacked::MAX_MOVE_COUNT {
                    for action in &*tree[tree.root_index()].actions() {
                        packed_position.push_move(action.mv(), action.visits() as u16);
                    }
                }

                let is_game_end = if position.board().side_to_move() == Side::WHITE {
                    Self::process_move::<true, false>(&mut position, best_move, &mut state)
                } else {
                    Self::process_move::<false, true>(&mut position, best_move, &mut state)
                };

                if is_game_end {
                    continue;
                }

                if move_count <= PolicyPacked::MAX_MOVE_COUNT && move_count != 0 {
                    packed_positions.push(packed_position);
                }
            }

            printer.add_position(packed_positions.len() as u64);

            let bytes = bytemuck::cast_slice(&packed_positions);
            save_queue.push(bytes.to_vec());
        }
    }

    fn process_move<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        position: &mut ChessPosition,
        best_move: Move,
        state: &mut GameState,
    ) -> bool {
        let mut no_moves = true;
        position.make_move::<STM_WHITE, NSTM_WHITE>(best_move);
        position.board().map_moves::<_, NSTM_WHITE, STM_WHITE>(|_| {
            no_moves = false;
        });

        if no_moves {
            *state = if position.board().is_in_check::<NSTM_WHITE, STM_WHITE>() {
                GameState::Lost(0)
            } else {
                GameState::Drawn
            };
            return true;
        } else if position.is_repetition()
            || position.board().is_insufficient_material()
            || position.board().half_move_counter() >= 100
        {
            *state = GameState::Drawn;
            return true;
        }

        return false;
    }
}
