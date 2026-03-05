use crate::{
    search_engine::{engine_options::EngineOptions, proof::get_proof_bonus, tree::NodeIndex},
    Node, SearchEngine, WDLScore,
};

impl SearchEngine {
    pub(super) fn select(&self, node_idx: NodeIndex, depth: f64) -> NodeIndex {
        let parent_node = &self.tree()[node_idx];
        let parent_score = parent_node.score().reversed();

        let cpuct = get_cpuct(self.options(), parent_node, depth);

        let start_idx = parent_node.children_index();
        let mut total_policy = 0.0;
        let mut k = 0;
        while k < parent_node.children_count() && total_policy < self.options().policy_percentage()
        {
            total_policy += self.tree[start_idx + k].policy();
            k += 1;
        }

        let mut limit = k.max(self.options().min_policy_actions() as usize);
        if parent_node.visits() >= self.options().initial_visit_threshold() as u32 {
            limit += (self.options().visit_increase_multi() * (parent_node.visits() as f64).log2()
                - self.options().visit_increase_offset())
            .floor() as usize;
        }

        #[allow(unused_assignments)]
        {
            limit = limit.min(parent_node.children_count());
        }

        #[cfg(feature = "datagen")]
        {
            limit = parent_node.children_count()
        }

        let draw_score = if depth as i64 % 2 == 0 {
            0.5
        } else {
            self.options().get_draw_score_blend(parent_node.score())
        };

        let is_stm_parent = depth as i64 % 2 != 0;

        self.tree()
            .select_child_by_key_with_limit(node_idx, limit, |child_node| {
                let score = get_score(
                    &parent_score,
                    child_node,
                    child_node.visits(),
                    child_node.policy(),
                    parent_node.children_count(),
                    self.options(),
                )
                .single_with_score(draw_score) as f64;

                let proof_bonus = if is_stm_parent {
                    get_proof_bonus(&parent_score, parent_node, child_node)
                } else {
                    0.0
                };

                let visit_scale = f64::from(child_node.visits() + 1).recip();

                let exploration_sac_bonus = if child_node.sac_strength() != 0
                    && is_stm_parent
                    && parent_score.single() > 0.51
                {
                    let sac_multiplier = 1.0
                        + (parent_score.single() - 0.75).max(0.0) * self.options().sac_scaling();
                    (self.options().exploration_sac_bonus()
                        + child_node.sac_strength() as f64 / 20000.0)
                        * sac_multiplier
                } else {
                    0.0
                };

                score
                    + child_node.policy() * cpuct * visit_scale
                    + proof_bonus
                    + exploration_sac_bonus
            })
            .expect("Failed to select a valid node.")
    }
}

#[inline(always)]
fn get_score(
    parent_score: &WDLScore,
    child_node: &Node,
    child_visits: u32,
    child_policy: f64,
    children_count: usize,
    options: &EngineOptions,
) -> WDLScore {
    let mut score = if child_visits == 0 {
        let reduction =
            (1.0 - child_policy * children_count as f64).max(0.0) * options.fpu_reduction();
        let w = (parent_score.win_chance() - reduction).max(0.0);
        WDLScore::new(w, parent_score.draw_chance())
    } else {
        child_node.score()
    };

    let threads = f64::from(child_node.threads());
    if threads > 0.0 {
        let v = f64::from(child_visits);
        let w =
            (score.win_chance() * v) / (v + 1.0 + options.virtual_loss_weight() * (threads - 1.0));
        let d =
            (score.draw_chance() * v) / (v + 1.0 + options.virtual_loss_weight() * (threads - 1.0));
        score = WDLScore::new(w, d)
    }

    score
}

fn get_cpuct(options: &EngineOptions, parent_node: &Node, depth: f64) -> f64 {
    let mut cpuct = options.end_cpuct()
        + (options.start_cpuct() - options.end_cpuct())
            * (-options.cpuct_depth_decay() * (depth - 1.0)).exp();

    let visit_scale = options.cpuct_visit_scale();
    cpuct *= 1.0 + ((f64::from(parent_node.visits()) + visit_scale) / visit_scale).ln();

    if parent_node.visits() > 1 {
        let var = (parent_node.squared_score() - parent_node.score().single().powi(2)).max(0.0);
        let mut variance = var.sqrt() / options.cpuct_variance_scale();
        variance +=
            (1.0 - variance) / (1.0 + options.cpuct_var_warmup() * parent_node.visits() as f64);
        cpuct *= 1.0 + options.cpuct_variance_weight() * (variance - 1.0);
    }

    cpuct *= (options.exploration_tau() * (parent_node.visits().max(1) as f64).ln()).exp();
    cpuct *= (options.gini_base()
        - options.gini_multiplier() * (parent_node.gini_impurity() + 0.001).ln())
    .max(options.gini_min())
    .min(options.gini_max());

    cpuct
}
