use chess::Move;

use crate::search_engine::tree::node::{Node, WDLScore};

#[derive(Debug, Clone, Default)]
pub struct PvLine(Vec<Node>);
impl PvLine {
    #[inline]
    pub fn new(node: &Node) -> Self {
        Self(vec![node.clone()])
    }

    #[inline]
    pub fn add_node(&mut self, node: &Node) {
        self.0.insert(0, node.clone());
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn first_move(&self) -> Move {
        self.0[0].mv()
    }

    #[inline]
    pub fn first_node(&self) -> Node {
        self.0[0].clone()
    }

    #[inline]
    pub fn score(&self) -> WDLScore {
        self.0[0].score()
    }

    #[inline]
    pub fn to_string(&self, chess960: bool) -> String {
        let mut result = String::new();
        for node in &self.0 {
            result.push_str(node.mv().to_string(chess960).as_str());
            result.push(' ');
        }

        result.trim().to_string()
    }

    #[inline]
    pub fn to_string_wrapped(&self, wrap_length: usize, chess960: bool) -> String {
        let mut result = String::new();
        for (idx, node) in self.0.iter().enumerate() {
            if idx == wrap_length - 1 && idx < self.0.len() - 1 {
                result.push_str(format!("({} more...)", self.0.len() - wrap_length + 1).as_str());
                break;
            }

            result.push_str(node.mv().to_string(chess960).as_str());
            result.push(' ');
        }

        result.trim().to_string()
    }
}