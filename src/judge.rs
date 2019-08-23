use crate::agaritilesets::AgariTilesets;
use crate::form::{Form, Point};
use crate::tilesets::Tilesets;
use std::cmp::Ordering;
use std::fmt;
use std::iter::once;

#[derive(Debug, Clone)]
pub struct Judge {
    forms: Vec<Form>,
    total: Point,
    tilesets: Tilesets,
}

impl Judge {
    fn new(forms: Vec<Form>, total: Point, tilesets: Tilesets) -> Option<Judge> {
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
        if forms.iter().any(|form| form.point().is_true_yakuman()) {
            forms.retain(|form| form.point().is_true_yakuman());
        }
    }

    fn from_tilesets(tilesets: Tilesets, mut forms: Vec<Form>) -> Option<Judge> {
        // 役を補正する。
        Judge::fix_forms(&mut forms);

        // 合計の翻数を計算する。
        let total: Point = forms.iter().map(|f| f.point()).sum();

        Judge::new(forms, total, tilesets)
    }

    fn from_agaritilesets(agari: AgariTilesets, mut forms: Vec<Form>) -> Option<Judge> {
        // 役を補正する。
        Judge::fix_forms(&mut forms);

        // 合計の翻数を計算する。
        let mut total: Point = forms.iter().map(|f| f.point()).sum();

        // 符計算をする。
        assert_eq!(total.fu, 0);
        total.fu = FuCalculator::new(&agari, &forms).calculate();

        Judge::new(forms, total, agari.tilesets)
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
            self.tilesets.context.place,
            self.tilesets.context.player,
            self.tilesets.context.player_name
        )?;

        writeln!(b, "{}", self.tilesets)?;

        for form in &self.forms {
            writeln!(b, "{}", form.display())?;
        }

        write!(
            b,
            "{}",
            self.total.display_full(self.tilesets.context.is_oya())
        )
    }
}

pub fn judge(tilesets: &Tilesets) -> Option<Judge> {
    judge_all(tilesets).max()
}

fn judge_all(tilesets: &Tilesets) -> impl Iterator<Item = Judge> {
    let qiduizi = judge_qiduizi(&tilesets);
    let kokusimusou = judge_kokusimusou(&tilesets);
    let jiulianbaodeng = judge_jiulianbaodeng(&tilesets);
    let agari = (AgariTilesets::enumerate(tilesets).into_iter()).filter_map(judge_agari);

    agari
        .chain(qiduizi)
        .chain(kokusimusou)
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
    use crate::form::*;

    let mut forms = Vec::with_capacity(10);
    forms.extend(forms_for_all_base(&agari.tilesets));
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
    forms.extend(check_shousangen(&agari));
    forms.extend(check_daisangen(&agari));
    forms.extend(check_shousushi_daisushi(&agari));

    Judge::from_agaritilesets(agari, forms)
}

fn judge_qiduizi(tilesets: &Tilesets) -> Option<Judge> {
    crate::form::special_check_qiduizi(tilesets)
        .map(|q| forms_for_all_base(tilesets).chain(once(q)).collect())
        .and_then(|forms| Judge::from_tilesets(tilesets.clone(), forms))
}

fn judge_kokusimusou(tilesets: &Tilesets) -> Option<Judge> {
    crate::form::special_check_kokusimusou(tilesets)
        .map(|k| forms_for_all_base(tilesets).chain(once(k)).collect())
        .and_then(|forms| Judge::from_tilesets(tilesets.clone(), forms))
}

fn judge_jiulianbaodeng(tilesets: &Tilesets) -> Option<Judge> {
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
        // 平和ツモは一律 20 符
        if self.is_pinghe_tumo() {
            return 20;
        }

        // アガリ基本符
        let fudi = 20;

        // アガリ方による符
        let agari_fu = self.calc_agari_fu();

        // 刻子、槓によるボーナス
        let kotu_fu = self.calc_kotu_fu();

        // 雀頭によるボーナス
        let janto_fu = self.calc_janto_fu();

        // 待ちによるボーナス
        let machi_fu = self.calc_machi_fu();

        let base = fudi + agari_fu + kotu_fu + janto_fu + machi_fu;
        let ceiled = (base + 9) / 10 * 10;

        // 喰い平和形では 20 符となるが、このときは 30 符に引き上げる
        if !self.agari.tilesets.is_tumo && ceiled == 20 {
            return 30;
        }

        ceiled
    }

    fn is_pinghe_tumo(&self) -> bool {
        self.agari.tilesets.is_tumo && self.forms.iter().any(|&form| form == Form::Pinghe)
    }

    fn calc_agari_fu(&self) -> u32 {
        if self.agari.tilesets.is_tumo {
            2
        } else if self.agari.tilesets.is_menzen() {
            10
        } else {
            0
        }
    }

    fn calc_kotu_fu(&self) -> u32 {
        let mut res = 0;

        // 明刻 (槓を除く)
        res += (self.agari.tilesets.pons.iter())
            .chain(self.agari.ronmin.iter_minko())
            .map(|tiles| tiles.first())
            .map(|tile| if tile.is_chunchan() { 2 } else { 4 })
            .sum::<u32>();

        // 暗刻 (槓を除く)
        res += (self.agari.kotus_in_hand())
            .map(|tiles| tiles.first())
            .map(|tile| if tile.is_chunchan() { 4 } else { 8 })
            .sum::<u32>();

        // 明槓
        res += (self.agari.tilesets.minkans.iter())
            .map(|tiles| tiles.first())
            .map(|tile| if tile.is_chunchan() { 8 } else { 16 })
            .sum::<u32>();

        // 暗槓
        res += (self.agari.tilesets.ankans.iter())
            .map(|tiles| tiles.first())
            .map(|tile| if tile.is_chunchan() { 16 } else { 32 })
            .sum::<u32>();

        res
    }

    fn calc_janto_fu(&self) -> u32 {
        (self.agari.janto())
            .first()
            .num_yakuhai(&self.agari.tilesets.context)
    }

    fn calc_machi_fu(&self) -> u32 {
        use crate::agaritilesets::MachiKind;
        match self.agari.machi {
            MachiKind::Ryanmen | MachiKind::Shanpon => 0,
            _ => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Context, Direction};
    use crate::tilesets::Tilesets;

    #[test]
    fn test_judge() {
        let tilesets: Tilesets = "1p1p1p2p2p2p3p3p3p4p4p4p5p ツモ5p".parse().unwrap();
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p1p1p2p2p2p3p3p3p4p4p4p5p ツモ5p\n13翻 四暗刻単騎\n48000点 役満"
        );

        let tilesets: Tilesets = "1s9s1m9m1p9p東南西北白發中 ツモ中"
            .parse()
            .unwrap();
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1s9s1m9m1p9p東南西北白發中 ツモ中\n13翻 国士無双13面待ち\n48000点 役満"
        );

        let tilesets: Tilesets = "1p1p2p2p3p3p4p4p5p5p6p6p7p ツモ7p".parse().unwrap();
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p1p2p2p3p3p4p4p5p5p6p6p7p ツモ7p\n1翻 門前清自摸和\n2翻5符 七対子\n6翻 清一色\n9翻 24000点 倍満"
        );

        let tilesets: Tilesets = "1p1p2p2p3p3p4p4p5p5p6p6p7p ツモ7p".parse().unwrap();
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p1p2p2p3p3p4p4p5p5p6p6p7p ツモ7p\n1翻 門前清自摸和\n2翻5符 七対子\n6翻 清一色\n9翻 24000点 倍満"
        );

        let tilesets: Tilesets = "1p1p1p2p2p2p3p3p3p5p ツモ5P ポン4p4p4p"
            .parse()
            .unwrap();
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 東家 \n1p1p1p2p2p2p3p3p3p5p ポン4p4p4p ツモ5P\n1翻 ドラ\n2翻 三暗刻\n2翻 対々和\n5翻 清一色\n10翻 24000点 倍満"
        );

        let tilesets: Tilesets = "5s6s7s4m5m6m4p4p4p5p6p西西 ロン西".parse().unwrap();
        let tilesets = Tilesets {
            context: Context {
                player: Direction::West,
                ..Context::default()
            },
            ..tilesets
        };
        let res = dbg!(judge(&tilesets)).unwrap();
        assert_eq!(
            res.to_string(),
            "東場 西家 \n5s6s7s4m5m6m4p4p4p5p6p西西 ロン西\n1翻 役牌\n1翻40符 1300点"
        );
    }
}
