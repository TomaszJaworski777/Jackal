use super::{
    print::SearchDisplay,
    search_limits::SearchLimits,
    tree::{Edge, GameState, NodeIndex},
    SearchHelpers, SearchStats, SearchTree,
};
use crate::options::EngineOptions;
use spear::{ChessPosition, Move, Side};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

pub struct Mcts<'a> {
    root_position: ChessPosition,
    tree: &'a SearchTree,
    interruption_token: &'a AtomicBool,
    options: &'a EngineOptions,
    stats: &'a SearchStats,
    limits: &'a SearchLimits,
}

impl<'a> Mcts<'a> {
    pub fn new(
        root_position: ChessPosition,
        tree: &'a SearchTree,
        interruption_token: &'a AtomicBool,
        options: &'a EngineOptions,
        stats: &'a SearchStats,
        limits: &'a SearchLimits,
    ) -> Self {
        Self {
            root_position,
            tree,
            interruption_token,
            options,
            stats,
            limits,
        }
    }

    pub fn search<PRINTER: SearchDisplay>(&self) -> (Move, f64) {
        PRINTER::print_search_start(self.stats, self.options, self.limits);

        //Check if root node is expanded, and if not then expand it
        let root_index = self.tree.root_index();
        if !self.tree[root_index].has_children() {
            let side_to_move = self.root_position.board().side_to_move();
            if side_to_move == Side::WHITE {
                self.expand::<true, false, true>(root_index, &self.root_position)
            } else {
                self.expand::<false, true, true>(root_index, &self.root_position)
            }
        }

        //Start mcts search loop
        if self.root_position.board().side_to_move() == Side::WHITE {
            self.main_loop::<PRINTER, true, false>()
        } else {
            self.main_loop::<PRINTER, false, true>()
        }

        let (best_move, best_score) = self.tree.get_best_move(root_index);
        self.stats.update_time_passed();
        PRINTER::print_search_raport(
            self.stats,
            self.options,
            self.limits,
            best_score,
            self.tree[self.tree.root_index()].state(),
            &self.tree.get_pv(),
        );
        PRINTER::print_search_result(best_move, best_score);
        (best_move, best_score)
    }

    fn main_loop<PRINTER: SearchDisplay, const STM_WHITE: bool, const NSTM_WHITE: bool>(&self) {
        let mut last_raport_time = Instant::now();
        let mut last_avg_depth = 0;
        loop {
            //Start tree desent
            let mut depth = 0;
            let mut position = self.root_position;
            let root_index = self.tree.root_index();
            self.process_deeper_node::<STM_WHITE, NSTM_WHITE, true>(
                root_index,
                NodeIndex::NULL,
                0,
                self.tree.root_edge(),
                &mut position,
                &mut depth,
            );

            //Increment search stats
            self.stats.add_iteration(depth);

            //Interrupt search when root becomes terminal node, so when there is a force mate on board
            if self.tree[root_index].is_termial() {
                self.interruption_token.store(true, Ordering::Relaxed)
            }

            //Update timer every few iterations to reduce the slowdown caused by obtaining time
            if self.stats.iters() % 128 == 0 {
                self.stats.update_time_passed()
            }

            //Check for end of the search
            if self.limits.is_limit_reached(self.stats, self.options) {
                self.interruption_token.store(true, Ordering::Relaxed)
            }

            //Break out of the search
            if self.interruption_token.load(Ordering::Relaxed) {
                break;
            }

            //draws report when avg_depth increases or if there wasnt any report for 1s
            if self.stats.avg_depth() > last_avg_depth
                || last_raport_time.elapsed().as_secs_f32() > 1.0
            {
                last_avg_depth = last_avg_depth.max(self.stats.avg_depth());
                last_raport_time = Instant::now();
                let (_, best_score) = self.tree.get_best_move(root_index);
                PRINTER::print_search_raport(
                    self.stats,
                    self.options,
                    self.limits,
                    best_score,
                    self.tree[self.tree.root_index()].state(),
                    &self.tree.get_pv(),
                )
            }
        }
    }

    fn process_deeper_node<const STM_WHITE: bool, const NSTM_WHITE: bool, const ROOT: bool>(
        &self,
        current_node_index: NodeIndex,
        edge_node_index: NodeIndex,
        action_index: usize,
        action_cpy: Edge,
        current_position: &mut ChessPosition,
        depth: &mut u32,
    ) -> f32 {
        //If current non-root node is terminal or it's first visit, we don't want to go deeper into the tree
        //therefore we just evaluate the node and thats where recursion ends
        let mut new_node_state = GameState::Unresolved;
        let score = if !ROOT
            && (self.tree[current_node_index].is_termial() || action_cpy.visits() == 0)
        {
            SearchHelpers::get_node_score::<STM_WHITE, NSTM_WHITE>(
                current_position,
                self.tree[current_node_index].state(),
            )
        } else {
            //On second visit we expand the node, if it wasn't already expanded.
            //This allows us to reduce amount of time we evaluate policy net
            if !self.tree[current_node_index].has_children() {
                self.expand::<STM_WHITE, NSTM_WHITE, false>(current_node_index, current_position)
            }

            //We then select the best action to evaluate and advance the position to the move of this action
            if !self.tree[current_node_index].has_children() {
                current_position.board().draw_board()
            }
            let best_action_index = self.select_action::<ROOT>(
                current_node_index,
                action_cpy.visits(),
                self.options.cpuct_value(),
            );
            let new_edge_cpy = self
                .tree
                .get_edge_clone(current_node_index, best_action_index);
            current_position.make_move::<STM_WHITE, NSTM_WHITE>(new_edge_cpy.mv());

            //Update the node on the tree
            let new_node_index = if !new_edge_cpy.index().is_null() {
                new_edge_cpy.index()
            } else {
                self.tree
                    .spawn_node(SearchHelpers::get_position_state::<NSTM_WHITE, STM_WHITE>(
                        current_position,
                    ))
            };
            self.tree
                .change_edge_node_index(current_node_index, best_action_index, new_node_index);

            //Desent deeper into the tree
            *depth += 1;
            let score = self.process_deeper_node::<NSTM_WHITE, STM_WHITE, false>(
                new_node_index,
                current_node_index,
                best_action_index,
                new_edge_cpy,
                current_position,
                depth,
            );

            //This line is reached then desent is over and now scores are backpropagated
            //up the tree. Now we can read the state of the node and it will be taking into
            //consideration state backpropagated deeper in the tree
            new_node_state = self.tree[new_node_index].state();
            score
        };

        //Inverse the score to adapt to side to move perspective.
        //MCTS always selects highest score move, and our opponents wants
        //to select worst move for us, so we have to alternate score as we
        //backpropagate it up the tree
        let score = 1.0 - score;
        self.tree
            .add_edge_score::<ROOT>(edge_node_index, action_index, score);

        //Backpropagate the terminal score up the tree
        self.tree
            .backpropagate_mates(current_node_index, new_node_state);

        score
    }

    pub fn expand<const STM_WHITE: bool, const NSTM_WHITE: bool, const ROOT: bool>(
        &self,
        node_index: NodeIndex,
        position: &ChessPosition,
    ) {
        let mut actions = self.tree[node_index].actions_mut();

        //Map moves into actions and set initial policy to 1
        position
            .board()
            .map_moves::<_, STM_WHITE, NSTM_WHITE>(|mv| actions.push(Edge::new(NodeIndex::NULL, mv, 1.0)));

        //Update the policy to 1/action_count for uniform policy
        let action_count = actions.len() as f32;
        for action in actions.iter_mut() {
            action.update_policy(1.0 / action_count)
        }
    }

    //PUCT formula V + C * P * (N.max(1).sqrt()/n + 1) where N = number of visits to parent node, n = number of visits to a child
    #[inline]
    pub fn select_action<const ROOT: bool>(
        &self,
        node_index: NodeIndex,
        visits_to_parent: u32,
        cpuct: f32,
    ) -> usize {
        let explore_value = cpuct * (visits_to_parent.max(1) as f32).sqrt();
        self.tree.get_best_action_by_key(node_index, |action| {
            let visits = action.visits();
            let score = if visits == 0 {
                0.5
            } else {
                action.score() as f32
            };

            score + (explore_value * action.policy() / (visits as f32 + 1.0))
        })
    }
}
