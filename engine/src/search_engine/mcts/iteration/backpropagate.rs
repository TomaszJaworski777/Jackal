use chess::ZobristKey;

use crate::{
    search_engine::tree::{NodeIndex, Tree},
    GameState, SearchEngine, WDLScore,
};

impl SearchEngine {
    pub(super) fn backpropagate(
        &self,
        node_idx: NodeIndex,
        child_idx: Option<NodeIndex>,
        score: WDLScore,
        key: ZobristKey,
        depth: f64,
    ) {
        self.tree().add_visit(node_idx, score);
        backprop_state(self.tree(), node_idx, child_idx);
        backprop_proof(self.tree(), node_idx, depth as i64 % 2 == 0);
        self.tree().hash_table().push(key, score.reversed());
    }
}

fn backprop_state(tree: &Tree, node_idx: NodeIndex, child_idx: Option<NodeIndex>) -> Option<()> {
    let child_idx = child_idx?;

    match tree[child_idx].state() {
        GameState::Loss(len) => {
            tree.set_state(node_idx, GameState::Win(len + 1));
        }
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
        }
        _ => (),
    }

    Some(())
}

fn backprop_proof(tree: &Tree, node_idx: NodeIndex, is_stm: bool) -> Option<()> {
    let node = &tree[node_idx];

    if node.proof() == 0 {
        return Some(());
    }

    match node.state() {
        GameState::Win(_) => {
            if is_stm {
                node.set_proof(u16::MAX);
            } else {
                node.set_proof(0);
            }
            return Some(());
        }
        GameState::Loss(_) => {
            if is_stm {
                node.set_proof(0);
            } else {
                node.set_proof(u16::MAX);
            }
            return Some(());
        }
        GameState::Draw => {
            node.set_proof(u16::MAX);
            return Some(());
        }
        _ => {}
    }

    let mut min_child_proof = u16::MAX;
    let mut sum_child_proof = 0u16;

    node.map_children(|child_idx| {
        let child = &tree[child_idx];
        let child_proof = child.proof();

        if child_proof < min_child_proof {
            min_child_proof = child_proof;
        }

        sum_child_proof = sum_child_proof.saturating_add(child_proof);
    });

    if node.children_count() > 0 {
        if is_stm {
            node.set_proof(min_child_proof);
        } else {
            node.set_proof(sum_child_proof);
        }
    }

    Some(())
}
