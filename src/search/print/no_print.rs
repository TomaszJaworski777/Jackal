use spear::ChessPosition;

use crate::EngineOptions;

use super::SearchDisplay;

pub struct NoPrint;
impl SearchDisplay for NoPrint {
    fn new(_position: &ChessPosition, _engine_options: &EngineOptions) -> Self {
        NoPrint
    }
}
