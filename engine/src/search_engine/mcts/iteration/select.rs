use crate::{search_engine::{engine_options::EngineOptions, tree::NodeIndex}, Node, SearchEngine, WDLScore};

impl SearchEngine {
    pub(super) fn select(&self, node_idx: NodeIndex, depth: f64) -> NodeIndex {
        let parent_node = &self.tree()[node_idx];

        let cpuct = get_cpuct(&self.options(), &parent_node, depth);

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

        #[cfg(feature = "policy_datagen")] {
            limit = parent_node.children_count()
        }

        let draw_score = if depth as i64 % 2 == 0 { 0.5 } else { self.options().draw_score() as f64 / 100.0 };
        self.tree().select_child_by_key_with_limit(node_idx, limit, |child_node| {
            let score = get_score(&parent_node.score(), child_node, child_node.visits()).single_with_score(draw_score) as f64;
            score + child_node.policy() * cpuct * f64::from(child_node.visits() + 1).recip()
        }).expect("Failed to select a valid node.")
    }
}

#[inline(always)]
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
    let mut cpuct = options.end_cpuct() + (options.start_cpuct() - options.end_cpuct()) * exp_f64(-options.cpuct_depth_decay() * (depth - 1.0));

    let visit_scale = options.cpuct_visit_scale();
    cpuct *= 1.0 + ln_f64((f64::from(parent_node.visits()) + visit_scale) / visit_scale);

    if parent_node.visits() > 1 {
        let var = (parent_node.squared_score() - (parent_node.score().single() as f64).powi(2)).max(0.0);
        let mut variance = sqrt_f64(var) / options.cpuct_variance_scale();
        variance += (1.0 - variance) / (1.0 + options.cpuct_var_warmup() * parent_node.visits() as f64);
        cpuct *= 1.0 + options.cpuct_variance_weight() * (variance - 1.0);
    }

    cpuct *= exp_f64(options.exploration_tau() * ln_f64(parent_node.visits().max(1) as f64));
    cpuct *= (options.gini_base() - options.gini_multiplier() * ln_f64(parent_node.gini_impurity() + 0.001)).max(options.gini_min()).min(options.gini_max());

    cpuct
}

#[inline(always)]
fn ln_f64(x: f64) -> f64 { fastapprox::fast::ln(x as f32) as f64 }
#[inline(always)]
fn exp_f64(x: f64) -> f64 { fastapprox::fast::exp(x as f32) as f64 }
#[inline(always)]
fn sqrt_f64(x: f64) -> f64 { <f32 as micromath::F32Ext>::sqrt(x as f32) as f64 }