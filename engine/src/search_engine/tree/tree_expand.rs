use chess::{ChessBoard, Move, Piece};

use crate::{NodeIndex, PolicyNetwork, Tree, networks::EndgamePolicyNetwork, search_engine::engine_options::EngineOptions};

impl Tree {
    pub fn expand_node(&self, node_idx: NodeIndex, board: &ChessBoard, engine_options: &EngineOptions) -> Option<()> {
        let children_idx = self[node_idx].children_index_mut();

        if self[node_idx].children_count() > 0 {
            return Some(());
        }

        assert_eq!(
            self[node_idx].children_count(),
            0,
            "Node {node_idx} already have children."
        );

        let mut policy = Vec::with_capacity(board.occupancy().pop_count() as usize);
        let mut max = f64::NEG_INFINITY;
        let mut total = 0f64;

        let pst = if node_idx == self.root_index() {
            engine_options.root_pst()
        } else {
            engine_options.base_pst()
        };

        let endgame = board.phase() <= 8;

        if endgame {
            let policy_base = EndgamePolicyNetwork.create_base(board);

            board.map_legal_moves(|mv| {
                let see = board.see(mv, -108);
                let mut p = EndgamePolicyNetwork.forward(board, &policy_base, mv, see, engine_options.chess960()) as f64 + usize::from(!see) as f64 * mva_lvv(mv, board, engine_options);
                p += self.butterfly_history().get_bonus(board.side(), mv, engine_options);
                policy.push((mv, p));
                max = max.max(p);
            });
        } else {
            let policy_base = PolicyNetwork.create_base(board);

            board.map_legal_moves(|mv| {
                let see = board.see(mv, -108);
                let mut p = PolicyNetwork.forward(board, &policy_base, mv, see, engine_options.chess960()) as f64 + usize::from(!see) as f64 * mva_lvv(mv, board, engine_options);
                p += self.butterfly_history().get_bonus(board.side(), mv, engine_options);
                policy.push((mv, p));
                max = max.max(p);
            });
        }

        let start_index = self.current_half().reserve_nodes(policy.len())?;

        for (_, p) in policy.iter_mut() {
            *p = ((*p - max)/pst).exp();
            total += *p;
        }

        children_idx.store(start_index);
        self[node_idx].set_children_count(policy.len());

        policy.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut squares = 0.0;
        for (idx, &(mv, p)) in policy.iter().enumerate() {
            let p = if policy.len() == 1 {
                1.0
            } else {
                 p / total
            };

            self[start_index + idx].clear(mv);
            self[start_index + idx].set_policy(p);

            squares += p * p;
        }

        let gini_impurity = (1.0 - squares).clamp(0.0, 1.0);
        self[node_idx].set_gini_impurity(gini_impurity);

        Some(())
    }

    const RELABEL_DEPTH: u8 = 1;
    pub fn relabel_root(&self, board: &ChessBoard, engine_options: &EngineOptions) {
        self.recurse_relabel(self.root_index(), Self::RELABEL_DEPTH, board, engine_options);
    }

    fn recurse_relabel(&self, node_idx: NodeIndex, depth: u8, board: &ChessBoard, engine_options: &EngineOptions) {
        if depth == 0 {
            return;
        }

        self.relabel_node(node_idx, Self::RELABEL_DEPTH + 1 - depth, board, engine_options);

        let mask = board.castle_rights().get_castle_mask();
        self[node_idx].map_children(|child_idx| {
            let mut board_copy = board.clone();
            board_copy.make_move(self[child_idx].mv(), &mask);

            self.recurse_relabel(child_idx, depth - 1, &board_copy, engine_options);
        });
    }

    fn relabel_node(&self, node_idx: NodeIndex, _depth: u8, board: &ChessBoard, engine_options: &EngineOptions) {
        let children_idx = self[node_idx].children_index();

        if self[node_idx].children_count() == 0 {
            return;
        }

        let pst = if node_idx == self.root_index() {
            engine_options.root_pst()
        } else {
            engine_options.base_pst()
        };

        let mut policy = Vec::with_capacity(board.occupancy().pop_count() as usize);
        let mut max = f64::NEG_INFINITY;
        let mut total = 0f64;

        let endgame = board.phase() <= 8;

        if endgame {
            let policy_base = EndgamePolicyNetwork.create_base(board);

            self[node_idx].map_children(|child_idx| {
                let mv = self[child_idx].mv();
                let see = board.see(mv, -108);
                let mut p = EndgamePolicyNetwork.forward(board, &policy_base, mv, see, engine_options.chess960()) as f64 + usize::from(!see) as f64 * mva_lvv(mv, board, engine_options);
                p += self.butterfly_history().get_bonus(board.side(), mv, engine_options);
                policy.push(p);
                max = max.max(p);
            });
        } else {
            let policy_base = PolicyNetwork.create_base(board);

            self[node_idx].map_children(|child_idx| {
                let mv = self[child_idx].mv();
                let see = board.see(mv, -108);
                let mut p = PolicyNetwork.forward(board, &policy_base, mv, see, engine_options.chess960()) as f64 + usize::from(!see) as f64 * mva_lvv(mv, board, engine_options);
                p += self.butterfly_history().get_bonus(board.side(), mv, engine_options);
                policy.push(p);
                max = max.max(p);
            });
        };

        for p in policy.iter_mut() {
            *p = ((*p - max)/pst).exp();
            total += *p;
        }

        let mut squares = 0.0;
        for (idx, p) in policy.iter().enumerate() {
            let p = if policy.len() == 1 {
                1.0
            } else {
                p / total
            };

            self[children_idx + idx].set_policy(p as f64);
            squares += p * p;
        }

        let gini_impurity = (1.0 - squares).clamp(0.0, 1.0);
        self[node_idx].set_gini_impurity(gini_impurity);
    }
}

fn mva_lvv(mv: Move, board: &ChessBoard, options: &EngineOptions) -> f64 {
    let attacker = board.piece_on_square(mv.from_square());
    let victim = board.piece_on_square(mv.to_square());

    if !mv.is_capture() || victim == Piece::NONE || attacker == Piece::KING {
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
        options.sac_queen_value()
        ];

    ((piece_values[usize::from(attacker)] - piece_values[usize::from(victim)]) * (options.policy_sac() as f64 / 10000.0)).max(0.0)
}