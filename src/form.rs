//! 役を定義する。

use crate::agaritilesets::{AgariTilesets, MachiKind};
use crate::config::Riichi;
use crate::tile::{Jihai, Order, Tile, TileKind};
use crate::tiles::Tiles;
use crate::tilesets::Tilesets;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter::once;

type SmallVec = smallvec::SmallVec<[Form; 4]>;

/// 翻数・符数。
#[derive(Debug, Clone, Copy)]
pub struct Point {
    /// 翻数。
    fan: u32,

    /// 負数。
    fu: u32,

    /// 役満かどうか。
    is_true_yakuman: bool,
}

impl Point {
    pub fn new(fan: u32) -> Point {
        Point {
            fan,
            fu: 0,
            is_true_yakuman: false,
        }
    }

    pub fn with_fu(fan: u32, fu: u32) -> Point {
        Point {
            fan,
            fu,
            is_true_yakuman: false,
        }
    }

    pub fn new_yakuman() -> Point {
        Point {
            fan: 13,
            fu: 0,
            is_true_yakuman: true,
        }
    }

    pub fn is_yakuman(&self) -> bool {
        self.is_true_yakuman || self.fan >= 13
    }
}

impl fmt::Display for Point {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        if self.is_yakuman() {
            write!(b, "役満")
        } else if self.fu == 0 {
            write!(b, "{}翻", self.fan)
        } else {
            write!(b, "{}翻{}符", self.fan, self.fu)
        }
    }
}

/// 役。
#[derive(Debug, Clone, Copy)]
pub struct Form {
    /// 名前
    name: &'static str,

    /// 翻数
    point: Point,
}

impl fmt::Display for Form {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{}　{}", self.point, self.name)
    }
}

/// [1]立直・[2]ダブルリーチ・[1]一発
fn special_check_lizhi(tilesets: &Tilesets) -> impl IntoIterator<Item = Form> {
    let lizhi = || Form {
        name: "立直",
        point: Point::new(1),
    };

    let ippatu = || Form {
        name: "一発",
        point: Point::new(1),
    };

    let double_lizhi = || Form {
        name: "ダブル立直",
        point: Point::new(1),
    };

    // 立直類は最初から指定されており、 Context として渡されている。
    match tilesets.context.riichi {
        Riichi::None => SmallVec::new(),
        Riichi::Riichi => SmallVec::from_elem(lizhi(), 1),
        Riichi::RiichiIppatu => {
            let mut v = SmallVec::new();
            v.push(lizhi());
            v.push(ippatu());
            v
        }
        Riichi::DoubleRiichi => SmallVec::from_elem(double_lizhi(), 1),
        Riichi::DoubleRiichiIppatu => {
            let mut v = SmallVec::new();
            v.push(double_lizhi());
            v.push(ippatu());
            v
        }
    }
}

/// [2.5]七対子
fn checkall_qiduizi(tilesets: &Tilesets) -> impl IntoIterator<Item = Form> {
    // ポン・チー・カンをしていたら七対子にならないので終了。
    if tilesets.did_furo() || !tilesets.ankans.is_empty() {
        return Vec::new();
    }

    let tiles = &tilesets.hand;
    assert_eq!(
        tiles.len(),
        14,
        "なぜか牌が足りません: {}",
        tiles.len()
    );

    // 七対子
    let mut map: HashMap<Tile, u8> = HashMap::new();
    for &tile in tiles.iter() {
        *map.entry(tile).or_default() += 1;
    }

    // 一つでも2枚じゃない牌があれば七対子ではない。
    if map.iter().any(|(_, &cnt)| cnt != 2) {
        return Vec::new();
    }

    // そうであれば七対子
    once(Form {
        name: "七対子",
        point: Point::with_fu(2, 25),
    })
    .chain(special_check_lizhi(tilesets))
    .chain(special_check_menqianqingzimohu(tilesets))
    .chain(special_check_ziyise(tilesets))
    .chain(special_check_hungyise_qingyise(tilesets))
    .chain(special_check_hunlaotou(tilesets))
    .chain(special_check_duanyaojiu(tilesets))
    .collect()
}

/// [1]門前清自摸和
///
/// - 門前でツモ上がりをした。
fn special_check_menqianqingzimohu(tilesets: &Tilesets) -> impl IntoIterator<Item = Form> {
    if tilesets.is_tumo && tilesets.is_menzen() {
        Some(Form {
            name: "門前清自摸和",
            point: Point::new(1),
        })
    } else {
        None
    }
}

/// [1]断么九
///
/// - 手牌が全て中張牌である。
fn special_check_duanyaojiu(tilesets: &Tilesets) -> impl IntoIterator<Item = Form> {
    let is_chunchan = tilesets.tiles_without_doras().all(|t| t.is_chunchan());

    if is_chunchan {
        Some(Form {
            name: "断么九",
            point: Point::new(1),
        })
    } else {
        None
    }
}

/// [6/5]清一色・[3/2]混一色
///
/// 〈清一色〉
/// - どれか一種類の牌だけで構成する。
///
/// 〈混一色〉
/// - どれか一種類の牌と字牌だけで構成する。
fn special_check_hungyise_qingyise(tilesets: &Tilesets) -> impl IntoIterator<Item = Form> {
    // 各面子・雀頭の種類
    let kinds = || {
        tilesets
            .tiles_without_doras()
            .map(|tile| tile.kind())
            .filter(|&kind| kind != TileKind::Jihai)
    };

    // 字牌があるかどうか
    let has_jihai = kinds().any(|kind| kind == TileKind::Jihai);

    // 字牌でない雀頭の種類
    let kinds_not_jihai = || kinds().filter(|&kind| kind != TileKind::Jihai);

    // 対象となる種類
    let target_kind = kinds_not_jihai().next()?;

    // 全てが同じかどうか
    let all_same = kinds_not_jihai().all(|kind| kind == target_kind);

    match (all_same, has_jihai) {
        (true, false) => Some(Form {
            name: "清一色",
            point: Point::new(if tilesets.is_menzen() { 6 } else { 5 }),
        }),
        (true, true) => Some(Form {
            name: "混一色",
            point: Point::new(if tilesets.is_menzen() { 3 } else { 2 }),
        }),
        _ => None,
    }
}

/// [2]混老頭
///
/// - 全ての面子が幺九牌で構成されている。
fn special_check_hunlaotou(tilesets: &Tilesets) -> impl IntoIterator<Item = Form> {
    let is_hunalotou = tilesets.tiles_without_doras().all(|tile| tile.is_yaochu());

    if is_hunalotou {
        Some(Form {
            name: "混老頭",
            point: Point::new(2),
        })
    } else {
        None
    }
}

/// 特別な形のある役 (国士無双、九蓮宝燈など)
///
/// `target` はベースとなる形 (国士無双なら 1s9s1m9m1p9p東南西北白發中 など) で、これプラスその形の
/// どれか一つの牌だけがダブっている状態がアガリとなる。 `name_genuine` は純正の場合、つまり最初から
/// `target` がそろっていて最後に引いた牌がダブっている場合につく。たとえば国士無双13面待ちなど。
fn special_check_certainform(
    tilesets: &Tilesets,
    mut target: Tiles,
    name_genuine: &'static str,
    name: &'static str,
    point: Point,
) -> impl IntoIterator<Item = Form> {
    // ポン・チー・カンをしていたらならないので終了。
    if tilesets.did_furo() || !tilesets.ankans.is_empty() {
        return None;
    }

    // 手札を追加する。
    let hand: Tiles = tilesets
        .hand
        .iter()
        .copied()
        .chain(once(tilesets.last))
        .collect();

    // これらの牌の一つを選んで、それを追加したものと手牌が一致するかどうかを確かめる。
    for add in target.clone().into_inner().into_iter() {
        target.push(add);

        // 一致した場合は国士無双成立。
        if target == hand {
            return if add == tilesets.last {
                Some(Form {
                    name: name_genuine,
                    point,
                })
            } else {
                Some(Form { name, point })
            };
        }

        target.pop();
    }

    None
}

/// [13]国士無双
fn special_check_kokusimusou(tilesets: &Tilesets) -> impl IntoIterator<Item = Form> {
    special_check_certainform(
        tilesets,
        Tiles::new(vec![
            Tile::Souzu(Order::new(1).unwrap()),
            Tile::Souzu(Order::new(9).unwrap()),
            Tile::Manzu(Order::new(1).unwrap()),
            Tile::Manzu(Order::new(9).unwrap()),
            Tile::Pinzu(Order::new(1).unwrap()),
            Tile::Pinzu(Order::new(9).unwrap()),
            Tile::Jihai(Jihai::East),
            Tile::Jihai(Jihai::South),
            Tile::Jihai(Jihai::West),
            Tile::Jihai(Jihai::North),
            Tile::Jihai(Jihai::Haku),
            Tile::Jihai(Jihai::Hatu),
            Tile::Jihai(Jihai::Chun),
        ]),
        "国士無双13面待ち",
        "国士無双",
        Point::new_yakuman(),
    )
}

/// [13]九蓮宝燈
fn special_check_jiulianbaodeng<'a>(tilesets: &'a Tilesets) -> impl IntoIterator<Item = Form> + 'a {
    let constructors: Vec<fn(Order) -> Tile> = vec![Tile::Souzu, Tile::Manzu, Tile::Pinzu];

    let orders = vec![
        Order::new(1).unwrap(),
        Order::new(1).unwrap(),
        Order::new(1).unwrap(),
        Order::new(2).unwrap(),
        Order::new(3).unwrap(),
        Order::new(4).unwrap(),
        Order::new(5).unwrap(),
        Order::new(6).unwrap(),
        Order::new(7).unwrap(),
        Order::new(8).unwrap(),
        Order::new(9).unwrap(),
        Order::new(9).unwrap(),
        Order::new(9).unwrap(),
    ];

    constructors
        .into_iter()
        .scan(orders, move |orders, ctor| {
            Some(special_check_certainform(
                tilesets,
                orders.iter().copied().map(|o| ctor(o)).collect(),
                "純正九蓮宝燈",
                "九蓮宝燈",
                Point::new_yakuman(),
            ))
        })
        .flatten()
}

/// [13]緑一色
///
/// - 全ての牌が緑一色を構成する牌である。
fn special_check_luyise(tilesets: &Tilesets) -> impl IntoIterator<Item = Form> {
    let is_luyise = tilesets.tiles_without_doras().all(|tile| tile.is_green());

    if is_luyise {
        Some(Form {
            name: "緑一色",
            point: Point::new_yakuman(),
        })
    } else {
        None
    }
}

/// [13]字一色
///
/// - 全ての牌が字牌である。
fn special_check_ziyise(tilesets: &Tilesets) -> impl IntoIterator<Item = Form> {
    let is_ziyise = tilesets
        .tiles_without_doras()
        .all(|tile| tile.kind() == TileKind::Jihai);

    if is_ziyise {
        Some(Form {
            name: "字一色",
            point: Point::new_yakuman(),
        })
    } else {
        None
    }
}

/// [13]清老頭
///
/// - 全ての牌が 1,9 牌のみである。
fn special_check_qinglaotou(tilesets: &Tilesets) -> impl IntoIterator<Item = Form> {
    let is_qinglaotou = tilesets
        .tiles_without_doras()
        .all(|tile| tile.kind() != TileKind::Jihai && tile.is_yaochu());

    if is_qinglaotou {
        Some(Form {
            name: "清老頭",
            point: Point::new_yakuman(),
        })
    } else {
        None
    }
}

/// [n]役牌
///
/// - 刻子・槓子が役牌である。一つにつき1翻。
fn check_fanpai(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    let mut sum = 0;
    for tile in agari.kotus() {
        sum += tile.first().num_yakuhai(&agari.tilesets.context);
    }

    if sum != 0 {
        Some(Form {
            name: "役牌",
            point: Point::new(sum),
        })
    } else {
        None
    }
}

/// [1/0]平和
///
/// - 4面子が順子である。
/// - 雀頭が役牌でない。
/// - 両面待ちである。
fn check_pinghe(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    if agari.tilesets.is_menzen()
        && agari.juntus().count() == 4
        && agari.janto().first().kind() == TileKind::Jihai
        && agari.machi == MachiKind::Ryanmen
    {
        Some(Form {
            name: "平和",
            point: Point::new(1),
        })
    } else {
        None
    }
}

/// [1/0]一盃口・[3/0]二盃口
///
/// - 門前である
///
/// 〈一盃口〉
/// - 同種の牌で同じ順序の順子が2面子ある。
///
/// 〈二盃口〉
/// - 同種の牌で同じ順序の順子が2面子、これが2組ある。一盃口二つ。
fn check_yibeikou_liangbeigou(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    if !agari.tilesets.is_menzen() {
        return None;
    }

    let mut map = HashMap::new();
    for juntu in agari.juntus() {
        *map.entry(juntu.first()).or_default() += 1;
    }

    let mut cnt = 0;
    for (_, num) in map {
        match num {
            // 同じ牌が 4 枚あるならそれで二盃口が構成される。
            4 => cnt += 2,
            // 同じ牌が 2 または 3 あるならそれで一盃口が構成される。
            // 今後他の牌についてまた一盃口が構成されれば二盃口となる。
            2 | 3 => cnt += 1,
            _ => {}
        }
    }

    match cnt {
        0 => None,
        1 => Some(Form {
            name: "一盃口",
            point: Point::new(1),
        }),
        2 => Some(Form {
            name: "二盃口",
            point: Point::new(3),
        }),
        _ => panic!("二盃口以上があります。"),
    }
}

/// [2/1]三色同順
///
/// - 索子・萬子・筒子で同じ数字から始まる順子を作る。
fn check_sanshoku_dojun(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    // 「その順序から始まる順子にはどの種類の牌があるか」を集める
    let mut map: HashMap<Option<Order>, HashSet<TileKind>> = HashMap::new();
    for tile in agari.juntus().map(|t| t.first()) {
        map.entry(tile.order()).or_default().insert(tile.kind());
    }

    // そのなかのある一つの順序について、索子も萬子も筒子もあるということなら三色同順
    let does_match = map.into_iter().any(|(_, kinds)| {
        kinds.contains(&TileKind::Souzu)
            && kinds.contains(&TileKind::Manzu)
            && kinds.contains(&TileKind::Pinzu)
    });

    if does_match {
        // 喰い下がりがあるので注意。
        Some(Form {
            name: "三色同順",
            point: Point::new(if agari.tilesets.is_menzen() { 2 } else { 1 }),
        })
    } else {
        None
    }
}

/// [2]三色同刻
///
/// - 索子・萬子・筒子で同じ数字からなる刻子を作る。
fn check_sanshoku_doko(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    // 「その順序から始まる刻子にはどの種類の牌があるか」を集める
    let mut map: HashMap<Option<Order>, HashSet<TileKind>> = HashMap::new();
    for tile in agari.kotus().map(|t| t.first()) {
        map.entry(tile.order()).or_default().insert(tile.kind());
    }

    // そのなかのある一つの順序について、索子も萬子も筒子もあるということなら三色同刻
    let does_match = map.into_iter().any(|(_, kinds)| {
        kinds.contains(&TileKind::Souzu)
            && kinds.contains(&TileKind::Manzu)
            && kinds.contains(&TileKind::Pinzu)
    });

    if does_match {
        Some(Form {
            name: "三色同順",
            point: Point::new(2),
        })
    } else {
        None
    }
}

/// [13]四暗刻・[2]三暗刻
///
/// 〈四暗刻〉
/// - 暗刻が4つある
///
/// 〈三暗刻〉
/// - 暗刻が3つある
fn check_sananke_sianke(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    if agari.ankos().count() == 4 {
        if agari.machi == MachiKind::Tanki {
            Some(Form {
                name: "四暗刻単騎",
                point: Point::new_yakuman(),
            })
        } else {
            Some(Form {
                name: "四暗刻",
                point: Point::new_yakuman(),
            })
        }
    } else if agari.ankos().count() == 3 {
        Some(Form {
            name: "三暗刻",
            point: Point::new(2),
        })
    } else {
        None
    }
}

/// [2/1]一気通貫
///
/// - どれか一種類の牌で 123 456 789 を達成する
fn check_ikki_tukan(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    let mut map: HashMap<TileKind, HashSet<Option<Order>>> = HashMap::new();
    for tile in agari.juntus().map(|t| t.first()) {
        map.entry(tile.kind()).or_default().insert(tile.order());
    }

    let does_match = map.into_iter().any(|(_, orders)| {
        orders.contains(&Some(Order::new(1).unwrap()))
            && orders.contains(&Some(Order::new(4).unwrap()))
            && orders.contains(&Some(Order::new(7).unwrap()))
    });

    if does_match {
        Some(Form {
            name: "一気通貫",
            point: Point::new(if agari.tilesets.is_menzen() { 2 } else { 1 }),
        })
    } else {
        None
    }
}

/// [2]対々和
///
/// - 刻子が4つある。
fn check_duiduihe(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    if agari.kotus().count() == 4 {
        Some(Form {
            name: "対々和",
            point: Point::new(2),
        })
    } else {
        None
    }
}

/// [2/1]混全帯幺九・[3/2]純全帯公九
///
/// 〈混全帯幺九〉
/// - 全ての面子と雀頭に幺九牌が絡んでいる。
/// 〈純全帯公九〉
/// - 全ての面子と雀頭に 1, 9 が絡んでいる。
fn check_hunquandaiyaojiu_chunquandaiyaojiu_hunlaotou(
    agari: &AgariTilesets,
) -> impl IntoIterator<Item = Form> {
    let mut has_jihai = false;
    let mut has_chunchan = false;

    for tiles in agari.mentus().chain(once(agari.janto())) {
        // その面子の牌の全てが中張牌であれば対象の役のどれも成立しえないので放置。
        if tiles.iter().all(|&tile| tile.is_chunchan()) {
            return None;
        }

        has_jihai = has_jihai || tiles.iter().any(|&tile| tile.kind() == TileKind::Jihai);
        has_chunchan = has_chunchan || tiles.iter().any(|tile| tile.is_chunchan());
    }

    match (has_jihai, has_chunchan) {
        (false, true) => Some(Form {
            name: "純全帯公九",
            point: Point::new(3),
        }),
        (true, true) => Some(Form {
            name: "混全帯幺九",
            point: Point::new(3),
        }),
        // 混老頭は別扱いのため、ここでは None
        _ => None,
    }
}

/// [13]四槓子・[2]三槓子
///
/// 〈四槓子〉
/// - 槓を4回行う
/// 〈三槓子〉
/// - 槓を3回行う
fn check_sangangzi(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    match agari.tilesets.ankans.len() + agari.tilesets.minkans.len() {
        4 => Some(Form {
            name: "四槓子",
            point: Point::new_yakuman(),
        }),
        3 => Some(Form {
            name: "三槓子",
            point: Point::new(2),
        }),
        _ => None,
    }
}

/// [4]小三元
///
/// - 雀頭が三元牌になっている。
/// - 遺りの二つを刻子または槓子で揃える。
fn check_shousangen(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    // まず雀頭が三元牌でないならアウト。
    if !agari.janto().first().is_sangen() {
        return None;
    }

    let num_sangen = agari
        .kotus()
        .filter(|tiles| tiles.first().is_sangen())
        .count();

    if num_sangen >= 2 {
        Some(Form {
            name: "小三元",
            point: Point::new(4),
        })
    } else {
        None
    }
}

/// [13]大三元
///
/// - 三元牌全てについてそれぞれ刻子を作る。
fn check_daisangen(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    let num_sangen = agari
        .kotus()
        .filter(|tiles| tiles.first().is_sangen())
        .count();

    // 刻子が3つあれば自動的に全種類で刻子を作っていることになるのでOK。そもそも数がないため。
    if num_sangen == 3 {
        Some(Form {
            name: "大三元",
            point: Point::new_yakuman(),
        })
    } else {
        None
    }
}

/// [13]大四喜・[13]小四喜
///
/// 〈大四喜〉
/// - 面子が全て風牌
///
/// 〈小四喜〉
/// - 雀頭と3面子が風牌
fn check_shousushi(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    let extract_jihai_kind = |tiles: &Tiles| {
        if let Tile::Jihai(kind) = tiles.first() {
            Some(kind)
        } else {
            None
        }
    };

    let mut set: HashSet<Jihai> = agari.kotus().filter_map(extract_jihai_kind).collect();

    let check = |set: &HashSet<Jihai>, name: &'static str| {
        if set.contains(&Jihai::East)
            && set.contains(&Jihai::South)
            && set.contains(&Jihai::West)
            && set.contains(&Jihai::North)
        {
            Some(Form {
                name,
                point: Point::new_yakuman(),
            })
        } else {
            None
        }
    };

    // まずここで大四喜を確認
    check(&set, "大四喜")?;

    // 続いて雀頭を追加し、小四喜を確認
    set.insert(extract_jihai_kind(agari.janto())?);

    check(&set, "小四喜")
}

#[cfg(test)]
pub mod tests {}
