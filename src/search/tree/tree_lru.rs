use std::sync::atomic::Ordering;

use spear::ChessPosition;

use crate::{search::SearchHelpers, GameState};

use super::{tree_base::SEGMENT_COUNT, NodeIndex, Tree};

impl Tree {
    pub fn get_node_index<const STM_WHITE: bool, const NSTM_WHITE: bool>(
        &self,
        position: &ChessPosition,
        child_index: NodeIndex,
        edge_index: NodeIndex,
        action_index: usize,
    ) -> Option<NodeIndex> {
        //When there is no node assigned to the selected move edge, we spawn new node
        //and return its index
        if child_index.is_null() {
            //Create mutable lock for actions to assure that they are not read or wrote during this process
            let actions = self[edge_index].actions_mut();

            //Create new node in the current segment
            let state = SearchHelpers::get_position_state::<STM_WHITE, NSTM_WHITE>(position);
            let new_index = self.current_segment().add(state, position.board().get_key().get_raw())?;

            //Assign new node index to the edge
            actions[action_index].set_node_index(new_index);

            Some(new_index)

        //When there is a node assigned to the selected move edge, but the assigned
        //node is in old tree segment, we want to copy it to the new tree segment
        } else if child_index.segment() != self.current_segment.load(Ordering::Relaxed) {
            //Create mutable lock for actions to assure that they are not read or wrote during this process
            let actions = self[edge_index].actions_mut();

            //Obtain new node index from the current segment
            let new_index = self.current_segment().add(GameState::Unresolved, 0)?;

            //Copy the node from old location to the new one and check it's index
            //in the edge
            self.copy_node(child_index, new_index);
            actions[action_index].set_node_index(new_index);

            Some(new_index)

        //When nthere is a node assigned to the selected move edge and it's located
        //in corrected segment, we can just return the index without changes
        } else {
            Some(child_index)
        }
    }

    //When segment is full we want to prepare new segment and swap our tree to it
    pub fn advance_segments(&self) {
        let old_root_index = self.root_index();

        //Calculate new segment index
        let current_segment_index = self.current_segment.load(Ordering::Relaxed);
        let new_segment_index = (current_segment_index + 1) % SEGMENT_COUNT;

        //Iterate through segments and kill all pointers to the segment we are about to clear
        for i in 0..SEGMENT_COUNT {
            if i != new_segment_index {
                self.segments[i].clear_references(new_segment_index as u32);
            }
        }

        //Clear the segment
        self.current_segment
            .store(new_segment_index, Ordering::Relaxed);
        self.segments[new_segment_index].clear();

        //Move root to the new segment
        let new_root_index = self.segments[new_segment_index]
            .add(GameState::Unresolved, 0)
            .unwrap();
        self[new_root_index].clear();

        self.copy_node(old_root_index, new_root_index);
    }

    pub(super) fn copy_node(&self, a: NodeIndex, b: NodeIndex) {
        if a == b {
            return;
        }

        let a_actions = &mut *self[a].actions_mut();
        let b_actions = &mut *self[b].actions_mut();

        self[b].set_state(self[a].state());
        self[b].set_key(self[a].key());

        if a_actions.is_empty() {
            return;
        }

        std::mem::swap(a_actions, b_actions);
    }
}
