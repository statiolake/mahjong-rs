pub type Result<T> = std::result::Result<T, TileError>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TileError {
    InvalidStringLen,
    InvalidChar,
    InvalidOrder,
    InvalidRed,
    InvalidNext,
    InvalidPrev,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tile {
    Souzu(Order),
    Manzu(Order),
    Pinzu(Order),
    Jihai(Jihai),
}

// PartialEq is manually implemented because `is_red` should be ignored in equality comparizon.
// Ord and PartialOrd is also manually implemented for the same reason.
#[derive(Debug, Copy, Clone, Eq)]
pub struct Order {
    order: u8,
    is_red: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Jihai {
    East,
    South,
    West,
    North,
    Haku,
    Hatu,
    Chun,
}

impl Tile {
    pub fn parse(from: &str) -> Result<Tile> {
        // jihai
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

        // other tiles
        //------------------------------
        if from.chars().count() != 2 {
            return Err(TileError::InvalidStringLen);
        }

        let mut chars = from.chars();
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

    pub fn next(&self, red_if_5: bool) -> Result<Tile> {
        let should_be_red = |o: &Order| {
            // the next will be 5 so now 4
            red_if_5 && o.order == 4
        };

        match self {
            Tile::Jihai(Jihai::East) => Ok(Tile::Jihai(Jihai::South)),
            Tile::Jihai(Jihai::South) => Ok(Tile::Jihai(Jihai::West)),
            Tile::Jihai(Jihai::West) => Ok(Tile::Jihai(Jihai::North)),
            Tile::Jihai(Jihai::North) => Ok(Tile::Jihai(Jihai::East)),
            Tile::Jihai(Jihai::Haku) => Ok(Tile::Jihai(Jihai::Hatu)),
            Tile::Jihai(Jihai::Hatu) => Ok(Tile::Jihai(Jihai::Chun)),
            Tile::Jihai(Jihai::Chun) => Ok(Tile::Jihai(Jihai::Haku)),
            Tile::Souzu(o) => Order::new(o.order + 1, should_be_red(&o)).map(Tile::Souzu),
            Tile::Manzu(o) => Order::new(o.order + 1, should_be_red(&o)).map(Tile::Manzu),
            Tile::Pinzu(o) => Order::new(o.order + 1, should_be_red(&o)).map(Tile::Pinzu),
        }
    }

    pub fn prev(&self, red_if_5: bool) -> Result<Tile> {
        let should_be_red = |o: &Order| {
            // the previous will be 5 so now 6
            red_if_5 && o.order == 6
        };

        match self {
            Tile::Jihai(Jihai::East) => Ok(Tile::Jihai(Jihai::North)),
            Tile::Jihai(Jihai::South) => Ok(Tile::Jihai(Jihai::East)),
            Tile::Jihai(Jihai::West) => Ok(Tile::Jihai(Jihai::South)),
            Tile::Jihai(Jihai::North) => Ok(Tile::Jihai(Jihai::West)),
            Tile::Jihai(Jihai::Haku) => Ok(Tile::Jihai(Jihai::Chun)),
            Tile::Jihai(Jihai::Hatu) => Ok(Tile::Jihai(Jihai::Haku)),
            Tile::Jihai(Jihai::Chun) => Ok(Tile::Jihai(Jihai::Hatu)),
            Tile::Souzu(o) => Order::new(o.order - 1, should_be_red(&o)).map(Tile::Souzu),
            Tile::Manzu(o) => Order::new(o.order - 1, should_be_red(&o)).map(Tile::Manzu),
            Tile::Pinzu(o) => Order::new(o.order - 1, should_be_red(&o)).map(Tile::Pinzu),
        }
    }

    pub fn is_red(&self) -> bool {
        match self {
            Tile::Jihai(_) => false,
            Tile::Souzu(o) | Tile::Manzu(o) | Tile::Pinzu(o) => o.is_red(),
        }
    }

    pub fn is_chuchan(&self) -> bool {
        match self {
            Tile::Jihai(_) => false,
            Tile::Souzu(o) | Tile::Manzu(o) | Tile::Pinzu(o) => o.is_chuchan(),
        }
    }

    pub fn is_yaochu(&self) -> bool {
        match self {
            Tile::Jihai(_) => false,
            Tile::Souzu(o) | Tile::Manzu(o) | Tile::Pinzu(o) => o.is_yaochu(),
        }
    }

    /// 風牌: 東南西北
    pub fn is_fon(&self) -> bool {
        match self {
            Tile::Jihai(Jihai::East) => true,
            Tile::Jihai(Jihai::South) => true,
            Tile::Jihai(Jihai::West) => true,
            Tile::Jihai(Jihai::North) => true,
            _ => false,
        }
    }

    /// 三元牌: 白發中
    pub fn is_sangen(&self) -> bool {
        match self {
            Tile::Jihai(Jihai::Haku) => true,
            Tile::Jihai(Jihai::Hatu) => true,
            Tile::Jihai(Jihai::Chun) => true,
            _ => false,
        }
    }

    /// check if the tile can consist of 緑一色.
    pub fn is_green(&self) -> bool {
        match self {
            Tile::Jihai(Jihai::Hatu) => true,
            Tile::Souzu(o) => o.is_green_order(),
            _ => false,
        }
    }
}

impl Order {
    pub fn new(order: u8, is_red: bool) -> Result<Order> {
        if order < 1 || 9 < order {
            return Err(TileError::InvalidOrder);
        }

        if is_red && order != 5 {
            return Err(TileError::InvalidRed);
        }

        Ok(Order { order, is_red })
    }

    pub fn is_red(&self) -> bool {
        self.is_red
    }

    pub fn is_chuchan(&self) -> bool {
        self.order != 1 && self.order != 9
    }

    pub fn is_yaochu(&self) -> bool {
        !self.is_chuchan()
    }

    pub fn is_green_order(&self) -> bool {
        match self.order {
            2 | 3 | 4 | 6 | 8 => true,
            _ => false,
        }
    }
}

use std::convert::TryFrom;

impl TryFrom<&str> for Tile {
    type Error = TileError;
    fn try_from(from: &str) -> Result<Tile> {
        Tile::parse(from)
    }
}

use std::fmt;

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
        // ignore is_red flag since red dora 5 is no difference than non-red version in everywhere
        // except dora calculation.
        self.order == other.order
    }
}

use std::cmp::Ordering;

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

        assert_eq!(s4.next(false), Ok(s5));
        assert_eq!(s5.prev(false), Ok(s4));
        assert_eq!(m4.next(true), Ok(rm5));
        assert_eq!(rm5.prev(false), Ok(m4));
        assert_eq!(p4.next(true), Ok(rp5));
        assert_eq!(rp5.prev(true), Ok(p4));

        assert_eq!(east.next(false), Ok(south));
        assert_eq!(south.next(false), Ok(west));
        assert_eq!(west.next(false), Ok(north));
        assert_eq!(north.next(false), Ok(east));
        assert_eq!(haku.next(false), Ok(hatu));
        assert_eq!(hatu.next(false), Ok(chun));
        assert_eq!(chun.next(false), Ok(haku));

        assert_eq!(east.prev(false), Ok(north));
        assert_eq!(south.prev(false), Ok(east));
        assert_eq!(west.prev(false), Ok(south));
        assert_eq!(north.prev(false), Ok(west));
        assert_eq!(haku.prev(false), Ok(chun));
        assert_eq!(hatu.prev(false), Ok(haku));
        assert_eq!(chun.prev(false), Ok(hatu));
    }
}
