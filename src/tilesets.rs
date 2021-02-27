//! 牌集合を定義する。

use crate::context::{Context, Lizhi};
use crate::tile::{Tile, TileKind};
use crate::tiles::Tiles;
use crate::tileset::ParseError as ParseTilesetError;
use crate::tileset::{Tag, Tileset};
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

    /// ドラが 2 回以上指定された。
    #[fail(display = "ドラが二回以上指定されています。")]
    DorasSpecifiedMoreThanOnce,

    /// 立直と副露が同時に行われている。
    #[fail(display = "立直と副露が同時に行われています。")]
    BothLizhiFulou,

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
#[derive(Debug, Clone)]
pub struct Tilesets {
    /// コンテキスト (場風・自風やリーチの状態など) 。
    pub context: Context,

    /// ツモかどうか。
    pub is_zimo: bool,

    /// アガリ牌。
    pub last: Tile,

    /// 手牌。
    pub hand: Tiles,

    /// ポン。
    pub pengs: Vec<Tiles>,

    /// チー。
    pub chis: Vec<Tiles>,

    /// 明槓。
    pub minggangs: Vec<Tiles>,

    /// 暗槓。
    pub angangs: Vec<Tiles>,

    /// ドラ。
    ///
    /// ドラ表示牌ではなくてその次の本来のドラの牌で表されている。
    pub doras: Tiles,
}

impl Tilesets {
    /// 牌集合の集合を作る。
    pub fn new(context: Context, tilesets: Vec<Tileset>) -> Result<Tilesets> {
        let cand = Tilesets::dispatch(context, tilesets)?;

        cand.check_lizhi_fulou()?;
        cand.check_num_same_tiles()?;
        cand.check_num_tiles()?;

        Ok(cand)
    }

    /// 副露をしたかどうか。
    ///
    /// 副露とはポン・チー・明槓のいずれかである。
    pub fn did_fulou(&self) -> bool {
        !self.pengs.is_empty() || !self.chis.is_empty() || !self.minggangs.is_empty()
    }

    /// 門前かどうか。
    ///
    /// これは副露をしていないことと同値。
    pub fn is_menqian(&self) -> bool {
        !self.did_fulou()
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

        let mut is_zimo = None;
        let mut last = None;
        let mut hand = None;

        let mut pengs = Vec::new();
        let mut chis = Vec::new();

        let mut minggangs = Vec::new();
        let mut angangs = Vec::new();

        let mut doras = None;

        for tileset in tilesets {
            match tileset.tag {
                tag @ Tag::Zimo | tag @ Tag::Ronghe => {
                    match tag {
                        Tag::Zimo => is_zimo = Some(true),
                        Tag::Ronghe => is_zimo = Some(false),
                        _ => unreachable!("only Zimo or Ronghe can reach here."),
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
                Tag::Peng => pengs.push(tileset.tiles),
                Tag::Chi => chis.push(tileset.tiles),
                Tag::Minggang => minggangs.push(tileset.tiles),
                Tag::Angang => angangs.push(tileset.tiles),
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
        let is_zimo = is_zimo.expect("last was some but is_zimo is none.");
        let hand = hand.ok_or(TilesetsError::HandNotFound)?;
        let doras = doras.unwrap_or_else(|| Tiles::new(Vec::new()));

        Ok(Tilesets {
            context,
            is_zimo,
            last,
            hand,
            pengs,
            chis,
            minggangs,
            angangs,
            doras,
        })
    }

    /// 立直と副露が同時に起きていないかを確かめる。
    fn check_lizhi_fulou(&self) -> Result<()> {
        if self.context.lizhi == Lizhi::None {
            return Ok(());
        }

        if self.did_fulou() {
            return Err(TilesetsError::BothLizhiFulou);
        }

        Ok(())
    }

    /// ドラ以外の全ての牌をまわすイテレータを得る。
    pub fn tiles_without_doras<'a>(&'a self) -> impl Iterator<Item = Tile> + 'a {
        use std::iter::once;
        once(self.last)
            .chain(self.hand.iter().copied())
            .chain(self.pengs.iter().flat_map(|i| i.iter().copied()))
            .chain(self.chis.iter().flat_map(|i| i.iter().copied()))
            .chain(self.minggangs.iter().flat_map(|i| i.iter().copied()))
            .chain(self.angangs.iter().flat_map(|i| i.iter().copied()))
    }

    /// ドラを含めた全ての牌をまわすイテレータを得る。
    fn tiles_all<'a>(&'a self) -> impl Iterator<Item = Tile> + 'a {
        self.tiles_without_doras()
            .chain(self.doras.iter().map(|tile| tile.wrapping_prev()))
    }

    /// 同じ牌の数を確認。同じ牌は 4 枚しかないはず。
    fn check_num_same_tiles(&self) -> Result<()> {
        use std::collections::HashMap;
        let mut nums = HashMap::new();

        for tile in self.tiles_all() {
            *nums.entry(tile).or_insert(0) += 1;
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
        let pengs = self.pengs.len() * 3;
        let chis = self.chis.len() * 3;

        // 槓は実際は 4 枚あるが、枚数確認では 3 枚と扱う。
        let minggangs = self.minggangs.len() * 3;
        let angangs = self.angangs.len() * 3;

        let tiles = last + hand + pengs + chis + minggangs + angangs;

        if tiles != 14 {
            return Err(TilesetsError::InvalidNumTiles(tiles as _));
        }

        Ok(())
    }

    pub fn display_en(&self) -> TilesetsDisplayEn {
        TilesetsDisplayEn(self)
    }
}

impl fmt::Display for Tilesets {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        if !self.doras.is_empty() {
            write!(b, "ドラ{} ", self.doras)?;
        }

        write!(b, "{}", self.hand)?;

        for peng in &self.pengs {
            write!(b, " ポン{}", peng)?;
        }

        for chi in &self.chis {
            write!(b, " チー{}", chi)?;
        }

        for minggang in &self.minggangs {
            write!(b, " 明槓{}", minggang)?;
        }

        for angang in &self.angangs {
            write!(b, " 暗槓{}", angang)?;
        }

        if self.is_zimo {
            write!(b, " ツモ{}", self.last)?;
        } else {
            write!(b, " ロン{}", self.last)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TilesetsDisplayEn<'a>(&'a Tilesets);

impl fmt::Display for TilesetsDisplayEn<'_> {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        let TilesetsDisplayEn(tilesets) = self;
        if !tilesets.doras.is_empty() {
            write!(b, "Dora {} ", tilesets.doras)?;
        }

        write!(b, "{}", tilesets.hand)?;

        for peng in &tilesets.pengs {
            write!(b, " Pon {}", peng)?;
        }

        for chi in &tilesets.chis {
            write!(b, " Chii {}", chi)?;
        }

        for minggang in &tilesets.minggangs {
            write!(b, " Kong {}", minggang)?;
        }

        for angang in &tilesets.angangs {
            write!(b, " Concealed Kong {}", angang)?;
        }

        if tilesets.is_zimo {
            write!(b, " Tsumo {}", tilesets.last)?;
        } else {
            write!(b, " Ron {}", tilesets.last)?;
        }

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "牌集合のパースに失敗しました: {}", 0)]
    ParseTilesetError(#[fail(cause)] ParseTilesetError),

    #[fail(display = "牌集合の生成に失敗しました: {}", 0)]
    TilesetsError(#[fail(cause)] TilesetsError),
}

impl From<ParseTilesetError> for ParseError {
    fn from(x: ParseTilesetError) -> ParseError {
        ParseError::ParseTilesetError(x)
    }
}

impl From<TilesetsError> for ParseError {
    fn from(x: TilesetsError) -> ParseError {
        ParseError::TilesetsError(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_num_same_tiles() {
        assert!(Tilesets::new(
            Context::default(),
            vec![
                Tileset::new(Tag::Hand, "1p1p1p1p2p3p1p2p3p4p5p6p3p".parse().unwrap()).unwrap(),
                Tileset::new(Tag::Zimo, "3p".parse().unwrap()).unwrap(),
            ]
        )
        .is_err());
    }
}
