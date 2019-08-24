//! 牌集合などを定義する。
//!
//! 牌集合は牌の集合である。順番に処理していくことで、より情報の多い牌集合ができる。
//!
//! - Tiles (牌のかたまり) : 牌集合と種類のタグがつく。ユーザー入力をパースしただけのもの。
//! - Tileset (牌集合) : 牌のかたまりを種別ごとに分類し、ありえない集合をエラーにしたもの。
//! - AgariTileset (アガリ牌集合) : Tileset をもとに役判定をし、手牌を分解したもの。

use crate::tiles::{Error as TilesError, ParseError as ParseTilesError, Tiles};
use failure::Fail;
use std::fmt;
use std::str::FromStr;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "牌集合のエラーです: {}", 0)]
    TilesError(#[fail(cause)] TilesError),
}

impl From<TilesError> for Error {
    fn from(err: TilesError) -> Error {
        Error::TilesError(err)
    }
}

/// 牌集合。例えば「手牌全体」「ポン」などなどが当たる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tileset {
    /// この牌集合の種類。
    pub tag: Tag,

    /// 実際に集合を構成している牌の集合。
    pub tiles: Tiles,
}

/// 牌集合に関連付けられるタグ。これはその牌集合が何を意味しているかを表している。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    /// アガリ牌で、それはツモ。
    Zimo,

    /// アガリ牌で、それはロン。
    Ronghe,

    /// 手牌。
    Hand,

    /// ポン。
    Peng,

    /// チー。
    Chi,

    /// 明槓。
    Minggang,

    /// 暗槓。
    Angang,

    /// ドラ。
    Dora,
}

impl Tileset {
    /// 牌集合を作る。
    pub fn new(tag: Tag, tiles: Tiles) -> Result<Tileset> {
        // まず牌の集合としておかしいものをチェックしつつ除く。
        let tiles = match tag {
            Tag::Zimo | Tag::Ronghe => tiles.check_last_tile()?,
            Tag::Peng => tiles.check_peng()?,
            Tag::Chi => tiles.check_chi()?,
            Tag::Minggang | Tag::Angang => tiles.check_gang()?,
            _ => tiles,
        };

        Ok(Tileset { tag, tiles })
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tag::Zimo => write!(b, "ツモ"),
            Tag::Ronghe => write!(b, "ロン"),
            Tag::Hand => write!(b, ""),
            Tag::Peng => write!(b, "ポン"),
            Tag::Chi => write!(b, "チー"),
            Tag::Minggang => write!(b, "明槓"),
            Tag::Angang => write!(b, "暗槓"),
            Tag::Dora => write!(b, "ドラ"),
        }
    }
}

impl fmt::Display for Tileset {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{}", self.tag)?;
        write!(b, "{}", self.tiles)?;
        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "不明なアノテーションです: {}", 0)]
    UnknownAnnotation(String),

    #[fail(display = "妙な牌集合です: {}", 0)]
    TilesetError(Error),

    #[fail(display = "牌集合のパースに失敗しました: {}", 0)]
    ParseTilesError(#[fail(cause)] ParseTilesError),
}

impl From<Error> for ParseError {
    fn from(err: Error) -> ParseError {
        ParseError::TilesetError(err)
    }
}

impl From<ParseTilesError> for ParseError {
    fn from(err: ParseTilesError) -> ParseError {
        ParseError::ParseTilesError(err)
    }
}

impl FromStr for Tileset {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Tileset, ParseError> {
        let annot: String = s
            .chars()
            .take_while(|&ch| !"123456789東南西北白發中".contains(ch))
            .collect();

        let (tag, rest) = match &*annot {
            "ツモ" => (Tag::Zimo, &s[6..]),
            "ロン" => (Tag::Ronghe, &s[6..]),
            "ポン" => (Tag::Peng, &s[6..]),
            "チー" => (Tag::Chi, &s[6..]),
            "明槓" => (Tag::Minggang, &s[6..]),
            "暗槓" => (Tag::Angang, &s[6..]),
            "ドラ" => (Tag::Dora, &s[6..]),
            "" => (Tag::Hand, &*s),
            _ => return Err(ParseError::UnknownAnnotation(annot)),
        };

        Ok(Tileset::new(tag, rest.parse()?)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{Order, Tile};

    #[test]
    fn parse() {
        assert_eq!(
            "ロン1p".parse::<Tileset>().unwrap(),
            Tileset::new(Tag::Ronghe, "1p".parse().unwrap()).unwrap()
        );

        assert_eq!(
            "暗槓1p1p1p1p".parse::<Tileset>().unwrap(),
            Tileset::new(Tag::Angang, "1p1p1p1p".parse().unwrap()).unwrap()
        );

        match "ポン1p2p3p".parse::<Tileset>() {
            Err(ParseError::TilesetError(Error::TilesError(TilesError::InvalidPeng(_)))) => {}
            _ => panic!("should cause invalid peng error"),
        }
    }

    #[test]
    fn tiles() {
        let s1 = Tile::Suozi(Order::new(1).unwrap());
        let s2 = Tile::Suozi(Order::new(2).unwrap());
        let s3 = Tile::Suozi(Order::new(3).unwrap());
        let tiles = Tiles::new(vec![s1, s2, s3]);

        assert_eq!(tiles.first(), s1);
        assert_eq!(tiles.middle(), s2);
        assert_eq!(tiles.last(), s3);
    }
}
