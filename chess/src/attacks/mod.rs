#[allow(clippy::module_inception)]
mod attacks;
mod king_attacks;
mod knight_attacks;
mod pawn_attacks;
mod rays;
mod slider_attacks;

pub use attacks::Attacks;
pub use king_attacks::KingAttacks;
pub use knight_attacks::KnightAttacks;
pub use pawn_attacks::PawnsAttacks;
pub use rays::Rays;
