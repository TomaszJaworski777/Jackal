use std::sync::{
    atomic::{AtomicU16, AtomicU32, AtomicU64, Ordering},
    RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use crate::spear::{ChessBoard, ChessPosition, Move, Piece, Side};

use crate::{
    search::{tree::Edge, NodeIndex, Score},
    EngineOptions, GameState, PolicyNetwork, Tree,
};

pub struct Node {
    actions: RwLock<Vec<Edge>>,
    state: AtomicU16,
    key: AtomicU64,
    threads: AtomicU16,
    gini_impurity: AtomicU32,
}

impl Default for Node {
    fn default() -> Self {
        Self::new(GameState::Unresolved)
    }
}

impl Node {
    pub fn new(state: GameState) -> Self {
        Self {
            actions: RwLock::new(Vec::new()),
            state: AtomicU16::new(u16::from(state)),
            key: AtomicU64::new(0),
            threads: AtomicU16::new(0),
            gini_impurity: AtomicU32::new(0),
        }
    }

    pub fn clear(&self) {
        self.replace(GameState::Unresolved, 0);
    }

    #[inline]
    pub fn replace(&self, state: GameState, key: u64) {
        *self.actions_mut() = Vec::new();
        self.state.store(u16::from(state), Ordering::Relaxed);
        self.key.store(key, Ordering::Relaxed);
        self.set_gini_impurity(0.0);
    }

    #[inline]
    pub fn state(&self) -> GameState {
        GameState::from(self.state.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn set_state(&self, state: GameState) {
        self.state.store(u16::from(state), Ordering::Relaxed)
    }

    pub fn key(&self) -> u64 {
        self.key.load(Ordering::Relaxed)
    }

    pub fn set_key(&self, key: u64) {
        self.key.store(key, Ordering::Relaxed)
    }

    #[inline]
    pub fn actions(&self) -> RwLockReadGuard<Vec<Edge>> {
        self.actions.read().unwrap()
    }

    #[inline]
    pub fn actions_mut(&self) -> RwLockWriteGuard<Vec<Edge>> {
        self.actions.write().unwrap()
    }

    pub fn threads(&self) -> u16 {
        self.threads.load(Ordering::Relaxed)
    }

    pub fn inc_threads(&self) -> u16 {
        self.threads.fetch_add(1, Ordering::Relaxed)
    }

    pub fn dec_threads(&self) -> u16 {
        self.threads.fetch_sub(1, Ordering::Relaxed)
    }

    pub fn gini_impurity(&self) -> f32 {
        f32::from_bits(self.gini_impurity.load(Ordering::Relaxed))
    }

    pub fn set_gini_impurity(&self, gini_impurity: f32) {
        self.gini_impurity
            .store(f32::to_bits(gini_impurity), Ordering::Relaxed);
    }

    #[inline]
    pub fn has_children(&self) -> bool {
        !self.actions().is_empty()
    }

    #[inline]
    pub fn is_terminal(&self) -> bool {
        self.state() != GameState::Unresolved
    }

    pub fn get_best_move(&self, tree: &Tree, draw_score: f32) -> (Move, Score) {
        //Extracts the best move from all possible root moves
        let action_index = self.get_best_action(tree, draw_score);

        //If no action was selected then return null move
        if action_index == usize::MAX {
            return (Move::NULL, Score::DRAW);
        }

        let edge_clone = &self.actions()[action_index];
        (edge_clone.mv(), edge_clone.score())
    }

    pub fn get_best_action(&self, tree: &Tree, draw_score: f32) -> usize {
        self.get_best_action_by_key(|action| {
            if action.visits() == 0 {
                f32::NEG_INFINITY
            } else if !action.node_index().is_null() {
                match tree[action.node_index()].state() {
                    GameState::Lost(n) => 1.0 + f32::from(n),
                    GameState::Won(n) => f32::from(n) - 256.0,
                    GameState::Drawn => 0.5,
                    GameState::Unresolved => action.score().single(draw_score),
                }
            } else {
                action.score().single(draw_score)
            }
        })
    }

    pub fn get_best_action_by_key<F: FnMut(&Edge) -> f32>(&self, mut method: F) -> usize {
        let mut best_action_index = usize::MAX;
        let mut best_score = f32::NEG_INFINITY;

        for (index, action) in self.actions().iter().enumerate() {
            let score = method(action);
            if score >= best_score {
                best_action_index = index;
                best_score = score;
            }
        }

        best_action_index
    }

    pub fn expand<const STM_WHITE: bool, const NSTM_WHITE: bool, const ROOT: bool>(
        &self,
        position: &ChessPosition,
        options: &EngineOptions,
    ) {
        let mut actions = self.actions_mut();

        let mut inputs: Vec<usize> = Vec::with_capacity(32);
        PolicyNetwork::map_policy_inputs::<_, STM_WHITE, NSTM_WHITE>(position.board(), |idx| {
            inputs.push(idx)
        });

        let vertical_flip = if position.board().side_to_move() == Side::WHITE {
            0
        } else {
            56
        };

        let pst = if ROOT {
            options.root_pst()
        } else {
            options.common_pst()
        };

        let mut max = f32::NEG_INFINITY;
        let mut total = 0.0;

        //Network output cache to prevent processing the same subnet twice
        let mut cache: [Option<Vec<f32>>; 192] = [const { None }; 192];

        const MULTIPLIER: f32 = 1000.0;
        position
            .board()
            .map_moves::<_, STM_WHITE, NSTM_WHITE>(|mv| {
                let policy = PolicyNetwork.forward::<STM_WHITE, NSTM_WHITE>(
                    position.board(),
                    &inputs,
                    mv,
                    vertical_flip,
                    &mut cache,
                ) + mva_lvv(mv, position.board(), options);
                actions.push(Edge::new(
                    NodeIndex::from_raw((policy * MULTIPLIER) as u32),
                    mv,
                    0.0,
                ));
                max = max.max(policy);
            });

        for action in actions.iter_mut() {
            let policy = action.node_index().get_raw() as f32 / MULTIPLIER;
            let policy = ((policy - max) / pst).exp();
            total += policy;
            action.set_node_index(NodeIndex::from_raw((policy * MULTIPLIER) as u32));
        }

        let mut policy_squares = 0.0;
        let is_single_action = actions.len() == 1;
        for action in actions.iter_mut() {
            let policy = action.node_index().get_raw() as f32 / MULTIPLIER;
            let policy = if is_single_action {
                1.0
            } else {
                policy / total
            };
            action.update_policy(policy);
            action.set_node_index(NodeIndex::NULL);
            policy_squares += policy * policy;
        }

        let gini_impurity = (1.0 - policy_squares).clamp(0.0, 1.0);
        self.set_gini_impurity(gini_impurity);
    }

    pub fn recalculate_policy<const STM_WHITE: bool, const NSTM_WHITE: bool, const ROOT: bool>(
        &self,
        position: &ChessPosition,
        options: &EngineOptions,
    ) {
        let mut actions = self.actions_mut();

        let mut inputs: Vec<usize> = Vec::with_capacity(32);
        PolicyNetwork::map_policy_inputs::<_, STM_WHITE, NSTM_WHITE>(position.board(), |idx| {
            inputs.push(idx)
        });

        let vertical_flip = if position.board().side_to_move() == Side::WHITE {
            0
        } else {
            56
        };

        let pst = if ROOT { options.root_pst() } else { 1.0 };

        //Network output cache to prevent processing the same subnet twice
        let mut cache: [Option<Vec<f32>>; 192] = [const { None }; 192];

        let mut policies = Vec::new();
        let mut max: f32 = f32::NEG_INFINITY;
        let mut total = 0.0;

        for action in actions.iter_mut() {
            let policy = PolicyNetwork.forward::<STM_WHITE, NSTM_WHITE>(
                position.board(),
                &inputs,
                action.mv(),
                vertical_flip,
                &mut cache,
            ) + mva_lvv(action.mv(), position.board(), options);
            policies.push(policy);
            max = max.max(policy);
        }

        let is_single_action = actions.len() == 1;
        for policy in policies.iter_mut() {
            if is_single_action {
                *policy = 1.0;
                total = 1.0;
            } else {
                *policy = ((*policy - max) / pst).exp();
                total += *policy;
            }
        }

        let mut policy_squares = 0.0;
        for (i, action) in actions.iter_mut().enumerate() {
            let policy = policies[i] / total;
            action.update_policy(policy);
            policy_squares += policy * policy;
        }

        let gini_impurity = (1.0 - policy_squares).clamp(0.0, 1.0);
        self.set_gini_impurity(gini_impurity);
    }
}

const MVA_LVV_PIECE_VALUES: [f32; 5] = [1.0, 3.0, 3.0, 5.0, 9.0];
fn mva_lvv(mv: Move, board: &ChessBoard, options: &EngineOptions) -> f32 {
    let attacker = board.get_piece_on_square(mv.get_from_square());
    let victim = board.get_piece_on_square(mv.get_to_square());

    if !mv.is_capture() || victim == Piece::NONE || attacker == Piece::KING {
        return 0.0;
    }

    (MVA_LVV_PIECE_VALUES[attacker.get_raw() as usize]
        - MVA_LVV_PIECE_VALUES[victim.get_raw() as usize])
        * options.policy_sac_bonus()
}
