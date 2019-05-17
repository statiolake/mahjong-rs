use crate::config::Context;
use crate::config::Riichi;
use crate::tile::{Tile, TileKind};

use std::fmt;
use std::ops::{Deref, DerefMut};

/// Error related to tileset.
pub enum TilesetError {
    /// The last drawn tile is specified as multiple tiles.
    InvalidLastTile(Tiles),

    /// The ポン has invalid length or tiles are not all same.
    InvalidPon(Tiles),

    /// The チー has invalid length or tile orders are not contigous, e.g. 2s4s5s.
    InvalidQi(Tiles),

    /// The カン has invalid length or tiles are not all same.
    InvalidKan(Tiles),

    /// 手牌 was not specified.
    HandNotFound,

    /// 手牌 was specified at least twice.
    HandSpecifiedMoreThanOnce,

    /// the last drawn tile wasn't specified
    LastTileNotFound,

    /// the last drawn tile was specified at least twice.
    LastTileSpecifiedMoreThanOnce,

    /// ドラ was not specified
    DorasNotFound,

    /// ドラ was specified at least twice.
    DorasSpecifiedMoreThanOnce,

    /// 立直 and 副露 are done at the same time.
    BothRiichiFuro,

    /// Too many 赤ドラ.
    InvalidNumRed(TileKind, u32),

    /// Too many number of same tiles in hand.
    InvalidNumSameTiles(Tile),

    /// Too many or too few number of tiles in hand.
    InvalidNumTiles(u32),
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
    /// the last drawn tile and that was ツモ (*tumo*).
    Tumo,

    /// the last drawn tile and that was ロン (*ron*).
    Ron,

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
            Tag::Tumo | Tag::Ron => Tileset::check_last_tile(tiles)?,
            Tag::Pon => Tileset::check_pon(tiles)?,
            Tag::Qi => Tileset::check_qi(tiles)?,
            Tag::Minkan | Tag::Ankan => Tileset::check_kan(tiles)?,
            _ => tiles,
        };

        Ok(Tileset { tag, tiles })
    }

    fn check_last_tile(tiles: Tiles) -> Result<Tiles> {
        if tiles.len() != 1 {
            return Err(TilesetError::InvalidLastTile(tiles));
        }

        Ok(tiles)
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
            Tag::Tumo => write!(b, "ツモ"),
            Tag::Ron => write!(b, "ロン"),
            Tag::Hand => write!(b, ""),
            Tag::Pon => write!(b, "ポン"),
            Tag::Qi => write!(b, "チー"),
            Tag::Minkan => write!(b, "明槓"),
            Tag::Ankan => write!(b, "暗槓"),
            Tag::Dora => write!(b, "ドラ"),
        }
    }
}

pub struct Tilesets {
    context: Context,

    is_tumo: bool,
    last: Tile,

    hand: Tiles,

    pons: Vec<Tiles>,
    qis: Vec<Tiles>,

    minkans: Vec<Tiles>,
    ankans: Vec<Tiles>,

    doras: Tiles,
}

impl Tilesets {
    pub fn new(context: Context, tilesets: Vec<Tileset>) -> Result<Tilesets> {
        let cand = Tilesets::dispatch(context, tilesets)?;

        cand.check_riichi_furo()?;
        cand.check_num_reds()?;
        cand.check_num_same_tiles()?;
        cand.check_num_tiles()?;

        Ok(cand)
    }

    pub fn did_furo(&self) -> bool {
        !self.pons.is_empty() || !self.qis.is_empty() || !self.minkans.is_empty()
    }

    fn dispatch(context: Context, tilesets: Vec<Tileset>) -> Result<Tilesets> {
        fn set<T>(storage: &mut Option<T>, value: T, error: TilesetError) -> Result<()> {
            if storage.is_none() {
                *storage = Some(value);
                Ok(())
            } else {
                return Err(error);
            }
        }

        let mut is_tumo = None;
        let mut last = None;
        let mut hand = None;

        let mut pons = Vec::new();
        let mut qis = Vec::new();

        let mut minkans = Vec::new();
        let mut ankans = Vec::new();

        let mut doras = None;

        for tileset in tilesets {
            match tileset.tag {
                tag @ Tag::Tumo | tag @ Tag::Ron => {
                    match tag {
                        Tag::Tumo => is_tumo = Some(true),
                        Tag::Ron => is_tumo = Some(false),
                        _ => unreachable!("only Tumo or Ron can reach here."),
                    }
                    set(
                        &mut last,
                        tileset.tiles,
                        TilesetError::LastTileSpecifiedMoreThanOnce,
                    )?
                }
                Tag::Hand => set(
                    &mut hand,
                    tileset.tiles,
                    TilesetError::HandSpecifiedMoreThanOnce,
                )?,
                Tag::Pon => pons.push(tileset.tiles),
                Tag::Qi => qis.push(tileset.tiles),
                Tag::Minkan => minkans.push(tileset.tiles),
                Tag::Ankan => ankans.push(tileset.tiles),
                Tag::Dora => set(
                    &mut doras,
                    tileset.tiles,
                    TilesetError::DorasSpecifiedMoreThanOnce,
                )?,
            }
        }

        // Get one tile as last.  Its length is already checked in Tileset::new().
        let last = last
            .ok_or(TilesetError::LastTileNotFound)?
            .into_inner()
            .into_iter()
            .next()
            .expect("last tile must have at least one tile.");
        let is_tumo = is_tumo.expect("last was some but is_tumo is none.");
        let hand = hand.ok_or(TilesetError::HandNotFound)?;
        let doras = doras.ok_or(TilesetError::DorasNotFound)?;

        Ok(Tilesets {
            context,
            is_tumo,
            last,
            hand,
            pons,
            qis,
            minkans,
            ankans,
            doras,
        })
    }

    fn check_riichi_furo(&self) -> Result<()> {
        if self.context.riichi == Riichi::None {
            return Ok(());
        }

        if self.did_furo() {
            return Err(TilesetError::BothRiichiFuro);
        }

        Ok(())
    }

    fn tiles_without_doras(&self) -> impl Iterator<Item = &Tile> {
        Some(&self.last)
            .into_iter()
            .chain(self.hand.iter())
            .chain(self.pons.iter().flat_map(|i| i.iter()))
            .chain(self.qis.iter().flat_map(|i| i.iter()))
            .chain(self.minkans.iter().flat_map(|i| i.iter()))
            .chain(self.ankans.iter().flat_map(|i| i.iter()))
    }

    fn tiles_all(&self) -> impl Iterator<Item = &Tile> {
        self.tiles_without_doras().chain(self.doras.iter())
    }

    fn check_num_reds(&self) -> Result<()> {
        let (mut red_s, mut red_m, mut red_p) = (0, 0, 0);
        self.tiles_without_doras().for_each(|tile| match tile {
            Tile::Souzu(o) if o.is_red() => red_s += 1,
            Tile::Manzu(o) if o.is_red() => red_m += 1,
            Tile::Pinzu(o) if o.is_red() => red_p += 1,
            _ => (),
        });

        let (kind, num) = match (red_s >= 2, red_m >= 2, red_p >= 2) {
            (true, _, _) => (TileKind::Souzu, red_s),
            (_, true, _) => (TileKind::Manzu, red_m),
            (_, _, true) => (TileKind::Pinzu, red_p),
            _ => return Ok(()),
        };

        Err(TilesetError::InvalidNumRed(kind, num))
    }

    fn check_num_same_tiles(&self) -> Result<()> {
        use std::collections::HashMap;
        let mut nums = HashMap::new();

        for tile in self.tiles_all() {
            nums.entry(tile).and_modify(|n| *n += 1).or_insert(0);
        }

        match nums.into_iter().find(|&(_, num)| num > 4) {
            None => Ok(()),
            Some((tile, _)) => Err(TilesetError::InvalidNumSameTiles(tile.clone())),
        }
    }

    fn check_num_tiles(&self) -> Result<()> {
        let last = 1;
        let hand = self.hand.len();
        let pons = self.pons.len() * 3;
        let qis = self.qis.len() * 3;

        // Though kan is actually 4, in judgement this is treated as kotu so count them as 3.
        let minkans = self.minkans.len() * 3;
        let ankans = self.ankans.len() * 3;

        let tiles = last + hand + pons + qis + minkans + ankans;

        if tiles != 14 {
            return Err(TilesetError::InvalidNumTiles(tiles as _));
        }

        Ok(())
    }
}
