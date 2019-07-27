//! 牌集合などを定義する。
//!
//! 牌集合は牌の集合である。順番に処理していくことで、より情報の多い牌集合ができる。
//!
//! - Tiles (牌のかたまり) : 牌集合と種類のタグがつく。ユーザー入力をパースしただけのもの。
//! - Tileset (牌集合) : 牌のかたまりを種別ごとに分類し、ありえない集合をエラーにしたもの。
//! - AgariTileset (アガリ牌集合) : Tileset をもとに役判定をし、手牌を分解したもの。

use crate::config::Context;
use crate::config::Riichi;
use crate::tile::{Tile, TileKind};

use std::fmt;
use std::ops::{Deref, DerefMut};

/// 牌集合に関するエラー。
pub enum TilesetError {
    /// アガリ牌が複数回指定されている。
    InvalidLastTile(Tiles),

    /// ポンの長さが変、または全ての牌が同じではない。
    InvalidPon(Tiles),

    /// チーの長さが変、または牌の番号が連続していない。例 : 2s4s5s
    InvalidQi(Tiles),

    /// カンの長さが変、または全ての牌が同じではない。
    InvalidKan(Tiles),

    /// 手牌が指定されていない。
    HandNotFound,

    /// 手牌が 2 回以上指定された。
    HandSpecifiedMoreThanOnce,

    /// アガリ牌が指定されなかった。
    LastTileNotFound,

    /// アガリ牌が 2 回以上指定された。
    LastTileSpecifiedMoreThanOnce,

    /// ドラが指定されなかった。
    DorasNotFound,

    /// ドラが 2 回以上指定された。
    DorasSpecifiedMoreThanOnce,

    /// 立直と副露が同時に行われている。
    BothRiichiFuro,

    /// 赤ドラが多すぎる。
    InvalidNumRed(TileKind, u32),

    /// 手牌に同じ牌が多すぎる。
    InvalidNumSameTiles(Tile),

    /// 手牌の枚数が多すぎるか少なすぎる (多牌か少牌) 。
    InvalidNumTiles(u32),
}

pub type Result<T> = std::result::Result<T, TilesetError>;

/// 牌のかたまり。
pub struct Tiles(Vec<Tile>);

impl Tiles {
    pub fn into_inner(self) -> Vec<Tile> {
        self.0
    }

    pub fn inner(&self) -> &Vec<Tile> {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut Vec<Tile> {
        &mut self.0
    }

    /// アガリ牌を確認する。
    ///
    /// - 枚数が 1 枚かどうか
    fn check_last_tile(self) -> Result<Tiles> {
        if self.len() != 1 {
            return Err(TilesetError::InvalidLastTile(self));
        }

        Ok(self)
    }

    /// ポンを確認する。
    ///
    /// - 枚数が 3 枚かどうか
    /// - 刻子になっているかどうか
    fn check_pon(self) -> Result<Tiles> {
        if self.len() != 3 {
            return Err(TilesetError::InvalidPon(self));
        }

        self.check_kotu()
    }

    /// チーを確認する。
    ///
    /// - 枚数が 3 枚かどうか
    /// - 順子になっているかどうか
    fn check_qi(self) -> Result<Tiles> {
        if self.len() != 3 {
            return Err(TilesetError::InvalidPon(self));
        }

        let mut expect = self[0];
        for tile in self.inner() {
            if *tile != expect {
                return Err(TilesetError::InvalidQi(self));
            }
            expect = expect.next();
        }

        Ok(self)
    }

    /// カンを確認する
    ///
    /// - 枚数が 4 枚かどうか
    /// - 刻子になっているかどうか
    fn check_kan(self) -> Result<Tiles> {
        if self.len() != 4 {
            return Err(TilesetError::InvalidPon(self));
        }

        self.check_kotu()
    }

    /// 刻子かどうか確認する
    fn check_kotu(self) -> Result<Tiles> {
        let expect = &self[0];
        for tile in &self[1..] {
            if tile != expect {
                return Err(TilesetError::InvalidPon(self));
            }
        }

        Ok(self)
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

/// 牌集合。例えば「手牌全体」「ポン」などなどが当たる。
// #[derive(Debug, Clone)]
pub struct Tileset {
    /// この牌集合の種類。
    tag: Tag,

    /// 実際に集合を構成している牌の集合。
    tiles: Tiles,
}

/// 牌集合に関連付けられるタグ。これはその牌集合が何を意味しているかを表している。
#[derive(Debug, Clone)]
pub enum Tag {
    /// アガリ牌で、それはツモ。
    Tumo,

    /// アガリ牌で、それはロン。
    Ron,

    /// 手牌。
    Hand,

    /// ポン。
    Pon,

    /// チー。
    Qi,

    /// 明槓。
    Minkan,

    /// 暗槓。
    Ankan,

    /// ドラ。
    Dora,
}

impl Tileset {
    /// 牌集合を作る。
    pub fn new(tag: Tag, mut tiles: Tiles) -> Result<Tileset> {
        tiles.sort();

        // まず牌の集合としておかしいものをチェックしつつ除く。
        let tiles = match tag {
            Tag::Tumo | Tag::Ron => tiles.check_last_tile()?,
            Tag::Pon => tiles.check_pon()?,
            Tag::Qi => tiles.check_qi()?,
            Tag::Minkan | Tag::Ankan => tiles.check_kan()?,
            _ => tiles,
        };

        Ok(Tileset { tag, tiles })
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

/// 牌集合の集合。これをもとに判定を行う。
pub struct Tilesets {
    /// コンテキスト (場風・自風やリーチの状態など) 。
    context: Context,

    /// ツモかどうか。
    is_tumo: bool,

    /// アガリ牌。
    last: Tile,

    /// 手牌。
    hand: Tiles,

    /// ポン。
    pons: Vec<Tiles>,

    /// チー。
    qis: Vec<Tiles>,

    /// 明槓。
    minkans: Vec<Tiles>,

    /// 暗槓。
    ankans: Vec<Tiles>,

    /// ドラ。
    ///
    /// ドラ表示牌ではなくてその次の本来のドラの牌で表されている。
    doras: Tiles,
}

impl Tilesets {
    /// 牌集合の集合を作る。
    pub fn new(context: Context, tilesets: Vec<Tileset>) -> Result<Tilesets> {
        let cand = Tilesets::dispatch(context, tilesets)?;

        cand.check_riichi_furo()?;
        cand.check_num_reds()?;
        cand.check_num_same_tiles()?;
        cand.check_num_tiles()?;

        Ok(cand)
    }

    /// 副露をしたかどうか。
    ///
    /// 副露とはポン・チー・明槓のいずれかである。
    pub fn did_furo(&self) -> bool {
        !self.pons.is_empty() || !self.qis.is_empty() || !self.minkans.is_empty()
    }

    /// 単純に牌集合の列を受け取って、整理した Tilesets を返す。
    fn dispatch(context: Context, tilesets: Vec<Tileset>) -> Result<Tilesets> {
        fn set<T>(storage: &mut Option<T>, value: T, error: TilesetError) -> Result<()> {
            if storage.is_none() {
                *storage = Some(value);
                Ok(())
            } else {
                Err(error)
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

    /// 立直と副露が同時に起きていないかを確かめる。
    fn check_riichi_furo(&self) -> Result<()> {
        if self.context.riichi == Riichi::None {
            return Ok(());
        }

        if self.did_furo() {
            return Err(TilesetError::BothRiichiFuro);
        }

        Ok(())
    }

    /// ドラ以外の全ての牌をまわすイテレータを得る。
    fn tiles_without_doras<'a>(&'a self) -> impl Iterator<Item = Tile> + 'a {
        use std::iter::once;
        once(self.last)
            .chain(self.hand.iter().copied())
            .chain(self.pons.iter().flat_map(|i| i.iter().copied()))
            .chain(self.qis.iter().flat_map(|i| i.iter().copied()))
            .chain(self.minkans.iter().flat_map(|i| i.iter().copied()))
            .chain(self.ankans.iter().flat_map(|i| i.iter().copied()))
    }

    /// ドラを含めた全ての牌をまわすイテレータを得る。
    fn tiles_all<'a>(&'a self) -> impl Iterator<Item = Tile> + 'a {
        self.tiles_without_doras().chain(self.doras.iter().copied())
    }

    /// 赤ドラの枚数を確認。各色に赤ドラは 1 枚ずつしかないはず。
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

    /// 同じ牌の数を確認。同じ牌は 4 枚しかないはず。
    fn check_num_same_tiles(&self) -> Result<()> {
        use std::collections::HashMap;
        let mut nums = HashMap::new();

        for tile in self.tiles_all() {
            nums.entry(tile).and_modify(|n| *n += 1).or_insert(0);
        }

        match nums.into_iter().find(|&(_, num)| num > 4) {
            None => Ok(()),
            Some((tile, _)) => Err(TilesetError::InvalidNumSameTiles(tile)),
        }
    }

    /// 牌の数を確認。必ず 14 枚のはず。
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
