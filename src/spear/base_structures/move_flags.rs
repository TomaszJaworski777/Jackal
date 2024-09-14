pub struct MoveFlag;
impl MoveFlag {
    pub const QUIET_MOVE: u16 = 0; //0000
    pub const DOUBLE_PUSH: u16 = 1; //0001
    pub const KING_SIDE_CASTLE: u16 = 2; //0010
    pub const QUEEN_SIDE_CASTLE: u16 = 3; //0011
    pub const CAPTURE: u16 = 4; //0100
    pub const EN_PASSANT: u16 = 5; //0101
    pub const KNIGHT_PROMOTION: u16 = 8; //1000
    pub const BISHOP_PROMOTION: u16 = 9; //1001
    pub const ROOK_PROMOTION: u16 = 10; //1010
    pub const QUEEN_PROMOTION: u16 = 11; //1011
    pub const KNIGHT_PROMOTION_CAPTURE: u16 = 12; //1100
    pub const BISHOP_PROMOTION_CAPTURE: u16 = 13; //1101
    pub const ROOK_PROMOTION_CAPTURE: u16 = 14; //1110
    pub const QUEEN_PROMOTION_CAPTURE: u16 = 15; //1111
}
