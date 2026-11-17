// 診断専用: src/heuristic.rsの座標修復を基にした行penalty試作。
//! 丸めたLP点の制約違反を座標修復する、証明には使わない主解探索。
use mip_codex::model::Model;
use std::time::Instant;
#[derive(Default, Debug, Clone)]
pub struct Stats {
    pub calls: usize,
    pub candidates: usize,
    pub moves: usize,
    pub work: usize,
}
pub struct Repair<'a> {
    model: &'a Model,
    incidence: Vec<Vec<(usize, f64)>>,
    weights: Vec<f64>,
    pub stats: Stats,
}
impl<'a> Repair<'a> {
    pub fn new(model: &'a Model) -> Self {
        let mut incidence = vec![vec![]; model.variables.len()];
        let weights = model
            .rows
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let mut scale: f64 = 1.;
                for &(j, a) in &r.coefficients {
                    incidence[j].push((i, a));
                    scale = scale.max(a.abs());
                }
                1. / scale
            })
            .collect();
        Self {
            model,
            incidence,
            weights,
            stats: Stats::default(),
        }
    }
    fn violation(&self, i: usize, activity: f64) -> f64 {
        let row = &self.model.rows[i];
        row.lower
            .map_or(0., |l| (l - activity).max(0.))
            .max(row.upper.map_or(0., |u| (activity - u).max(0.)))
            * self.weights[i]
    }
    pub fn candidate_check(&self, x: &[f64]) -> bool {
        if self.model.check(x, true, 1e-7).is_err() {
            return false;
        }
        // この追加ヒューリスティックからのincumbentは、桁落ちで合格させない。
        self.model.rows.iter().all(|r| {
            let (mut lo, mut hi) = (0.0f64, 0.0f64);
            for &(j, a) in &r.coefficients {
                let term = a * x[j];
                lo = (lo + term.next_down()).next_down();
                hi = (hi + term.next_up()).next_up();
            }
            lo.is_finite()
                && hi.is_finite()
                && r.lower.is_none_or(|l| lo >= l - 1e-7)
                && r.upper.is_none_or(|u| hi <= u + 1e-7)
        })
    }
    /// 失敗・予算切れは探索木や下界に影響させない。
    pub fn search(
        &mut self,
        point: &[f64],
        lower: &[Option<f64>],
        upper: &[Option<f64>],
        deadline: Instant,
        budget: usize,
        seed: u64,
    ) -> Option<Vec<f64>> {
        self.search_impl(point,lower,upper,deadline,budget,seed,0.,1.)
    }
    /// 費用を正規化した修復順位を診断する。実行可能性判定は元のguardを使う。
    pub fn search_priced(&mut self,point:&[f64],lower:&[Option<f64>],upper:&[Option<f64>],deadline:Instant,budget:usize,seed:u64,weight:f64,scale:f64)->Option<Vec<f64>> {
        if !weight.is_finite()||weight<0.||!scale.is_finite()||scale<=0.{return None;}
        self.search_impl(point,lower,upper,deadline,budget,seed,weight,scale)
    }
    fn search_impl(&mut self,point:&[f64],lower:&[Option<f64>],upper:&[Option<f64>],deadline:Instant,budget:usize,seed:u64,weight:f64,scale:f64)->Option<Vec<f64>> {
        self.stats.calls += 1;
        let mut rng = seed.wrapping_add(0x9e3779b97f4a7c15);
        let mut work = 0usize;
        let screen = std::env::var("MIP_REPAIR_CHECK_SCREEN").map_or(true, |v| v == "1");
        for attempt in 0..4 {
            if Instant::now() >= deadline || work >= budget {
                break;
            }
            let mut x = point.to_vec();
            for (j, v) in self.model.variables.iter().enumerate() {
                if v.integer {
                    rng ^= rng << 13;
                    rng ^= rng >> 7;
                    rng ^= rng << 17;
                    x[j] = if attempt == 0 {
                        x[j].round()
                    } else {
                        let probability = (rng >> 11) as f64 / ((1u64 << 53) as f64);
                        if probability < x[j] - x[j].floor() {
                            x[j].ceil()
                        } else {
                            x[j].floor()
                        }
                    };
                }
                if let Some(l) = lower[j] {
                    x[j] = x[j].max(if v.integer { l.ceil() } else { l });
                }
                if let Some(u) = upper[j] {
                    x[j] = x[j].min(if v.integer { u.floor() } else { u });
                }
            }
            let mut activities: Vec<_> = self
                .model
                .rows
                .iter()
                .map(|r| r.coefficients.iter().map(|&(j, a)| a * x[j]).sum::<f64>())
                .collect();
            work = work.saturating_add(
                self.model
                    .rows
                    .iter()
                    .map(|r| r.coefficients.len() + 1)
                    .sum::<usize>(),
            );
            loop {
                if Instant::now() >= deadline || work >= budget {
                    break;
                }
                // 許容誤差の境界から少し内側で候補を受理し、独立検査も別途行う。
                // 活動値は候補受理の証明には使わず、明白な違反時の全検査だけ省く。
                // 丸め誤差による探索取り逃しを抑えるため、受理閾値より広い余裕を使う。
                let plausible = !screen || activities.iter().enumerate().all(|(i, &a)| {
                    let row = &self.model.rows[i];
                    !a.is_finite() || (row.lower.is_none_or(|l| a >= l - 1e-6)
                        && row.upper.is_none_or(|u| a <= u + 1e-6))
                });
                if plausible && self.candidate_check(&x) {
                    self.stats.candidates += 1;
                    self.stats.work += work;
                    return Some(x);
                }
                let worst = activities
                    .iter()
                    .enumerate()
                    .map(|(i, &a)| (i, self.violation(i, a)))
                    .filter(|(_, v)| v.is_finite() && *v > 0.)
                    .max_by(|a, b| a.1.total_cmp(&b.1));
                work = work.saturating_add(self.model.rows.len() + 1);
                let Some((i, _)) = worst else {
                    break;
                };
                let row = &self.model.rows[i];
                let target = if row.lower.is_some_and(|l| activities[i] < l) {
                    row.lower.unwrap()
                } else {
                    row.upper.unwrap()
                };
                let mut best: Option<(usize, f64, f64, f64)> = None;
                for &(j, a) in &row.coefficients {
                    if a == 0. {
                        continue;
                    }
                    let raw = x[j] + (target - activities[i]) / a;
                    let choices = if self.model.variables[j].integer {
                        [raw.floor(), raw.ceil()]
                    } else {
                        [raw, raw]
                    };
                    for mut value in choices {
                        if let Some(l) = lower[j] {
                            value = value.max(l);
                        }
                        if let Some(u) = upper[j] {
                            value = value.min(u);
                        }
                        let delta = value - x[j];
                        if !value.is_finite() || delta == 0. {
                            continue;
                        }
                        if self.model.variables[j].integer && value != value.round() {
                            continue;
                        }
                        let mut gain = 0.;
                        for &(r, c) in &self.incidence[j] {
                            gain += self.violation(r, activities[r])
                                - self.violation(r, activities[r] + c * delta);
                        }
                        work = work.saturating_add(self.incidence[j].len());
                        let cost = (if self.model.maximize { -1. } else { 1. })
                            * self.model.variables[j].objective
                            * delta;
                        let score=if weight==0.{gain}else{gain/(1.+weight*(cost.max(0.)/scale))};
                        if gain > 1e-12 && (weight==0. || score.is_finite())
                            && best.is_none_or(|(_, _, g, c)| {
                                score > g + 1e-12 || ((score - g).abs() <= 1e-12 && cost < c)
                            })
                        {
                            best = Some((j, value, score, cost));
                        }
                    }
                    if work >= budget || Instant::now() >= deadline {
                        break;
                    }
                }
                let Some((j, value, _, _)) = best else {
                    // 診断専用: 局所停滞時、違反中の行のpenaltyを増やす。
                    // 実行可能性の受理条件や元のモデルは変更しない。
                    let mut changed=false;
                    for (r,&activity)in activities.iter().enumerate(){
                        if self.violation(r,activity)>1e-12&&self.weights[r]<1e12{
                            self.weights[r]*=2.;changed=true;
                        }
                    }
                    work=work.saturating_add(activities.len());
                    if changed{continue;}else{break;}
                };
                let delta = value - x[j];
                x[j] = value;
                for &(r, c) in &self.incidence[j] {
                    activities[r] += c * delta;
                }
                self.stats.moves += 1;
            }
        }
        self.stats.work += work;
        None
    }
}

