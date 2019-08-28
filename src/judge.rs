use crate::agaritilesets::AgariTilesets;
use crate::form::{Form, Point};
use crate::tilesets::Tilesets;
use log::debug;
use std::cmp::Ordering;
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
        // 役を小さい順に並べる。
        forms.sort();

        // もし真の役満が含まれているなら、それ以外を除く。
        if forms.iter().any(|form| form.point().is_true_yiman()) {
            forms.retain(|form| form.point().is_true_yiman());
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

impl PartialEq for Judge {
    fn eq(&self, other: &Judge) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Judge {}

impl PartialOrd for Judge {
    fn partial_cmp(&self, other: &Judge) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Judge {
    fn cmp(&self, other: &Judge) -> Ordering {
        // まずは得点で比較し、もし同じならば役の個数が少ない方をとる。
        (self.total.value(false))
            .cmp(&other.total.value(false))
            .then_with(|| self.forms.len().cmp(&other.forms.len()).reverse())
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
    let res = judge_all(tilesets).max();

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
        .chain(special_check_hungyise_qingyise(tilesets))
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
        let tilesets = parse("1p1p1p2p2p2p3p3p3p4p4p4p5p ツモ5p");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p1p1p2p2p2p3p3p3p4p4p4p5p ツモ5p\n(1p1p1p 2p2p2p 3p3p3p 4p4p4p 5p5p 待ち: 単騎)\n13翻 四暗刻単騎\n48000点 役満"
        );
    }

    #[test]
    fn judge_kokushimuso() {
        let tilesets = parse("1s9s1m9m1p9p東南西北白發中 ツモ中");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1s9s1m9m1p9p東南西北白發中 ツモ中\n13翻 国士無双13面待ち\n48000点 役満"
        );
    }

    #[test]
    fn judge_qiduizi_liangbeigou() {
        let tilesets = parse("1p1p2p2p3p3p4p4p5p5p6p6p7p ツモ7p");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p1p2p2p3p3p4p4p5p5p6p6p7p ツモ7p\n(1p2p3p 1p2p3p 5p6p7p 5p6p7p 4p4p 待ち: 両面)\n1翻 門前清自摸和\n1翻 平和\n3翻 二盃口\n6翻 清一色\n11翻 36000点 三倍満"
        );
    }

    #[test]
    fn judge_peng() {
        let tilesets = parse("1p1p1p2p2p2p3p3p3p5p ツモ5P ポン4p4p4p");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p1p1p2p2p2p3p3p3p5p ポン4p4p4p ツモ5P\n(4p4p4p 1p1p1p 2p2p2p 3p3p3p 5p5P 待ち: 単騎)\n1翻 ドラ\n2翻 三暗刻\n2翻 対々和\n5翻 清一色\n10翻 24000点 倍満"
        );
    }

    #[test]
    fn judge_xijia() {
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
        let tilesets = parse("1s2s3s4s5s6s6s7s8s8s9s西西 ロン7s ドラ1s中6s2p");
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1s2s3s4s5s6s6s7s8s8s9s西西 ロン7s\n(6s7s8s 1s2s3s 4s5s6s 7s8s9s 西西 待ち: カンチャン)\n2翻 一気通貫\n3翻 混一色\n3翻 ドラ\n8翻 24000点 倍満"
        );
    }
}
