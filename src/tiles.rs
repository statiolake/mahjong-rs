use crate::tile::{ParseError as ParseTileError, Tile};
use failure::Fail;
use std::fmt;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

/// 牌のかたまりに関するエラー。
#[derive(Debug, Fail)]
pub enum Error {
    /// アガリ牌が複数回指定されている。
    #[fail(display = "アガリ牌が複数枚指定されています: {}", 0)]
    InvalidLastTile(Tiles),

    /// ポンの長さが変、または全ての牌が同じではない。
    #[fail(display = "変なポンです: {}", 0)]
    InvalidPeng(Tiles),

    /// チーの長さが変、または牌の番号が連続していない。例 : 2s4s5s
    #[fail(display = "変なチーです: {}", 0)]
    InvalidChi(Tiles),

    /// カンの長さが変、または全ての牌が同じではない。
    #[fail(display = "変なカンです: {}", 0)]
    InvalidGang(Tiles),
}

pub type Result<T> = std::result::Result<T, Error>;

/// 牌のかたまり。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

        self.inner()[1]
    }

    /// アガリ牌を確認する。
    ///
    /// - 枚数が 1 枚かどうか
    pub fn check_last_tile(self) -> Result<Tiles> {
        if self.len() != 1 {
            return Err(Error::InvalidLastTile(self));
        }

        Ok(self)
    }

    /// ポンを確認する。
    ///
    /// - 枚数が 3 枚かどうか
    /// - 刻子になっているかどうか
    pub fn check_peng(self) -> Result<Tiles> {
        if self.len() != 3 {
            return Err(Error::InvalidPeng(self));
        }

        self.check_kezi()
    }

    /// チーを確認する。
    ///
    /// - 枚数が 3 枚かどうか
    /// - 順子になっているかどうか
    pub fn check_chi(self) -> Result<Tiles> {
        if self.len() != 3 {
            return Err(Error::InvalidPeng(self));
        }

        let mut expect = self[0];
        for tile in self.inner() {
            if *tile != expect {
                return Err(Error::InvalidChi(self));
            }
            expect = expect.wrapping_next();
        }

        Ok(self)
    }

    /// カンを確認する
    ///
    /// - 枚数が 4 枚かどうか
    /// - 刻子になっているかどうか
    pub fn check_gang(self) -> Result<Tiles> {
        if self.len() != 4 {
            return Err(Error::InvalidPeng(self));
        }

        self.check_kezi()
    }

    /// 刻子かどうか確認する
    fn check_kezi(self) -> Result<Tiles> {
        let expect = &self[0];
        for tile in &self[1..] {
            if tile != expect {
                return Err(Error::InvalidPeng(self));
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

impl FromIterator<Tile> for Tiles {
    fn from_iter<I: IntoIterator<Item = Tile>>(iter: I) -> Tiles {
        Tiles::new(iter.into_iter().collect())
    }
}

#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "牌のパースに失敗しました: {}", 0)]
    ParseTileError(#[fail(cause)] ParseTileError),
}

impl From<ParseTileError> for ParseError {
    fn from(err: ParseTileError) -> ParseError {
        ParseError::ParseTileError(err)
    }
}

impl FromStr for Tiles {
    type Err = ParseError;
    fn from_str(mut s: &str) -> std::result::Result<Tiles, ParseError> {
        let mut res = Vec::new();
        while let Some(ch) = s.chars().next() {
            let bytes = if ch.is_numeric() {
                // 数字なら 1s などなので2バイト
                2
            } else {
                // そうでなければ白發中などなので3バイト
                3
            };

            res.push(s[0..bytes].parse()?);
            s = &s[bytes..];
        }

        Ok(Tiles::new(res))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tiles() {
        assert_eq!(
            &*"1s2s3s".parse::<Tiles>().unwrap(),
            &[
                "1s".parse().unwrap(),
                "2s".parse().unwrap(),
                "3s".parse().unwrap(),
            ]
        );

        assert_eq!(
            &*"白發中".parse::<Tiles>().unwrap(),
            &[
                "白".parse().unwrap(),
                "發".parse().unwrap(),
                "中".parse().unwrap(),
            ]
        );

        assert!("1sあいうえお".parse::<Tiles>().is_err());
        assert!("あいうえお".parse::<Tiles>().is_err());
    }
}
