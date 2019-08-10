//! アガリ形を保持する牌集合を定義する。

use crate::tile::Order;
use crate::tile::Tile;
use crate::tiles::Tiles;
use crate::tilesets::Tilesets;
use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachiKind {
    /// 両面待ち。
    ///
    /// 例: 23 に対して 1,4 を待つ形。
    Ryanmen,

    /// シャンポン待ち。
    ///
    /// 例: 22, 33 に対して 2,3 を待つ形。
    Shanpon,

    /// ペンチャン待ち。
    ///
    /// 形上は両面と同じだが、 123 または 789 を構成する 3 か 7 であるために実は1通りしか待ちがな
    /// い形。
    Penchan,

    /// カンチャン待ち
    ///
    /// 例: 24 に対して間の 3 を待つ形。
    Kanchan,

    /// 単騎待ち
    ///
    /// 4面子が既に完成していて、雀頭が片方しかない状態。例: 1112223334449 で 9 を待つ形。
    Tanki,

    /// ノベタン
    ///
    /// 4枚の数字が連続している形。両端のいずれかが来ればそれを雀頭に残りを順子にすることで上がれ
    /// る。例: 1234 で 1,4 を待つ形。 1 が来れば 11 と 234 、 4 が来れば 123 と 44 になる。
    Nobetan,
}

impl fmt::Display for MachiKind {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MachiKind::Ryanmen => write!(b, "両面"),
            MachiKind::Shanpon => write!(b, "シャンポン"),
            MachiKind::Penchan => write!(b, "ペンチャン"),
            MachiKind::Kanchan => write!(b, "カンチャン"),
            MachiKind::Tanki => write!(b, "単騎"),
            MachiKind::Nobetan => write!(b, "ノベタン"),
        }
    }
}

pub fn enumerate_machis(
    janto: &Tiles,
    anjuns: &[Tiles],
    ankos: &[Tiles],
    last: Tile,
) -> Vec<MachiKind> {
    MachiEnumerator::new(janto, anjuns, ankos, last).enumerate()
}

/// 待ちを列挙するヘルパー
struct MachiEnumerator<'a> {
    janto: &'a Tiles,
    anjuns: &'a [Tiles],
    ankos: &'a [Tiles],
    last: Tile,
}

impl<'a> MachiEnumerator<'a> {
    fn new(
        janto: &'a Tiles,
        anjuns: &'a [Tiles],
        ankos: &'a [Tiles],
        last: Tile,
    ) -> MachiEnumerator<'a> {
        MachiEnumerator {
            janto,
            anjuns,
            ankos,
            last,
        }
    }

    /// 待ちを列挙する
    fn enumerate(&self) -> Vec<MachiKind> {
        let mut result = Vec::new();

        if self.is_ryanmen() {
            result.push(MachiKind::Ryanmen);
        }

        if self.is_shanpon() {
            result.push(MachiKind::Shanpon);
        }

        if self.is_penchan() {
            result.push(MachiKind::Penchan);
        }

        if self.is_kanchan() {
            result.push(MachiKind::Kanchan);
        }

        if self.is_tanki() {
            result.push(MachiKind::Tanki);
        }

        if self.is_nobetan() {
            result.push(MachiKind::Nobetan);
        }

        result
    }

    /// 両面待ちにあたるかどうか。
    fn is_ryanmen(&self) -> bool {
        self.is_ryanmen_penchan() == Some(MachiKind::Ryanmen)
    }

    /// シャンポン待ちにあたるかどうか。
    fn is_shanpon(&self) -> bool {
        // 最後に引いてきたやつが刻子を構成しているなら、雀頭と合わせてシャンポン待ちになっていたは
        // ず。
        self.ankos.iter().any(|kotu| kotu.first() == self.last)
    }

    /// ペンチャン待ちにあたるかどうか。
    fn is_penchan(&self) -> bool {
        self.is_ryanmen_penchan() == Some(MachiKind::Penchan)
    }

    /// カンチャン待ちにあたるかどうか。
    fn is_kanchan(&self) -> bool {
        self.anjuns.iter().any(|juntu| juntu.middle() == self.last)
    }

    /// 単騎待ちにあたるかどうか。
    fn is_tanki(&self) -> bool {
        self.is_tanki_nobetan() == Some(MachiKind::Tanki)
    }

    /// ノベタンにあたるかどうか。
    fn is_nobetan(&self) -> bool {
        self.is_tanki_nobetan() == Some(MachiKind::Nobetan)
    }

    fn is_ryanmen_penchan(&self) -> Option<MachiKind> {
        // まずオーダーをとる。そもそも字牌なら両面もペンチャンもありえない。
        let order_last = self.last.order()?;

        for juntu in self.anjuns.iter() {
            // そもそも両端でないなら次へ
            if self.last != juntu.first() && self.last != juntu.last() {
                continue;
            }

            let order_juntu_first = match juntu.first().order() {
                Some(o) => o,
                None => continue,
            };

            let order_juntu_last = match juntu.last().order() {
                Some(o) => o,
                None => continue,
            };

            // このときは両面かペンチャンかが発生する。

            // この順子の左端か右端は最後に引いてきた牌。
            // この順子の右端が3かつ3を引いてきた場合か、
            // この順子の左端が7かつ7を引いてきた場合はペンチャン待ち。
            let order_3 = Order::new(3).unwrap();
            if order_juntu_last == order_3 && order_last == order_3 {
                return Some(MachiKind::Penchan);
            }

            let order_7 = Order::new(7).unwrap();
            if order_juntu_first == order_7 && order_last == order_7 {
                return Some(MachiKind::Penchan);
            }

            // ここまでこれば、この牌で両面待ちとできる。
            return Some(MachiKind::Ryanmen);
        }

        None
    }

    fn is_tanki_nobetan(&self) -> Option<MachiKind> {
        // まず最後の牌が雀頭になっている必要がある。
        if self.janto.first() != self.last {
            return None;
        }

        // その上でその牌が順子と連続していなければ単騎待ちとなる。逆にそうでなければノベタンとな
        // る。なぜなら、もしある順子が雀頭と連続していれば、もともと4連続の順子だったところにその両
        // 端の牌を引いてきたことになり、これはノベタンとなるからである。
        for juntu in self.anjuns {
            if Some(juntu.first()) == self.last.next() {
                return Some(MachiKind::Nobetan);
            }

            if Some(juntu.last()) == self.last.prev() {
                return Some(MachiKind::Nobetan);
            }
        }

        Some(MachiKind::Tanki)
    }
}

/// ロンによる明刻・明順を保持する。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum RonMin {
    Minko(Tiles),
    Minjun(Tiles),
    None,
}

impl RonMin {
    fn minko(&self) -> Option<&Tiles> {
        match self {
            RonMin::Minko(tiles) => Some(tiles),
            RonMin::Minjun(_) => None,
            RonMin::None => None,
        }
    }

    fn minjun(&self) -> Option<&Tiles> {
        match self {
            RonMin::Minko(_) => None,
            RonMin::Minjun(tiles) => Some(tiles),
            RonMin::None => None,
        }
    }

    fn iter_minko(&self) -> impl Iterator<Item = &Tiles> {
        self.minko().into_iter()
    }

    fn iter_minjun(&self) -> impl Iterator<Item = &Tiles> {
        self.minjun().into_iter()
    }
}

/// アガリ形に整理された牌集合たち。
#[derive(Debug, Clone)]
pub struct AgariTilesets {
    pub tilesets: Tilesets,
    machi: MachiKind,
    ronmin: RonMin,
    janto: Tiles,
    kotus_in_hand: Vec<Tiles>,
    juntus_in_hand: Vec<Tiles>,
}

impl AgariTilesets {
    pub fn enumerate(tilesets: Tilesets) -> Vec<AgariTilesets> {
        // ツモにせよロンにせよ、とりあえず最後に引いた牌を手牌にくっつけておく。
        let hand = {
            let mut v = tilesets.hand.clone();
            v.push(tilesets.last);
            v
        };
        let mut res = Vec::new();

        // 雀頭のありえかたを列挙する
        for (janto, rest) in enumerate_janto(&hand) {
            // 刻子を全て列挙する
            for (kotus_in_hand, rest) in enumerate_kotu(&rest) {
                // 順子を全てとりだす
                let juntus_in_hand = match extract_juntu(rest) {
                    // あがれなかった (取り出し方が不適切、など)
                    None => continue,

                    // あがれた
                    Some(t) => t,
                };

                // 待ちを全て列挙する
                let last = tilesets.last;
                let machis = enumerate_machis(&janto, &juntus_in_hand, &kotus_in_hand, last);

                // それら全てに対して AgariTilesets を作成する
                for machi in machis {
                    res.push(AgariTilesets::new(
                        tilesets.clone(),
                        machi,
                        janto.clone(),
                        kotus_in_hand.clone(),
                        juntus_in_hand.clone(),
                    ));
                }
            }
        }

        res
    }

    fn new(
        mut tilesets: Tilesets,
        machi: MachiKind,
        janto: Tiles,
        mut kotus_in_hand: Vec<Tiles>,
        mut juntus_in_hand: Vec<Tiles>,
    ) -> AgariTilesets {
        let ronmin = fix_ron_an_mins(
            &mut tilesets,
            &janto,
            &mut kotus_in_hand,
            &mut juntus_in_hand,
        );

        AgariTilesets {
            tilesets,
            machi,
            ronmin,
            janto,
            kotus_in_hand,
            juntus_in_hand,
        }
    }

    /// 明刻。ポンと明槓、ロンによる明刻を合わせたもの。
    pub fn minkos(&self) -> impl Iterator<Item = &Tiles> {
        self.tilesets
            .pons
            .iter()
            .chain(self.tilesets.minkans.iter())
            .chain(self.ronmin.iter_minko())
    }

    /// 暗刻。手札の刻子と暗槓を合わせたもの。
    pub fn ankos(&self) -> impl Iterator<Item = &Tiles> {
        self.kotus_in_hand.iter().chain(self.tilesets.ankans.iter())
    }

    /// 明順。チーとロンによる順子を合わせたもの。
    pub fn minjuns(&self) -> impl Iterator<Item = &Tiles> {
        self.tilesets.qis.iter().chain(self.ronmin.iter_minjun())
    }

    /// 暗順。手札の順子のみ。
    pub fn anjuns(&self) -> impl Iterator<Item = &Tiles> {
        self.juntus_in_hand.iter()
    }

    /// 刻子。明刻、暗刻を合わせたもの。
    pub fn kotus(&self) -> impl Iterator<Item = &Tiles> {
        self.minkos().chain(self.ankos())
    }

    /// 順子。明順、暗順を合わせたもの。
    pub fn juntus(&self) -> impl Iterator<Item = &Tiles> {
        self.minjuns().chain(self.anjuns())
    }

    /// 面子。刻子と順子を合わせたもの。
    pub fn mentus(&self) -> impl Iterator<Item = &Tiles> {
        self.kotus().chain(self.juntus())
    }
}

/// ロンによる暗刻と明刻を調整する。ロンによってできた刻子は明刻として扱うルールがあるため、最初は
/// 手牌の暗刻として扱われているものを一つ明刻へ移さなければならない。
fn fix_ron_an_mins(
    tilesets: &mut Tilesets,
    janto: &Tiles,
    kotus_in_hand: &mut Vec<Tiles>,
    juntus_in_hand: &mut Vec<Tiles>,
) -> RonMin {
    // ツモなら関係がない。
    if tilesets.is_tumo {
        return RonMin::None;
    }

    let last = tilesets.last;

    // まず、暗順に `last` を含む順子があるなら、ロンした牌は常にその順子を作ったと考える。刻子では
    // 暗刻か明刻かが重要になるのに対し、順子に関しては鳴いたかどうかは問題でも暗順か明順かの区別は
    // 一切無関係である。よって、どのような場合も常に順子を優先的に明順にする方がよい。
    for anjuns in &mut [juntus_in_hand] {
        if let Some(pos) = anjuns.iter().position(|jun| jun.contains(&last)) {
            return RonMin::Minjun(anjuns.remove(pos));
        }
    }

    // そうでないなら仕方ないので刻子を確認する。
    for ankos in &mut [kotus_in_hand, &mut tilesets.ankans] {
        if let Some(pos) = ankos.iter().position(|ko| ko.contains(&last)) {
            return RonMin::Minko(ankos.remove(pos));
        }
    }

    // いずれでもなければ必ず雀頭になっているはず。
    assert_eq!(
        tilesets.last,
        janto.first(),
        "ロンした牌によって順子も刻子も雀頭も作られていません。"
    );

    RonMin::None
}

fn range_same_tiles(tiles: &Tiles) -> Vec<Range<usize>> {
    let mut res = Vec::new();
    let mut start = 0;
    let mut curr = None;

    for (index, &tile) in tiles.iter().enumerate() {
        if curr.is_none() {
            curr = Some(tile);
            continue;
        }

        if curr != Some(tile) {
            res.push(start..index);
            curr = Some(tile);
            start = index;
        }
    }

    res.push(start..tiles.len());

    res
}

/// 雀頭を全てのパターンで抽出して列挙する。
fn enumerate_janto(tiles: &Tiles) -> Vec<(Tiles, Tiles)> {
    range_same_tiles(tiles)
        .into_iter()
        .filter(|range| range.len() >= 2)
        .map(|range| range.start..range.start + 2)
        .map(|range| {
            let mut tiles = tiles.clone();
            (tiles.drain(range).collect(), tiles)
        })
        .collect()
}

/// 刻子を全てのパターンで抽出して列挙する。
fn enumerate_kotu(tiles: &Tiles) -> Vec<(Vec<Tiles>, Tiles)> {
    assert!(
        tiles.len() % 3 == 0,
        "残りの牌の個数が3の倍数ではありません : {}",
        tiles.len()
    );

    // 全パターンを入れる Vec
    let mut res = Vec::new();

    // 候補を全て洗い出す。
    let cand_ranges: Vec<_> = range_same_tiles(tiles)
        .into_iter()
        .filter(|range| range.len() >= 3)
        .map(|range| range.start..range.start + 3)
        .collect();

    // 刻子はせいぜい4つしかないはず
    assert!(cand_ranges.len() <= 4, "刻子が5つ以上あります。");

    // 高々 2^4 == 8 通りさえ試せばよい。ビットでやる。
    for set in 0..(1u8 << cand_ranges.len()) {
        // 編集用にコピーしておく。
        let mut tiles = tiles.clone();

        // set で指定された牌を集合にしたもの。
        let mut kotus = Vec::new();

        // 抜きとった要素の個数。
        let mut num_removed = 0;

        for (i, range) in cand_ranges.iter().enumerate() {
            if (set >> i) & 1 != 0 {
                // 既に抜きとった要素の分、正しいインデックスが前にずれているので補正する。
                let start = range.start - num_removed;
                let end = range.end - num_removed;
                let range = start..end;

                // 修正した range の牌を抜きとる。
                num_removed += range.len();
                kotus.push(Tiles::new(tiles.drain(range).collect()));
            }
        }

        assert_eq!(
            set.count_ones() as usize,
            kotus.len(),
            "to_use で立っているビットの個数と実際に選ばれた Tiles の個数が違います。"
        );

        // 分解した刻子と残った牌をおいておく。
        res.push((kotus, tiles));
    }

    res
}

/// 刻子をのぞいた牌から順子を貪欲に分解する。
fn extract_juntu(mut tiles: Tiles) -> Option<Vec<Tiles>> {
    assert!(
        tiles.len() % 3 == 0,
        "残りの牌の個数が3の倍数ではありません : {}",
        tiles.len()
    );

    // チェックしたいんだけど is_sorted() が unstable であった。ある時点で牌集合にいる一番後ろの牌が
    // 「最大」であることに依存するのでソートされている必要がある。

    // assert!(
    //     juntu_tiles.is_sorted(),
    //     "順子がソートされていません。"
    // );

    // 残りは順子しかないはずなので貪欲に分解する。
    let mut juntus = Vec::new();

    // なくなるまでループ
    while !tiles.is_empty() {
        fn pop_last(tiles: &mut Tiles, t: Tile) -> Option<Tile> {
            // 順序を保ちたいので swap_remove() は使えない
            tiles.iter().rposition(|&s| s == t).map(|i| tiles.remove(i))
        }

        // とりあえず最後の牌をとる (前からやるよりたぶんはやいよね？)
        let last = tiles.pop()?;

        // 最後の牌の一つ前を見つけてきて取り出す。
        let mid = pop_last(&mut tiles, last.prev()?)?;

        // 真ん中の牌の一つ前を見つけてきて取り出す。
        let first = pop_last(&mut tiles, mid.prev()?)?;

        // 以下はとり方から成立するはずのことたち
        assert_eq!(first.next(), Some(mid));
        assert_eq!(mid.next(), Some(last));
        assert!(tiles.len() % 3 == 0);

        juntus.push(Tiles::new(vec![first, mid, last]));
    }

    Some(juntus)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Context;
    use crate::tileset::{Tag, Tileset};

    #[test]
    fn machi() {
        let janto = &"4s4s".parse().unwrap();
        let anjuns = &["1s2s3s".parse().unwrap()];
        let ankos = &[
            "1s1s1s".parse().unwrap(),
            "1m1m1m".parse().unwrap(),
            "2m2m2m".parse().unwrap(),
        ];

        assert_eq!(
            &[MachiKind::Ryanmen, MachiKind::Shanpon],
            &*enumerate_machis(janto, anjuns, ankos, "1s".parse().unwrap())
        );

        assert_eq!(
            &[MachiKind::Kanchan],
            &*enumerate_machis(janto, anjuns, ankos, "2s".parse().unwrap())
        );

        assert_eq!(
            &[MachiKind::Penchan],
            &*enumerate_machis(janto, anjuns, ankos, "3s".parse().unwrap())
        );

        assert_eq!(
            &[MachiKind::Shanpon],
            &*enumerate_machis(janto, anjuns, ankos, "1m".parse().unwrap())
        );

        assert_eq!(
            &[MachiKind::Kanchan, MachiKind::Tanki],
            &*enumerate_machis(
                &"2s2s".parse().unwrap(),
                anjuns,
                ankos,
                "2s".parse().unwrap()
            )
        );

        assert_eq!(
            &[MachiKind::Nobetan],
            &*enumerate_machis(janto, anjuns, ankos, "4s".parse().unwrap())
        );
    }

    fn tiles(tiles: &str) -> Tiles {
        tiles.parse().unwrap()
    }

    fn tilesets(hand: &str, last: &str) -> Tilesets {
        let tilesets = vec![
            Tileset::new(Tag::Hand, tiles(hand)).unwrap(),
            Tileset::new(Tag::Tumo, tiles(last)).unwrap(),
            Tileset::new(Tag::Dora, Tiles::new(Vec::new())).unwrap(),
        ];

        Tilesets::new(Context::default(), tilesets).unwrap()
    }

    #[test]
    fn same_tile_counting() {
        let v: Vec<_> = range_same_tiles(&tiles("1s1s1s2s2s2s3s3s3s4s4s4s東東"));
        assert_eq!(v, [0..3, 3..6, 6..9, 9..12, 12..14]);
    }

    #[test]
    fn janto() {
        let hand = tiles("1s1s1s2s2s2s3s3s3s4s4s4s東東");
        let cands = enumerate_janto(&hand);

        assert_eq!(
            cands,
            [
                (tiles("1s1s"), tiles("1s2s2s2s3s3s3s4s4s4s東東")),
                (tiles("2s2s"), tiles("1s1s1s2s3s3s3s4s4s4s東東")),
                (tiles("3s3s"), tiles("1s1s1s2s2s2s3s4s4s4s東東")),
                (tiles("4s4s"), tiles("1s1s1s2s2s2s3s3s3s4s東東")),
                (tiles("東東"), tiles("1s1s1s2s2s2s3s3s3s4s4s4s")),
            ]
        )
    }

    #[test]
    fn decompose() {
        let tilesets = tilesets("1s1s1s2s2s2s3s3s3s4s4s4s東", "東");
        let agaris = AgariTilesets::enumerate(tilesets);

        assert_eq!(agaris.len(), 3);
        eprintln!("{:#?}", agaris);
    }
}
