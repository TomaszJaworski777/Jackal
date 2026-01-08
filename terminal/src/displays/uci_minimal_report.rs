use engine::{SearchEngine, SearchLimits, SearchReport, SearchStats, WDLScore};

pub struct UciMinimalReport;
impl SearchReport for UciMinimalReport {
    fn refresh_rate_per_second() -> f64 {
        1.0
    }

    fn search_ended(_: &SearchLimits, search_stats: &SearchStats, search_engine: &SearchEngine) { 
        let search_stats_data = search_stats.aggregate();
        let depth = search_stats_data.avg_depth();
        let max_depth = search_stats_data.max_depth();

        let draw_score = search_engine.params().draw_score() as f64 / 100.0;

        let pv = search_engine.tree().get_best_pv(0, draw_score);

        let pv_score = pv.score();

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
            _ => pv.score()
        };

        let wdl = if search_engine.params().show_wdl() {
            format!(" wdl {:.0} {:.0} {:.0}", 
                wdl.win_chance() * 1000.0, 
                wdl.draw_chance() * 1000.0,
                wdl.lose_chance() * 1000.0
            )
        } else {
            String::new() 
        };
        
        let time = search_stats.elapsed_ms();
        let nodes = if search_engine.params().iters_as_nodes() {
            search_stats_data.iterations()
        } else {
            search_stats_data.cumulative_depth()
        };

        let nps = (nodes as u128 * 1000) / time.max(1);

        let hashfull = search_engine.tree().current_size() * 1000 / search_engine.tree().max_size();

        let pv_string = pv.to_string(search_engine.params().chess960());

        println!("info depth {depth} seldepth {max_depth} score {score}{wdl} time {time} nodes {nodes} nps {nps} hashfull {hashfull} multipv 1 pv {pv_string}");

        println!(
            "bestmove {}",
            pv.first_move().to_string(search_engine.params().chess960())
        );
    }
}