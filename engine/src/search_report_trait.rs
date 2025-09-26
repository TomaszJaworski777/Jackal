use crate::{search_engine::SearchStats, SearchEngine, SearchLimits};

pub trait SearchReport {
    fn refresh_rate_per_second() -> f64;
    #[allow(unused)]
    fn search_started(search_limits: &SearchLimits, search_engine: &SearchEngine) { }
    #[allow(unused)]
    fn search_report(search_limits: &SearchLimits, search_stats: &SearchStats, search_engine: &SearchEngine) { }
    #[allow(unused)]
    fn search_ended(search_limits: &SearchLimits, search_stats: &SearchStats, search_engine: &SearchEngine) { }
}

pub struct NoReport;
impl SearchReport for NoReport {
    fn refresh_rate_per_second() -> f64 {
        1.0
    }
}