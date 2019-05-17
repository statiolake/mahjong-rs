use crate::tile::Tile;

use std::fmt;
use std::ops::{Deref, DerefMut};

/// Error related to tileset.
pub enum TilesetError {
    InvalidPon(Tiles),
    InvalidQi(Tiles),
    InvalidKan(Tiles),
}

pub type Result<T> = std::result::Result<T, TilesetError>;

/// Wrapper for Vec<Tile>.  This is to implement Display for Vec<Tile>.
pub struct Tiles(Vec<Tile>);

impl Tiles {
    /// Moves out inner Vec.
    pub fn into_inner(self) -> Vec<Tile> {
        self.0
    }

    /// Returns shared reference to inner Vec.
    pub fn inner(&self) -> &Vec<Tile> {
        &self.0
    }

    /// Returns mutable reference to inner Vec.
    pub fn inner_mut(&mut self) -> &mut Vec<Tile> {
        &mut self.0
    }
}

impl Deref for Tiles {
    type Target = Vec<Tile>;
    fn deref(&self) -> &Vec<Tile> {
        self.inner()
    }
}

impl DerefMut for Tiles {
    fn deref_mut(&mut self) -> &mut Vec<Tile> {
        self.inner_mut()
    }
}

impl fmt::Display for Tiles {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        for tile in self.inner() {
            write!(b, "{}", tile)?;
        }
        Ok(())
    }
}

/// A block of tiles.  For example 手牌 (*tehai*), ポン (*pon*), or anything else.
/// #[derive(Debug, Clone)]
pub struct Tileset {
    tag: Tag,
    tiles: Tiles,
}

/// A tag tied with Tileset.  It has the kind of that tileset.
#[derive(Debug, Clone)]
pub enum Tag {
    /// 手牌 (*tehai*).
    Hand,

    /// ポン (*pon*).
    Pon,

    /// チー (*qi*).
    Qi,

    /// 明槓 (*minkan*).
    Minkan,

    /// 暗槓 (*ankan*).
    Ankan,

    /// ドラ (*dora*).
    Dora,
}

impl Tileset {
    /// Creates new instance of Tileset.
    pub fn new(tag: Tag, mut tiles: Tiles) -> Result<Tileset> {
        tiles.sort();

        // check tiles according to tag
        let tiles = match tag {
            Tag::Pon => Tileset::check_pon(tiles)?,
            Tag::Qi => Tileset::check_qi(tiles)?,
            Tag::Minkan | Tag::Ankan => Tileset::check_kan(tiles)?,
            _ => tiles,
        };

        Ok(Tileset { tag, tiles })
    }

    fn check_pon(tiles: Tiles) -> Result<Tiles> {
        if tiles.len() != 3 {
            return Err(TilesetError::InvalidPon(tiles));
        }

        Tileset::check_kotu(tiles)
    }

    fn check_qi(tiles: Tiles) -> Result<Tiles> {
        if tiles.len() != 3 {
            return Err(TilesetError::InvalidPon(tiles));
        }

        let mut expect = tiles[0].clone();
        for tile in tiles.inner() {
            if *tile != expect {
                return Err(TilesetError::InvalidQi(tiles));
            }
            expect = expect.next(false);
        }

        Ok(tiles)
    }

    fn check_kan(tiles: Tiles) -> Result<Tiles> {
        if tiles.len() != 4 {
            return Err(TilesetError::InvalidPon(tiles));
        }

        Tileset::check_kotu(tiles)
    }

    fn check_kotu(tiles: Tiles) -> Result<Tiles> {
        let expect = &tiles[0];
        for tile in &tiles[1..] {
            if tile != expect {
                return Err(TilesetError::InvalidPon(tiles));
            }
        }

        Ok(tiles)
    }
}

impl fmt::Display for Tileset {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{}", self.tag)?;
        write!(b, "{}", self.tiles)?;
        Ok(())
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tag::Hand => write!(b, ""),
            Tag::Pon => write!(b, "ポン"),
            Tag::Qi => write!(b, "チー"),
            Tag::Minkan => write!(b, "明槓"),
            Tag::Ankan => write!(b, "暗槓"),
            Tag::Dora => write!(b, "ドラ"),
        }
    }
}
