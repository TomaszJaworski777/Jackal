use colored::Colorize;
use spear::{ChessBoard, Move};

use crate::{utils::heat_color, GameState};

use super::{Edge, NodeIndex, Tree};

impl Tree {
    pub fn draw_tree<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
        board: &ChessBoard,
        node_idx: NodeIndex,
        depth: u32,
    ) {
        self.print_usage_text();
        let mut node_depth = 0;
        let is_root = node_idx == self.root_index();
        if is_root {
            let edge = self.root_edge();
            edge.print::<true>(edge.policy(), edge.policy(), self[node_idx].state(), true);
        } else {
            let result = self.find_position_by_key::<_, NSTM_WHITE, STM_WHITE>(
                board,
                self.root_index(),
                255,
                &|_, idx| idx == node_idx,
            );
            if let Some((edge_idx, action_idx, depth)) = result {
                node_depth = depth;
                let edge = self.get_edge_clone(edge_idx, action_idx, 1);
                edge.print::<true>(
                    edge.policy(),
                    edge.policy(),
                    self[node_idx].state(),
                    node_depth % 2 == 0,
                );
            } else {
                return;
            }
        }
        self.draw_tree_internal(node_idx, depth - 1, &String::new(), node_depth % 2 == 1)
    }

    fn print_usage_text(&self) {
        let usage = self.total_usage();
        let usage_percent = heat_color(
            format!("{}%", (usage * 100.0) as u32).as_str(),
            1.0 - usage,
            0.0,
            1.0,
        );
        let used_bytes = format!(
            "{}B",
            Self::bytes_to_string((self.tree_size_in_bytes as f32 * usage) as u128)
        )
        .white();
        let total_bytes = format!(
            "{}B",
            Self::bytes_to_string(self.tree_size_in_bytes as u128)
        )
        .white();
        println!(
            "{}",
            format!(
                "\nUsage: {}/{} ({})\n",
                used_bytes, total_bytes, usage_percent
            )
            .bright_black()
        );
    }

    fn bytes_to_string(number: u128) -> String {
        match number {
            0..=1023 => format!("{number}"),
            1024..=1_048_575 => format!("{:.2}K", number as f64 / 1024.0),
            1_048_576..=1_073_741_823 => format!("{:.2}M", number as f64 / 1_048_576.0),
            1_073_741_824.. => format!("{:.2}G", number as f64 / 1_073_741_824.0),
        }
    }

    fn find_position_by_key<
        F: Fn(ChessBoard, NodeIndex) -> bool,
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
    >(
        &self,
        previous_board: &ChessBoard,
        node_idx: NodeIndex,
        depth: u8,
        method: &F,
    ) -> Option<(NodeIndex, usize, u8)> {
        let actions = &*self[node_idx].actions();
        for (idx, action) in actions.iter().enumerate() {
            let child_idx = action.node_index();
            if child_idx == NodeIndex::NULL {
                continue;
            }

            let mut board_clone = *previous_board;
            board_clone.make_move::<STM_WHITE, NSTM_WHITE>(action.mv());
            if method(board_clone, child_idx) {
                return Some((node_idx, idx, 1));
            }

            if depth - 1 > 0 {
                let result = self.find_position_by_key::<F, NSTM_WHITE, STM_WHITE>(
                    &board_clone,
                    child_idx,
                    depth - 1,
                    method,
                );
                if let Some((edge_idx, action_idx, depth)) = result {
                    return Some((edge_idx, action_idx, depth + 1));
                }
            }
        }

        None
    }

    fn draw_tree_internal(
        &self,
        node_index: NodeIndex,
        depth: u32,
        prefix: &String,
        flip_score: bool,
    ) {
        if self.total_usage() == 0.0 {
            return;
        }

        let max_policy = self[node_index]
            .actions()
            .iter()
            .max_by(|&a, &b| {
                a.policy()
                    .partial_cmp(&b.policy())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&Edge::new(NodeIndex::NULL, Move::NULL, 0.0))
            .policy();

        let min_policy = self[node_index]
            .actions()
            .iter()
            .min_by(|&a, &b| {
                a.policy()
                    .partial_cmp(&b.policy())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&Edge::new(NodeIndex::NULL, Move::NULL, 0.0))
            .policy();

        let actions_len = self[node_index].actions().len();
        for (idx, action) in self[node_index].actions().iter().enumerate() {
            let is_last = idx == actions_len - 1;
            let state = if action.node_index().is_null() {
                GameState::Unresolved
            } else {
                self[action.node_index()].state()
            };
            print!("{}{} ", prefix, if is_last { "└─>" } else { "├─>" });
            action.print::<false>(min_policy, max_policy, state, flip_score);
            if !action.node_index().is_null()
                && self[action.node_index()].has_children()
                && depth > 0
            {
                let prefix_add = if is_last { "    " } else { "│   " };
                self.draw_tree_internal(
                    action.node_index(),
                    depth - 1,
                    &format!("{}{}", prefix, prefix_add),
                    !flip_score,
                )
            }
        }
    }
}
