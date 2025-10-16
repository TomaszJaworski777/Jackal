use crate::{search_engine::{engine_options::EngineOptions, tree::NodeIndex}, Node, SearchEngine, WDLScore};

impl SearchEngine {
    pub(super) fn select(&self, node_idx: NodeIndex, depth: f64, nodes_left: u64) -> NodeIndex {
        let parent_node = &self.tree()[node_idx];

        let cpuct = get_cpuct(&self.options(), &parent_node, depth);
        let exploration_scale = get_exploration_scale(self.options(), &parent_node);

        let expl = cpuct * exploration_scale;

        let start_idx = parent_node.children_index();
        let mut total_policy = 0.0;
        let mut k = 0;
        while k < parent_node.children_count() && total_policy < self.options().policy_percentage() {
            total_policy += self.tree[start_idx + k].policy();
            k += 1;
        }

        let mut limit = k.max(self.options().min_policy_actions() as usize);
        if parent_node.visits() >= self.options().initial_visit_threshold() as u32 {
            limit += (self.options().visit_increase_multi() * (parent_node.visits() as f64).log2() - self.options().visit_increase_offset()).floor() as usize;
        }

        #[allow(unused_assignments)]{
            limit = limit.min(parent_node.children_count());
        }

        #[cfg(feature = "datagen")] {
            limit = parent_node.children_count()
        }

        let best_score = {
            let mut best_score = f64::MIN;

            self.tree()[node_idx].map_children_with_limit(limit, |child_idx| {
                let child_node = &self.tree()[child_idx];
                best_score = best_score.max(child_node.score().single())
            });

            best_score
        };

        let optimistic_score = (best_score + 0.1).clamp(0.0, 1.0);

        self.tree().select_child_by_key_with_limit(node_idx, limit, |child_node| {
            let child_visits = child_node.visits();
            let optimistic_score = (child_node.score().single() * child_visits as f64 + optimistic_score * nodes_left as f64) / (child_visits as f64 + nodes_left as f64); 

            if optimistic_score <= best_score && !child_node.is_terminal() {
                return -69420.0;
            }

            let score = get_score(&parent_node.score(), child_node, child_visits).single_with_score(if depth as i64 % 2 == 0 {
                0.5
            } else {
                self.options().draw_score() as f64 / 100.0
            }) as f64;
            score + child_node.policy() * expl / f64::from(child_node.visits() + 1)
        }).expect("Failed to select a valid node.")
    }
}

fn get_score(parent_score: &WDLScore, child_node: &Node, child_visits: u32) -> WDLScore {
    let mut score = if child_visits == 0 {
        parent_score.reversed()
    } else {
        child_node.score()
    };

    let threads = f64::from(child_node.threads());
    if threads > 0.0 {
        let v = f64::from(child_visits);
        let w = (score.win_chance() * v) / (v + threads);
        let d = (score.draw_chance() * v) / (v + threads);
        score = WDLScore::new(w, d)
    }

    score
}

fn get_cpuct(options: &EngineOptions, parent_node: &Node, depth: f64) -> f64 {
    let mut cpuct = options.end_cpuct() + (options.start_cpuct() - options.end_cpuct()) * (-options.cpuct_depth_decay() * (depth - 1.0)).exp();

    let visit_scale = options.cpuct_visit_scale();
    cpuct *= 1.0 + ((f64::from(parent_node.visits()) + visit_scale) / visit_scale).ln();

    if parent_node.visits() > 1 {
        let var = (parent_node.squared_score() - (parent_node.score().single() as f64).powi(2)).max(0.0);
        let mut variance = var.sqrt() / options.cpuct_variance_scale();
        variance += (1.0 - variance) / (1.0 + options.cpuct_var_warmup() * parent_node.visits() as f64);
        cpuct *= 1.0 + options.cpuct_variance_weight() * (variance - 1.0);
    }

    cpuct
}

#[allow(unused_mut)]
fn get_exploration_scale(options: &EngineOptions, parent_node: &Node) -> f64 {
    let mut exp = (options.exploration_tau() * (parent_node.visits().max(1) as f64).ln()).exp();

    #[cfg(not(feature = "datagen"))] {
        exp *= (0.463 - 1.567 * (parent_node.gini_impurity() + 0.001).ln()).min(1.5).max(1.0);
    }

    exp
}