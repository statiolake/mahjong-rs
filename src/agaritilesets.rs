//! アガリ形を保持する牌集合を定義する。

use crate::tile::Order;
use crate::tile::Tile;
use crate::tileset::Tiles;
use crate::tilesets::Tilesets;
use std::fmt;

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
pub struct AgariTilesets {
    tilesets: Tilesets,
    machi: MachiKind,
    ronmin: RonMin,
    janto: Tiles,
    kotus_in_hand: Vec<Tiles>,
    juntus_in_hand: Vec<Tiles>,
}

impl AgariTilesets {
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
    fn minkos(&self) -> impl Iterator<Item = &Tiles> {
        self.tilesets
            .pons
            .iter()
            .chain(self.tilesets.minkans.iter())
            .chain(self.ronmin.iter_minko())
    }

    /// 暗刻。手札の刻子と暗槓を合わせたもの。
    fn ankos(&self) -> impl Iterator<Item = &Tiles> {
        self.kotus_in_hand.iter().chain(self.tilesets.ankans.iter())
    }

    /// 明順。チーとロンによる順子を合わせたもの。
    fn minjuns(&self) -> impl Iterator<Item = &Tiles> {
        self.tilesets.qis.iter().chain(self.ronmin.iter_minjun())
    }

    /// 暗順。手札の順子のみ。
    fn anjuns(&self) -> impl Iterator<Item = &Tiles> {
        self.juntus_in_hand.iter()
    }

    /// 刻子。明刻、暗刻を合わせたもの。
    fn kotus(&self) -> impl Iterator<Item = &Tiles> {
        self.minkos().chain(self.ankos())
    }

    /// 順子。明順、暗順を合わせたもの。
    fn juntus(&self) -> impl Iterator<Item = &Tiles> {
        self.minjuns().chain(self.anjuns())
    }

    /// 面子。刻子と順子を合わせたもの。
    fn mentus(&self) -> impl Iterator<Item = &Tiles> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machi() {
        let order_1 = Order::new(1).unwrap();
        let order_2 = Order::new(2).unwrap();
        let order_3 = Order::new(3).unwrap();
        let order_4 = Order::new(4).unwrap();

        let m1 = Tile::Manzu(order_1);
        let m2 = Tile::Manzu(order_2);
        let s1 = Tile::Souzu(order_1);
        let s2 = Tile::Souzu(order_2);
        let s3 = Tile::Souzu(order_3);
        let s4 = Tile::Souzu(order_4);

        let janto = &Tiles::new(vec![s4, s4]);
        let anjuns = &[Tiles::new(vec![s1, s2, s3])];
        let ankos = &[
            Tiles::new(vec![s1, s1, s1]),
            Tiles::new(vec![m1, m1, m1]),
            Tiles::new(vec![m2, m2, m2]),
        ];

        assert_eq!(
            &[MachiKind::Ryanmen, MachiKind::Shanpon],
            &*enumerate_machis(janto, anjuns, ankos, s1)
        );

        assert_eq!(
            &[MachiKind::Kanchan],
            &*enumerate_machis(janto, anjuns, ankos, s2)
        );

        assert_eq!(
            &[MachiKind::Penchan],
            &*enumerate_machis(janto, anjuns, ankos, s3)
        );

        assert_eq!(
            &[MachiKind::Shanpon],
            &*enumerate_machis(janto, anjuns, ankos, m1)
        );

        assert_eq!(
            &[MachiKind::Kanchan, MachiKind::Tanki],
            &*enumerate_machis(&Tiles::new(vec![s2, s2]), anjuns, ankos, s2)
        );

        assert_eq!(
            &[MachiKind::Nobetan],
            &*enumerate_machis(janto, anjuns, ankos, s4)
        );
    }
}
