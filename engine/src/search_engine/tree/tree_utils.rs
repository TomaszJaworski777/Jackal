use crate::{GameState, search_engine::tree::{NodeIndex, Tree, node::Node, pv_line::PvLine}};

impl Tree {
    pub fn bytes_to_size(bytes: usize) -> usize {
        bytes / std::mem::size_of::<Node>()
    }

    pub fn size_to_bytes(size: usize) -> usize {
        size * std::mem::size_of::<Node>()
    }

    pub fn select_child_by_key<F: FnMut(&Node) -> f64>(
        &self,
        parent_idx: NodeIndex,
        key: F,
    ) -> Option<NodeIndex> {
        self.select_child_by_key_with_limit(parent_idx, self[parent_idx].children_count(), key)
    }

    pub fn select_child_by_key_with_limit<F: FnMut(&Node) -> f64>(
        &self,
        parent_idx: NodeIndex,
        limit: usize,
        mut key: F,
    ) -> Option<NodeIndex> {
        let mut best_idx = None;
        let mut best_score = f64::NEG_INFINITY;

        self[parent_idx].map_children_with_limit(limit, |child_idx| {
            let new_score = key(&self[child_idx]);
            if new_score > best_score {
                best_idx = Some(child_idx);
                best_score = new_score;
            }
        });

        best_idx
    }

    pub fn select_best_child(&self, parent_idx: NodeIndex, draw_score: f64) -> Option<NodeIndex> {
        self.select_child_by_key(parent_idx, |node| {
            match node.state() {
                GameState::Loss(x) => 256.0 - x as f64,
                GameState::Win(x) => -256.0 + x as f64,
                _ => node.score().single_with_score(draw_score) as f64
            }
        })
    }

    pub fn get_pv(&self, node_idx: NodeIndex, draw_score: f64, flip: bool) -> PvLine {
        let node = &self[node_idx];

        if node.children_count() == 0 {
            return PvLine::new(node);
        }

        let best_child_idx = self.select_best_child(node_idx, draw_score);

        if best_child_idx.is_none() {
            return PvLine::new(node);
        }

        let best_child_idx = best_child_idx.unwrap();

        let mut result = self.get_pv(best_child_idx, if flip {
            0.5
        } else {
            draw_score
        }, !flip);
        result.add_node(node);

        result
    }

    pub fn get_best_pv(&self, index: usize, draw_score: f64) -> PvLine {
        let mut chilren_nodes = Vec::new();
        let root = self.root_node();

        root.map_children(|child_idx| {
            let node = &self[child_idx];

            if node.visits() == 0 {
                return;
            }

            chilren_nodes.push((child_idx, node.score().single_with_score(draw_score)))
        });

        if chilren_nodes.is_empty() {
            return PvLine::new(&Node::new());
        }

        chilren_nodes.sort_by(|(_, a), (_, b)| b.total_cmp(a));

        let (pv_node_idx, _) = chilren_nodes[index.min(chilren_nodes.len() - 1)];
        self.get_pv(pv_node_idx, draw_score, false)
    }

    pub fn find_node_depth(&self, start_node_idx: NodeIndex, target_node_idx: NodeIndex) -> Option<u64> {

        if start_node_idx == target_node_idx {
            return Some(0);
        }

        let node = &self[start_node_idx];

        let mut chilren_nodes = Vec::new();
        let mut found_node = false;

        node.map_children(|child_idx| {
            if child_idx == target_node_idx {
                found_node = true
            }

            if self[child_idx].children_count() == 0 {
                return;
            }

            chilren_nodes.push(child_idx)
        });


        if found_node {
            return Some(1);
        }

        for child_idx in chilren_nodes {
            let result = self.find_node_depth(child_idx, target_node_idx);
            if result.is_none() {
                continue;
            }

            return Some(result.unwrap() + 1);
        }

        return None;
    }
}