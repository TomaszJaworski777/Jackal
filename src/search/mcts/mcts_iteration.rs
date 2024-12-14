use core::f32;

use spear::ChessPosition;

use crate::search::{tree::Edge, NodeIndex, Score, SearchHelpers};

use super::Mcts;

impl<'a> Mcts<'a> {
    pub(super) fn process_deeper_node<
        const STM_WHITE: bool,
        const NSTM_WHITE: bool,
        const ROOT: bool,
        const US: bool,
        const NOT_US: bool,
    >(
        &self,
        current_node_index: NodeIndex,
        action_cpy: &Edge,
        current_position: &mut ChessPosition,
        depth: &mut u32,
    ) -> Option<Score> {
        //If current non-root node is terminal or it's first visit, we don't want to go deeper into the tree
        //therefore we just evaluate the node and thats where recursion ends
        let score = if !ROOT
            && (self.tree[current_node_index].is_terminal() || action_cpy.visits() == 0)
        {
            let current_material = Self::calculate_stm_material(
                &current_position,
                self.root_position.board().side_to_move(),
            );
            SearchHelpers::get_node_score::<STM_WHITE, NSTM_WHITE, US>(
                current_position,
                self.tree[current_node_index].state(),
                self.tree[current_node_index].key(),
                self.tree,
                self.start_material - current_material,
                self.options,
                self.contempt_parms,
            )
        } else {
            //On second visit we expand the node, if it wasn't already expanded.
            //This allows us to reduce amount of time we evaluate policy net
            if !self.tree[current_node_index].has_children() {
                self.tree[current_node_index]
                    .expand::<STM_WHITE, NSTM_WHITE, false>(current_position, self.options)
            }

            //Calculate asymetrical draw contempt
            let draw_score = if US {
                self.options.draw_score()
            } else {
                self.options.draw_score_opp()
            };

            //We then select the best action to evaluate and advance the position to the move of this action
            let best_action_index = self.select_action::<ROOT>(
                current_node_index,
                action_cpy,
                draw_score
            );

            let new_edge_cpy = self
                .tree
                .get_edge_clone(current_node_index, best_action_index);
            current_position.make_move::<STM_WHITE, NSTM_WHITE>(new_edge_cpy.mv());

            //Process the new action on the tree and obtain it's updated index
            let new_node_index = self.tree.get_node_index::<NSTM_WHITE, STM_WHITE>(
                current_position,
                new_edge_cpy.node_index(),
                current_node_index,
                best_action_index,
            )?;

            //Increase amount of threads visiting this node
            self.tree[new_node_index].inc_threads();

            //Descend deeper into the tree
            *depth += 1;
            let opt_score = self.process_deeper_node::<NSTM_WHITE, STM_WHITE, false, NOT_US, US>(
                new_node_index,
                &new_edge_cpy,
                current_position,
                depth,
            );

            //When thread leaves the node, decrease the counter
            self.tree[new_node_index].dec_threads();
            let score = opt_score?;

            //Backpropagate the score up the tree
            self.tree
                .add_edge_score(current_node_index, best_action_index, score);

            //Backpropagate mates to assure our engine avoids/follows mating line
            self.tree
                .backpropagate_mates(current_node_index, self.tree[new_node_index].state());

            score
        };

        Some(score.reversed())
    }

    //PUCT formula V + C * P * (N.max(1).sqrt()/n + 1) where N = number of visits to parent node, n = number of visits to a child
    #[inline]
    fn select_action<const ROOT: bool>(
        &self,
        node_idx: NodeIndex,
        parent: &Edge,
        draw_score: f32
    ) -> usize {
        assert!(self.tree[node_idx].has_children());

        let parent_visits = parent.visits();

        let mut cpuct = if ROOT {
            self.options.root_cpuct_value()
        } else {
            self.options.cpuct_value()
        };

        //Variance scaling
        if parent_visits > 1 {
            let frac = parent.variance(draw_score).sqrt() / self.options.cpuct_variance_scale();
            cpuct *= 1.0 + self.options.cpuct_variance_weight() * (frac - 1.0);
        }

        let scale = self.options.cpuct_visits_scale() * 128.0;
        cpuct *= 1.0 + ((parent_visits as f32 + scale) / scale).ln();

        //Exploration scaling with visits and gini impurity
        let mut explore_scale =
            (self.options.exploration_tau() * (parent_visits.max(1) as f32).ln()).exp();
        explore_scale *=
            (0.679 - 1.634 * (self.tree[node_idx].gini_impurity() + 0.001).ln()).min(2.1);

        let explore_value = cpuct * explore_scale;
        self.tree[node_idx].get_best_action_by_key(|action| {
            let visits = action.visits();

            let mut score = if visits == 0 {
                parent.score().reversed()
            } else {
                action.score()
            };

            //Virtual loss
            let idx = action.node_index();
            if !idx.is_null() {
                let thrds = f64::from(self.tree[idx].threads());
                let v = f64::from(visits);

                if thrds > 0.0 {
                    //Score adjusted by the amount of thread visits
                    let w = f64::from(score.win_chance()) * v / (v + thrds);
                    let d = f64::from(score.draw_chance()) * v / (v + thrds);
                    score = Score::new(w as f32, d as f32);
                }
            }

            let score = score.single(draw_score);
            score + (explore_value * action.policy() / (visits as f32 + 1.0))
        })
    }
}
