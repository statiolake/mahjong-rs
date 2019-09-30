use crate::agaritilesets::AgariTilesets;
use crate::form::{Form, Point};
use crate::tilesets::Tilesets;
use log::debug;
use std::fmt;
use std::iter::once;

#[derive(Debug, Clone)]
enum JudgeTilesets {
    Tilesets(Tilesets),
    AgariTilesets(AgariTilesets),
}

#[derive(Debug, Clone)]
pub struct Judge {
    forms: Vec<Form>,
    total: Point,
    tilesets: JudgeTilesets,
}

impl Judge {
    fn new(forms: Vec<Form>, total: Point, tilesets: JudgeTilesets) -> Option<Judge> {
        if forms.is_empty() {
            None
        } else {
            Some(Judge {
                forms,
                total,
                tilesets,
            })
        }
    }

    fn fix_forms(forms: &mut Vec<Form>) {
        // 役を翻数の順に並べる。
        forms.sort_by_key(|f| f.point());

        // もし真の役満が含まれているなら、それ以外を除く。
        if forms.iter().any(|form| form.point().is_true_yiman()) {
            forms.retain(|form| form.point().is_true_yiman());
        }

        // もし役がドラのみであれば、それでは上がれないので役を空にする
        if forms.iter().all(|form| form.is_dora()) {
            forms.clear();
        }
    }

    fn from_tilesets(tilesets: Tilesets, mut forms: Vec<Form>) -> Option<Judge> {
        // 役を補正する。
        Judge::fix_forms(&mut forms);

        // 合計の翻数を計算する。
        let total: Point = forms.iter().map(|f| f.point()).sum();

        Judge::new(forms, total, JudgeTilesets::Tilesets(tilesets))
    }

    fn from_agaritilesets(agari: AgariTilesets, mut forms: Vec<Form>) -> Option<Judge> {
        // 役を補正する。
        Judge::fix_forms(&mut forms);

        // 合計の翻数を計算する。
        let mut total: Point = forms.iter().map(|f| f.point()).sum();

        // 符計算をする。
        assert_eq!(total.fu, 0);
        total.fu = FuCalculator::new(&agari, &forms).calculate();

        Judge::new(forms, total, JudgeTilesets::AgariTilesets(agari))
    }

    fn tilesets(&self) -> &Tilesets {
        match &self.tilesets {
            JudgeTilesets::Tilesets(tilesets) => tilesets,
            JudgeTilesets::AgariTilesets(agari) => agari.tilesets(),
        }
    }
}

impl fmt::Display for Judge {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            b,
            "{}場 {}家 {}",
            self.tilesets().context.place,
            self.tilesets().context.player,
            self.tilesets().context.player_name
        )?;

        writeln!(b, "{}", self.tilesets())?;

        if let JudgeTilesets::AgariTilesets(ref agari) = self.tilesets {
            writeln!(b, "({})", agari)?;
        }

        for form in &self.forms {
            writeln!(b, "{}", form.display())?;
        }

        write!(
            b,
            "{}",
            self.total.display_full(self.tilesets().context.is_parent())
        )
    }
}

pub fn judge(tilesets: &Tilesets) -> Option<Judge> {
    debug!("判定を開始します。");
    debug!("対象: {}", tilesets);
    let res = judge_all(tilesets).max_by(|x, y| {
        // まずは得点で比較し、等しければ役の数が少ない方をとる。
        (x.total)
            .cmp(&y.total)
            .then((x.forms.len()).cmp(&y.forms.len()).reverse())
    });

    debug!(
        "判定が終わりました。結論は: {:?}",
        res.as_ref().map(|j| j.to_string())
    );
    res
}

fn judge_all(tilesets: &Tilesets) -> impl Iterator<Item = Judge> {
    let qiduizi = judge_qiduizi(&tilesets);
    let kokushimuso = judge_kokushimuso(&tilesets);
    let jiulianbaodeng = judge_jiulianbaodeng(&tilesets);
    let agari = (AgariTilesets::enumerate(tilesets).into_iter()).filter_map(judge_agari);

    agari
        .chain(qiduizi)
        .chain(kokushimuso)
        .chain(jiulianbaodeng)
}

fn forms_for_all_base(tilesets: &Tilesets) -> impl Iterator<Item = Form> {
    use crate::form::*;
    (tilesets.context.lucky_forms.clone().into_iter())
        .chain(special_check_lizhi(tilesets))
        .chain(special_check_menqianqingzimohu(tilesets))
        .chain(special_check_duanyaojiu(tilesets))
        .chain(special_check_ziyise(tilesets))
        .chain(special_check_luyise(tilesets))
        .chain(special_check_hungyise_qingyise(tilesets))
        .chain(special_check_qinglaotou(tilesets))
        .chain(special_check_hunlaotou(tilesets))
        .chain(special_check_dora(tilesets))
}

fn judge_agari(agari: AgariTilesets) -> Option<Judge> {
    debug!("-> 次のアガリ形について判定: {}", agari);
    use crate::form::*;

    let mut forms = Vec::with_capacity(10);
    forms.extend(forms_for_all_base(agari.tilesets()));
    forms.extend(check_fanpai(&agari));
    forms.extend(check_pinghe(&agari));
    forms.extend(check_yibeikou_liangbeigou(&agari));
    forms.extend(check_sanshoku_dojun(&agari));
    forms.extend(check_sanshoku_doko(&agari));
    forms.extend(check_sananke_sianke(&agari));
    forms.extend(check_ikki_tukan(&agari));
    forms.extend(check_duiduihe(&agari));
    forms.extend(check_hunquandaiyaojiu_chunquandaiyaojiu(&agari));
    forms.extend(check_sangangzi_sigangzi(&agari));
    forms.extend(check_shousanyuan(&agari));
    forms.extend(check_daisanyuan(&agari));
    forms.extend(check_shousushi_daisushi(&agari));

    Judge::from_agaritilesets(agari, forms)
}

fn judge_qiduizi(tilesets: &Tilesets) -> Option<Judge> {
    debug!("-> 七対子を判定...");
    crate::form::special_check_qiduizi(tilesets)
        .map(|q| forms_for_all_base(tilesets).chain(once(q)).collect())
        .and_then(|forms| Judge::from_tilesets(tilesets.clone(), forms))
}

fn judge_kokushimuso(tilesets: &Tilesets) -> Option<Judge> {
    debug!("-> 国士無双を判定...");
    crate::form::special_check_kokushimuso(tilesets)
        .map(|k| forms_for_all_base(tilesets).chain(once(k)).collect())
        .and_then(|forms| Judge::from_tilesets(tilesets.clone(), forms))
}

fn judge_jiulianbaodeng(tilesets: &Tilesets) -> Option<Judge> {
    debug!("-> 九蓮宝燈を判定...");
    crate::form::special_check_jiulianbaodeng(tilesets)
        .map(|j| forms_for_all_base(tilesets).chain(once(j)).collect())
        .and_then(|forms| Judge::from_tilesets(tilesets.clone(), forms))
}

struct FuCalculator<'a> {
    agari: &'a AgariTilesets,
    forms: &'a [Form],
}

impl<'a> FuCalculator<'a> {
    fn new(agari: &'a AgariTilesets, forms: &'a [Form]) -> FuCalculator<'a> {
        FuCalculator { agari, forms }
    }

    fn calculate(&self) -> u32 {
        debug!("符計算を行います。");
        // 平和ツモは一律 20 符
        if self.is_pinghe_zimo() {
            debug!("-> 平和ツモのため 20 符です。");
            return 20;
        }

        // アガリ基本符
        let fudi = 20;
        debug!("-> 副底は {} 符", fudi);

        // アガリ方による符
        let agari_fu = self.calc_agari_fu();
        debug!("-> アガリ方による符は {} 符", agari_fu);

        // 刻子、槓によるボーナス
        let kezi_fu = self.calc_kezi_fu();
        debug!("-> 刻子・槓によるボーナスが {} 符", kezi_fu);

        // 雀頭によるボーナス
        let quetou_fu = self.calc_quetou_fu();
        debug!("-> 雀頭によるボーナスが {} 符", quetou_fu);

        // 待ちによるボーナス
        let machi_fu = self.calc_machi_fu();
        debug!("-> 待ちによるボーナスが {} 符", machi_fu);

        let base = fudi + agari_fu + kezi_fu + quetou_fu + machi_fu;
        debug!("-> 従って、基本の合計が {} 符", base);

        let ceiled = crate::utils::ceil_at(base, 10);
        debug!("-> これを切り上げると {} 符", ceiled);

        // 喰い平和形では 20 符となるが、このときは 30 符に引き上げる
        if !self.agari.is_zimo() && ceiled == 20 {
            debug!("-> これは喰い平和形なので 30 符に切り上げます。");
            return 30;
        }

        ceiled
    }

    fn is_pinghe_zimo(&self) -> bool {
        self.agari.is_zimo() && self.forms.iter().any(|&form| form == Form::Pinghe)
    }

    fn calc_agari_fu(&self) -> u32 {
        if self.agari.is_zimo() {
            2
        } else if self.agari.is_menqian() {
            10
        } else {
            0
        }
    }

    fn calc_kezi_fu(&self) -> u32 {
        let mut res = 0;

        // 明刻 (槓を除く)
        res += (self.agari.pengs())
            .chain(self.agari.ronghe_mingke())
            .map(|tiles| tiles.first())
            .map(|tile| if tile.is_zhongzhang() { 2 } else { 4 })
            .sum::<u32>();

        // 暗刻 (槓を除く)
        res += (self.agari.kezis_in_hand())
            .map(|tiles| tiles.first())
            .map(|tile| if tile.is_zhongzhang() { 4 } else { 8 })
            .sum::<u32>();

        // 明槓
        res += (self.agari.minggangs())
            .map(|tiles| tiles.first())
            .map(|tile| if tile.is_zhongzhang() { 8 } else { 16 })
            .sum::<u32>();

        // 暗槓
        res += (self.agari.angangs())
            .map(|tiles| tiles.first())
            .map(|tile| if tile.is_zhongzhang() { 16 } else { 32 })
            .sum::<u32>();

        res
    }

    fn calc_quetou_fu(&self) -> u32 {
        (self.agari.quetou()).first().num_fan(&self.agari.context())
    }

    fn calc_machi_fu(&self) -> u32 {
        use crate::agaritilesets::MachiKind;
        match self.agari.machi() {
            MachiKind::Liangmian | MachiKind::Shuangpeng => 0,
            _ => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Context, Direction};
    use crate::tilesets::Tilesets;

    fn parse(from: &str) -> Tilesets {
        let tilesets = from
            .split_whitespace()
            .map(|tileset| tileset.parse().unwrap())
            .collect();

        Tilesets::new(Context::default(), tilesets).unwrap()
    }

    fn with_direction(tilesets: Tilesets, player: Direction, place: Direction) -> Tilesets {
        Tilesets {
            context: Context {
                player,
                place,
                ..Context::default()
            },
            ..tilesets
        }
    }

    #[test]
    fn judge_simple() {
        crate::logger::init_once();

        let tilesets = parse("1p1p1p2p2p2p3p3p3p4p4p4p5p ツモ5p");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p1p1p2p2p2p3p3p3p4p4p4p5p ツモ5p\n(1p1p1p 2p2p2p 3p3p3p 4p4p4p 5p5p 待ち: 単騎)\n13翻 四暗刻単騎\n48000点 役満"
        );
    }

    #[test]
    fn judge_kokushimuso() {
        crate::logger::init_once();

        let tilesets = parse("1s9s1m9m1p9p東南西北白發中 ツモ中");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1s9s1m9m1p9p東南西北白發中 ツモ中\n13翻 国士無双13面待ち\n48000点 役満"
        );
    }

    #[test]
    fn judge_qiduizi() {
        crate::logger::init_once();

        let tilesets = parse("3s3s5s5s1p6p6p東東白白中中 ツモ1p");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n3s3s5s5s1p6p6p東東白白中中 ツモ1p\n1翻 門前清自摸和\n2翻25符 七対子\n3翻25符 4800点"
        );
    }

    #[test]
    fn judge_qiduizi_liangbeigou() {
        crate::logger::init_once();

        let tilesets = parse("1p1p2p2p3p3p4p4p5p5p6p6p7p ツモ7p");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p1p2p2p3p3p4p4p5p5p6p6p7p ツモ7p\n(1p2p3p 1p2p3p 5p6p7p 5p6p7p 4p4p 待ち: 両面)\n1翻 門前清自摸和\n1翻 平和\n3翻 二盃口\n6翻 清一色\n11翻 36000点 三倍満"
        );
    }

    #[test]
    fn judge_peng() {
        crate::logger::init_once();

        let tilesets = parse("1p1p1p2p2p2p3p3p3p5p ツモ5P ポン4p4p4p");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p1p1p2p2p2p3p3p3p5p ポン4p4p4p ツモ5P\n(4p4p4p 1p1p1p 2p2p2p 3p3p3p 5p5P 待ち: 単騎)\n1翻 ドラ\n2翻 三暗刻\n2翻 対々和\n5翻 清一色\n10翻 24000点 倍満"
        );
    }

    #[test]
    fn judge_xijia() {
        crate::logger::init_once();

        let tilesets = parse("5s6s7s4m5m6m4p4p4p5p6p西西 ロン西");
        let tilesets = with_direction(tilesets, Direction::West, Direction::East);

        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 西家 \n5s6s7s4m5m6m4p4p4p5p6p西西 ロン西\n(西西西 5s6s7s 4m5m6m 4p5p6p 4p4p 待ち: シャンポン)\n1翻 役牌\n1翻40符 1300点"
        );
    }

    #[test]
    fn judge_qingyise() {
        crate::logger::init_once();

        let tilesets = parse("1s2s3s4s5s6s6s7s8s8s9s西西 ロン7s ドラ1s中6s2p");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1s2s3s4s5s6s6s7s8s8s9s西西 ロン7s\n(6s7s8s 1s2s3s 4s5s6s 7s8s9s 西西 待ち: カンチャン)\n2翻 一気通貫\n3翻 混一色\n3翻 ドラ\n8翻 24000点 倍満"
        );
    }

    #[test]
    fn judge_40fu() {
        crate::logger::init_once();

        let tilesets = parse("1m2m3m7m7m4s5s6s8s8s西西西 ツモ7m");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n4s5s6s8s8s1m2m3m7m7m西西西 ツモ7m\n(7m7m7m 西西西 4s5s6s 1m2m3m 8s8s 待ち: シャンポン)\n1翻 門前清自摸和\n1翻40符 2000点"
        );
    }

    #[test]
    fn judge_pinghe() {
        use crate::context::Lizhi;
        crate::logger::init_once();

        let tilesets = parse("1p1p2p3p4p4p5p6p6p7p7p8p9p ツモ5P ドラ9p9p東9m");
        let tilesets = Tilesets {
            context: Context {
                lizhi: Lizhi::Lizhi,
                ..Context::default()
            },
            ..tilesets
        };
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p1p2p3p4p4p5p6p6p7p7p8p9p ツモ5P\n(2p3p4p 4p5p6p 5P6p7p 7p8p9p 1p1p 待ち: 両面)\n1翻 立直\n1翻 門前清自摸和\n1翻 平和\n3翻 ドラ\n6翻 清一色\n12翻 36000点 三倍満"
        )
    }

    #[test]
    fn judge_qinglaotou() {
        crate::logger::init_once();
        let tilesets = parse("1s1s1s9s9s9p9p ロン9p 暗槓1p1p1p1p ポン1m1m1m ドラ8s9p");
        let tilesets = with_direction(tilesets, Direction::West, Direction::East);
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 西家 \n1s1s1s9s9s9p9p ポン1m1m1m 暗槓1p1p1p1p ロン9p\n(1m1m1m 9p9p9p 1s1s1s 1p1p1p1p 9s9s 待ち: シャンポン)\n13翻 清老頭\n32000点 役満"
        );
    }

    #[test]
    fn judge_luyise() {
        crate::logger::init_once();
        let tilesets = parse("2s2s2s2s3s4s4s6s6s6s8s8s8s ロン3s");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n2s2s2s2s3s4s4s6s6s6s8s8s8s ロン3s\n(6s6s6s 8s8s8s 2s3s4s 2s3s4s 2s2s 待ち: カンチャン)\n13翻 緑一色\n48000点 役満"
        );
    }

    #[test]
    fn judge_hunlaotou() {
        crate::logger::init_once();
        let tilesets = parse("1m1m9m9m1s1s1s東東東 ロン9m ポン白白白");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1s1s1s1m1m9m9m東東東 ポン白白白 ロン9m\n(白白白 9m9m9m 1s1s1s 東東東 1m1m 待ち: シャンポン)\n2翻 混老頭\n2翻 対々和\n3翻 役牌\n7翻 18000点 跳満"
        );
    }

    #[test]
    fn judge_bianzhang() {
        use crate::context::Lizhi;

        crate::logger::init_once();
        let tilesets = parse("2s2s1p1p1p3m4m5m5s6s7s8s9s ツモ7s ドラ4m");
        let tilesets = Tilesets {
            context: Context {
                lizhi: Lizhi::Lizhi,
                ..Context::default()
            },
            ..tilesets
        };

        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n2s2s5s6s7s8s9s3m4m5m1p1p1p ツモ7s\n(1p1p1p 5s6s7s 7s8s9s 3m4m5m 2s2s 待ち: ペンチャン)\n1翻 立直\n1翻 門前清自摸和\n1翻 ドラ\n3翻40符 7700点"
        );
    }

    #[test]
    fn judge_true_yiman() {
        crate::logger::init_once();
        let tilesets = parse("2m2m2m3m3m3m4m4m4m5m5m5m1m ツモ1m ドラ2m3m4m");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1m2m2m2m3m3m3m4m4m4m5m5m5m ツモ1m\n(2m2m2m 3m3m3m 4m4m4m 5m5m5m 1m1m 待ち: 単騎)\n13翻 四暗刻単騎\n48000点 役満"
        );
    }

    #[test]
    fn judge_pinghe_zipai() {
        crate::logger::init_once();
        let tilesets = parse("1p2p2p3p3p4p6p6p7p7p8p南南 ロン8p");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p2p2p3p3p4p6p6p7p7p8p南南 ロン8p\n(6p7p8p 1p2p3p 2p3p4p 6p7p8p 南南 待ち: 両面)\n1翻 平和\n1翻 一盃口\n3翻 混一色\n5翻 12000点 満貫"
        );
    }

    #[test]
    fn judge_pinghe_zipai_ng() {
        crate::logger::init_once();
        let tilesets = parse("1p2p2p3p3p4p6p6p7p7p8p東東 ロン8p");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p2p2p3p3p4p6p6p7p7p8p東東 ロン8p\n(6p7p8p 1p2p3p 2p3p4p 6p7p8p 東東 待ち: 両面)\n1翻 一盃口\n3翻 混一色\n4翻40符 12000点 満貫"
        );
    }

    #[test]
    fn judge_dora_only() {
        crate::logger::init_once();
        let tilesets = parse("2m3m4m2s2s4s5s6p7p8p ツモ6s チー3m1m2m ドラ2m");
        assert!(judge(&tilesets).is_none());
    }
}
