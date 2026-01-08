use chess::ChessPosition;

use crate::{search_engine::engine_params::EngineParams, NodeIndex, Tree};

impl Tree {
    pub fn try_reuse(&self, position: &ChessPosition, target: &ChessPosition, params: &EngineParams) -> Option<()> {
        if position.board().hash() == target.board().hash() {
            return Some(())
        }
       
        let result = self.find_node(self.root_index(), position, target, 2);
        if result.is_none() {
            self.clear();
            return None;
        }

        let result = result.unwrap();

        let new_root = &self[result];
        let children_idx = new_root.children_index();
        let count = new_root.children_count();

        if children_idx.is_null() {
            self.clear();
            return None;
        }

        let old_root_children_idx = self.root_node().children_index();

        self[self.root_index()].set_to(new_root);
        self[self.root_index()].set_children_count(count);

        self.copy_across(children_idx, count, old_root_children_idx);

        self.relabel_root(target.board(), params);

        Some(())
    }

    fn find_node(&self, node_idx: NodeIndex, position: &ChessPosition, target: &ChessPosition, depth: u8) -> Option<NodeIndex> {
        if position.board().hash() == target.board().hash() {
            return Some(node_idx)
        }

        if depth == 0 {
            return None
        }

        let mut children = Vec::new();
        self[node_idx].map_children(|child_idx| {
            children.push(child_idx)
        });

        children.sort_by_key(|&a| u32::MAX - self[a].visits());

        let mask = position.board().castle_rights().get_castle_mask();
        for child_idx in children {
            let mut new_pos = position.clone();
            new_pos.make_move(self[child_idx].mv(), &mask);
            let result = self.find_node(child_idx, &new_pos, target, depth - 1);
            if result.is_none() {
                continue
            }

            return result
        }

        None
    }
}