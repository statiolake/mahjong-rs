//! 牌などを定義する。

use crate::config::Context;
use crate::config::Direction;
use failure::Fail;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::hash;
use std::str::FromStr;

pub type Result<T> = std::result::Result<T, TileError>;

/// 牌を作るときまたはパース中に生じたエラー。
#[derive(Debug, Fail, Copy, Clone, PartialEq, Eq)]
pub enum TileError {
    /// (パース) 文字列の長さが変。
    #[fail(display = "文字列の長さが変です。")]
    InvalidStringLen,

    /// (パース) 予期しない文字が出現した。
    #[fail(display = "予期しない文字です: {}。", 0)]
    InvalidChar(char),

    /// 番号がおかしい。 (例: 10m など)
    #[fail(display = "索子・萬子・筒子の番号が範囲外です。")]
    InvalidOrder,

    /// 5 以外の牌で赤ドラが指定された。
    #[fail(display = "5 以外は赤ドラになれません。")]
    InvalidRed,
}

/// 牌の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn next(self) -> Option<Tile> {
        match self {
            Tile::Jihai(_) => None,
            Tile::Souzu(o) => o.next().map(Tile::Souzu),
            Tile::Manzu(o) => o.next().map(Tile::Manzu),
            Tile::Pinzu(o) => o.next().map(Tile::Pinzu),
        }
    }

    pub fn prev(self) -> Option<Tile> {
        match self {
            Tile::Jihai(_) => None,
            Tile::Souzu(o) => o.prev().map(Tile::Souzu),
            Tile::Manzu(o) => o.prev().map(Tile::Manzu),
            Tile::Pinzu(o) => o.prev().map(Tile::Pinzu),
        }
    }

    /// 今の牌の次の牌を返す。常に赤ドラではない牌を返す。9の次は1に戻る。
    pub fn wrapping_next(self) -> Tile {
        match self {
            Tile::Jihai(Jihai::East) => Tile::Jihai(Jihai::South),
            Tile::Jihai(Jihai::South) => Tile::Jihai(Jihai::West),
            Tile::Jihai(Jihai::West) => Tile::Jihai(Jihai::North),
            Tile::Jihai(Jihai::North) => Tile::Jihai(Jihai::East),
            Tile::Jihai(Jihai::Haku) => Tile::Jihai(Jihai::Hatu),
            Tile::Jihai(Jihai::Hatu) => Tile::Jihai(Jihai::Chun),
            Tile::Jihai(Jihai::Chun) => Tile::Jihai(Jihai::Haku),
            Tile::Souzu(o) => Tile::Souzu(o.wrapping_next()),
            Tile::Manzu(o) => Tile::Manzu(o.wrapping_next()),
            Tile::Pinzu(o) => Tile::Pinzu(o.wrapping_next()),
        }
    }

    /// 今の牌の前の牌を返す。常に赤ドラではない牌を返す。9の次は1に戻る。
    pub fn wrapping_prev(self) -> Tile {
        match self {
            Tile::Jihai(Jihai::East) => Tile::Jihai(Jihai::North),
            Tile::Jihai(Jihai::South) => Tile::Jihai(Jihai::East),
            Tile::Jihai(Jihai::West) => Tile::Jihai(Jihai::South),
            Tile::Jihai(Jihai::North) => Tile::Jihai(Jihai::West),
            Tile::Jihai(Jihai::Haku) => Tile::Jihai(Jihai::Chun),
            Tile::Jihai(Jihai::Hatu) => Tile::Jihai(Jihai::Haku),
            Tile::Jihai(Jihai::Chun) => Tile::Jihai(Jihai::Hatu),
            Tile::Souzu(o) => Tile::Souzu(o.wrapping_prev()),
            Tile::Manzu(o) => Tile::Manzu(o.wrapping_prev()),
            Tile::Pinzu(o) => Tile::Pinzu(o.wrapping_prev()),
        }
    }

    /// オーダーを取得する。もし字牌なら None となる。
    pub fn order(self) -> Option<Order> {
        match self {
            Tile::Jihai(_) => None,
            Tile::Souzu(o) | Tile::Manzu(o) | Tile::Pinzu(o) => Some(o),
        }
    }

    /// 赤ドラかどうかを変更した牌を作る。
    pub fn with_red(self, is_red: bool) -> Result<Tile> {
        match self {
            Tile::Jihai(_) => Err(TileError::InvalidRed),
            Tile::Souzu(o) => Ok(Tile::Souzu(o.with_red(is_red)?)),
            Tile::Manzu(o) => Ok(Tile::Manzu(o.with_red(is_red)?)),
            Tile::Pinzu(o) => Ok(Tile::Pinzu(o.with_red(is_red)?)),
        }
    }

    /// 種類を調べる。
    pub fn kind(self) -> TileKind {
        match self {
            Tile::Manzu(_) => TileKind::Manzu,
            Tile::Souzu(_) => TileKind::Souzu,
            Tile::Pinzu(_) => TileKind::Pinzu,
            Tile::Jihai(_) => TileKind::Jihai,
        }
    }

    /// 赤ドラかどうか調べる。
    pub fn is_red(self) -> bool {
        self.order().map(|o| o.is_red()).unwrap_or(false)
    }

    /// 中張牌かどうか調べる
    pub fn is_chunchan(self) -> bool {
        self.order().map(|o| o.is_chunchan()).unwrap_or(false)
    }

    /// 么九牌かどうか調べる。 `!self.is_chunchan()` と同じ。
    pub fn is_yaochu(self) -> bool {
        self.order().map(|o| o.is_yaochu()).unwrap_or(false)
    }

    /// 風牌かどうか調べる。風牌は「東南西北」のどれか。
    pub fn is_fan(self) -> bool {
        match self {
            Tile::Jihai(Jihai::East) => true,
            Tile::Jihai(Jihai::South) => true,
            Tile::Jihai(Jihai::West) => true,
            Tile::Jihai(Jihai::North) => true,
            _ => false,
        }
    }

    /// 三元牌かどうか調べる。三元牌は「白發中」のどれか。
    pub fn is_sangen(self) -> bool {
        match self {
            Tile::Jihai(Jihai::Haku) => true,
            Tile::Jihai(Jihai::Hatu) => true,
            Tile::Jihai(Jihai::Chun) => true,
            _ => false,
        }
    }

    /// 緑一色を構成できる牌かどうか調べる。
    pub fn is_green(self) -> bool {
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
    pub fn num_yakuhai(self, ctx: &Context) -> u32 {
        match self {
            Tile::Jihai(jihai) => {
                let mut res = 0;

                if jihai == ctx.place {
                    res += 1;
                }

                if jihai == ctx.player {
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

impl FromStr for Tile {
    type Err = TileError;

    fn from_str(from: &str) -> Result<Tile> {
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
        let (order, suit) = match (chars.next(), chars.next(), chars.next()) {
            (Some(order), Some(suit), None) => (order, suit),
            _ => return Err(TileError::InvalidStringLen),
        };

        let (tile_constructor, is_red): (fn(Order) -> Tile, bool) = match suit {
            's' => (Tile::Souzu, false),
            'm' => (Tile::Manzu, false),
            'p' => (Tile::Pinzu, false),

            'S' => (Tile::Souzu, true),
            'M' => (Tile::Manzu, true),
            'P' => (Tile::Pinzu, true),

            ch => return Err(TileError::InvalidChar(ch)),
        };

        let order = Order::new(order as u8 - b'0')?.with_red(is_red)?;

        Ok(tile_constructor(order))
    }
}

impl Order {
    /// 牌の順序を作る。
    ///
    /// # エラー
    ///
    /// もし `order` が範囲外になっていればエラーを返す。
    pub fn new(order: u8) -> Result<Order> {
        if order < 1 || 9 < order {
            return Err(TileError::InvalidOrder);
        }

        Ok(Order {
            order,
            is_red: false,
        })
    }

    /// 赤ドラにした同じ番号のものを作る。
    pub fn with_red(self, is_red: bool) -> Result<Order> {
        if is_red && self.order != 5 {
            return Err(TileError::InvalidRed);
        }

        Ok(Order { is_red, ..self })
    }

    /// 次の番号。9であれば1に戻る。
    pub fn wrapping_next(self) -> Order {
        assert!(1 <= self.order && self.order <= 9);
        self.next().unwrap_or_else(|| Order::new(1).unwrap())
    }

    /// 前の番号。1であれば9に戻る。
    pub fn wrapping_prev(self) -> Order {
        assert!(1 <= self.order && self.order <= 9);
        self.prev().unwrap_or_else(|| Order::new(9).unwrap())
    }

    /// 次の番号。9であればNoneとなる。
    pub fn next(self) -> Option<Order> {
        assert!(1 <= self.order && self.order <= 9);
        Order::new(self.order + 1).ok()
    }

    /// 前の番号。1であればNoneとなる。
    pub fn prev(self) -> Option<Order> {
        assert!(1 <= self.order && self.order <= 9);
        Order::new(self.order - 1).ok()
    }

    /// 赤ドラかどうか調べる。
    pub fn is_red(self) -> bool {
        self.is_red
    }

    /// 中張牌かどうか調べる。
    pub fn is_chunchan(self) -> bool {
        self.order != 1 && self.order != 9
    }

    /// 么九牌かどうか調べる。
    pub fn is_yaochu(self) -> bool {
        !self.is_chunchan()
    }

    /// **順序として** 緑一色を構成できる牌かどうかを調べる。
    ///
    /// そもそも索子でなければありえないが、こういった牌の種類はこちらからは知りようもないし無視す
    /// る。それらを考慮するのは牌側の仕事である。順序が 2, 3, 4, 6, 8 になっていることを確かめる。
    pub fn is_green_order(self) -> bool {
        match self.order {
            2 | 3 | 4 | 6 | 8 => true,
            _ => false,
        }
    }
}

impl TryFrom<&str> for Tile {
    type Error = TileError;
    fn try_from(from: &str) -> Result<Tile> {
        from.parse()
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

    fn order_of(order: u8, is_red: bool) -> Order {
        Order::new(order)
            .expect("不正な順番です。")
            .with_red(is_red)
            .expect("不正な赤ドラです。")
    }

    #[test]
    fn smp() {
        assert_eq!("4s", Tile::Souzu(order_of(4, false)).to_string());
        assert_eq!("5m", Tile::Manzu(order_of(5, false)).to_string());
        assert_eq!("6p", Tile::Pinzu(order_of(6, false)).to_string());
        assert_eq!("5S", Tile::Souzu(order_of(5, true)).to_string());
        assert_eq!("5M", Tile::Manzu(order_of(5, true)).to_string());
        assert_eq!("5P", Tile::Pinzu(order_of(5, true)).to_string());
        assert_eq!("東", Tile::Jihai(Jihai::East).to_string());
        assert_eq!("南", Tile::Jihai(Jihai::South).to_string());
        assert_eq!("西", Tile::Jihai(Jihai::West).to_string());
        assert_eq!("北", Tile::Jihai(Jihai::North).to_string());
        assert_eq!("白", Tile::Jihai(Jihai::Haku).to_string());
        assert_eq!("發", Tile::Jihai(Jihai::Hatu).to_string());
        assert_eq!("中", Tile::Jihai(Jihai::Chun).to_string());
    }

    #[test]
    #[should_panic]
    fn smp_invalid() {
        order_of(0, false);
        order_of(10, false);
        order_of(4, true);
    }

    #[test]
    fn parse() {
        assert_eq!(
            "4s".parse::<Tile>().unwrap(),
            Tile::Souzu(order_of(4, false))
        );
        assert_eq!(
            "5m".parse::<Tile>().unwrap(),
            Tile::Manzu(order_of(5, false))
        );
        assert_eq!(
            "6p".parse::<Tile>().unwrap(),
            Tile::Pinzu(order_of(6, false))
        );
        assert_eq!(
            "5S".parse::<Tile>().unwrap(),
            Tile::Souzu(order_of(5, true))
        );
        assert_eq!(
            "5M".parse::<Tile>().unwrap(),
            Tile::Manzu(order_of(5, true))
        );
        assert_eq!(
            "5P".parse::<Tile>().unwrap(),
            Tile::Pinzu(order_of(5, true))
        );
        assert_eq!("東".parse::<Tile>().unwrap(), Tile::Jihai(Jihai::East));
        assert_eq!("南".parse::<Tile>().unwrap(), Tile::Jihai(Jihai::South));
        assert_eq!("西".parse::<Tile>().unwrap(), Tile::Jihai(Jihai::West));
        assert_eq!("北".parse::<Tile>().unwrap(), Tile::Jihai(Jihai::North));
        assert_eq!("白".parse::<Tile>().unwrap(), Tile::Jihai(Jihai::Haku));
        assert_eq!("發".parse::<Tile>().unwrap(), Tile::Jihai(Jihai::Hatu));
        assert_eq!("中".parse::<Tile>().unwrap(), Tile::Jihai(Jihai::Chun));
        assert!("あ".parse::<Tile>().is_err());
        assert!("あい".parse::<Tile>().is_err());
    }

    #[test]
    fn ordering() {
        let s4 = Tile::Souzu(order_of(4, false));
        let m5 = Tile::Manzu(order_of(5, false));
        let p6 = Tile::Pinzu(order_of(6, false));
        let rs5 = Tile::Souzu(order_of(5, true));
        let rm5 = Tile::Manzu(order_of(5, true));
        let rp5 = Tile::Pinzu(order_of(5, true));
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
        let s4 = Tile::Souzu(order_of(4, false));
        let s5 = Tile::Souzu(order_of(5, false));
        let m4 = Tile::Manzu(order_of(4, false));
        let rm5 = Tile::Manzu(order_of(5, true));
        let m9 = Tile::Manzu(order_of(9, false));
        let m1 = Tile::Manzu(order_of(1, false));
        let east = Tile::Jihai(Jihai::East);
        let south = Tile::Jihai(Jihai::South);
        let west = Tile::Jihai(Jihai::West);
        let north = Tile::Jihai(Jihai::North);
        let haku = Tile::Jihai(Jihai::Haku);
        let hatu = Tile::Jihai(Jihai::Hatu);
        let chun = Tile::Jihai(Jihai::Chun);

        assert_eq!(s4.wrapping_next(), s5);
        assert_eq!(s5.wrapping_prev(), s4);
        assert_eq!(rm5.wrapping_prev(), m4);
        assert_eq!(m9.wrapping_next(), m1);
        assert_eq!(m1.wrapping_prev(), m9);
        assert_eq!(m9.next(), None);
        assert_eq!(m1.prev(), None);

        assert_eq!(east.wrapping_next(), south);
        assert_eq!(south.wrapping_next(), west);
        assert_eq!(west.wrapping_next(), north);
        assert_eq!(north.wrapping_next(), east);
        assert_eq!(haku.wrapping_next(), hatu);
        assert_eq!(hatu.wrapping_next(), chun);
        assert_eq!(chun.wrapping_next(), haku);

        assert_eq!(east.wrapping_prev(), north);
        assert_eq!(south.wrapping_prev(), east);
        assert_eq!(west.wrapping_prev(), south);
        assert_eq!(north.wrapping_prev(), west);
        assert_eq!(haku.wrapping_prev(), chun);
        assert_eq!(hatu.wrapping_prev(), haku);
        assert_eq!(chun.wrapping_prev(), hatu);
    }

    #[test]
    fn yakuhai() {
        let make_ctx = |player: Direction, place: Direction| Context {
            player,
            place,
            ..Context::default()
        };

        let east = Tile::Jihai(Jihai::East);
        let haku = Tile::Jihai(Jihai::Haku);

        assert_eq!(
            east.num_yakuhai(&make_ctx(Direction::East, Direction::East)),
            2
        );
        assert_eq!(
            east.num_yakuhai(&make_ctx(Direction::East, Direction::West)),
            1
        );
        assert_eq!(
            east.num_yakuhai(&make_ctx(Direction::North, Direction::West)),
            0
        );
        assert_eq!(
            haku.num_yakuhai(&make_ctx(Direction::East, Direction::East)),
            1
        );
    }
}
