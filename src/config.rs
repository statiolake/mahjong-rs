/// Direction.  It represents the play's place (e.g. East for 東場) or player's home direction
/// (e.g. East for 東家).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
