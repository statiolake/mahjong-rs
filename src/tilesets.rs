//! 牌集合を定義する。

use crate::config::{Context, Riichi};
use crate::tile::{Tile, TileKind};
use crate::tileset::{Tag, Tiles, Tileset};
use failure::Fail;
use std::fmt;

pub type Result<T> = std::result::Result<T, TilesetsError>;

#[derive(Debug, Fail)]
pub enum TilesetsError {
    /// 手牌が指定されていない。
    #[fail(display = "手牌が指定されていません。")]
    HandNotFound,

    /// 手牌が 2 回以上指定された。
    #[fail(display = "手牌が二回以上指定されています。")]
    HandSpecifiedMoreThanOnce,

    /// アガリ牌が指定されなかった。
    #[fail(display = "アガリ牌が指定されていません。")]
    LastTileNotFound,

    /// アガリ牌が 2 回以上指定された。
    #[fail(display = "アガリ牌が二回以上指定されています。")]
    LastTileSpecifiedMoreThanOnce,

    /// ドラが指定されなかった。
    #[fail(display = "ドラが指定されていません。")]
    DorasNotFound,

    /// ドラが 2 回以上指定された。
    #[fail(display = "ドラが二回以上指定されています。")]
    DorasSpecifiedMoreThanOnce,

    /// 立直と副露が同時に行われている。
    #[fail(display = "立直と副露が同時に行われています。")]
    BothRiichiFuro,

    /// 赤ドラが多すぎる。
    #[fail(display = "{} に対する赤ドラが {} 枚もあります。", 0, 1)]
    InvalidNumRed(TileKind, u32),

    /// 手牌に同じ牌が多すぎる。
    #[fail(display = "{} の数が多すぎます。", 0)]
    InvalidNumSameTiles(Tile),

    /// 手牌の枚数が多すぎるか少なすぎる (多牌か少牌) 。
    #[fail(display = "手牌の数が変です: {} 枚あります。", 0)]
    InvalidNumTiles(u32),
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
        fn set<T>(storage: &mut Option<T>, value: T, error: TilesetsError) -> Result<()> {
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
                        TilesetsError::LastTileSpecifiedMoreThanOnce,
                    )?
                }
                Tag::Hand => set(
                    &mut hand,
                    tileset.tiles,
                    TilesetsError::HandSpecifiedMoreThanOnce,
                )?,
                Tag::Pon => pons.push(tileset.tiles),
                Tag::Qi => qis.push(tileset.tiles),
                Tag::Minkan => minkans.push(tileset.tiles),
                Tag::Ankan => ankans.push(tileset.tiles),
                Tag::Dora => set(
                    &mut doras,
                    tileset.tiles,
                    TilesetsError::DorasSpecifiedMoreThanOnce,
                )?,
            }
        }

        // Get one tile as last.  Its length is already checked in Tileset::new().
        let last = last
            .ok_or(TilesetsError::LastTileNotFound)?
            .into_inner()
            .into_iter()
            .next()
            .expect("last tile must have at least one tile.");
        let is_tumo = is_tumo.expect("last was some but is_tumo is none.");
        let hand = hand.ok_or(TilesetsError::HandNotFound)?;
        let doras = doras.ok_or(TilesetsError::DorasNotFound)?;

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
            return Err(TilesetsError::BothRiichiFuro);
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
        self.tiles_without_doras()
            .chain(self.doras.iter().map(|tile| tile.wrapping_prev()))
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

        Err(TilesetsError::InvalidNumRed(kind, num))
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
            Some((tile, _)) => Err(TilesetsError::InvalidNumSameTiles(tile)),
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
            return Err(TilesetsError::InvalidNumTiles(tiles as _));
        }

        Ok(())
    }
}

impl fmt::Display for Tilesets {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{}", self.hand)?;

        for pon in &self.pons {
            write!(b, " ポン{}", pon)?;
        }

        for qi in &self.qis {
            write!(b, " チー{}", qi)?;
        }

        for minkan in &self.minkans {
            write!(b, " 明槓{}", minkan)?;
        }

        for ankan in &self.ankans {
            write!(b, " 暗槓{}", ankan)?;
        }

        if self.is_tumo {
            write!(b, " ツモ{}", self.last)?;
        } else {
            write!(b, " ロン{}", self.last)?;
        }

        Ok(())
    }
}
