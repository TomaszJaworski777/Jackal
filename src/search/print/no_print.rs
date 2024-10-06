use spear::ChessPosition;

use crate::EngineOptions;

use super::SearchDisplay;

pub struct NoPrint;
impl SearchDisplay for NoPrint {
    const REFRESH_RATE: f32 = f32::MAX;

    fn new(_position: &ChessPosition, _engine_options: &EngineOptions) -> Self {
        NoPrint
    }
}
