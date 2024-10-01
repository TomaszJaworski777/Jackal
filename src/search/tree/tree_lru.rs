use std::sync::atomic::Ordering;

use spear::ChessPosition;

use crate::{search::SearchHelpers, GameState};

use super::{tree_base::SEGMENT_COUNT, NodeIndex, Tree};

impl Tree {
    pub fn get_node_index<const STM_WHITE: bool, const NSTM_WHITE: bool>(&self, position: &ChessPosition, child_index: NodeIndex, edge_index: NodeIndex, action_index: usize) -> Option<NodeIndex> {

        //When there is no node assigned to the selected move edge, we spawn new node
        //and return its index
        if child_index.is_null() {

            //We spawn a new node and update the corresponding edge. If the segment returned None,
            //then it means segment is full, we return that instantly and process it in the search
            let state = SearchHelpers::get_position_state::<STM_WHITE, NSTM_WHITE>(position);
            let new_index = self.current_segment().add(state)?;

            self[edge_index].actions()[action_index].set_node_index(new_index);

            Some(new_index)

        //When there is a node assigned to the selected move edge, but the assigned
        //node is in old tree segment, we want to copy it to the new tree segment
        } else if child_index.segment() != self.current_segment.load(Ordering::Relaxed) {

            //We get new index from the segment. If the index is None, then segment is
            //full. When that happens we return it instantly and process it in the search
            let new_index = self.current_segment().add(GameState::Unresolved)?;

            //Next, we copy the actions from the old node to the new one and
            self.copy_node(child_index, new_index);
            self[edge_index].actions()[action_index].set_node_index(new_index);

            Some(new_index)

        //When nthere is a node assigned to the selected move edge and it's located
        //in corrected segment, we can just return the index without changes
        } else {
            Some(child_index)
        }
    }

    pub fn advance_segments(&self) {
        let old_root_index = self.root_index();

        let current_segment_index = self.current_segment.load(Ordering::Relaxed);
        let new_segment_index = (current_segment_index + 1) % SEGMENT_COUNT;

        for i in 0..SEGMENT_COUNT {
            if i != new_segment_index {
                self.segments[i].clear_references(new_segment_index as u32);
            }
        }

        self.current_segment
            .store(new_segment_index, Ordering::Relaxed);
        self.segments[new_segment_index].clear();

        let new_root_index = self.segments[new_segment_index].add(GameState::Unresolved).unwrap();
        self[new_root_index].replace(GameState::Unresolved);

        self.copy_node(old_root_index, new_root_index);
    }

    pub(super) fn copy_node(&self, a: NodeIndex, b: NodeIndex) {
        if a == b {
            return;
        }

        let a_actions = &mut *self[a].actions_mut();
        let b_actions = &mut *self[b].actions_mut();
        
        self[b].set_state(self[a].state());

        if a_actions.is_empty() {
            return;
        }

        std::mem::swap(a_actions, b_actions);
    }
}