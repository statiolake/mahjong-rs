//! 役を定義する。

use crate::agaritilesets::{AgariTilesets, MachiKind};
use crate::context::Lizhi;
use crate::tile::{Order, Tile, TileKind, Zipai};
use crate::tiles::Tiles;
use crate::tilesets::Tilesets;
use log::debug;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter::once;
use std::iter::Sum;

type SmallVec = smallvec::SmallVec<[Form; 4]>;

/// 翻数・符数。
#[derive(Debug, Clone, Copy)]
pub struct Point {
    /// 翻数。
    pub fan: u32,

    /// 負数。
    pub fu: u32,

    /// 何倍役満か。
    pub yiman: u32,
}

impl Point {
    pub fn new(fan: u32) -> Point {
        Point {
            fan,
            fu: 0,
            yiman: 0,
        }
    }

    pub fn with_fu(fan: u32, fu: u32) -> Point {
        Point { fan, fu, yiman: 0 }
    }

    pub fn new_manguan() -> Point {
        Point::new(5)
    }

    pub fn new_yiman() -> Point {
        Point {
            fan: 13,
            fu: 0,
            yiman: 1,
        }
    }

    pub fn is_yiman(self) -> bool {
        self.fan >= 13
    }

    pub fn is_true_yiman(self) -> bool {
        self.yiman > 0
    }

    pub fn value(self, is_parent: bool) -> u32 {
        debug!("-> 点数計算を行います。");

        let calc_few = |fan: u32, fu: u32| {
            debug!("--> 少翻の点数計算を行います。");

            // 符の倍率
            let mul = if is_parent { 6 } else { 4 };
            debug!(
                "    符の基本の倍率は{}{}倍です。",
                if is_parent { "親なので" } else { "" },
                mul
            );

            debug!("    比較のため満貫の点数を計算します...");
            let manguan = Point::new_manguan().value(is_parent);
            debug!("    満貫の点数の計算は終わりました。");

            // 最後の +2 は場ゾロあるいはバンバンと呼ばれる。
            let raw = fu * mul * 2u32.pow(fan + 2);
            debug!("    補正なしの点数は{}点です。", raw);

            if raw > manguan {
                // 満貫を越えていたら満貫に強制。
                debug!("    満貫の点数を越えているため、満貫に強制します。");
                manguan
            } else {
                // それ以外の場合は定義の計算式に従う。百の位以下を切り上げる。
                let ceiled = crate::utils::ceil_at(raw, 100);
                debug!("    切り上げて{}点です。", ceiled);
                ceiled
            }
        };

        let value = match self.yiman {
            0 => match (self.fan, is_parent) {
                (0..=4, is_parent) => {
                    let manguan = || Point::new_manguan().value(is_parent);
                    match (self.fan, self.fu) {
                        // 4翻30符と3翻60符は切り上げ満貫
                        (fan @ 4, fu @ 30) | (fan @ 3, fu @ 60) => {
                            debug!(
                                "    {}翻{}符のため、切り上げ満貫です。",
                                fan, fu
                            );
                            manguan()
                        }

                        // それ以外は通常の計算ルールに従う
                        (fan, fu) => calc_few(fan, fu),
                    }
                }

                (5, true) => 12000,
                (5, false) => 8000,

                (6..=7, true) => 18000,
                (6..=7, false) => 12000,

                (8..=10, true) => 24000,
                (8..=10, false) => 16000,

                (11..=12, true) => 36000,
                (11..=12, false) => 24000,

                (_, is_parent) => Point::new_yiman().value(is_parent),
            },

            n => n * if is_parent { 48000 } else { 32000 },
        };

        debug!("    結果は{}点です。", value);

        value
    }

    pub fn rank(self, is_parent: bool) -> Option<Cow<'static, str>> {
        let calc_few = || {
            let value = self.value(is_parent);
            let manguan = Point::new_manguan().value(is_parent);

            assert!(
                value <= manguan,
                "4翻以下で満貫を越えることはありません。"
            );

            if value == manguan {
                Point::new_manguan().rank(is_parent)
            } else {
                // 満貫もないときは特に何も表示しない
                None
            }
        };

        match self.yiman {
            0 => match self.fan {
                0..=4 => calc_few(),
                5 => Some("満貫".into()),
                6..=7 => Some("跳満".into()),
                8..=10 => Some("倍満".into()),
                11..=12 => Some("三倍満".into()),
                x if x >= 13 => Point::new_yiman().rank(is_parent),
                _ => unreachable!(),
            },
            1 => Some("役満".into()),
            2 => Some("ダブル役満".into()),
            3 => Some("トリプル役満".into()),
            4 => Some("四倍役満".into()),
            5 => Some("五倍役満".into()),
            n => Some(format!("{}倍役満", n).into()),
        }
    }

    pub fn display_full(self, is_parent: bool) -> PointDisplayFull {
        PointDisplayFull {
            point: self,
            is_parent,
        }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Point {}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Point) -> Ordering {
        self.fan
            .cmp(&other.fan)
            .then_with(|| self.fu.cmp(&other.fu))
    }
}

impl fmt::Display for Point {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        let fan = self.fan;
        let fu = self.fu;

        if fu == 0 || fan > 4 {
            write!(b, "{}翻", fan)
        } else {
            write!(b, "{}翻{}符", fan, fu)
        }
    }
}

impl Sum for Point {
    fn sum<I: Iterator<Item = Point>>(iter: I) -> Point {
        let mut fan = 0;
        let mut fu = 0;
        let mut yiman = 0;

        for point in iter {
            fan += point.fan;
            fu += point.fu;
            yiman += point.yiman;
        }

        Point { fan, fu, yiman }
    }
}

pub struct PointDisplayFull {
    point: Point,
    is_parent: bool,
}

impl fmt::Display for PointDisplayFull {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        let &PointDisplayFull { point, is_parent } = self;

        if !point.is_yiman() {
            write!(b, "{} ", point)?;
        }

        write!(b, "{}点", point.value(is_parent))?;

        if let Some(rank) = point.rank(is_parent) {
            write!(b, " {}", rank)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// 立直
    Lizhi,

    /// 一発
    Ippatsu,

    /// 門前清自摸
    Menqianqingzimohu,

    /// 役牌
    ///
    /// `u32`: 役牌の個数。
    Fanpai(u32),

    /// 断么九
    Duanyaojiu,

    /// 平和
    Pinghe,

    /// 一盃口
    Yibeikou,

    /// 海底撈月
    Haidimoyue,

    /// 河底撈魚
    Hedilaoyu,

    /// 嶺上開花
    Lingshangkaihua,

    /// 槍槓
    Chenggang,

    /// ダブル立直
    Doublelizhi,

    /// 三色同順
    ///
    /// `bool`: 門前かどうか
    Sanshokudojun(bool),

    /// 三色同刻
    Sanshokudoko,

    /// 三暗刻
    Sananke,

    /// 一気通貫
    ///
    /// `bool`: 門前かどうか
    Ikkitsukan(bool),

    /// 七対子
    Qiduizi,

    /// 対々和
    Duiduihe,

    /// 混全帯幺九
    ///
    /// `bool`: 門前かどうか
    Hunquandaiyaojiu(bool),

    /// 三槓子
    Sangangzi,

    /// 二盃口
    Liangbeigou,

    /// 純全帯公九
    ///
    /// `bool`: 門前かどうか
    Chunquandaiyaojiu(bool),

    /// 混一色
    ///
    /// `bool`: 門前かどうか
    Hungyise(bool),

    /// 小三元
    Shousangen,

    /// 混老頭
    Hunlaotou,

    /// 清一色
    ///
    /// `bool`: 門前かどうか
    Qingyise(bool),

    /// 四暗刻
    ///
    /// `bool`: 単騎待ちかどうか
    Sianke(bool),

    /// 大三元
    Daisangen,

    /// 国士無双
    ///
    /// `bool`: 13面待ちかどうか
    Kokushimuso(bool),

    /// 緑一色
    Luyise,

    /// 字一色
    Ziyise,

    /// 清老頭
    Qinglaotou,

    /// 四槓子
    Sigangzi,

    /// 小四喜
    Shousushi,

    /// 大四喜
    Daisushi,

    /// 九蓮宝燈
    ///
    /// `bool`: 純正かどうか
    Jiulianbaodeng(bool),

    /// 地和
    Dihe,

    /// 天和
    Tianhe,

    /// ドラ
    ///
    /// `u32`: ドラの枚数
    Dora(u32),
}

impl Form {
    pub fn name(self) -> &'static str {
        match self {
            Form::Lizhi => "立直",
            Form::Ippatsu => "一発",
            Form::Menqianqingzimohu => "門前清自摸和",
            Form::Fanpai(_) => "役牌",
            Form::Duanyaojiu => "断么九",
            Form::Pinghe => "平和",
            Form::Yibeikou => "一盃口",
            Form::Haidimoyue => "海底撈月",
            Form::Hedilaoyu => "河底撈魚",
            Form::Lingshangkaihua => "嶺上開花",
            Form::Chenggang => "槍槓",
            Form::Doublelizhi => "ダブル立直",
            Form::Sanshokudojun(_) => "三色同順",
            Form::Sanshokudoko => "三色同刻",
            Form::Sananke => "三暗刻",
            Form::Ikkitsukan(_) => "一気通貫",
            Form::Qiduizi => "七対子",
            Form::Duiduihe => "対々和",
            Form::Hunquandaiyaojiu(_) => "混全帯幺九",
            Form::Sangangzi => "三槓子",
            Form::Liangbeigou => "二盃口",
            Form::Chunquandaiyaojiu(_) => "純全帯公九",
            Form::Hungyise(_) => "混一色",
            Form::Shousangen => "小三元",
            Form::Hunlaotou => "混老頭",
            Form::Qingyise(_) => "清一色",
            Form::Sianke(is_danqi) => {
                if is_danqi {
                    "四暗刻単騎"
                } else {
                    "四暗刻"
                }
            }
            Form::Daisangen => "大三元",
            Form::Kokushimuso(is_genuine) => {
                if is_genuine {
                    "国士無双13面待ち"
                } else {
                    "国士無双"
                }
            }
            Form::Luyise => "緑一色",
            Form::Ziyise => "字一色",
            Form::Qinglaotou => "清老頭",
            Form::Sigangzi => "四槓子",
            Form::Shousushi => "小四喜",
            Form::Daisushi => "大四喜",
            Form::Jiulianbaodeng(is_genuine) => {
                if is_genuine {
                    "純正九蓮宝燈"
                } else {
                    "九蓮宝燈"
                }
            }
            Form::Dihe => "地和",
            Form::Tianhe => "天和",
            Form::Dora(_) => "ドラ",
        }
    }

    pub fn point(self) -> Point {
        match self {
            Form::Lizhi => Point::new(1),
            Form::Ippatsu => Point::new(1),
            Form::Menqianqingzimohu => Point::new(1),
            Form::Fanpai(n) => Point::new(n),
            Form::Duanyaojiu => Point::new(1),
            Form::Pinghe => Point::new(1),
            Form::Yibeikou => Point::new(1),
            Form::Haidimoyue => Point::new(1),
            Form::Hedilaoyu => Point::new(1),
            Form::Lingshangkaihua => Point::new(1),
            Form::Chenggang => Point::new(1),
            Form::Doublelizhi => Point::new(2),
            Form::Sanshokudojun(is_menqian) => Point::new(if is_menqian { 2 } else { 1 }),
            Form::Sanshokudoko => Point::new(2),
            Form::Sananke => Point::new(2),
            Form::Ikkitsukan(is_menqian) => Point::new(if is_menqian { 2 } else { 1 }),
            Form::Qiduizi => Point::with_fu(2, 25),
            Form::Duiduihe => Point::new(2),
            Form::Hunquandaiyaojiu(is_menqian) => Point::new(if is_menqian { 2 } else { 1 }),
            Form::Sangangzi => Point::new(2),
            Form::Liangbeigou => Point::new(3),
            Form::Chunquandaiyaojiu(is_menqian) => Point::new(if is_menqian { 2 } else { 1 }),
            Form::Hungyise(is_menqian) => Point::new(if is_menqian { 3 } else { 2 }),
            Form::Shousangen => Point::new(4),
            Form::Hunlaotou => Point::new(2),
            Form::Qingyise(is_menqian) => Point::new(if is_menqian { 6 } else { 5 }),
            Form::Sianke(_) => Point::new_yiman(),
            Form::Daisangen => Point::new_yiman(),
            Form::Kokushimuso(_) => Point::new_yiman(),
            Form::Luyise => Point::new_yiman(),
            Form::Ziyise => Point::new_yiman(),
            Form::Qinglaotou => Point::new_yiman(),
            Form::Sigangzi => Point::new_yiman(),
            Form::Shousushi => Point::new_yiman(),
            Form::Daisushi => Point::new_yiman(),
            Form::Jiulianbaodeng(_) => Point::new_yiman(),
            Form::Dihe => Point::new_yiman(),
            Form::Tianhe => Point::new_yiman(),
            Form::Dora(n) => Point::new(n),
        }
    }

    pub fn display(self) -> FormDisplay {
        let name = self.name();
        let point = self.point();
        FormDisplay { name, point }
    }
}

/// 役。
#[derive(Debug, Clone, Copy)]
pub struct FormDisplay {
    /// 名前
    name: &'static str,

    /// 翻数
    point: Point,
}

impl fmt::Display for FormDisplay {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{} {}", self.point, self.name)
    }
}

/// [1]立直・[2]ダブルリーチ・[1]一発
pub fn special_check_lizhi(tilesets: &Tilesets) -> SmallVec {
    debug!("--> 立直類を判定...");
    // 立直類は最初から指定されており、 Context として渡されている。
    match tilesets.context.lizhi {
        Lizhi::None => {
            debug!("   立直ではありません。");
            SmallVec::new()
        }
        Lizhi::Lizhi => {
            debug!("    立直です。");
            SmallVec::from_elem(Form::Lizhi, 1)
        }
        Lizhi::LizhiIppatsu => {
            debug!("    立直・一発です。");
            let mut v = SmallVec::new();
            v.push(Form::Lizhi);
            v.push(Form::Ippatsu);
            v
        }
        Lizhi::DoubleLizhi => {
            debug!("    ダブル立直です。");
            SmallVec::from_elem(Form::Doublelizhi, 1)
        }
        Lizhi::DoubleLizhiIppatsu => {
            debug!("    ダブル立直・一発です。");
            let mut v = SmallVec::new();
            v.push(Form::Doublelizhi);
            v.push(Form::Ippatsu);
            v
        }
    }
}

pub fn special_check_dora(tilesets: &Tilesets) -> Option<Form> {
    debug!("--> ドラを判定...");
    let count_dora = |tile: Tile| {
        let num_dora = tilesets.doras.iter().filter(|&&dora| tile == dora).count();
        let red_dora = if tile.is_red() { 1 } else { 0 };
        num_dora + red_dora
    };

    let num_dora: usize = tilesets.tiles_without_doras().map(count_dora).sum();

    debug!("    ドラは {} 枚です。", num_dora);
    if num_dora > 0 {
        Some(Form::Dora(num_dora as _))
    } else {
        None
    }
}

/// [2.5]七対子
pub fn special_check_qiduizi(tilesets: &Tilesets) -> Option<Form> {
    debug!("--> 七対子を判定...");
    // ポン・チー・カンをしていたら七対子にならないので終了。
    if tilesets.did_fulou() || !tilesets.angangs.is_empty() {
        debug!("    副露または暗槓があります。");
        return None;
    }

    let tiles: Tiles = (tilesets.hand.iter().copied())
        .chain(once(tilesets.last))
        .collect();

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
        debug!("    二枚組でない牌があります。");
        return None;
    }

    // そうであれば七対子
    debug!("    七対子です。");
    Some(Form::Qiduizi)
}

/// [1]門前清自摸和
///
/// - 門前でツモ上がりをした。
pub fn special_check_menqianqingzimohu(tilesets: &Tilesets) -> Option<Form> {
    debug!("--> 門前清自摸和を判定...");
    if !tilesets.is_zimo {
        debug!("    ツモではありません。");
        return None;
    }

    if !tilesets.is_menqian() {
        debug!("    門前ではありません。");
        return None;
    }

    debug!("    門前清自摸和です。");
    Some(Form::Menqianqingzimohu)
}

/// [1]断么九
///
/// - 手牌が全て中張牌である。
pub fn special_check_duanyaojiu(tilesets: &Tilesets) -> Option<Form> {
    debug!("--> 断么九を判定...");
    let has_yaojiu = tilesets.tiles_without_doras().all(|t| t.is_zhongzhang());

    if !has_yaojiu {
        debug!("    幺九牌があります。");
        return None;
    }

    debug!("    断么九です。");
    Some(Form::Duanyaojiu)
}

/// [6/5]清一色・[3/2]混一色
///
/// 〈清一色〉
/// - どれか一種類の牌だけで構成する。
///
/// 〈混一色〉
/// - どれか一種類の牌と字牌だけで構成する。
pub fn special_check_hungyise_qingyise(tilesets: &Tilesets) -> Option<Form> {
    debug!("--> 清一色・混一色を判定...");

    // 各面子・雀頭の種類
    let kinds = || tilesets.tiles_without_doras().map(|tile| tile.kind());

    // 字牌があるかどうか
    let has_zipai = kinds().any(|kind| kind == TileKind::Zipai);
    debug!(
        "    字牌はありま{}。",
        if has_zipai { "す" } else { "せん" }
    );

    // 字牌でない雀頭の種類
    let kinds_not_zipai = || kinds().filter(|&kind| kind != TileKind::Zipai);

    // 対象となる種類
    let target_kind = kinds_not_zipai().next()?;
    debug!(
        "    字牌でない牌の種類の一つは {} です。",
        target_kind
    );

    // 全てが同じかどうか
    let all_same = kinds_not_zipai().all(|kind| kind == target_kind);
    debug!(
        "    字牌以外の牌の種類は全て等し{}。",
        if all_same {
            "いです"
        } else {
            "くありません"
        }
    );

    match (all_same, has_zipai) {
        (true, false) => {
            debug!("    清一色です。");
            Some(Form::Qingyise(tilesets.is_menqian()))
        }
        (true, true) => {
            debug!("    混一色です。");
            Some(Form::Hungyise(tilesets.is_menqian()))
        }
        _ => {
            debug!("    清一色・混一色ではありません。");
            None
        }
    }
}

/// [2]混老頭
///
/// - 全ての面子が幺九牌で構成されている。
pub fn special_check_hunlaotou(tilesets: &Tilesets) -> Option<Form> {
    debug!("--> 混老頭を判定...");
    let has_zhongzhang = tilesets.tiles_without_doras().all(|tile| tile.is_yaojiu());

    if !has_zhongzhang {
        debug!("    中張牌があります。");
        return None;
    }

    debug!("    混老頭です。");
    Some(Form::Hunlaotou)
}

/// 特別な形のある役 (国士無双、九蓮宝燈など)
///
/// `target` はベースとなる形 (国士無双なら 1s9s1m9m1p9p東南西北白發中 など) で、これプラスその形の
/// どれか一つの牌だけがダブっている状態がアガリとなる。 `name_genuine` は純正の場合、つまり最初から
/// `target` がそろっていて最後に引いた牌がダブっている場合につく。たとえば国士無双13面待ちなど。
pub fn special_check_certadebugrm(
    tilesets: &Tilesets,
    mut target: Tiles,
    form_constructor: fn(bool) -> Form,
) -> Option<Form> {
    // ポン・チー・カンをしていたらならないので終了。
    if tilesets.did_fulou() || !tilesets.angangs.is_empty() {
        debug!("    副露があるため所定の形にできません。");
        return None;
    }

    // 手札に最後に引いてきた牌を追加する。
    let hand: Tiles = (tilesets.hand.iter().copied())
        .chain(once(tilesets.last))
        .collect();

    // これらの牌の一つを選んで、それを追加したものと手牌が一致するかどうかを確かめる。
    for add in target.clone().into_inner().into_iter() {
        target.push(add);

        // 一致した場合は国士無双などが成立。
        if target == hand {
            let form = form_constructor(add == tilesets.last);
            debug!(
                "    形が一致したので {} 成立です。",
                form.name()
            );
            return Some(form);
        }

        target.remove(
            target
                .iter()
                .position(|&tile| tile == add)
                .expect("追加した牌を見つけられませんでした。"),
        );
    }

    debug!("    形が一致しませんでした。");
    None
}

/// [13]国士無双
pub fn special_check_kokushimuso(tilesets: &Tilesets) -> Option<Form> {
    debug!("--> 国士無双を判定...");
    special_check_certadebugrm(
        tilesets,
        Tiles::new(vec![
            Tile::Suozi(Order::new(1).unwrap()),
            Tile::Suozi(Order::new(9).unwrap()),
            Tile::Wanzi(Order::new(1).unwrap()),
            Tile::Wanzi(Order::new(9).unwrap()),
            Tile::Tongzi(Order::new(1).unwrap()),
            Tile::Tongzi(Order::new(9).unwrap()),
            Tile::Zipai(Zipai::East),
            Tile::Zipai(Zipai::South),
            Tile::Zipai(Zipai::West),
            Tile::Zipai(Zipai::North),
            Tile::Zipai(Zipai::Bai),
            Tile::Zipai(Zipai::Fa),
            Tile::Zipai(Zipai::Zhong),
        ]),
        Form::Kokushimuso,
    )
}

/// [13]九蓮宝燈
pub fn special_check_jiulianbaodeng(tilesets: &Tilesets) -> Option<Form> {
    debug!("--> 九蓮宝燈を判定...");
    let constructors: Vec<fn(Order) -> Tile> = vec![Tile::Suozi, Tile::Wanzi, Tile::Tongzi];

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
        .filter_map(move |ctor| {
            special_check_certadebugrm(
                tilesets,
                orders.iter().map(|&o| ctor(o)).collect(),
                Form::Jiulianbaodeng,
            )
        })
        .next()
}

/// [13]緑一色
///
/// - 全ての牌が緑一色を構成する牌である。
pub fn special_check_luyise(tilesets: &Tilesets) -> Option<Form> {
    debug!("--> 緑一色を判定...");
    let all_green = tilesets.tiles_without_doras().all(|tile| tile.is_green());

    if !all_green {
        debug!("    緑色でない牌が混ざっています。");
        return None;
    }

    debug!("    緑一色です。");
    Some(Form::Luyise)
}

/// [13]字一色
///
/// - 全ての牌が字牌である。
pub fn special_check_ziyise(tilesets: &Tilesets) -> Option<Form> {
    debug!("--> 字一色を判定...");
    let all_zipai = tilesets
        .tiles_without_doras()
        .all(|tile| tile.kind() == TileKind::Zipai);

    if !all_zipai {
        debug!("    字牌以外の牌が混ざっています。");
        return None;
    }

    debug!("    字一色です。");
    Some(Form::Ziyise)
}

/// [13]清老頭
///
/// - 全ての牌が 1,9 牌のみである。
pub fn special_check_qinglaotou(tilesets: &Tilesets) -> Option<Form> {
    debug!("--> 清老頭を判定...");

    let all_19 = tilesets
        .tiles_without_doras()
        .all(|tile| tile.kind() != TileKind::Zipai && tile.is_yaojiu());

    if !all_19 {
        debug!("    1, 9 以外の牌が混ざっています。");
        return None;
    }

    debug!("    清老頭です。");
    Some(Form::Qinglaotou)
}

/// [n]役牌
///
/// - 刻子・槓子が役牌である。一つにつき1翻。
pub fn check_fanpai(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 役牌を判定...");

    let sum = agari
        .kezis()
        .map(|tile| {
            let num = tile.first().num_fan(agari.context());
            debug!("    {}の役は{}翻です。", tile, num);
            num
        })
        .sum();

    debug!("    役の合計は{}翻です。", sum);
    if sum != 0 {
        Some(Form::Fanpai(sum))
    } else {
        None
    }
}

/// [1/0]平和
///
/// - 4面子が順子である。
/// - 雀頭が役牌でない。
/// - 両面待ちである。
pub fn check_pinghe(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 平和を判定...");

    if !agari.is_menqian() {
        debug!("    門前ではありません。");
        return None;
    }

    if agari.shunzis().count() != 4 {
        debug!("    順子以外の面子があります。");
        return None;
    }

    if agari.quetou().first().kind() == TileKind::Zipai {
        debug!("    雀頭が字牌です。");
        return None;
    }

    if agari.machi() != MachiKind::Liangmian {
        debug!("    両面待ちではありません。");
        return None;
    }

    debug!("    平和です。");
    Some(Form::Pinghe)
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
pub fn check_yibeikou_liangbeigou(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 一盃口・二盃口を判定...");
    if !agari.is_menqian() {
        debug!("    門前ではありません。");
        return None;
    }

    let mut map = HashMap::new();
    for shunzi in agari.shunzis() {
        *map.entry(shunzi.first()).or_default() += 1;
    }

    let mut cnt = 0;
    for (_, num) in map {
        match num {
            // 同じ順子が 2 組あるならそれで二盃口が構成される。
            4 => {
                debug!("    同じ順子が2組ありました。");
                cnt += 2;
            }
            // 同じ順子が 2 または 3 あるならそれで一盃口が構成される。
            // 今後他の牌についてまた一盃口が構成されれば二盃口となる。
            2 | 3 => {
                debug!("    同じ順子が1組ありました。");
                cnt += 1;
            }
            _ => {}
        }
    }

    match cnt {
        0 => {
            debug!("    一盃口・二盃口ではありません。");
            None
        }
        1 => {
            debug!("    一盃口です。");
            Some(Form::Yibeikou)
        }
        2 => {
            debug!("    二盃口です。");
            Some(Form::Liangbeigou)
        }
        _ => panic!("二盃口以上があります。"),
    }
}

/// [2/1]三色同順
///
/// - 索子・萬子・筒子で同じ数字から始まる順子を作る。
pub fn check_sanshoku_dojun(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 三色同順を判定...");

    // 「その順序から始まる順子にはどの種類の牌があるか」を集める
    let mut map: HashMap<Option<Order>, HashSet<TileKind>> = HashMap::new();
    for tile in agari.shunzis().map(|t| t.first()) {
        map.entry(tile.order()).or_default().insert(tile.kind());
    }

    // そのなかのある一つの順序について、索子も萬子も筒子もあるということなら三色同順
    let does_match = map.into_iter().any(|(_, kinds)| {
        kinds.contains(&TileKind::Suozi)
            && kinds.contains(&TileKind::Wanzi)
            && kinds.contains(&TileKind::Tongzi)
    });

    if !does_match {
        debug!("    全ての種類が揃っている順子はありませんでした。");
        return None;
    }

    debug!("    三色同順です。");
    // 喰い下がりがあるので注意。
    Some(Form::Sanshokudojun(agari.is_menqian()))
}

/// [2]三色同刻
///
/// - 索子・萬子・筒子で同じ数字からなる刻子を作る。
pub fn check_sanshoku_doko(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 三色同刻を判定...");

    // 「その順序から始まる刻子にはどの種類の牌があるか」を集める
    let mut map: HashMap<Option<Order>, HashSet<TileKind>> = HashMap::new();
    for tile in agari.kezis().map(|t| t.first()) {
        map.entry(tile.order()).or_default().insert(tile.kind());
    }

    // そのなかのある一つの順序について、索子も萬子も筒子もあるということなら三色同刻
    let does_match = map.into_iter().any(|(_, kinds)| {
        kinds.contains(&TileKind::Suozi)
            && kinds.contains(&TileKind::Wanzi)
            && kinds.contains(&TileKind::Tongzi)
    });

    if !does_match {
        debug!("    全ての種類が揃っている刻子はありませんでした。");
        return None;
    }

    debug!("    三色同刻です。");
    Some(Form::Sanshokudoko)
}

/// [13]四暗刻・[2]三暗刻
///
/// 〈四暗刻〉
/// - 暗刻が4つある
///
/// 〈三暗刻〉
/// - 暗刻が3つある
pub fn check_sananke_sianke(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 四暗刻・三暗刻を判定...");

    let count = agari.ankes().count();

    if count == 4 {
        debug!("    四暗刻です。");
        Some(Form::Sianke(agari.machi() == MachiKind::Danqi))
    } else if count == 3 {
        debug!("    三暗刻です。");
        Some(Form::Sananke)
    } else {
        debug!("    暗刻は{}つしかありません。", count);
        None
    }
}

/// [2/1]一気通貫
///
/// - どれか一種類の牌で 123 456 789 を達成する
pub fn check_ikki_tukan(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 一気通貫を判定...");

    let mut map: HashMap<TileKind, HashSet<Option<Order>>> = HashMap::new();
    for tile in agari.shunzis().map(|t| t.first()) {
        map.entry(tile.kind()).or_default().insert(tile.order());
    }

    let does_match = map.into_iter().any(|(_, orders)| {
        orders.contains(&Some(Order::new(1).unwrap()))
            && orders.contains(&Some(Order::new(4).unwrap()))
            && orders.contains(&Some(Order::new(7).unwrap()))
    });

    if !does_match {
        debug!("    123 456 789 を達成している牌はありませんでした。");
        return None;
    }

    debug!("    一気通貫です。");
    Some(Form::Ikkitsukan(agari.is_menqian()))
}

/// [2]対々和
///
/// - 刻子が4つある。
pub fn check_duiduihe(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 対々和を判定...");

    let count = agari.kezis().count();

    if count != 4 {
        debug!("    刻子が{}個しかありません。", count);
        return None;
    }

    debug!("    対々和です。");
    Some(Form::Duiduihe)
}

/// [2/1]混全帯幺九・[3/2]純全帯公九
///
/// 〈混全帯幺九〉
/// - 全ての面子と雀頭に幺九牌が絡んでいる。
/// 〈純全帯公九〉
/// - 全ての面子と雀頭に 1, 9 が絡んでいる。
pub fn check_hunquandaiyaojiu_chunquandaiyaojiu(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 混全帯幺九・純全帯公九を判定...");

    let mut has_zipai = false;
    let mut has_zhongzhang = false;

    for tiles in agari.mianzis().chain(once(agari.quetou())) {
        // その面子の牌の全てが中張牌であれば対象の役のどれも成立しえないので放置。
        if tiles.iter().all(|&tile| tile.is_zhongzhang()) {
            debug!(
                "    全てが中張牌で構成された面子{}があったため、いずれも成立しません。",
                tiles
            );
            return None;
        }

        has_zipai = has_zipai || tiles.iter().any(|&tile| tile.kind() == TileKind::Zipai);
        has_zhongzhang = has_zhongzhang || tiles.iter().any(|tile| tile.is_zhongzhang());
    }

    debug!(
        "    字牌があり{}。中張牌はあり{}。",
        if has_zipai { "ます" } else { "ません" },
        if has_zhongzhang {
            "ます"
        } else {
            "ません"
        }
    );

    match (has_zipai, has_zhongzhang) {
        (false, true) => {
            debug!("    純全帯公九です。");
            Some(Form::Chunquandaiyaojiu(agari.is_menqian()))
        }
        (true, true) => {
            debug!("    混全帯幺九です。");
            Some(Form::Hunquandaiyaojiu(agari.is_menqian()))
        }
        // 混老頭は別扱いのため、ここでは None
        _ => {
            debug!("    純全帯公九でも混全帯幺九でもありません。");
            None
        }
    }
}

/// [13]四槓子・[2]三槓子
///
/// 〈四槓子〉
/// - 槓を4回行う
/// 〈三槓子〉
/// - 槓を3回行う
pub fn check_sangangzi_sigangzi(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 四槓子・三槓子を判定...");
    match agari.angangs().count() + agari.minggangs().count() {
        4 => {
            debug!("    四槓子です。");
            Some(Form::Sigangzi)
        }
        3 => {
            debug!("    三槓子です。");
            Some(Form::Sangangzi)
        }
        n => {
            debug!("    槓は{}回しか行われていません。", n);
            None
        }
    }
}

/// [4]小三元
///
/// - 雀頭が三元牌になっている。
/// - 遺りの二つを刻子または槓子で揃える。
pub fn check_shousanyuan(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 小三元を判定...");

    // まず雀頭が三元牌でないならアウト。
    if !agari.quetou().first().is_sanyuan() {
        debug!("    雀頭が三元牌ではありません。");
        return None;
    }

    let num_sanyuan = agari
        .kezis()
        .filter(|tiles| tiles.first().is_sanyuan())
        .count();

    if num_sanyuan >= 2 {
        debug!(
            "    三元牌が雀頭を除いて {} 枚あるので小三元です。",
            num_sanyuan
        );
        Some(Form::Shousangen)
    } else {
        debug!(
            "    三元牌は雀頭を除いて{}枚しかありません。",
            num_sanyuan
        );
        None
    }
}

/// [13]大三元
///
/// - 三元牌全てについてそれぞれ刻子を作る。
pub fn check_daisanyuan(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 大三元を判定...");
    let num_sanyuan = agari
        .kezis()
        .filter(|tiles| tiles.first().is_sanyuan())
        .count();

    // 刻子が3つあれば自動的に全種類で刻子を作っていることになるのでOK。そもそも数がないため。
    if num_sanyuan == 3 {
        debug!(
            "    三元牌が{}枚あるので大三元です。",
            num_sanyuan
        );
        Some(Form::Daisangen)
    } else {
        debug!(
            "    三元牌が雀頭を除いて{}枚しかありません。",
            num_sanyuan
        );
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
pub fn check_shousushi_daisushi(agari: &AgariTilesets) -> Option<Form> {
    debug!("--> 大四喜・小四喜を判定...");

    let extract_zipai_kind = |tiles: &Tiles| match tiles.first() {
        Tile::Zipai(kind) => Some(kind),
        _ => None,
    };

    let check = |set: &HashSet<Zipai>, form: Form| {
        let ok = [Zipai::East, Zipai::South, Zipai::West, Zipai::North]
            .iter()
            .all(|d| set.contains(d));
        if ok {
            debug!(
                "   全方位を含んでいるので{}成立です。",
                form.name()
            );

            Some(form)
        } else {
            debug!(
                "    方位が足りず{}は成立しませんでした。",
                form.name()
            );
            None
        }
    };

    let mut set: HashSet<Zipai> = agari.kezis().filter_map(extract_zipai_kind).collect();
    // まずここで大四喜を確認。
    check(&set, Form::Daisushi).or_else(|| {
        // 続いて雀頭を追加し、小四喜を確認。
        set.insert(extract_zipai_kind(agari.quetou())?);
        check(&set, Form::Shousushi)
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn judge_order() {
        assert_eq!(
            (Form::Ikkitsukan(false).point()).cmp(&Form::Hungyise(false).point()),
            Ordering::Less
        );
    }
}
