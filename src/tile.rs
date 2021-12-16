//! 牌などを定義する。

use crate::context::Context;
use crate::context::Direction;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::hash;
use std::str::FromStr;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// 牌を作るときまたはパース中に生じたエラー。
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    /// 番号がおかしい。 (例: 10m など)
    #[error("索子・萬子・筒子の番号が範囲外です。")]
    InvalidOrder,

    /// 字牌で赤ドラが指定された。
    #[error("字牌は赤ドラになれません。")]
    InvalidRed,
}

/// 牌の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileKind {
    /// 索子。
    Suozi,

    /// 萬子。
    Wanzi,

    /// 筒子。
    Tongzi,

    /// 字牌。
    Zipai,
}

/// 牌。
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tile {
    /// 索子。
    Suozi(Order),

    /// 萬子。
    Wanzi(Order),

    /// 筒子。
    Tongzi(Order),

    /// 字牌。
    Zipai(Zipai),
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
pub enum Zipai {
    /// 東。
    East,

    /// 南。
    South,

    /// 西。
    West,

    /// 北。
    North,

    /// 白。
    Bai,

    /// 發。
    Fa,

    /// 中。
    Zhong,
}

impl Tile {
    pub fn next(self) -> Option<Tile> {
        match self {
            Tile::Zipai(_) => None,
            Tile::Suozi(o) => o.next().map(Tile::Suozi),
            Tile::Wanzi(o) => o.next().map(Tile::Wanzi),
            Tile::Tongzi(o) => o.next().map(Tile::Tongzi),
        }
    }

    pub fn prev(self) -> Option<Tile> {
        match self {
            Tile::Zipai(_) => None,
            Tile::Suozi(o) => o.prev().map(Tile::Suozi),
            Tile::Wanzi(o) => o.prev().map(Tile::Wanzi),
            Tile::Tongzi(o) => o.prev().map(Tile::Tongzi),
        }
    }

    /// 今の牌の次の牌を返す。常に赤ドラではない牌を返す。9の次は1に戻る。
    pub fn wrapping_next(self) -> Tile {
        match self {
            Tile::Zipai(Zipai::East) => Tile::Zipai(Zipai::South),
            Tile::Zipai(Zipai::South) => Tile::Zipai(Zipai::West),
            Tile::Zipai(Zipai::West) => Tile::Zipai(Zipai::North),
            Tile::Zipai(Zipai::North) => Tile::Zipai(Zipai::East),
            Tile::Zipai(Zipai::Bai) => Tile::Zipai(Zipai::Fa),
            Tile::Zipai(Zipai::Fa) => Tile::Zipai(Zipai::Zhong),
            Tile::Zipai(Zipai::Zhong) => Tile::Zipai(Zipai::Bai),
            Tile::Suozi(o) => Tile::Suozi(o.wrapping_next()),
            Tile::Wanzi(o) => Tile::Wanzi(o.wrapping_next()),
            Tile::Tongzi(o) => Tile::Tongzi(o.wrapping_next()),
        }
    }

    /// 今の牌の前の牌を返す。常に赤ドラではない牌を返す。9の次は1に戻る。
    pub fn wrapping_prev(self) -> Tile {
        match self {
            Tile::Zipai(Zipai::East) => Tile::Zipai(Zipai::North),
            Tile::Zipai(Zipai::South) => Tile::Zipai(Zipai::East),
            Tile::Zipai(Zipai::West) => Tile::Zipai(Zipai::South),
            Tile::Zipai(Zipai::North) => Tile::Zipai(Zipai::West),
            Tile::Zipai(Zipai::Bai) => Tile::Zipai(Zipai::Zhong),
            Tile::Zipai(Zipai::Fa) => Tile::Zipai(Zipai::Bai),
            Tile::Zipai(Zipai::Zhong) => Tile::Zipai(Zipai::Fa),
            Tile::Suozi(o) => Tile::Suozi(o.wrapping_prev()),
            Tile::Wanzi(o) => Tile::Wanzi(o.wrapping_prev()),
            Tile::Tongzi(o) => Tile::Tongzi(o.wrapping_prev()),
        }
    }

    /// オーダーを取得する。もし字牌なら None となる。
    pub fn order(self) -> Option<Order> {
        match self {
            Tile::Zipai(_) => None,
            Tile::Suozi(o) | Tile::Wanzi(o) | Tile::Tongzi(o) => Some(o),
        }
    }

    /// 赤ドラかどうかを変更した牌を作る。
    pub fn with_red(self, is_red: bool) -> Result<Tile> {
        match self {
            Tile::Zipai(_) => Err(Error::InvalidRed),
            Tile::Suozi(o) => Ok(Tile::Suozi(o.with_red(is_red))),
            Tile::Wanzi(o) => Ok(Tile::Wanzi(o.with_red(is_red))),
            Tile::Tongzi(o) => Ok(Tile::Tongzi(o.with_red(is_red))),
        }
    }

    /// 種類を調べる。
    pub fn kind(self) -> TileKind {
        match self {
            Tile::Wanzi(_) => TileKind::Wanzi,
            Tile::Suozi(_) => TileKind::Suozi,
            Tile::Tongzi(_) => TileKind::Tongzi,
            Tile::Zipai(_) => TileKind::Zipai,
        }
    }

    /// 赤ドラかどうか調べる。
    pub fn is_red(self) -> bool {
        self.order().map(|o| o.is_red()).unwrap_or(false)
    }

    /// 中張牌かどうか調べる
    pub fn is_zhongzhang(self) -> bool {
        self.order().map(|o| o.is_zhongzhang()).unwrap_or(false)
    }

    /// 么九牌かどうか調べる。 `!self.is_zhongzhang()` と同じ。
    pub fn is_yaojiu(self) -> bool {
        // order がない場合は字牌なので幺九牌
        self.order().map(|o| o.is_yaojiu()).unwrap_or(true)
    }

    /// 風牌かどうか調べる。風牌は「東南西北」のどれか。
    pub fn is_feng(self) -> bool {
        match self {
            Tile::Zipai(Zipai::East) => true,
            Tile::Zipai(Zipai::South) => true,
            Tile::Zipai(Zipai::West) => true,
            Tile::Zipai(Zipai::North) => true,
            _ => false,
        }
    }

    /// 三元牌かどうか調べる。三元牌は「白發中」のどれか。
    pub fn is_sanyuan(self) -> bool {
        match self {
            Tile::Zipai(Zipai::Bai) => true,
            Tile::Zipai(Zipai::Fa) => true,
            Tile::Zipai(Zipai::Zhong) => true,
            _ => false,
        }
    }

    /// 緑一色を構成できる牌かどうか調べる。
    pub fn is_green(self) -> bool {
        match self {
            Tile::Zipai(Zipai::Fa) => true,
            Tile::Suozi(o) => o.is_green_order(),
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
    pub fn num_fan(self, ctx: &Context) -> u32 {
        match self {
            // 場風、自風、三元牌ならそれぞれ +1 する
            Tile::Zipai(zipai) => [zipai == ctx.place, zipai == ctx.player, self.is_sanyuan()]
                .iter()
                .filter(|&&x| x)
                .count() as _,
            _ => 0,
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    /// 文字列の長さが変。
    #[error("文字列の長さが変です。")]
    InvalidStringLen,

    /// 予期しない文字が出現した。
    #[error("予期しない文字です: {}", 0)]
    InvalidChar(char),

    /// その他、牌をつくるときにエラーが起きた
    #[error("牌を作成できませんでした: {}", 0)]
    TileError(Error),
}

impl From<Error> for ParseError {
    fn from(err: Error) -> ParseError {
        ParseError::TileError(err)
    }
}

impl FromStr for Tile {
    type Err = ParseError;

    fn from_str(from: &str) -> std::result::Result<Tile, ParseError> {
        // 字牌
        //------------------------------
        match from {
            "東" | "1z" => return Ok(Tile::Zipai(Zipai::East)),
            "南" | "2z" => return Ok(Tile::Zipai(Zipai::South)),
            "西" | "3z" => return Ok(Tile::Zipai(Zipai::West)),
            "北" | "4z" => return Ok(Tile::Zipai(Zipai::North)),
            "白" | "5z" => return Ok(Tile::Zipai(Zipai::Bai)),
            "發" | "6z" => return Ok(Tile::Zipai(Zipai::Fa)),
            "中" | "7z" => return Ok(Tile::Zipai(Zipai::Zhong)),
            _ => (),
        };

        // 他の牌
        //------------------------------
        let mut chars = from.chars();
        let (order, kind) = match (chars.next(), chars.next(), chars.next()) {
            (Some(order), Some(kind), None) => (order, kind),
            _ => return Err(ParseError::InvalidStringLen),
        };

        let (tile_constructor, is_red): (fn(Order) -> Tile, bool) = match kind {
            's' => (Tile::Suozi, false),
            'm' => (Tile::Wanzi, false),
            'p' => (Tile::Tongzi, false),

            'S' => (Tile::Suozi, true),
            'M' => (Tile::Wanzi, true),
            'P' => (Tile::Tongzi, true),

            ch => return Err(ParseError::InvalidChar(ch)),
        };

        let order = Order::new(order as u8 - b'0')?.with_red(is_red);

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
            return Err(Error::InvalidOrder);
        }

        Ok(Order {
            order,
            is_red: false,
        })
    }

    /// 赤ドラにした同じ番号のものを作る。
    pub fn with_red(self, is_red: bool) -> Order {
        Order { is_red, ..self }
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
    pub fn is_zhongzhang(self) -> bool {
        self.order != 1 && self.order != 9
    }

    /// 么九牌かどうか調べる。
    pub fn is_yaojiu(self) -> bool {
        !self.is_zhongzhang()
    }

    /// **順序として** 緑一色を構成できる牌かどうかを調べる。
    ///
    /// そもそも索子でなければありえないが、こういった牌の種類はこちらからは知りようもないし無視す
    /// る。それらを考慮するのは牌側の仕事である。順序が 2, 3, 4, 6, 8 になっていることを確かめる。
    pub fn is_green_order(self) -> bool {
        matches!(self.order, 2 | 3 | 4 | 6 | 8)
    }
}

impl TryFrom<&str> for Tile {
    type Error = ParseError;
    fn try_from(from: &str) -> std::result::Result<Tile, ParseError> {
        from.parse()
    }
}

impl fmt::Display for TileKind {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TileKind::Suozi => write!(b, "索子"),
            TileKind::Wanzi => write!(b, "萬子"),
            TileKind::Tongzi => write!(b, "筒子"),
            TileKind::Zipai => write!(b, "字牌"),
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Tile::Suozi(order) if order.is_red => format!("{}S", order),
            Tile::Suozi(order) => format!("{}s", order),
            Tile::Wanzi(order) if order.is_red => format!("{}M", order),
            Tile::Wanzi(order) => format!("{}m", order),
            Tile::Tongzi(order) if order.is_red => format!("{}P", order),
            Tile::Tongzi(order) => format!("{}p", order),
            Tile::Zipai(zipai) => format!("{}", zipai),
        };

        write!(b, "{}", disp)
    }
}

impl fmt::Display for Order {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{}", self.order)
    }
}

impl fmt::Display for Zipai {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Zipai::East => "東",
            Zipai::South => "南",
            Zipai::West => "西",
            Zipai::North => "北",
            Zipai::Bai => "白",
            Zipai::Fa => "發",
            Zipai::Zhong => "中",
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

impl PartialEq<Direction> for Zipai {
    fn eq(&self, other: &Direction) -> bool {
        match (self, other) {
            (Zipai::East, Direction::East) => true,
            (Zipai::South, Direction::South) => true,
            (Zipai::West, Direction::West) => true,
            (Zipai::North, Direction::North) => true,
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
    }

    #[test]
    fn smp() {
        assert_eq!("4s", Tile::Suozi(order_of(4, false)).to_string());
        assert_eq!("5m", Tile::Wanzi(order_of(5, false)).to_string());
        assert_eq!("6p", Tile::Tongzi(order_of(6, false)).to_string());
        assert_eq!("5S", Tile::Suozi(order_of(5, true)).to_string());
        assert_eq!("5M", Tile::Wanzi(order_of(5, true)).to_string());
        assert_eq!("5P", Tile::Tongzi(order_of(5, true)).to_string());
        assert_eq!("東", Tile::Zipai(Zipai::East).to_string());
        assert_eq!("南", Tile::Zipai(Zipai::South).to_string());
        assert_eq!("西", Tile::Zipai(Zipai::West).to_string());
        assert_eq!("北", Tile::Zipai(Zipai::North).to_string());
        assert_eq!("白", Tile::Zipai(Zipai::Bai).to_string());
        assert_eq!("發", Tile::Zipai(Zipai::Fa).to_string());
        assert_eq!("中", Tile::Zipai(Zipai::Zhong).to_string());
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
            Tile::Suozi(order_of(4, false))
        );
        assert_eq!(
            "5m".parse::<Tile>().unwrap(),
            Tile::Wanzi(order_of(5, false))
        );
        assert_eq!(
            "6p".parse::<Tile>().unwrap(),
            Tile::Tongzi(order_of(6, false))
        );
        assert_eq!(
            "5S".parse::<Tile>().unwrap(),
            Tile::Suozi(order_of(5, true))
        );
        assert_eq!(
            "5M".parse::<Tile>().unwrap(),
            Tile::Wanzi(order_of(5, true))
        );
        assert_eq!(
            "5P".parse::<Tile>().unwrap(),
            Tile::Tongzi(order_of(5, true))
        );
        assert_eq!("東".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::East));
        assert_eq!("南".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::South));
        assert_eq!("西".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::West));
        assert_eq!("北".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::North));
        assert_eq!("白".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::Bai));
        assert_eq!("發".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::Fa));
        assert_eq!("中".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::Zhong));
        assert_eq!("1z".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::East));
        assert_eq!("2z".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::South));
        assert_eq!("3z".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::West));
        assert_eq!("4z".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::North));
        assert_eq!("5z".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::Bai));
        assert_eq!("6z".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::Fa));
        assert_eq!("7z".parse::<Tile>().unwrap(), Tile::Zipai(Zipai::Zhong));
        assert!("あ".parse::<Tile>().is_err());
        assert!("あい".parse::<Tile>().is_err());
    }

    #[test]
    fn ordering() {
        let s4 = Tile::Suozi(order_of(4, false));
        let m5 = Tile::Wanzi(order_of(5, false));
        let p6 = Tile::Tongzi(order_of(6, false));
        let rs5 = Tile::Suozi(order_of(5, true));
        let rm5 = Tile::Wanzi(order_of(5, true));
        let rp5 = Tile::Tongzi(order_of(5, true));
        let east = Tile::Zipai(Zipai::East);
        let west = Tile::Zipai(Zipai::West);

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
        let s4 = Tile::Suozi(order_of(4, false));
        let s5 = Tile::Suozi(order_of(5, false));
        let m4 = Tile::Wanzi(order_of(4, false));
        let rm5 = Tile::Wanzi(order_of(5, true));
        let m9 = Tile::Wanzi(order_of(9, false));
        let m1 = Tile::Wanzi(order_of(1, false));
        let east = Tile::Zipai(Zipai::East);
        let south = Tile::Zipai(Zipai::South);
        let west = Tile::Zipai(Zipai::West);
        let north = Tile::Zipai(Zipai::North);
        let haku = Tile::Zipai(Zipai::Bai);
        let hatu = Tile::Zipai(Zipai::Fa);
        let chun = Tile::Zipai(Zipai::Zhong);

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
    fn fanpai() {
        let make_ctx = |player: Direction, place: Direction| Context {
            player,
            place,
            ..Context::default()
        };

        let east = Tile::Zipai(Zipai::East);
        let bai = Tile::Zipai(Zipai::Bai);

        assert_eq!(east.num_fan(&make_ctx(Direction::East, Direction::East)), 2);
        assert_eq!(east.num_fan(&make_ctx(Direction::East, Direction::West)), 1);
        assert_eq!(
            east.num_fan(&make_ctx(Direction::North, Direction::West)),
            0
        );
        assert_eq!(bai.num_fan(&make_ctx(Direction::East, Direction::East)), 1);
    }

    #[test]
    fn order() {
        assert!("4s".parse::<Tile>().unwrap() < "4m".parse::<Tile>().unwrap());
        assert!(Tile::Suozi(Order::new(4).unwrap()) < Tile::Wanzi(Order::new(4).unwrap()));
    }
}
