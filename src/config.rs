//! リーチやコンテキストなどを定義する。

use crate::form::Form;
use std::fmt;

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

impl fmt::Display for Direction {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::East => write!(b, "東"),
            Direction::South => write!(b, "南"),
            Direction::West => write!(b, "西"),
            Direction::North => write!(b, "北"),
        }
    }
}

/// どの種類のリーチか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lizhi {
    /// 立直なし
    None,

    /// 立直
    Lizhi,

    /// 立直一発
    LizhiIppatsu,

    /// ダブル立直
    DoubleLizhi,

    /// ダブル立直一発
    DoubleLizhiIppatsu,
}

/// アガリ牌がどういうものだったか。
#[derive(Debug, Clone, Copy)]
pub enum LastDraw {
    /// ツモ
    Zimo,

    /// ロン
    Ronghe,
}

/// 牌を解釈する状況。
#[derive(Debug, Clone)]
pub struct Context {
    pub lizhi: Lizhi,
    pub lucky_forms: Vec<Form>,
    pub place: Direction,
    pub player: Direction,
    pub player_name: String,
}

impl Context {
    pub fn is_parent(&self) -> bool {
        self.player == Direction::East
    }
}

impl Default for Context {
    fn default() -> Context {
        Context {
            lizhi: Lizhi::None,
            lucky_forms: Vec::new(),
            place: Direction::East,
            player: Direction::East,
            player_name: String::new(),
        }
    }
}
