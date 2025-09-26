use chess::Move;

use crate::{NodeIndex, Tree};

impl Tree {
    pub fn swap_half(&self) {
        let old_root = self.root_index();
        let old_half = self.current_half.fetch_xor(1, std::sync::atomic::Ordering::Relaxed) as usize;

        self.halves[old_half].clear_references();
        self.halves[old_half ^ 1].clear();

        let new_root = self.halves[old_half ^ 1].reserve_nodes(1).unwrap();
        self[new_root].clear(Move::NULL);

        self.copy_across(old_root, 1, new_root);
    }

    pub fn update_node(&self, node_idx: NodeIndex) -> Option<()> {
        if self[node_idx].children_index().half() == self.current_half_index() {
            return Some(());
        }

        let mut children_idx = self[node_idx].children_index_mut();

        if children_idx.half() == self.current_half_index() {
            return Some(());
        }

        let children_count = self[node_idx].children_count();
        let new_idx = self.current_half().reserve_nodes(children_count)?;

        self.copy_across(*children_idx, children_count, new_idx);

        *children_idx = new_idx;

        Some(())
    }

    pub fn copy_across(&self, from_idx: NodeIndex, count: usize, to_idx: NodeIndex) {
        if from_idx == to_idx {
            return;
        }

        for child_idx in 0..count {
            let from = &self[from_idx + child_idx];
            let to = &self[to_idx + child_idx];

            let from_children = from.children_index_mut();
            let mut to_children = to.children_index_mut();

            to.set_to(&from);
            to.set_children_count(from.children_count());
            *to_children = *from_children;
        }
    }
}