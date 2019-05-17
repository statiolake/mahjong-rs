/// Represents the play's place (e.g. East for 東場) or player's home direction
/// (e.g. East for 東家).
#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
    /// 東場 or 東家.
    East,

    /// 南場 or 南家.
    South,

    /// 西場 or 西家.
    West,

    /// 北場 or 北家.
    North,
}

/// 立直 (*riichi*). Represents the kind of riichi.
#[derive(Debug, PartialEq, Eq)]
pub enum Riichi {
    /// No 立直.
    None,

    /// 立直.
    Riichi,

    /// ダブル立直, riichi which is done in the first draw.
    DoubleRiichi,
}

/// Represents how the last draw was made.
#[derive(Debug)]
pub enum LastDraw {
    /// ツモ (*tumo*). The last tile is drawn by player.
    Tumo,

    /// ロン (*ron*). The last tile is from other player's 捨牌 (*sutehai*, tile thrown away).
    Ron,
}

/// The context tiles are interpreted in.
#[derive(Debug)]
pub struct Context {
    pub riichi: Riichi,
    pub place: Direction,
    pub player: Direction,
}
