//! 牌などを定義する。

use crate::config::Direction;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::hash;

pub type Result<T> = std::result::Result<T, TileError>;

/// 牌を作るときまたはパース中に生じたエラー。
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TileError {
    /// (パース) 文字列の長さが変。
    InvalidStringLen,

    /// (パース) 予期しない文字が出現した。
    InvalidChar,

    /// 番号がおかしい。 (例: 10m など)
    InvalidOrder,

    /// 5 以外の牌で赤ドラが指定された。
    InvalidRed,
}

/// 牌の種類。
pub enum TileKind {
    /// 索子。
    Souzu,

    /// 萬子。
    Manzu,

    /// 筒子。
    Pinzu,

    /// 字牌。
    Jihai,
}

/// 牌。
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tile {
    /// 索子。
    Souzu(Order),

    /// 萬子。
    Manzu(Order),

    /// 筒子。
    Pinzu(Order),

    /// 字牌。
    Jihai(Jihai),
}

/// 索子、萬子、筒子の番号。
///
/// 赤ドラかどうかも持つ。
// 比較で赤ドラかどうかは無視したいので PartialEq は手動で実装する。それに伴って Hash も手動で実装す
// る。
#[derive(Debug, Copy, Clone, Eq)]
pub struct Order {
    order: u8,
    is_red: bool,
}

/// 字牌。
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Jihai {
    /// 東。
    East,

    /// 南。
    South,

    /// 西。
    West,

    /// 北。
    North,

    /// 白。
    Haku,

    /// 發。
    Hatu,

    /// 中。
    Chun,
}

impl Tile {
    /// 与えられた文字列をパースして牌を作る。
    ///
    /// # エラー
    ///
    /// もしフォーマットがおかしければエラーを返す。
    pub fn parse(from: &str) -> Result<Tile> {
        // 字牌
        //------------------------------
        match from {
            "東" => return Ok(Tile::Jihai(Jihai::East)),
            "南" => return Ok(Tile::Jihai(Jihai::South)),
            "西" => return Ok(Tile::Jihai(Jihai::West)),
            "北" => return Ok(Tile::Jihai(Jihai::North)),
            "白" => return Ok(Tile::Jihai(Jihai::Haku)),
            "發" => return Ok(Tile::Jihai(Jihai::Hatu)),
            "中" => return Ok(Tile::Jihai(Jihai::Chun)),
            _ => (),
        };

        // 他の牌
        //------------------------------
        let mut chars = from.chars();
        // 長さは既にチェックしたので unwrap() してよい。
        let order = chars.next().expect("length checked but no char");
        let suit = chars.next().expect("length checked but no char");
        assert!(chars.next().is_none());

        let (tile_constructor, is_red): (fn(Order) -> Tile, bool) = match suit {
            's' => (Tile::Souzu, false),
            'm' => (Tile::Manzu, false),
            'p' => (Tile::Pinzu, false),

            'S' => (Tile::Souzu, true),
            'M' => (Tile::Manzu, true),
            'P' => (Tile::Pinzu, true),

            _ => return Err(TileError::InvalidChar),
        };

        let order = Order::new(order as u8 - b'0', is_red)?;

        Ok(tile_constructor(order))
    }

    /// 今の牌の次の牌を返す。 red_if_5 が true なら、次の牌が 5 だったときには赤ドラになる。
    pub fn next(&self, red_if_5: bool) -> Tile {
        // 赤ドラになるかどうかをチェック
        let should_be_red = |o: &Order| {
            // 赤ドラにするよう指定されていて今が 4 なら、次の牌は赤ドラにするべき
            red_if_5 && o.order == 4
        };

        match self {
            Tile::Jihai(Jihai::East) => Tile::Jihai(Jihai::South),
            Tile::Jihai(Jihai::South) => Tile::Jihai(Jihai::West),
            Tile::Jihai(Jihai::West) => Tile::Jihai(Jihai::North),
            Tile::Jihai(Jihai::North) => Tile::Jihai(Jihai::East),
            Tile::Jihai(Jihai::Haku) => Tile::Jihai(Jihai::Hatu),
            Tile::Jihai(Jihai::Hatu) => Tile::Jihai(Jihai::Chun),
            Tile::Jihai(Jihai::Chun) => Tile::Jihai(Jihai::Haku),
            Tile::Souzu(o) if o.order == 9 => {
                Tile::Souzu(Order::new(1, should_be_red(&o)).unwrap())
            }
            Tile::Manzu(o) if o.order == 9 => {
                Tile::Manzu(Order::new(1, should_be_red(&o)).unwrap())
            }
            Tile::Pinzu(o) if o.order == 9 => {
                Tile::Pinzu(Order::new(1, should_be_red(&o)).unwrap())
            }
            Tile::Souzu(o) => Tile::Souzu(Order::new(o.order + 1, should_be_red(&o)).unwrap()),
            Tile::Manzu(o) => Tile::Manzu(Order::new(o.order + 1, should_be_red(&o)).unwrap()),
            Tile::Pinzu(o) => Tile::Pinzu(Order::new(o.order + 1, should_be_red(&o)).unwrap()),
        }
    }

    /// 今の牌の前の牌を返す。 red_if_5 が true なら、次の牌が 5 だったときには赤ドラになる。
    pub fn prev(&self, red_if_5: bool) -> Tile {
        // 赤ドラになるかどうかをチェック
        let should_be_red = |o: &Order| {
            // 赤ドラにするよう指定されていて今が 4 なら、次の牌は赤ドラにするべき
            red_if_5 && o.order == 6
        };

        match self {
            Tile::Jihai(Jihai::East) => Tile::Jihai(Jihai::North),
            Tile::Jihai(Jihai::South) => Tile::Jihai(Jihai::East),
            Tile::Jihai(Jihai::West) => Tile::Jihai(Jihai::South),
            Tile::Jihai(Jihai::North) => Tile::Jihai(Jihai::West),
            Tile::Jihai(Jihai::Haku) => Tile::Jihai(Jihai::Chun),
            Tile::Jihai(Jihai::Hatu) => Tile::Jihai(Jihai::Haku),
            Tile::Jihai(Jihai::Chun) => Tile::Jihai(Jihai::Hatu),
            Tile::Souzu(o) if o.order == 1 => {
                Tile::Souzu(Order::new(9, should_be_red(&o)).unwrap())
            }
            Tile::Manzu(o) if o.order == 1 => {
                Tile::Manzu(Order::new(9, should_be_red(&o)).unwrap())
            }
            Tile::Pinzu(o) if o.order == 1 => {
                Tile::Pinzu(Order::new(9, should_be_red(&o)).unwrap())
            }
            Tile::Souzu(o) => Tile::Souzu(Order::new(o.order - 1, should_be_red(&o)).unwrap()),
            Tile::Manzu(o) => Tile::Souzu(Order::new(o.order - 1, should_be_red(&o)).unwrap()),
            Tile::Pinzu(o) => Tile::Souzu(Order::new(o.order - 1, should_be_red(&o)).unwrap()),
        }
    }

    /// 赤ドラかどうか調べる。
    pub fn is_red(&self) -> bool {
        match self {
            Tile::Jihai(_) => false,
            Tile::Souzu(o) | Tile::Manzu(o) | Tile::Pinzu(o) => o.is_red(),
        }
    }

    /// 中張牌かどうか調べる
    pub fn is_chunchan(&self) -> bool {
        match self {
            Tile::Jihai(_) => false,
            Tile::Souzu(o) | Tile::Manzu(o) | Tile::Pinzu(o) => o.is_chunchan(),
        }
    }

    /// 么九牌かどうか調べる。 `!self.is_chunchan()` と同じ。
    pub fn is_yaochu(&self) -> bool {
        match self {
            Tile::Jihai(_) => false,
            Tile::Souzu(o) | Tile::Manzu(o) | Tile::Pinzu(o) => o.is_yaochu(),
        }
    }

    /// 風牌かどうか調べる。風牌は「東南西北」のどれか。
    pub fn is_fan(&self) -> bool {
        match self {
            Tile::Jihai(Jihai::East) => true,
            Tile::Jihai(Jihai::South) => true,
            Tile::Jihai(Jihai::West) => true,
            Tile::Jihai(Jihai::North) => true,
            _ => false,
        }
    }

    /// 三元牌かどうか調べる。三元牌は「白發中」のどれか。
    pub fn is_sangen(&self) -> bool {
        match self {
            Tile::Jihai(Jihai::Haku) => true,
            Tile::Jihai(Jihai::Hatu) => true,
            Tile::Jihai(Jihai::Chun) => true,
            _ => false,
        }
    }

    /// 緑一色を構成できる牌かどうか調べる。
    pub fn is_green(&self) -> bool {
        match self {
            Tile::Jihai(Jihai::Hatu) => true,
            Tile::Souzu(o) => o.is_green_order(),
            _ => false,
        }
    }

    /// この牌の役牌の数を返す。
    ///
    /// 役牌とは
    ///
    /// - 場風が自風と一致している
    /// - 三元牌のいずれかである
    ///
    /// である。返す値は
    ///
    /// - 役牌ではない : 0
    /// - ダブってはないが役牌である : 1
    /// - 東場の東家のようにダブ東である : 2
    pub fn num_yakuhai(&self, place: Direction, player: Direction) -> u32 {
        match self {
            Tile::Jihai(jihai) => {
                let mut res = 0;

                if *jihai == place {
                    res += 1;
                }

                if *jihai == player {
                    res += 1;
                }

                if self.is_sangen() {
                    res += 1;
                }

                res
            }
            _ => 0,
        }
    }
}

impl Order {
    /// 牌の順序を作る。
    ///
    /// # エラー
    ///
    /// もし `order` が範囲外になっているか、 5 でもないのに `is_red` で赤ドラ指定されていれば `Err`
    /// を返す。
    pub fn new(order: u8, is_red: bool) -> Result<Order> {
        if order < 1 || 9 < order {
            return Err(TileError::InvalidOrder);
        }

        if is_red && order != 5 {
            return Err(TileError::InvalidRed);
        }

        Ok(Order { order, is_red })
    }

    /// 赤ドラかどうか調べる。
    pub fn is_red(&self) -> bool {
        self.is_red
    }

    /// 中張牌かどうか調べる。
    pub fn is_chunchan(&self) -> bool {
        self.order != 1 && self.order != 9
    }

    /// 么九牌かどうか調べる。
    pub fn is_yaochu(&self) -> bool {
        !self.is_chunchan()
    }

    /// **順序として** 緑一色を構成できる牌かどうかを調べる。
    ///
    /// そもそも索子でなければありえないが、こういった牌の種類はこちらからは知りようもないし無視す
    /// る。それらを考慮するのは牌側の仕事である。順序が 2, 3, 4, 6, 8 になっていることを確かめる。
    pub fn is_green_order(&self) -> bool {
        match self.order {
            2 | 3 | 4 | 6 | 8 => true,
            _ => false,
        }
    }
}

impl TryFrom<&str> for Tile {
    type Error = TileError;
    fn try_from(from: &str) -> Result<Tile> {
        Tile::parse(from)
    }
}

impl fmt::Display for TileKind {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TileKind::Souzu => write!(b, "索子"),
            TileKind::Manzu => write!(b, "萬子"),
            TileKind::Pinzu => write!(b, "筒子"),
            TileKind::Jihai => write!(b, "字牌"),
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Tile::Souzu(order) if order.is_red => format!("{}S", order),
            Tile::Souzu(order) => format!("{}s", order),
            Tile::Manzu(order) if order.is_red => format!("{}M", order),
            Tile::Manzu(order) => format!("{}m", order),
            Tile::Pinzu(order) if order.is_red => format!("{}P", order),
            Tile::Pinzu(order) => format!("{}p", order),
            Tile::Jihai(jihai) => format!("{}", jihai),
        };

        write!(b, "{}", disp)
    }
}

impl fmt::Display for Order {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{}", self.order)
    }
}

impl fmt::Display for Jihai {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Jihai::East => "東",
            Jihai::South => "南",
            Jihai::West => "西",
            Jihai::North => "北",
            Jihai::Haku => "白",
            Jihai::Hatu => "發",
            Jihai::Chun => "中",
        };

        write!(b, "{}", disp)
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Order) -> bool {
        // 赤ドラかどうかは全く影響しないので無視する。
        self.order == other.order
    }
}

impl hash::Hash for Order {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        // 赤ドラかどうかは影響しないのでむし っ
        hasher.write_u8(self.order);
    }
}

impl PartialEq<Direction> for Jihai {
    fn eq(&self, other: &Direction) -> bool {
        match (self, other) {
            (Jihai::East, Direction::East) => true,
            (Jihai::South, Direction::South) => true,
            (Jihai::West, Direction::West) => true,
            (Jihai::North, Direction::North) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Order) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Order) -> Ordering {
        self.order.cmp(&other.order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smp() {
        assert_eq!("4s", Tile::Souzu(Order::new(4, false).unwrap()).to_string());
        assert_eq!("5m", Tile::Manzu(Order::new(5, false).unwrap()).to_string());
        assert_eq!("6p", Tile::Pinzu(Order::new(6, false).unwrap()).to_string());
        assert_eq!("5S", Tile::Souzu(Order::new(5, true).unwrap()).to_string());
        assert_eq!("5M", Tile::Manzu(Order::new(5, true).unwrap()).to_string());
        assert_eq!("5P", Tile::Pinzu(Order::new(5, true).unwrap()).to_string());
        assert_eq!("東", Tile::Jihai(Jihai::East).to_string());
        assert_eq!("南", Tile::Jihai(Jihai::South).to_string());
        assert_eq!("西", Tile::Jihai(Jihai::West).to_string());
        assert_eq!("北", Tile::Jihai(Jihai::North).to_string());
        assert_eq!("白", Tile::Jihai(Jihai::Haku).to_string());
        assert_eq!("發", Tile::Jihai(Jihai::Hatu).to_string());
        assert_eq!("中", Tile::Jihai(Jihai::Chun).to_string());
        assert!(Order::new(0, false).is_err());
        assert!(Order::new(10, false).is_err());
        assert!(Order::new(4, true).is_err());
    }

    #[test]
    fn parse() {
        assert_eq!(
            Tile::parse("4s").unwrap(),
            Tile::Souzu(Order::new(4, false).unwrap())
        );
        assert_eq!(
            Tile::parse("5m").unwrap(),
            Tile::Manzu(Order::new(5, false).unwrap())
        );
        assert_eq!(
            Tile::parse("6p").unwrap(),
            Tile::Pinzu(Order::new(6, false).unwrap())
        );
        assert_eq!(
            Tile::parse("5S").unwrap(),
            Tile::Souzu(Order::new(5, true).unwrap())
        );
        assert_eq!(
            Tile::parse("5M").unwrap(),
            Tile::Manzu(Order::new(5, true).unwrap())
        );
        assert_eq!(
            Tile::parse("5P").unwrap(),
            Tile::Pinzu(Order::new(5, true).unwrap())
        );
        assert_eq!(Tile::parse("東").unwrap(), Tile::Jihai(Jihai::East));
        assert_eq!(Tile::parse("南").unwrap(), Tile::Jihai(Jihai::South));
        assert_eq!(Tile::parse("西").unwrap(), Tile::Jihai(Jihai::West));
        assert_eq!(Tile::parse("北").unwrap(), Tile::Jihai(Jihai::North));
        assert_eq!(Tile::parse("白").unwrap(), Tile::Jihai(Jihai::Haku));
        assert_eq!(Tile::parse("發").unwrap(), Tile::Jihai(Jihai::Hatu));
        assert_eq!(Tile::parse("中").unwrap(), Tile::Jihai(Jihai::Chun));
        assert!(Tile::parse("あ").is_err());
        assert!(Tile::parse("あい").is_err());
    }

    #[test]
    fn ordering() {
        let s4 = Tile::Souzu(Order::new(4, false).unwrap());
        let m5 = Tile::Manzu(Order::new(5, false).unwrap());
        let p6 = Tile::Pinzu(Order::new(6, false).unwrap());
        let rs5 = Tile::Souzu(Order::new(5, true).unwrap());
        let rm5 = Tile::Manzu(Order::new(5, true).unwrap());
        let rp5 = Tile::Pinzu(Order::new(5, true).unwrap());
        let east = Tile::Jihai(Jihai::East);
        let west = Tile::Jihai(Jihai::West);

        assert!(s4 < m5);
        assert!(m5 > s4);
        assert!(s4 < rs5);
        assert_eq!(m5, rm5);
        assert!(rp5 < p6);
        assert!(rs5 < rm5);
        assert!(east < west);
        assert!(west > east);
    }

    #[test]
    fn next_prev() {
        let s4 = Tile::Souzu(Order::new(4, false).unwrap());
        let s5 = Tile::Souzu(Order::new(5, false).unwrap());
        let m4 = Tile::Manzu(Order::new(4, false).unwrap());
        let rm5 = Tile::Manzu(Order::new(5, true).unwrap());
        let p4 = Tile::Pinzu(Order::new(4, false).unwrap());
        let rp5 = Tile::Pinzu(Order::new(5, true).unwrap());
        let east = Tile::Jihai(Jihai::East);
        let south = Tile::Jihai(Jihai::South);
        let west = Tile::Jihai(Jihai::West);
        let north = Tile::Jihai(Jihai::North);
        let haku = Tile::Jihai(Jihai::Haku);
        let hatu = Tile::Jihai(Jihai::Hatu);
        let chun = Tile::Jihai(Jihai::Chun);

        assert_eq!(s4.next(false), s5);
        assert_eq!(s5.prev(false), s4);
        assert_eq!(m4.next(true), rm5);
        assert_eq!(rm5.prev(false), m4);
        assert_eq!(p4.next(true), rp5);
        assert_eq!(rp5.prev(true), p4);

        assert_eq!(east.next(false), south);
        assert_eq!(south.next(false), west);
        assert_eq!(west.next(false), north);
        assert_eq!(north.next(false), east);
        assert_eq!(haku.next(false), hatu);
        assert_eq!(hatu.next(false), chun);
        assert_eq!(chun.next(false), haku);

        assert_eq!(east.prev(false), north);
        assert_eq!(south.prev(false), east);
        assert_eq!(west.prev(false), south);
        assert_eq!(north.prev(false), west);
        assert_eq!(haku.prev(false), chun);
        assert_eq!(hatu.prev(false), haku);
        assert_eq!(chun.prev(false), hatu);
    }

    #[test]
    fn yakuhai() {
        let east = Tile::Jihai(Jihai::East);
        let haku = Tile::Jihai(Jihai::Haku);

        assert_eq!(east.num_yakuhai(Direction::East, Direction::East), 2);
        assert_eq!(east.num_yakuhai(Direction::East, Direction::West), 1);
        assert_eq!(east.num_yakuhai(Direction::North, Direction::West), 0);

        assert_eq!(haku.num_yakuhai(Direction::East, Direction::East), 1);
    }
}
