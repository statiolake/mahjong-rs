#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TileError {
    InvalidStringLen,
    InvalidChar,
    InvalidOrder,
    InvalidRed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Tile {
    Souzu(Order),
    Manzu(Order),
    Pinzu(Order),
    Jihai(Jihai),
}

// PartialEq is manually implemented because `is_red` should be ignored in equality comparizon.
#[derive(Debug, Copy, Clone, Eq)]
pub struct Order {
    order: u8,
    is_red: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Jihai {
    East,
    South,
    North,
    West,
    Haku,
    Hatu,
    Chun,
}

impl Tile {
    pub fn parse(from: &str) -> Result<Tile, TileError> {
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
}

impl Order {
    pub fn new(order: u8, is_red: bool) -> Result<Order, TileError> {
        if order < 1 || 9 < order {
            return Err(TileError::InvalidOrder);
        }

        if is_red && order != 5 {
            return Err(TileError::InvalidRed);
        }

        Ok(Order { order, is_red })
    }
}

use std::convert::TryFrom;

impl TryFrom<&str> for Tile {
    type Error = TileError;
    fn try_from(from: &str) -> Result<Tile, TileError> {
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
}
