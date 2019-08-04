//! 牌集合などを定義する。
//!
//! 牌集合は牌の集合である。順番に処理していくことで、より情報の多い牌集合ができる。
//!
//! - Tiles (牌のかたまり) : 牌集合と種類のタグがつく。ユーザー入力をパースしただけのもの。
//! - Tileset (牌集合) : 牌のかたまりを種別ごとに分類し、ありえない集合をエラーにしたもの。
//! - AgariTileset (アガリ牌集合) : Tileset をもとに役判定をし、手牌を分解したもの。

use crate::tile::Tile;
use failure::Fail;
use std::fmt;
use std::ops::{Deref, DerefMut};

/// 牌集合に関するエラー。
#[derive(Debug, Fail)]
pub enum TilesetError {
    /// アガリ牌が複数回指定されている。
    #[fail(display = "アガリ牌が複数枚指定されています: {}", 0)]
    InvalidLastTile(Tiles),

    /// ポンの長さが変、または全ての牌が同じではない。
    #[fail(display = "変なポンです: {}", 0)]
    InvalidPon(Tiles),

    /// チーの長さが変、または牌の番号が連続していない。例 : 2s4s5s
    #[fail(display = "変なチーです: {}", 0)]
    InvalidQi(Tiles),

    /// カンの長さが変、または全ての牌が同じではない。
    #[fail(display = "変なカンです: {}", 0)]
    InvalidKan(Tiles),
}

pub type Result<T> = std::result::Result<T, TilesetError>;

/// 牌のかたまり。
#[derive(Debug)]
pub struct Tiles(Vec<Tile>);

impl Tiles {
    /// 牌のかたまりを生成する。渡された牌をソートする。
    pub fn new(mut tiles: Vec<Tile>) -> Tiles {
        tiles.sort();
        Tiles(tiles)
    }

    pub fn into_inner(self) -> Vec<Tile> {
        self.0
    }

    pub fn inner(&self) -> &Vec<Tile> {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut Vec<Tile> {
        &mut self.0
    }

    /// 牌のかたまりの最初の牌を確認する。
    pub fn first(&self) -> Tile {
        self.inner().first().copied().unwrap()
    }

    /// 牌のかたまりの最後の牌を確認する。
    pub fn last(&self) -> Tile {
        self.inner().last().copied().unwrap()
    }

    /// 牌のかたまりの真ん中の牌を確認する。これは個数が3つでないとき panic! する。
    pub fn middle(&self) -> Tile {
        assert_eq!(
            self.inner().len(),
            3,
            "牌の個数が{}つなのに middle() が呼ばれました。",
            self.inner().len()
        );

        self.inner()[2]
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
    pub tag: Tag,

    /// 実際に集合を構成している牌の集合。
    pub tiles: Tiles,
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
    pub fn new(tag: Tag, tiles: Tiles) -> Result<Tileset> {
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
