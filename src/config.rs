//! リーチやコンテキストなどを定義する。

/// 場風や自風を表す。例 : 東家、東場
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// 東場 / 東家。
    East,

    /// 南場 / 南家。
    South,

    /// 西場 / 西家。
    West,

    /// 北場 / 北家。
    North,
}

/// どの種類のリーチか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Riichi {
    /// 立直なし
    None,

    /// 立直
    Riichi,

    /// ダブル立直
    DoubleRiichi,
}

/// アガリ牌がどういうものだったか。
#[derive(Debug, Clone, Copy)]
pub enum LastDraw {
    /// ツモ
    Tumo,

    /// ロン
    Ron,
}

/// 牌を解釈する状況。
#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub riichi: Riichi,
    pub place: Direction,
    pub player: Direction,
}

impl Default for Context {
    fn default() -> Context {
        Context {
            riichi: Riichi::None,
            place: Direction::East,
            player: Direction::East,
        }
    }
}
