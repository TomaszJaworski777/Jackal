use chess::{ChessBoard, Move, Piece};

use crate::{
    search_engine::engine_options::EngineOptions, BasePolicyNetwork, NodeIndex,
    Stage1PolicyNetwork, Stage2PolicyNetwork, Stage3PolicyNetwork, Tree, WDLScore,
};

impl Tree {
    pub fn expand_node(
        &self,
        node_idx: NodeIndex,
        board: &ChessBoard,
        engine_options: &EngineOptions,
        depth: i32,
        parent_score: WDLScore,
    ) -> Option<()> {
        let children_idx = self[node_idx].children_index_mut();

        if self[node_idx].children_count() > 0 {
            return Some(());
        }

        assert_eq!(
            self[node_idx].children_count(),
            0,
            "Node {node_idx} already have children."
        );

        let network = if depth % 2 == 1 && board.phase() > 8 {
            if parent_score.win_chance() > 0.9 {
                &BasePolicyNetwork
            } else if parent_score.win_chance() > 0.575 {
                &Stage3PolicyNetwork
            } else if parent_score.win_chance() > 0.325 {
                &Stage2PolicyNetwork
            } else {
                &Stage1PolicyNetwork
            }
        } else {
            &BasePolicyNetwork
        };

        let policy_base = network.create_base(board);

        let pst = if node_idx == self.root_index() {
            engine_options.root_pst()
        } else {
            engine_options.base_pst()
        };

        let mut policy = Vec::with_capacity(board.occupancy().pop_count() as usize);
        let mut max = f64::NEG_INFINITY;
        let mut total = 0f64;

        board.map_legal_moves(|mv| {
            let see_108 = board.see(mv, -108);
            let mut p =
                network.forward(board, &policy_base, mv, see_108, engine_options.chess960()) as f64;

            let mva = mva_lvv(mv, board, engine_options);

            let is_sacrifice = if !see_108 {
                true
            } else if mva > 0.0 && mva <= 108.0 {
                !board.see(mv, 0)
            } else {
                false
            };

            let policy_bonus = usize::from(!see_108) as f64 * mva * engine_options.policy_sac();
            let history_bonus =
                self.butterfly_history()
                    .get_bonus(board.side(), mv, engine_options);
            let sac_strength = if is_sacrifice {
                (mva / 5.0).round() as u8
            } else {
                0
            };

            p += policy_bonus + history_bonus;

            policy.push((mv, p, sac_strength));
            max = max.max(p);
        });

        let start_index = self.current_half().reserve_nodes(policy.len())?;

        for (_, p, _) in policy.iter_mut() {
            *p = ((*p - max) / pst).exp();
            total += *p;
        }

        children_idx.store(start_index);
        self[node_idx].set_children_count(policy.len());

        policy.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut squares = 0.0;
        for (idx, &(mv, p, sac_strength)) in policy.iter().enumerate() {
            let p = if policy.len() == 1 { 1.0 } else { p / total };

            self[start_index + idx].clear(mv);
            self[start_index + idx].set_policy(p);
            self[start_index + idx].set_sac_strength(sac_strength);

            squares += p * p;
        }

        let gini_impurity = (1.0 - squares).clamp(0.0, 1.0);
        self[node_idx].set_gini_impurity(gini_impurity);

        Some(())
    }

    pub fn relabel_root(&self, board: &ChessBoard, engine_options: &EngineOptions) {
        let root_score = if self.root_node().visits() == 0 {
            WDLScore::DRAW
        } else {
            self.root_node().score()
        };

        self.relabel_node(self.root_index(), board, engine_options, 1, root_score);
    }

    fn relabel_node(
        &self,
        node_idx: NodeIndex,
        board: &ChessBoard,
        engine_options: &EngineOptions,
        depth: i32,
        parent_score: WDLScore,
    ) {
        let children_idx = self[node_idx].children_index();

        if self[node_idx].children_count() == 0 {
            return;
        }

        let network = if depth % 2 == 1 && board.phase() > 8 {
            if parent_score.win_chance() > 0.9 {
                &BasePolicyNetwork
            } else if parent_score.win_chance() > 0.575 {
                &Stage3PolicyNetwork
            } else if parent_score.win_chance() > 0.325 {
                &Stage2PolicyNetwork
            } else {
                &Stage1PolicyNetwork
            }
        } else {
            &BasePolicyNetwork
        };

        let policy_base = network.create_base(board);

        let pst = if node_idx == self.root_index() {
            engine_options.root_pst()
        } else {
            engine_options.base_pst()
        };

        let mut policy = Vec::with_capacity(board.occupancy().pop_count() as usize);
        let mut max = f64::NEG_INFINITY;
        let mut total = 0f64;

        self[node_idx].map_children(|child_idx| {
            let mv = self[child_idx].mv();

            let see_108 = board.see(mv, -108);
            let mut p =
                network.forward(board, &policy_base, mv, see_108, engine_options.chess960()) as f64;

            let mva = mva_lvv(mv, board, engine_options);

            let is_sacrifice = if !see_108 {
                true
            } else if mva > 0.0 && mva <= 108.0 {
                !board.see(mv, 0)
            } else {
                false
            };

            let policy_bonus = usize::from(!see_108) as f64 * mva * engine_options.policy_sac();
            let history_bonus =
                self.butterfly_history()
                    .get_bonus(board.side(), mv, engine_options);
            let sac_strength = if is_sacrifice {
                (mva / 5.0).round() as u8
            } else {
                0
            };

            p += policy_bonus + history_bonus;

            policy.push((p, sac_strength));
            max = max.max(p);
        });

        for (p, _) in policy.iter_mut() {
            *p = ((*p - max) / pst).exp();
            total += *p;
        }

        let mut squares = 0.0;
        for (idx, &(p, sac_strength)) in policy.iter().enumerate() {
            let p = if policy.len() == 1 { 1.0 } else { p / total };

            self[children_idx + idx].set_policy(p);
            self[children_idx + idx].set_sac_strength(sac_strength);
            squares += p * p;
        }

        let gini_impurity = (1.0 - squares).clamp(0.0, 1.0);
        self[node_idx].set_gini_impurity(gini_impurity);
    }
}

fn mva_lvv(mv: Move, board: &ChessBoard, options: &EngineOptions) -> f64 {
    let attacker = board.piece_on_square(mv.from_square());
    let victim = board.piece_on_square(mv.to_square());

    if attacker == Piece::KING {
        return 0.0;
    }

    if board.phase() <= 8 {
        return 0.0;
    }

    let piece_values = [
        options.sac_pawn_value(),
        options.sac_knight_value(),
        options.sac_bishop_value(),
        options.sac_rook_value(),
        options.sac_queen_value(),
    ];

    if !mv.is_capture() || victim == Piece::NONE {
        piece_values[usize::from(attacker)]
    } else {
        (piece_values[usize::from(attacker)] - piece_values[usize::from(victim)]).max(0.0)
    }
}
