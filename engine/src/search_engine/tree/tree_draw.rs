use utils::{bytes_to_string, heat_color, number_to_string, AlignString, Colors, Theme};

use crate::{SearchEngine, WDLScore, search_engine::{contempt::Contempt, tree::{GameState, NodeIndex, Tree, node::Node}}};

impl Tree {
    pub fn draw_tree<const FLIP_SCORE: bool>(&self, depth: Option<u8>, node_idx: Option<NodeIndex>, search_engine: &SearchEngine) {
        let depth = depth.unwrap_or(1);
        let node_idx = node_idx.unwrap_or(self.root_index());

        let tree_size = self.max_size();
        let current_size = self.current_size().min(tree_size);

        let current_size_nodes = number_to_string(current_size as u128).secondary(0.0);
        let tree_size_nodes = number_to_string(tree_size as u128).secondary(0.0);

        let current_size_mem = format!(
            "{}B",
            bytes_to_string((current_size as usize * std::mem::size_of::<Node>()) as u128)
        )
        .secondary(3.0 / 29.0);
        let tree_size_mem = format!(
            "{}B",
            bytes_to_string((tree_size * std::mem::size_of::<Node>()) as u128)
        )
        .secondary(3.0 / 29.0);

        let usage = current_size as f32 / tree_size as f32;
        let usage = heat_color(
            format!("{:.2}%", usage * 100.0).as_str(),
            1.0 - usage,
            0.0,
            1.0,
        );

        println!(
            "{}",
            format!(
                "\nNodes:  {}{}{}",
                current_size_nodes,
                "/".gray(),
                tree_size_nodes
            )
            .primary(0.0)
        );
        println!(
            "{}",
            format!(
                "Memory: {}{}{}",
                current_size_mem,
                "/".gray(),
                tree_size_mem
            )
            .primary(3.0 / 29.0)
        );
        println!("{}", format!("Usage:  {usage}\n").primary(5.0 / 29.0));

        let node_depth = self.find_node_depth(self.root_index(), node_idx);

        if node_depth.is_none() {
            return;
        }

        let policy = self[node_idx].policy() as f32;

        self.print_branch::<FLIP_SCORE>(
            node_idx,
            0,
            depth,
            String::new(),
            false,
            true,
            node_depth.unwrap() % 2 == 0,
            0,
            20,
            policy,
            policy,
            search_engine
        );
    }

    fn print_branch<const FLIP_SCORE: bool>(
        &self,
        node_idx: NodeIndex,
        depth: u8,
        max_depth: u8,
        mut prefix: String,
        is_last: bool,
        is_root: bool,
        mut flip: bool,
        iter_idx: usize,
        iter_size: usize,
        mut min_policy: f32,
        mut max_policy: f32,
        search_engine: &SearchEngine
    ) {
        self.print_node(
            node_idx,
            &prefix,
            is_root,
            is_last,
            flip && FLIP_SCORE,
            iter_idx,
            iter_size,
            min_policy,
            max_policy,
            search_engine
        );

        if FLIP_SCORE {
            flip = !flip;
        }

        if depth >= max_depth {
            return;
        }

        if !is_root {
            let new_prefix = if is_last { "    " } else { "│   " };
            prefix = format!("{prefix}{new_prefix}")
        }

        let mut children = Vec::new();

        self[node_idx].map_children(|child_idx| {
            if self[child_idx].visits() == 0 {
                return;
            }

            children.push(child_idx);
        });

        children.sort_by(|&a, &b| self[b].visits().cmp(&self[a].visits()));

        min_policy = f32::INFINITY;
        max_policy = f32::NEG_INFINITY;

        for &child_idx in &children {
            let policy = self[child_idx].policy() as f32;
            min_policy = min_policy.min(policy);
            max_policy = max_policy.max(policy);
        }

        for (idx, &child_idx) in (&children).into_iter().enumerate() {
            self.print_branch::<FLIP_SCORE>(
                child_idx,
                depth + 1,
                max_depth,
                prefix.clone(),
                idx + 1 == children.len(),
                false,
                flip,
                idx,
                children.len(),
                min_policy,
                max_policy,
                search_engine
            );
        }
    }

    fn print_node(
        &self,
        node_idx: NodeIndex,
        prefix: &String,
        is_root: bool,
        is_last: bool,
        flip: bool,
        iter_idx: usize,
        iter_size: usize,
        min_policy: f32,
        max_policy: f32,
        search_engine: &SearchEngine
    ) {
        let node = &self[node_idx];
        let color_gradient = (iter_idx + 5) as f32 / (iter_size + 10) as f32;

        let arrow = if is_root {
            ""
        } else {
            if is_last {
                "└─> "
            } else {
                "├─> "
            }
        };

        let prefix = if is_root {
            if node_idx == self.root_index() {
                String::from("root")
            } else {
                node.mv().to_string(search_engine.options().chess960()).align_to_left(5)
            }
            .primary(color_gradient)
        } else {
            format!(
                "{prefix}{arrow}{}{} {}",
                format!("{}", node_idx)
                    .align_to_right(18)
                    .primary(color_gradient),
                ">".secondary(color_gradient),
                node.mv()
                    .to_string(search_engine.options().chess960())
                    .align_to_left(5)
                    .primary(color_gradient)
            )
        }
        .white();

        let score = if flip {
            node.score().reversed()
        } else {
            node.score()
        };

        let mut v = score.win_chance() - score.lose_chance();
        let mut d = score.draw_chance();

        Contempt::rescale(&mut v, &mut d, 1.0, true, search_engine.options());

        let pv_score = WDLScore::new((1.0 + v - d) / 2.0, d);

        let state = if flip {
            match node.state() {
                GameState::Loss(x) => GameState::Win(x),
                GameState::Win(x) => GameState::Loss(x),
                _ => node.state()
            }
        } else {
            node.state()
        };

        let score = match state {
            GameState::Loss(len) => format!("+M{}", (len + 1).div_ceil(2)),
            GameState::Win(len) => format!("-M{}", (len + 1).div_ceil(2)),
            _ => format!("{}{:.2}", if pv_score.single() < 0.5 { "-" } else { "+" }, pv_score.cp().abs() as f32 / 100.0)
        };

        let score = heat_color(&score.align_to_right(6), pv_score.single() as f32, 0.0, 1.0);

        let visits = format!("{}", node.visits()).align_to_right(9);

        let policy = heat_color(&format!("{:.2}%", node.policy() * 100.0).align_to_right(6), node.policy() as f32, min_policy, max_policy);

        let state = match node.state() {
            GameState::Draw => String::from("DRAW"),
            GameState::Win(len) => format!("WIN IN {len}"),
            GameState::Loss(len) => format!("LOSS IN {len}"),
            _ => String::new(),
        };

        println!(
            "{}",
            format!(
                "{prefix}  {score} score  {} visits  {} policy  {:.4} gini {}",
                visits.to_string().white(),
                policy,
                node.gini_impurity(),
                state.white()
            )
            .secondary(color_gradient)
        );
    }
}
