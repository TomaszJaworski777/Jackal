use crate::{Node, WDLScore};

pub fn get_proof_bonus(parent_score: &WDLScore, parent_node: &Node, child_node: &Node) -> f64 {
    let q_magnitude = (parent_score.single() - 0.5).abs() * 2.0;
    let proof_scale = 0.1 * q_magnitude * q_magnitude;

    if parent_node.visits() > parent_node.children_count() as u32 * 6
        && child_node.visits() > parent_node.visits() / 64
    {
        proof_scale * (parent_node.proof() as f64 / child_node.proof().max(1) as f64).min(1.0)
    } else {
        0.0
    }
}
