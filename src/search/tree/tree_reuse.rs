use spear::{ChessBoard, Side};

use crate::GameState;

use super::{Edge, NodeIndex, Tree};

impl Tree {
    pub fn reuse_tree(&mut self, previous_board: &ChessBoard, current_board: &ChessBoard) {
        if self.total_usage() == 0.0 {
            _ = self.current_segment().add(GameState::Unresolved);
            return;
        }

        if previous_board == current_board {
            return;
        }

        let (node_index, edge) = if previous_board.side_to_move() == Side::WHITE {
            self.recurse_find::<_, true, false>(self.root_index(), previous_board, self.root_edge().clone(), 2, &|board, _| board == current_board)
        } else {
            self.recurse_find::<_, false, true>(self.root_index(), previous_board, self.root_edge().clone(), 2, &|board, _| board == current_board)
        };

        if !node_index.is_null() && self[node_index].has_children() {
            self[self.root_index()].replace(GameState::Unresolved);
            self.copy_node(node_index, self.root_index());
            self.root_edge = edge;
        } else {
            self.clear();
            _ = self.current_segment().add(GameState::Unresolved);
        }
    }

    fn recurse_find<F: Fn(&ChessBoard, NodeIndex) -> bool, const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
        start: NodeIndex,
        board: &ChessBoard,
        edge: Edge,
        depth: u8,
        method: &F
    ) -> (NodeIndex, Edge) {
        if method(board, start) {
            return (start, edge);
        }

        if start.is_null() || depth == 0 {
            return (NodeIndex::NULL, Edge::default());
        }

        let node = &self[start];

        for action in node.actions().iter() {
            let child_index = action.node_index();
            let mut child_board = board.clone();

            child_board.make_move::<STM_WHITE, NSTM_WHITE>(action.mv());

            let (idx, edge) =
                self.recurse_find::<F, NSTM_WHITE, STM_WHITE>(child_index, &child_board, action.clone(), depth - 1, method);

            if !idx.is_null() {
                return (idx, edge);
            }
        }

        (NodeIndex::NULL, Edge::default())
    }
}