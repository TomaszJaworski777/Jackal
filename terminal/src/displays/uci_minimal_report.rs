use engine::{SearchEngine, SearchLimits, SearchReport, SearchStats, WDLScore};

pub struct UciMinimalReport;
impl SearchReport for UciMinimalReport {
    fn refresh_rate_per_second() -> f64 {
        1.0
    }

    fn search_ended(_: &SearchLimits, search_stats: &SearchStats, search_engine: &SearchEngine) { 
        let depth = search_stats.avg_depth();
        let max_depth = search_stats.max_depth();

        let draw_score = *search_engine.options().draw_score() as f64 / 100.0;

        let pv = search_engine.tree().get_best_pv(0, draw_score);

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

        let wdl = if *search_engine.options().show_wdl() {
            format!(" wdl {:.0} {:.0} {:.0}", 
                pv_score.win_chance() * 1000.0, 
                pv_score.draw_chance() * 1000.0,
                pv_score.lose_chance() * 1000.0
            )
        } else {
            String::new() 
        };
        
        let time = search_stats.time_passesd_ms();
        let nodes = if *search_engine.options().iters_as_nodes() {
            search_stats.iterations()
        } else {
            search_stats.cumulative_depth()
        };

        let nps = (nodes as u128 * 1000) / time.max(1);

        let hashfull = search_engine.tree().current_size() * 1000 / search_engine.tree().max_size();

        let pv_string = pv.to_string(*search_engine.options().chess960());

        println!("info depth {depth} seldepth {max_depth} score {score}{wdl} time {time} nodes {nodes} nps {nps} hashfull {hashfull} multipv 1 pv {pv_string}");

        println!(
            "bestmove {}",
            pv.first_move().to_string(*search_engine.options().chess960())
        );
    }
}