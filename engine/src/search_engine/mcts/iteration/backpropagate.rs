use chess::ZobristKey;

use crate::{search_engine::tree::{NodeIndex, Tree}, GameState, SearchEngine, WDLScore};

impl SearchEngine {
    pub(super) fn backpropagate(&self, node_idx: NodeIndex, child_idx: Option<NodeIndex>, score: WDLScore, key: ZobristKey) {
        self.tree().add_visit(node_idx, score);
        backprop_state(self.tree(), node_idx, child_idx);
        self.tree().hash_table().push(key, score.reversed());
    }
}

fn backprop_state(tree: &Tree, node_idx: NodeIndex, child_idx: Option<NodeIndex>) -> Option<()> {
    let child_idx = child_idx?;

    match tree[child_idx].state() {
        GameState::Loss(len) => {
            tree.set_state(node_idx, GameState::Win(len + 1));
        },
        GameState::Win(len) => {
            let mut proven_loss = true;
            let mut proven_loss_length = len;

            tree[node_idx].map_children(|child_idx| {
                if let GameState::Win(x) = tree[child_idx].state() {
                    proven_loss_length = x.max(proven_loss_length);
                } else {
                    proven_loss = false;
                }
            });

            if proven_loss {
                tree.set_state(node_idx, GameState::Loss(proven_loss_length + 1));
            }
        },
        _ => (),
    }

    Some(())
}