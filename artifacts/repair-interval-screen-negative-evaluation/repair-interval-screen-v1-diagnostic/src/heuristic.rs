//! 丸めたLP点の制約違反を座標修復する、証明には使わない主解探索。
use crate::model::Model;
use std::time::Instant;
#[derive(Default, Debug, Clone)]
pub struct Stats {
    pub calls: usize,
    pub candidates: usize,
    pub moves: usize,
    pub work: usize,
}
// 元f64係数の実数積和を外側から包む。非有限なら候補を除外しない。
#[derive(Clone,Copy)]
struct ActivityInterval { lo:f64, hi:f64 }
impl ActivityInterval {
    fn new(terms:impl Iterator<Item=(f64,f64)>)->Self {
        let(mut lo,mut hi)=(0f64,0f64);
        for(a,x)in terms{let p=a*x;lo=(lo+p.next_down()).next_down();hi=(hi+p.next_up()).next_up();}
        Self{lo,hi}
    }
    fn replace(&mut self,a:f64,old:f64,new:f64){
        let(p,q)=(a*new,a*old);
        self.lo=(self.lo+(p.next_down()-q.next_up()).next_down()).next_down();
        self.hi=(self.hi+(p.next_up()-q.next_down()).next_up()).next_up();
    }
    fn plausible(&self,row:&crate::model::Row)->bool{
        !self.lo.is_finite()||!self.hi.is_finite()||
            (row.lower.is_none_or(|l|self.hi>=l-1e-6)&&row.upper.is_none_or(|u|self.lo<=u+1e-6))
    }
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
        let interval_screen = screen && std::env::var("MIP_REPAIR_INTERVAL_SCREEN").is_ok_and(|v|v=="1");
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
            let mut intervals=interval_screen.then(||self.model.rows.iter().map(|r|ActivityInterval::new(r.coefficients.iter().map(|&(j,a)|(a,x[j])))).collect::<Vec<_>>());
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
                let plausible = !screen || if let Some(intervals)=intervals.as_ref(){
                    intervals.iter().zip(&self.model.rows).all(|(a,row)|a.plausible(row))
                }else{activities.iter().enumerate().all(|(i, &a)| {
                    let row = &self.model.rows[i];
                    !a.is_finite() || (row.lower.is_none_or(|l| a >= l - 1e-6)
                        && row.upper.is_none_or(|u| a <= u + 1e-6))
                })};
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
                    break;
                };
                let old=x[j];let delta = value - old;
                x[j] = value;
                for &(r, c) in &self.incidence[j] {
                    activities[r] += c * delta;
                    if let Some(intervals)=intervals.as_mut(){intervals[r].replace(c,old,value);}
                }
                self.stats.moves += 1;
            }
        }
        self.stats.work += work;
        None
    }
}

#[derive(Default, Debug, serde::Serialize)]
pub struct BoxStats {
    pub calls: usize,
    pub candidates: usize,
    pub improvements: usize,
    pub moves: usize,
    pub work: usize,
    pub seconds: f64,
    pub first_seconds: Option<f64>,
}
impl Repair<'_> {

    /// 元入力区間とJSON点誤差から報告目的の精度を確認する。
    pub(crate) fn objective_reliable(&self, point: &[f64], value: f64) -> bool {
        if !value.is_finite(){return false;}
        let mut error=crate::model::scalar_error(self.model.offset,self.model.offset_enclosure);
        for (v,&x) in self.model.variables.iter().zip(point) {
            let cost_error=crate::model::scalar_error(v.objective,v.objective_enclosure);
            let point_error=crate::model::point_error(x);
            if x!=0. {crate::model::add_error(&mut error,(x.abs()*cost_error).next_up());}
            if point_error!=0. {crate::model::add_error(&mut error,((v.objective.abs()+cost_error).next_up()*point_error).next_up());}
        }
        crate::model::add_error(&mut error,(value.next_up()-value).abs().max((value-value.next_down()).abs()));
        error.is_finite() && error<=(1e-8*(1.+value.abs())).next_down()
    }
    /// 既存の実行可能点から目的を改善する。元点を保持し、元問題の検査後だけ受理する。
    pub fn descend(&self, seed: &[f64], deadline: Instant) -> Option<(Vec<f64>,BoxStats)> {
        let started=Instant::now();
        if started>=deadline || !self.candidate_check(seed){return None;}
        let sign=if self.model.maximize{-1.}else{1.};
        let value=sign*self.model.objective(seed);if !value.is_finite(){return None;}
        let mut point=seed.to_vec();let mut stats=BoxStats{calls:1,..BoxStats::default()};
        let mut activities:Vec<_>=self.model.rows.iter().map(|r|crate::exact::dot(r.coefficients.iter().map(|&(j,a)|(a,point[j]))).unwrap_or(f64::NAN)).collect();
        let mut order=Vec::new();
        for(j,v)in self.model.variables.iter().enumerate(){
            let target=if sign*v.objective>0.{v.lower.map(|l|if v.integer{l.ceil()}else{l})}else if sign*v.objective<0.{v.upper.map(|u|if v.integer{u.floor()}else{u})}else{None};
            if let Some(target)=target.filter(|v|v.is_finite()) {let gain=-sign*v.objective*(target-point[j]);if gain.is_finite()&&gain>1e-12{order.push((j,target,gain/(1+self.incidence[j].len())as f64));}}
        }
        order.sort_unstable_by(|a,b|b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
        for(j,target,_)in order{if Instant::now()>=deadline{break;}let delta=target-point[j];if !delta.is_finite(){continue;}stats.work+=self.incidence[j].len();let allowed=self.incidence[j].iter().all(|&(i,a)|{let next=crate::exact::dot([(activities[i],1.),(a,delta)].into_iter()).unwrap_or(f64::NAN);let r=&self.model.rows[i];next.is_finite()&&r.lower.is_none_or(|l|l-next<=1e-8)&&r.upper.is_none_or(|u|next-u<=1e-8)});if !allowed{continue;}point[j]=target;for &(i,a)in &self.incidence[j]{activities[i]=crate::exact::dot([(activities[i],1.),(a,delta)].into_iter()).unwrap_or(f64::NAN);}stats.moves+=1;}
        if self.candidate_check(&point) && self.objective_reliable(&point,self.model.objective(&point)) && sign*self.model.objective(&point)<=value {stats.candidates=1;let improved=sign*self.model.objective(&point)<value;if improved{stats.improvements=1;stats.first_seconds=Some(started.elapsed().as_secs_f64());}}else{point=seed.to_vec();}
        stats.seconds=started.elapsed().as_secs_f64();Some((point,stats))

    }
    /// Feasible box endpoints and objective descent; no bound or infeasibility claims.
    pub fn box_search(&self, deadline: Instant) -> (Option<Vec<f64>>, BoxStats) {
        let started = Instant::now();
        let mut stats = BoxStats {
            calls: 1,
            ..BoxStats::default()
        };
        let sign = if self.model.maximize { -1. } else { 1. };
        let mut best = None;
        let mut best_value = f64::INFINITY;
        let mut remember = |point: &[f64], stats: &mut BoxStats| {
            if !self.candidate_check(point) {
                return false;
            }
            let value = sign * self.model.objective(point);
            if !value.is_finite() {
                return false;
            }
            stats.candidates += 1;
            stats
                .first_seconds
                .get_or_insert_with(|| started.elapsed().as_secs_f64());
            if value < best_value {
                best_value = value;
                best = Some(point.to_vec());
                stats.improvements += 1;
            }
            true
        };
        let domain = |j: usize| {
            let v = &self.model.variables[j];
            (
                v.lower.map(|b| if v.integer { b.ceil() } else { b }),
                v.upper.map(|b| if v.integer { b.floor() } else { b }),
            )
        };
        for mode in 0..4 {
            if Instant::now() >= deadline {
                break;
            }
            let mut point = Vec::with_capacity(self.model.variables.len());
            for (j, v) in self.model.variables.iter().enumerate() {
                let (lower, upper) = domain(j);
                let raw = match mode {
                    1 => lower.unwrap_or(0.),
                    2 => upper.unwrap_or(0.),
                    3 if sign * v.objective < 0. => upper.unwrap_or(0.),
                    3 if sign * v.objective > 0. => lower.unwrap_or(0.),
                    _ => 0.,
                };
                point.push(
                    raw.max(lower.unwrap_or(f64::NEG_INFINITY))
                        .min(upper.unwrap_or(f64::INFINITY)),
                );
            }
            stats.work += point.len();
            if !remember(&point, &mut stats) {
                continue;
            }
            let mut activities: Vec<_> = self
                .model
                .rows
                .iter()
                .map(|row| {
                    crate::exact::dot(row.coefficients.iter().map(|&(j, a)| (a, point[j])))
                        .unwrap_or(f64::NAN)
                })
                .collect();
            let mut order = Vec::new();
            for (j, v) in self.model.variables.iter().enumerate() {
                let (lower, upper) = domain(j);
                let target = if sign * v.objective < 0. {
                    upper
                } else if sign * v.objective > 0. {
                    lower
                } else {
                    None
                };
                let Some(target) = target.filter(|x| x.is_finite()) else {
                    continue;
                };
                let gain = -sign * v.objective * (target - point[j]);
                if gain.is_finite() && gain > 1e-12 {
                    order.push((j, target, gain / (1 + self.incidence[j].len()) as f64));
                }
            }
            order.sort_unstable_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
            for (j, target, _) in order {
                if Instant::now() >= deadline {
                    break;
                }
                let delta = target - point[j];
                if !delta.is_finite() {
                    continue;
                }
                stats.work += self.incidence[j].len();
                let allowed = self.incidence[j].iter().all(|&(i, a)| {
                    let next = crate::exact::dot([(activities[i], 1.), (a, delta)].into_iter())
                        .unwrap_or(f64::NAN);
                    let row = &self.model.rows[i];
                    next.is_finite()
                        && row.lower.is_none_or(|l| l - next <= 1e-8)
                        && row.upper.is_none_or(|u| next - u <= 1e-8)
                });
                if !allowed {
                    continue;
                }
                point[j] = target;
                for &(i, a) in &self.incidence[j] {
                    activities[i] =
                        crate::exact::dot([(activities[i], 1.), (a, delta)].into_iter())
                            .unwrap_or(f64::NAN);
                }
                stats.moves += 1;
            }
            // Approximate activity updates only choose moves. The original guard decides acceptance.
            remember(&point, &mut stats);
        }
        stats.seconds = started.elapsed().as_secs_f64();
        (best, stats)
    }
}

#[cfg(test)]mod activity_interval_tests{
 use super::*;use std::time::Duration;
 #[test]fn accumulated_replacements_enclose_an_independent_exact_dot(){let mut rng=87281u64;for case in 0..32{let mut next=||{rng=rng.wrapping_mul(6364136223846793005).wrapping_add(1);rng};let a:Vec<_>=(0..12).map(|j|{let s=if next()>>63==0{1.}else{-1.};s*2f64.powi([-500,-100,0,50,500][(j+case)%5])*(1.+(next()%8)as f64/8.)}).collect();let mut x:Vec<_>=(0..12).map(|_|(next()%32)as f64/8.).collect();let mut bounds=ActivityInterval::new(a.iter().copied().zip(x.iter().copied()));for _ in 0..64{let j=next()as usize%12;let value=(next()%32)as f64/8.;bounds.replace(a[j],x[j],value);x[j]=value;let exact=crate::exact::dot_bounds(a.iter().copied().zip(x.iter().copied())).unwrap();assert!(bounds.lo<=exact[0]&&bounds.hi>=exact[1],"case{case}: {:?} outside [{},{}]",exact,bounds.lo,bounds.hi);}}}
 #[test]fn a_feasible_zero_point_survives_a_large_incremental_activity_error(){let m=crate::mps::parse(std::io::Cursor::new("NAME drift\nROWS\n N obj\n E r\nCOLUMNS\n x0 obj 1 r 9007199254740992\n x1 obj 1 r 1\n x2 obj 1 r -9007199254740992\nRHS\n rhs r 0\nBOUNDS\n BV b x0\n BV b x1\n BV b x2\nENDATA\n")).unwrap();let a=[9007199254740992.,1.,-9007199254740992.];let mut x=[1.;3];let mut raw=a.iter().zip(x).map(|(a,x)|a*x).sum::<f64>();let mut bounds=ActivityInterval::new(a.into_iter().zip(x));for j in [2,1,0]{raw+=a[j]*(0.-x[j]);bounds.replace(a[j],x[j],0.);x[j]=0.;}assert_eq!(raw,-1.);assert!(Repair::new(&m).candidate_check(&x));assert!(bounds.plausible(&m.rows[0]));assert!(raw<m.rows[0].lower.unwrap()-1e-6);let mut repair=Repair::new(&m);assert!(repair.search(&x,&vec![Some(0.);3],&vec![Some(1.);3],Instant::now()+Duration::from_secs(1),1000,0).is_some());}
 #[test]fn nonfinite_enclosures_defer_to_the_original_candidate_checker(){let m=crate::mps::parse(std::io::Cursor::new("NAME n\nROWS\n N obj\n G r\nCOLUMNS\n x obj 1 r 1\nRHS\n rhs r 1\nBOUNDS\n BV b x\nENDATA\n")).unwrap();for a in [ActivityInterval{lo:f64::NAN,hi:f64::NAN},ActivityInterval{lo:f64::NEG_INFINITY,hi:f64::INFINITY}]{assert!(a.plausible(&m.rows[0]));}assert!(!Repair::new(&m).candidate_check(&[f64::NAN]));}
}
