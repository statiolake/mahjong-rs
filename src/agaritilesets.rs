//! アガリ形を保持する牌集合を定義する。

use crate::context::Context;
use crate::tile::{Order, Tile};
use crate::tiles::Tiles;
use crate::tilesets::Tilesets;
use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachiKind {
    /// 両面待ち。
    ///
    /// 例: 23 に対して 1,4 を待つ形。
    Liangmian,

    /// シャンポン待ち。
    ///
    /// 例: 22, 33 に対して 2,3 を待つ形。
    Shuangpeng,

    /// ペンチャン待ち。
    ///
    /// 形上は両面と同じだが、 123 または 789 を構成する 3 か 7 であるために実は1通りしか待ちがな
    /// い形。
    Bianzhang,

    /// カンチャン待ち
    ///
    /// 例: 24 に対して間の 3 を待つ形。
    Qianzhang,

    /// 単騎待ち
    ///
    /// 4面子が既に完成していて、雀頭が片方しかない状態。例: 1112223334449 で 9 を待つ形。
    Danqi,

    /// ノベタン
    ///
    /// 4枚の数字が連続している形。両端のいずれかが来ればそれを雀頭に残りを順子にすることで上がれ
    /// る。例: 1234 で 1,4 を待つ形。 1 が来れば 11 と 234 、 4 が来れば 123 と 44 になる。
    Yandan,
}

impl fmt::Display for MachiKind {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MachiKind::Liangmian => write!(b, "両面"),
            MachiKind::Shuangpeng => write!(b, "シャンポン"),
            MachiKind::Bianzhang => write!(b, "ペンチャン"),
            MachiKind::Qianzhang => write!(b, "カンチャン"),
            MachiKind::Danqi => write!(b, "単騎"),
            MachiKind::Yandan => write!(b, "ノベタン"),
        }
    }
}

pub fn enumerate_machis(
    quetou: &Tiles,
    anshuns: &[Tiles],
    ankes: &[Tiles],
    last: Tile,
) -> Vec<MachiKind> {
    MachiEnumerator::new(quetou, anshuns, ankes, last).enumerate()
}

/// 待ちを列挙するヘルパー
struct MachiEnumerator<'a> {
    quetou: &'a Tiles,
    anshuns: &'a [Tiles],
    ankes: &'a [Tiles],
    last: Tile,
}

impl<'a> MachiEnumerator<'a> {
    fn new(
        quetou: &'a Tiles,
        anshuns: &'a [Tiles],
        ankes: &'a [Tiles],
        last: Tile,
    ) -> MachiEnumerator<'a> {
        MachiEnumerator {
            quetou,
            anshuns,
            ankes,
            last,
        }
    }

    /// 待ちを列挙する
    fn enumerate(&self) -> Vec<MachiKind> {
        let mut result = Vec::new();

        if self.is_liangmian() {
            result.push(MachiKind::Liangmian);
        }

        if self.is_shuangpeng() {
            result.push(MachiKind::Shuangpeng);
        }

        if self.is_bianzhang() {
            result.push(MachiKind::Bianzhang);
        }

        if self.is_qianzhang() {
            result.push(MachiKind::Qianzhang);
        }

        if self.is_danqi() {
            result.push(MachiKind::Danqi);
        }

        if self.is_yandan() {
            result.push(MachiKind::Yandan);
        }

        result
    }

    /// 両面待ちにあたるかどうか。
    fn is_liangmian(&self) -> bool {
        self.is_liangmian_bianzhang() == Some(MachiKind::Liangmian)
    }

    /// シャンポン待ちにあたるかどうか。
    fn is_shuangpeng(&self) -> bool {
        // 最後に引いてきたやつが刻子を構成しているなら、雀頭と合わせてシャンポン待ちになっていたは
        // ず。
        self.ankes.iter().any(|kezi| kezi.first() == self.last)
    }

    /// ペンチャン待ちにあたるかどうか。
    fn is_bianzhang(&self) -> bool {
        self.is_liangmian_bianzhang() == Some(MachiKind::Bianzhang)
    }

    /// カンチャン待ちにあたるかどうか。
    fn is_qianzhang(&self) -> bool {
        (self.anshuns.iter()).any(|shunzi| shunzi.middle() == self.last)
    }

    /// 単騎待ちにあたるかどうか。
    fn is_danqi(&self) -> bool {
        self.is_danqi_yandan() == Some(MachiKind::Danqi)
    }

    /// ノベタンにあたるかどうか。
    fn is_yandan(&self) -> bool {
        self.is_danqi_yandan() == Some(MachiKind::Yandan)
    }

    fn is_liangmian_bianzhang(&self) -> Option<MachiKind> {
        // まずオーダーをとる。そもそも字牌なら両面もペンチャンもありえない。
        let order_last = self.last.order()?;

        for shunzi in self.anshuns.iter() {
            // そもそも両端でないなら次へ
            if self.last != shunzi.first() && self.last != shunzi.last() {
                continue;
            }

            let order_shunzi_first = match shunzi.first().order() {
                Some(o) => o,
                None => continue,
            };

            let order_shunzi_last = match shunzi.last().order() {
                Some(o) => o,
                None => continue,
            };

            // このときは両面かペンチャンかが発生する。

            // この順子の左端か右端は最後に引いてきた牌。
            // この順子の右端が3かつ3を引いてきた場合か、
            // この順子の左端が7かつ7を引いてきた場合はペンチャン待ち。
            let order_3 = Order::new(3).unwrap();
            if order_shunzi_last == order_3 && order_last == order_3 {
                return Some(MachiKind::Bianzhang);
            }

            let order_7 = Order::new(7).unwrap();
            if order_shunzi_first == order_7 && order_last == order_7 {
                return Some(MachiKind::Bianzhang);
            }

            // ここまでこれば、この牌で両面待ちとできる。
            return Some(MachiKind::Liangmian);
        }

        None
    }

    fn is_danqi_yandan(&self) -> Option<MachiKind> {
        // まず最後の牌が雀頭になっている必要がある。
        if self.quetou.first() != self.last {
            return None;
        }

        // その上でその牌が順子と連続していなければ単騎待ちとなる。逆にそうでなければノベタンとな
        // る。なぜなら、もしある順子が雀頭と連続していれば、もともと4連続の順子だったところにその両
        // 端の牌を引いてきたことになり、これはノベタンとなるからである。
        for shunzi in self.anshuns {
            if Some(shunzi.first()) == self.last.next() {
                return Some(MachiKind::Yandan);
            }

            if Some(shunzi.last()) == self.last.prev() {
                return Some(MachiKind::Yandan);
            }
        }

        Some(MachiKind::Danqi)
    }
}

/// ロンによる明刻・明順を保持する。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Rongming {
    Mingke(Tiles),
    Mingshun(Tiles),
    None,
}

impl Rongming {
    pub fn mingke(&self) -> Option<&Tiles> {
        match self {
            Rongming::Mingke(tiles) => Some(tiles),
            Rongming::Mingshun(_) => None,
            Rongming::None => None,
        }
    }

    pub fn mingshun(&self) -> Option<&Tiles> {
        match self {
            Rongming::Mingke(_) => None,
            Rongming::Mingshun(tiles) => Some(tiles),
            Rongming::None => None,
        }
    }

    pub fn iter_mingke(&self) -> impl Iterator<Item = &Tiles> {
        self.mingke().into_iter()
    }

    pub fn iter_mingshun(&self) -> impl Iterator<Item = &Tiles> {
        self.mingshun().into_iter()
    }
}

/// アガリ形に整理された牌集合たち。
#[derive(Debug, Clone)]
pub struct AgariTilesets {
    tilesets: Tilesets,
    machi: MachiKind,
    rongming: Rongming,
    quetou: Tiles,
    kezis_in_hand: Vec<Tiles>,
    shunzis_in_hand: Vec<Tiles>,
}

impl AgariTilesets {
    pub fn enumerate(tilesets: &Tilesets) -> Vec<AgariTilesets> {
        // ツモにせよロンにせよ、とりあえず最後に引いた牌を手牌にくっつけておく。
        let hand = {
            let mut v = tilesets.hand.clone();
            v.push(tilesets.last);
            v
        };
        let mut res = Vec::new();

        // 雀頭のありえかたを列挙する
        for (quetou, rest) in enumerate_quetou(&hand) {
            // 刻子を全て列挙する
            for (kezis_in_hand, rest) in enumerate_kezi(&rest) {
                // 順子を全てとりだす
                let shunzis_in_hand = match extract_shunzi(rest) {
                    // あがれなかった (取り出し方が不適切、など)
                    None => continue,

                    // あがれた
                    Some(t) => t,
                };

                // 待ちを全て列挙する
                let last = tilesets.last;
                let machis = enumerate_machis(&quetou, &shunzis_in_hand, &kezis_in_hand, last);

                // それら全てに対して AgariTilesets を作成する
                for machi in machis {
                    res.push(AgariTilesets::new(
                        tilesets.clone(),
                        machi,
                        quetou.clone(),
                        kezis_in_hand.clone(),
                        shunzis_in_hand.clone(),
                    ));
                }
            }
        }

        res
    }

    fn new(
        mut tilesets: Tilesets,
        machi: MachiKind,
        quetou: Tiles,
        mut kezis_in_hand: Vec<Tiles>,
        mut shunzis_in_hand: Vec<Tiles>,
    ) -> AgariTilesets {
        let rongming = fix_rong_an_mings(
            &mut tilesets,
            &quetou,
            &mut kezis_in_hand,
            &mut shunzis_in_hand,
        );

        AgariTilesets {
            tilesets,
            machi,
            rongming,
            quetou,
            kezis_in_hand,
            shunzis_in_hand,
        }
    }

    /// ポン
    pub fn pengs(&self) -> impl Iterator<Item = &Tiles> {
        self.tilesets.pengs.iter()
    }

    /// チー
    pub fn chis(&self) -> impl Iterator<Item = &Tiles> {
        self.tilesets.chis.iter()
    }

    /// 明槓
    pub fn minggangs(&self) -> impl Iterator<Item = &Tiles> {
        self.tilesets.minggangs.iter()
    }

    /// 暗槓
    pub fn angangs(&self) -> impl Iterator<Item = &Tiles> {
        self.tilesets.angangs.iter()
    }

    /// 手札の刻子。
    pub fn kezis_in_hand(&self) -> impl Iterator<Item = &Tiles> {
        self.kezis_in_hand.iter()
    }

    /// 手札の順子。
    pub fn shunzis_in_hand(&self) -> impl Iterator<Item = &Tiles> {
        self.shunzis_in_hand.iter()
    }

    /// 明刻。ポンと明槓、ロンによる明刻を合わせたもの。
    pub fn mingkes(&self) -> impl Iterator<Item = &Tiles> {
        self.pengs()
            .chain(self.minggangs())
            .chain(self.ronghe_mingke())
    }

    /// 暗刻。手札の刻子と暗槓を合わせたもの。
    pub fn ankes(&self) -> impl Iterator<Item = &Tiles> {
        self.kezis_in_hand().chain(self.angangs())
    }

    /// 明順。チーとロンによる順子を合わせたもの。
    pub fn mingshuns(&self) -> impl Iterator<Item = &Tiles> {
        self.chis().chain(self.ronghe_mingshun())
    }

    /// 暗順。手札の順子のみ。
    pub fn anshuns(&self) -> impl Iterator<Item = &Tiles> {
        self.shunzis_in_hand()
    }

    /// 刻子。明刻、暗刻を合わせたもの。
    pub fn kezis(&self) -> impl Iterator<Item = &Tiles> {
        self.mingkes().chain(self.ankes())
    }

    /// 順子。明順、暗順を合わせたもの。
    pub fn shunzis(&self) -> impl Iterator<Item = &Tiles> {
        self.mingshuns().chain(self.anshuns())
    }

    /// 面子。刻子と順子を合わせたもの。
    pub fn mianzis(&self) -> impl Iterator<Item = &Tiles> {
        self.kezis().chain(self.shunzis())
    }

    /// 雀頭。
    pub fn quetou(&self) -> &Tiles {
        &self.quetou
    }

    /// ロンによる明刻。
    pub fn ronghe_mingke(&self) -> impl Iterator<Item = &Tiles> {
        self.rongming.iter_mingke()
    }

    /// ロンによる明順。
    pub fn ronghe_mingshun(&self) -> impl Iterator<Item = &Tiles> {
        self.rongming.iter_mingshun()
    }

    /// 待ち。
    pub fn machi(&self) -> MachiKind {
        self.machi
    }

    /// 門前かどうか
    pub fn is_menqian(&self) -> bool {
        self.tilesets.is_menqian()
    }

    /// ツモかどうか
    pub fn is_zimo(&self) -> bool {
        self.tilesets.is_zimo
    }

    /// コンテキスト。
    pub fn context(&self) -> &Context {
        &self.tilesets.context
    }

    /// 牌集合。
    pub fn tilesets(&self) -> &Tilesets {
        &self.tilesets
    }
}

impl fmt::Display for AgariTilesets {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        for mianzi in self.mianzis() {
            write!(b, "{} ", mianzi)?;
        }

        write!(b, "{} ", self.quetou())?;
        write!(b, "待ち: {}", self.machi)?;

        Ok(())
    }
}

/// ロンによる暗刻と明刻を調整する。ロンによってできた刻子は明刻として扱うルールがあるため、最初は
/// 手牌の暗刻として扱われているものを一つ明刻へ移さなければならない。
fn fix_rong_an_mings(
    tilesets: &mut Tilesets,
    quetou: &Tiles,
    kezis_in_hand: &mut Vec<Tiles>,
    shunzis_in_hand: &mut Vec<Tiles>,
) -> Rongming {
    // ツモなら関係がない。
    if tilesets.is_zimo {
        return Rongming::None;
    }

    let last = tilesets.last;

    // まず、暗順に `last` を含む順子があるなら、ロンした牌は常にその順子を作ったと考える。刻子では
    // 暗刻か明刻かが重要になるのに対し、順子に関しては鳴いたかどうかは問題でも暗順か明順かの区別は
    // 一切無関係である。よって、どのような場合も常に順子を優先的に明順にする方がよい。
    for anshuns in &mut [shunzis_in_hand] {
        if let Some(pos) = anshuns.iter().position(|jun| jun.contains(&last)) {
            return Rongming::Mingshun(anshuns.remove(pos));
        }
    }

    // そうでないなら仕方ないので刻子を確認する。
    for ankes in &mut [kezis_in_hand, &mut tilesets.angangs] {
        if let Some(pos) = ankes.iter().position(|ko| ko.contains(&last)) {
            return Rongming::Mingke(ankes.remove(pos));
        }
    }

    // いずれでもなければ必ず雀頭になっているはず。
    assert_eq!(
        tilesets.last,
        quetou.first(),
        "ロンした牌によって順子も刻子も雀頭も作られていません。"
    );

    Rongming::None
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
fn enumerate_quetou(tiles: &Tiles) -> Vec<(Tiles, Tiles)> {
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
fn enumerate_kezi(tiles: &Tiles) -> Vec<(Vec<Tiles>, Tiles)> {
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
        let mut kezis = Vec::new();

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
                kezis.push(Tiles::new(tiles.drain(range).collect()));
            }
        }

        assert_eq!(
            set.count_ones() as usize,
            kezis.len(),
            "to_use で立っているビットの個数と実際に選ばれた Tiles の個数が違います。"
        );

        // 分解した刻子と残った牌をおいておく。
        res.push((kezis, tiles));
    }

    res
}

/// 刻子をのぞいた牌から順子を貪欲に分解する。
fn extract_shunzi(mut tiles: Tiles) -> Option<Vec<Tiles>> {
    assert!(
        tiles.len() % 3 == 0,
        "残りの牌の個数が3の倍数ではありません : {}",
        tiles.len()
    );

    // チェックしたいんだけど is_sorted() が unstable であった。ある時点で牌集合にいる一番後ろの牌が
    // 「最大」であることに依存するのでソートされている必要がある。

    // assert!(
    //     shunzi_tiles.is_sorted(),
    //     "順子がソートされていません。"
    // );

    // 残りは順子しかないはずなので貪欲に分解する。
    let mut shunzis = Vec::new();

    // なくなるまでループ
    while !tiles.is_empty() {
        fn pop_first(tiles: &mut Tiles, t: Tile) -> Option<Tile> {
            // 順序を保ちたいので swap_remove() は使えない
            tiles.iter().position(|&s| s == t).map(|i| tiles.remove(i))
        }

        // とりあえず最初の牌をとる
        let first = tiles.remove(0);

        // 最後の牌の一つ前を見つけてきて取り出す。
        let mid = pop_first(&mut tiles, first.next()?)?;

        // 真ん中の牌の一つ前を見つけてきて取り出す。
        let last = pop_first(&mut tiles, mid.next()?)?;

        // 以下はとり方から成立するはずのことたち
        assert_eq!(first.next(), Some(mid));
        assert_eq!(mid.next(), Some(last));
        assert!(tiles.len() % 3 == 0);

        shunzis.push(Tiles::new(vec![first, mid, last]));
    }

    Some(shunzis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::tileset::{Tag, Tileset};

    #[test]
    fn machi() {
        let quetou = &"4s4s".parse().unwrap();
        let anshuns = &["1s2s3s".parse().unwrap()];
        let ankes = &[
            "1s1s1s".parse().unwrap(),
            "1m1m1m".parse().unwrap(),
            "2m2m2m".parse().unwrap(),
        ];

        assert_eq!(
            &[MachiKind::Liangmian, MachiKind::Shuangpeng],
            &*enumerate_machis(quetou, anshuns, ankes, "1s".parse().unwrap())
        );

        assert_eq!(
            &[MachiKind::Qianzhang],
            &*enumerate_machis(quetou, anshuns, ankes, "2s".parse().unwrap())
        );

        assert_eq!(
            &[MachiKind::Bianzhang],
            &*enumerate_machis(quetou, anshuns, ankes, "3s".parse().unwrap())
        );

        assert_eq!(
            &[MachiKind::Shuangpeng],
            &*enumerate_machis(quetou, anshuns, ankes, "1m".parse().unwrap())
        );

        assert_eq!(
            &[MachiKind::Qianzhang, MachiKind::Danqi],
            &*enumerate_machis(
                &"2s2s".parse().unwrap(),
                anshuns,
                ankes,
                "2s".parse().unwrap()
            )
        );

        assert_eq!(
            &[MachiKind::Yandan],
            &*enumerate_machis(quetou, anshuns, ankes, "4s".parse().unwrap())
        );
    }

    fn tiles(tiles: &str) -> Tiles {
        tiles.parse().unwrap()
    }

    fn tilesets(hand: &str, last: &str) -> Tilesets {
        let tilesets = vec![
            Tileset::new(Tag::Hand, tiles(hand)).unwrap(),
            Tileset::new(Tag::Zimo, tiles(last)).unwrap(),
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
    fn quetou() {
        let hand = tiles("1s1s1s2s2s2s3s3s3s4s4s4s東東");
        let cands = enumerate_quetou(&hand);

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
    fn decompose1() {
        let tilesets = tilesets("1s1s1s2s2s2s3s3s3s4s4s4s東", "東");
        let agaris = AgariTilesets::enumerate(&tilesets);

        assert_eq!(agaris.len(), 3);
        eprintln!("{:#?}", agaris);
    }

    #[test]
    fn decompose2() {
        let tilesets = tilesets("中中中白白白發發發東東東西", "西");
        let agaris = AgariTilesets::enumerate(&tilesets);

        assert_eq!(agaris.len(), 1);
        eprintln!("{:#?}", agaris);
    }
}
