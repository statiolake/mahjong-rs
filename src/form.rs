//! 役を定義する。

use crate::agaritilesets::AgariTilesets;
use crate::config::Riichi;
use std::fmt;

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
            fan: 0,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// 立直
    Lizhi,

    /// 一発
    Ippatu,

    /// 門前清自摸和
    MenqianQingZimohu,

    /// 役牌
    Fanpai,

    /// 断么九
    DuanyaoJiu,

    /// 平和
    Pinghe,

    /// 一盃口
    Yibeikou,

    /// 海底摸月
    HaidiMoyue,

    /// 河底撈魚
    HediLaoyu,

    /// 嶺上開花
    LingshangKaihua,

    /// 槍槓
    ChengGang,

    /// ダブル立直
    DoubleLizhi,

    /// 三色同順
    SanshokuDojun,

    /// 三色同刻
    SanshokuDoko,

    /// 三暗刻
    Sananke,

    /// 一気通貫
    IkkiTukan,

    /// 七対子
    QiDuizi,

    /// 対々和
    Duiduihe,

    /// 混全帯幺九
    HunquanDaiYaojiu,

    /// 三槓子
    SanGangzi,

    /// 二盃口
    Liangbeigou,

    /// 純全帯公九
    ChunquanDaiyaojiu,

    /// 混一色
    Hunyise,

    /// 小三元
    Shousangen,

    /// 混老頭
    Hunlaotou,

    /// 清一色
    Qingyise,

    /// 四暗刻
    Sianke,

    /// 大三元
    Daisangen,

    /// 国士無双
    KokusiMusou,

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
    JiulianBaodeng,

    /// 地和
    Dihe,

    /// 天和
    Tianhe,
}

impl Form {
    fn name(self) -> &'static str {
        match *self {
            Form::Lizhi => "立直",
            Form::Ippatu => "一発",
            Form::MenqianQingZimohu => "門前清自摸和",
            Form::Fanpai => "役牌",
            Form::DuanyaoJiu => "断么九",
            Form::Pinghe => "平和",
            Form::Yibeikou => "一盃口",
            Form::HaidiMoyue => "海底摸月",
            Form::HediLaoyu => "河底撈魚",
            Form::LingshangKaihua => "嶺上開花",
            Form::ChengGang => "槍槓",
            Form::DoubleLizhi => "ダブル立直",
            Form::SanshokuDojun => "三色同順",
            Form::SanshokuDoko => "三色同刻",
            Form::Sananke => "三暗刻",
            Form::IkkiTukan => "一気通貫",
            Form::QiDuizi => "七対子",
            Form::Duiduihe => "対々和",
            Form::HunquanDaiYaojiu => "混全帯幺九",
            Form::SanGangzi => "三槓子",
            Form::Liangbeigou => "二盃口",
            Form::ChunquanDaiyaojiu => "純全帯公九",
            Form::Hunyise => "混一色",
            Form::Shousangen => "小三元",
            Form::Hunlaotou => "混老頭",
            Form::Qingyise => "清一色",
            Form::Sianke => "四暗刻",
            Form::Daisangen => "大三元",
            Form::KokusiMusou => "国士無双",
            Form::Luyise => "緑一色",
            Form::Ziyise => "字一色",
            Form::Qinglaotou => "清老頭",
            Form::Sigangzi => "四槓子",
            Form::Shousushi => "小四喜",
            Form::Daisushi => "大四喜",
            Form::JiulianBaodeng => "九蓮宝燈",
            Form::Dihe => "地和",
            Form::Tianhe => "天和",
        }
    }

    fn point(self, is_menzen: bool) -> Point {
        match *self {
            Form::Lizhi => Point::new(1),
            Form::Ippatu => Point::new(1),
            Form::MenqianQingZimohu => unimplemented!(),
            Form::Fanpai => unimplemented!(),
            Form::DuanyaoJiu => unimplemented!(),
            Form::Pinghe => unimplemented!(),
            Form::Yibeikou => unimplemented!(),
            Form::HaidiMoyue => unimplemented!(),
            Form::HediLaoyu => unimplemented!(),
            Form::LingshangKaihua => unimplemented!(),
            Form::ChengGang => unimplemented!(),
            Form::DoubleLizhi => Point::new(2),
            Form::SanshokuDojun => unimplemented!(),
            Form::SanshokuDoko => unimplemented!(),
            Form::Sananke => unimplemented!(),
            Form::IkkiTukan => unimplemented!(),
            Form::QiDuizi => unimplemented!(),
            Form::Duiduihe => unimplemented!(),
            Form::HunquanDaiYaojiu => unimplemented!(),
            Form::SanGangzi => unimplemented!(),
            Form::Liangbeigou => unimplemented!(),
            Form::ChunquanDaiyaojiu => unimplemented!(),
            Form::Hunyise => unimplemented!(),
            Form::Shousangen => unimplemented!(),
            Form::Hunlaotou => unimplemented!(),
            Form::Qingyise => unimplemented!(),
            Form::Sianke => unimplemented!(),
            Form::Daisangen => unimplemented!(),
            Form::KokusiMusou => unimplemented!(),
            Form::Luyise => unimplemented!(),
            Form::Ziyise => unimplemented!(),
            Form::Qinglaotou => unimplemented!(),
            Form::Sigangzi => unimplemented!(),
            Form::Shousushi => unimplemented!(),
            Form::Daisushi => unimplemented!(),
            Form::JiulianBaodeng => unimplemented!(),
            Form::Dihe => unimplemented!(),
            Form::Tianhe => unimplemented!(),
        }
    }

    fn display(self, is_menzen: bool) -> FormDisplay {
        FormDisplay {
            name: self.name(),
            point: self.point(is_menzen),
        }
    }
}

/// 役の表示。
#[derive(Debug, Clone, Copy)]
pub struct FormDisplay {
    /// 役の名前。
    name: &'static str,

    /// 翻数。
    point: Point,
}

impl fmt::Display for FormDisplay {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "{}　{}", self.point, self.name)
    }
}

/// 立直
fn check_lizhi(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    // 立直類は最初から指定されており、 Context として渡されている。
    match agari.tilesets.context.riichi {
        Riichi::None => SmallVec::new(),
        Riichi::Riichi => SmallVec::from_elem(Form::Lizhi, 1),
        Riichi::RiichiIppatu => {
            let mut v = SmallVec::new();
            v.push(Form::Lizhi);
            v.push(Form::Ippatu);
            v
        }
        Riichi::DoubleRiichi => SmallVec::from_elem(Form::DoubleLizhi, 1),
        Riichi::DoubleRiichiIppatu => {
            let mut v = SmallVec::new();
            v.push(Form::DoubleLizhi);
            v.push(Form::Ippatu);
            v
        }
    }
}

/// 門前清自摸和
fn check_menqian_qing_zimohu(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 役牌
fn check_fanpai(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 断么九
fn check_duanyao_jiu(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 平和
fn check_pinghe(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 一盃口
fn check_yibeikou(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 海底摸月
fn check_haidi_moyue(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 河底撈魚
fn check_hedi_laoyu(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 嶺上開花
fn check_lingshang_kaihua(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 槍槓
fn check_cheng_gang(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// ダブル立直
fn check_double_lizhi(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 三色同順
fn check_sanshoku_dojun(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 三色同刻
fn check_sanshoku_doko(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 三暗刻
fn check_sananke(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 一気通貫
fn check_ikki_tukan(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 七対子
fn check_qi_duizi(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 対々和
fn check_duiduihe(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 混全帯幺九
fn check_hunquan_dai_yaojiu(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 三槓子
fn check_san_gangzi(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 二盃口
fn check_liangbeigou(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 純全帯公九
fn check_chunquan_daiyaojiu(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 混一色
fn check_hunyise(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 小三元
fn check_shousangen(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 混老頭
fn check_hunlaotou(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 清一色
fn check_qingyise(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 四暗刻
fn check_sianke(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 大三元
fn check_daisangen(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 国士無双
fn check_kokusi_musou(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 緑一色
fn check_luyise(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 字一色
fn check_ziyise(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 清老頭
fn check_qinglaotou(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 四槓子
fn check_sigangzi(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 小四喜
fn check_shousushi(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 大四喜
fn check_daisushi(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 九蓮宝燈
fn check_jiulian_baodeng(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 地和
fn check_dihe(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}

/// 天和
fn check_tianhe(agari: &AgariTilesets) -> impl IntoIterator<Item = Form> {
    unimplemented!()
}
