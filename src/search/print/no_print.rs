use crate::spear::ChessPosition;

use crate::{EngineOptions, Tree};

use super::SearchDisplay;

pub struct NoPrint;
impl SearchDisplay for NoPrint {
    const REFRESH_RATE: f32 = f32::MAX;

    fn new(_position: &ChessPosition, _engine_options: &EngineOptions, _tree: &Tree) -> Self {
        NoPrint
    }
}
