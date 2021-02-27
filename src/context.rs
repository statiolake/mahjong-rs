//! リーチやコンテキストなどを定義する。

use crate::form::Form;
use failure::Fail;
use num_derive::FromPrimitive;
use std::fmt;
use std::str::FromStr;

/// 場風や自風を表す。例 : 東家、東場
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
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

impl Direction {
    pub fn display_en(self) -> DirectionDisplayEn {
        DirectionDisplayEn(self)
    }
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::East
    }
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

pub struct DirectionDisplayEn(Direction);

impl fmt::Display for DirectionDisplayEn {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        let DirectionDisplayEn(dir) = self;
        match dir {
            Direction::East => write!(b, "East"),
            Direction::South => write!(b, "South"),
            Direction::West => write!(b, "West"),
            Direction::North => write!(b, "North"),
        }
    }
}

#[derive(Debug, Fail)]
#[fail(display = "不明な方角です: {}", 0)]
pub struct UnknownDirection(String);

impl FromStr for Direction {
    type Err = UnknownDirection;
    fn from_str(from: &str) -> Result<Direction, UnknownDirection> {
        match from {
            "東" | "East" => Ok(Direction::East),
            "南" | "South" => Ok(Direction::South),
            "西" | "West" => Ok(Direction::West),
            "北" | "North" => Ok(Direction::North),
            _ => Err(UnknownDirection(from.to_string())),
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

impl Default for Lizhi {
    fn default() -> Lizhi {
        Lizhi::None
    }
}

/// アガリ牌がどういうものだったか。
#[derive(Debug, Clone, Copy)]
pub enum LastDraw {
    /// ツモ
    Zimo,

    /// ロン
    Ronghe,
}

impl Default for LastDraw {
    fn default() -> LastDraw {
        LastDraw::Zimo
    }
}

/// 牌を解釈する状況。
#[derive(Debug, Clone, Default)]
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
