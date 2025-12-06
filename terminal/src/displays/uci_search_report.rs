use engine::{SearchEngine, SearchLimits, SearchReport, SearchStats, WDLScore};

pub struct UciSearchReport;
impl SearchReport for UciSearchReport {
    fn refresh_rate_per_second() -> f64 {
        1.0
    }

    fn search_report(_: &SearchLimits, search_stats: &SearchStats, search_engine: &SearchEngine) {
        let search_stats_data = search_stats.aggregate();
        let depth = search_stats_data.avg_depth();
        let max_depth = search_stats_data.max_depth();

        let pv_count = search_engine.tree().root_node().children_count().min(search_engine.options().multi_pv() as usize);

        let draw_score = search_engine.options().draw_score() as f64 / 100.0;

        for pv_idx in 0..pv_count {
            let pv = search_engine.tree().get_best_pv(pv_idx as usize, draw_score);

            let score = pv.score();
            let mut v = score.win_chance() - score.lose_chance();
            let mut d = score.draw_chance();

            search_engine.contempt().rescale(&mut v, &mut d, 1.0, true, search_engine.options());

            let pv_score = WDLScore::new((1.0 + v - d) / 2.0, d);

            let state = pv.first_node().state();
            let score = match state {
                engine::GameState::Loss(len) => format!("mate {}", (len + 1).div_ceil(2)),
                engine::GameState::Win(len) => format!("mate -{}", (len + 1).div_ceil(2)),
                _ => format!("cp {}", pv_score.cp())
            };

            let wdl = match pv.first_node().state() {
                engine::GameState::Loss(_) => WDLScore::WIN,
                engine::GameState::Win(_) => WDLScore::LOSE,
                engine::GameState::Draw => WDLScore::DRAW,
                _ => pv_score
            };

            let wdl = if search_engine.options().show_wdl() {
                format!(" wdl {:.0} {:.0} {:.0}", 
                    wdl.win_chance() * 1000.0, 
                    wdl.draw_chance() * 1000.0,
                    wdl.lose_chance() * 1000.0
                )
            } else {
               String::new() 
            };
            
            let time = search_stats.elapsed_ms();
            let nodes = if search_engine.options().iters_as_nodes() {
                search_stats_data.iterations()
            } else {
                search_stats_data.cumulative_depth()
            };

            let nps = (nodes as u128 * 1000) / time.max(1);

            let hashfull = search_engine.tree().current_size() * 1000 / search_engine.tree().max_size();

            let pv = pv.to_string(search_engine.options().chess960());

            println!("info depth {depth} seldepth {max_depth} score {score}{wdl} time {time} nodes {nodes} nps {nps} hashfull {hashfull} multipv {} pv {pv}", pv_idx + 1)   
        }
    }

    fn search_ended(_: &SearchLimits, _: &SearchStats, search_engine: &SearchEngine) {
        let draw_score = search_engine.options().draw_score() as f64 / 100.0;
        let best_node_idx = search_engine.tree().select_best_child(search_engine.tree().root_index(), draw_score);

        if best_node_idx.is_none() {
            return;
        }

        println!(
            "bestmove {}",
            search_engine
                .tree()[best_node_idx.unwrap()]
                .mv()
                .to_string(search_engine.options().chess960())
        );
    }
}