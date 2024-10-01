#[derive(Clone, Copy, PartialEq, Default)]
pub enum GameState {
    #[default]
    Unresolved,
    Lost(u8),
    Drawn,
    Won(u8),
}

impl From<GameState> for u16 {
    fn from(value: GameState) -> Self {
        match value {
            GameState::Unresolved => 0,
            GameState::Drawn => 1 << 8,
            GameState::Lost(x) => (2 << 8) ^ u16::from(x),
            GameState::Won(x) => (3 << 8) ^ u16::from(x),
        }
    }
}

impl From<u16> for GameState {
    fn from(value: u16) -> Self {
        let x = value as u8;

        match value >> 8 {
            0 => GameState::Unresolved,
            1 => GameState::Drawn,
            2 => GameState::Lost(x),
            3 => GameState::Won(x),
            _ => unreachable!(),
        }
    }
}
