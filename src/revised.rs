//! Bounded-variable revised primal simplex. Optimization logic is original Rust;
//! faer is used solely for general-purpose sparse LU factorizations/solves.
use crate::lp::Outcome;
use faer::{
    Col,
    linalg::solvers::Solve,
    sparse::{SparseColMat, Triplet, linalg::solvers::Lu},
};
use std::{sync::Arc, time::Instant};
const FEAS: f64 = 1e-8;
const PIVOT: f64 = 1e-10;
#[derive(Clone, Debug)]
pub struct Problem {
    pub columns: Vec<Vec<(usize, f64)>>,
    pub rhs: Vec<f64>,
    pub costs: Vec<f64>,
    pub upper: Vec<f64>,
    pub equality: Vec<bool>,
    pub slack_upper: Vec<f64>,
}
#[derive(Default, Clone, Debug)]
pub struct Stats {
    pub pivots: usize,
    pub flips: usize,
    pub refactors: usize,
    pub warm_attempts: usize,
    pub warm_successes: usize,
    pub warm_fallbacks: usize,
    pub warm_rejections: usize,
    pub dual_pivots: usize,
    pub dual_flips: usize,
}
#[derive(Debug)]
pub struct Answer {
    pub outcome: Outcome,
    pub dual: Option<f64>,
    pub stats: Stats,
    pub warm: Option<Arc<WarmStart>>,
}
#[derive(Debug)]
struct Eta {
    row: usize,
    column: Vec<(usize, f64)>,
    pivot: f64,
}
#[derive(Debug)]
enum Factor {
    Diagonal(Vec<f64>),
    Sparse(Lu<usize, f64>),
}
#[derive(Clone, Debug)]
struct Basis {
    factor: Arc<Factor>,
    updates: Vec<Arc<Eta>>,
}
impl Basis {
    fn forward(&self, v: &[f64]) -> Vec<f64> {
        let mut x = match self.factor.as_ref() {
            Factor::Diagonal(d) => v.iter().zip(d).map(|(v, d)| v / d).collect(),
            Factor::Sparse(lu) => {
                let rhs = Col::from_fn(v.len(), |i| v[i]);
                let x = lu.solve(&rhs);
                (0..v.len()).map(|i| x[i]).collect::<Vec<_>>()
            }
        };
        for e in &self.updates {
            let t = x[e.row] / e.pivot;
            for &(i, a) in &e.column {
                if i != e.row {
                    x[i] -= a * t;
                }
            }
            x[e.row] = t;
        }
        x
    }
    fn backward(&self, v: &[f64]) -> Vec<f64> {
        let mut x = v.to_vec();
        for e in self.updates.iter().rev() {
            let mut t = x[e.row];
            for &(i, a) in &e.column {
                if i != e.row {
                    t -= a * x[i];
                }
            }
            x[e.row] = t / e.pivot;
        }
        match self.factor.as_ref() {
            Factor::Diagonal(d) => x.iter().zip(d).map(|(v, d)| v / d).collect(),
            Factor::Sparse(lu) => {
                let rhs = Col::from_fn(v.len(), |i| x[i]);
                let x = lu.solve_transpose(&rhs);
                (0..v.len()).map(|i| x[i]).collect()
            }
        }
    }
    fn update(&mut self, row: usize, d: &[f64]) {
        self.updates.push(Arc::new(Eta {
            row,
            column: d
                .iter()
                .enumerate()
                .filter(|(_, v)| **v != 0.0)
                .map(|(i, &v)| (i, v))
                .collect(),
            pivot: d[row],
        }));
    }
}
/// 親の拡大行列を保持するため、RHS依存の人工列番号も子で変化しない。
#[derive(Debug)]
struct Structure {
    columns: Arc<Vec<Vec<(usize, f64)>>>,
    costs: Vec<f64>,
    equality: Vec<bool>,
    slack_upper: Vec<f64>,
    extra_upper: Vec<f64>,
    artificial: Vec<usize>,
}
#[derive(Debug)]
pub struct WarmStart {
    structure: Arc<Structure>,
    basis: Vec<usize>,
    at_upper: Vec<bool>,
    factor: Basis,
}
impl WarmStart {
    fn compatible(&self, p: &Problem) -> bool {
        let a = &self.structure;
        a.costs == p.costs
            && a.equality == p.equality
            && a.slack_upper == p.slack_upper
            && a.columns.len() >= p.columns.len()
            && a.columns[..p.columns.len()] == p.columns
    }
}
fn snapshot(p: &Problem, s: &Simplex, structure: Option<Arc<Structure>>) -> Arc<WarmStart> {
    let structure = structure.unwrap_or_else(|| {
        Arc::new(Structure {
            columns: s.columns.clone(),
            costs: p.costs.clone(),
            equality: p.equality.clone(),
            slack_upper: p.slack_upper.clone(),
            extra_upper: s.upper[p.costs.len()..].to_vec(),
            artificial: s.artificial.clone(),
        })
    });
    Arc::new(WarmStart {
        structure,
        basis: s.basis.clone(),
        at_upper: s.at_upper.clone(),
        factor: s.factor.clone(),
    })
}
struct Simplex {
    columns: Arc<Vec<Vec<(usize, f64)>>>,
    rhs: Vec<f64>,
    upper: Vec<f64>,
    basis: Vec<usize>,
    basic: Vec<bool>,
    at_upper: Vec<bool>,
    x: Vec<f64>,
    factor: Basis,
    stats: Stats,
    limit: usize,
    deadline: Option<Instant>,
    artificial: Vec<usize>,
    original: usize,
}
impl Simplex {
    fn refactor(&mut self) -> Result<(), Outcome> {
        if self.basis.is_empty() {
            return Ok(());
        }
        let triplets: Vec<_> = self
            .basis
            .iter()
            .enumerate()
            .flat_map(|(k, &j)| {
                self.columns[j]
                    .iter()
                    .map(move |&(i, a)| Triplet::new(i, k, a))
            })
            .collect();
        let matrix = SparseColMat::try_new_from_triplets(self.rhs.len(), self.rhs.len(), &triplets)
            .map_err(|_| Outcome::NumericalError)?;
        let lu = matrix.sp_lu().map_err(|_| Outcome::NumericalError)?;
        self.factor = Basis {
            factor: Arc::new(Factor::Sparse(lu)),
            updates: vec![],
        };
        self.stats.refactors += 1;
        self.recompute()
    }
    fn recompute(&mut self) -> Result<(), Outcome> {
        let mut residual = self.rhs.clone();
        for j in 0..self.columns.len() {
            if self.basic[j] {
                continue;
            }
            self.x[j] = if self.at_upper[j] { self.upper[j] } else { 0.0 };
            if self.x[j] != 0.0 {
                for &(i, a) in &self.columns[j] {
                    residual[i] -= a * self.x[j];
                }
            }
        }
        let x = self.factor.forward(&residual);
        for (i, &j) in self.basis.iter().enumerate() {
            self.x[j] = x[i];
        }
        if self.x.iter().any(|v| !v.is_finite()) {
            return Err(Outcome::NumericalError);
        }
        Ok(())
    }
    fn multipliers(&self, c: &[f64]) -> Vec<f64> {
        self.factor
            .backward(&self.basis.iter().map(|&j| c[j]).collect::<Vec<_>>())
    }
    fn reduced(&self, c: &[f64], y: &[f64], j: usize) -> f64 {
        c[j] - self.columns[j].iter().map(|&(i, a)| a * y[i]).sum::<f64>()
    }
    /// Validate primal and bounded-variable dual residuals independently of pricing.
    fn certificate(&self, c: &[f64]) -> Result<(f64, f64), Outcome> {
        let mut residual = self.rhs.clone();
        for j in 0..self.columns.len() {
            if !self.x[j].is_finite() || self.x[j] < -1e-7 || self.x[j] > self.upper[j] + 1e-7 {
                return Err(Outcome::NumericalError);
            }
            for &(i, a) in &self.columns[j] {
                residual[i] -= a * self.x[j];
            }
        }
        if residual.iter().any(|r| !r.is_finite() || r.abs() > 1e-6) {
            return Err(Outcome::NumericalError);
        }
        let y = self.multipliers(c);
        let mut dual: f64 = y.iter().zip(&self.rhs).map(|(y, b)| y * b).sum();
        for j in 0..self.columns.len() {
            let r = self.reduced(c, &y, j);
            if !r.is_finite() {
                return Err(Outcome::NumericalError);
            }
            if self.upper[j].is_finite() {
                dual += r.min(0.0) * self.upper[j];
            } else if r < -1e-7 {
                return Err(Outcome::NumericalError);
            }
        }
        let primal: f64 = c.iter().zip(&self.x).map(|(c, x)| c * x).sum();
        if !primal.is_finite()
            || !dual.is_finite()
            || (primal - dual).abs() > 1e-6 * (1.0 + primal.abs())
        {
            return Err(Outcome::NumericalError);
        }
        Ok((primal, dual.min(primal)))
    }
    /// 任意の行乗数と全box境界で矛盾を確認する。確証が弱ければcoldへ戻す。
    fn impossible_row(&self, w: &[f64]) -> bool {
        let rhs: f64 = w.iter().zip(&self.rhs).map(|(w, b)| w * b).sum();
        if !rhs.is_finite() {
            return false;
        }
        let mut lower = 0.0;
        let mut upper = 0.0;
        for j in 0..self.columns.len() {
            let a: f64 = self.columns[j].iter().map(|&(i, v)| v * w[i]).sum();
            if !a.is_finite() {
                return false;
            }
            if self.upper[j].is_finite() {
                let value = a * self.upper[j];
                if !value.is_finite() {
                    return false;
                }
                lower += value.min(0.0);
                upper += value.max(0.0);
            } else if a > 0.0 {
                upper = f64::INFINITY;
            } else if a < 0.0 {
                lower = f64::NEG_INFINITY;
            }
        }
        (lower.is_finite() && rhs < lower - 1e-7 * (1.0 + rhs.abs() + lower.abs()))
            || (upper.is_finite() && rhs > upper + 1e-7 * (1.0 + rhs.abs() + upper.abs()))
    }
    /// 有界変数のdual simplex。有限幅を使い切る候補をまとめてflipする。
    fn dual_run(&mut self, c: &[f64]) -> Result<(f64, f64), Outcome> {
        let mut degenerate = 0usize;
        loop {
            if self.deadline.is_some_and(|d| Instant::now() >= d) {
                return Err(Outcome::TimeLimit);
            }
            if self.stats.pivots + self.stats.flips >= self.limit {
                return Err(Outcome::IterationLimit);
            }
            let y = self.multipliers(c);
            if y.iter().any(|v| !v.is_finite()) {
                return Err(Outcome::NumericalError);
            }
            let mut reduced = vec![0.0; c.len()];
            for j in 0..c.len() {
                if self.basic[j] || self.upper[j] == 0.0 {
                    continue;
                }
                let r = self.reduced(c, &y, j);
                let q = if self.at_upper[j] { -r } else { r };
                if !r.is_finite() || q < -1e-8 {
                    return Err(Outcome::NumericalError);
                }
                reduced[j] = r;
            }
            let mut row = None;
            let mut largest = FEAS;
            for (i, &j) in self.basis.iter().enumerate() {
                let violation = (-self.x[j]).max(self.x[j] - self.upper[j]);
                if violation > FEAS
                    && (if degenerate > 25 {
                        row.is_none_or(|r| j < self.basis[r])
                    } else {
                        violation > largest
                    })
                {
                    row = Some(i);
                    largest = violation;
                }
            }
            let Some(r) = row else {
                self.recompute()?;
                if self
                    .basis
                    .iter()
                    .any(|&j| self.x[j] < -FEAS || self.x[j] > self.upper[j] + FEAS)
                {
                    continue;
                }
                return match self.certificate(c) {
                    Ok(answer) => Ok(answer),
                    Err(_) if !self.factor.updates.is_empty() => {
                        self.refactor()?;
                        self.certificate(c)
                    }
                    Err(e) => Err(e),
                };
            };
            let leaving = self.basis[r];
            let t = if self.x[leaving] < 0.0 { 1.0 } else { -1.0 };
            let target = if t > 0.0 { 0.0 } else { self.upper[leaving] };
            let mut unit = vec![0.0; self.rhs.len()];
            unit[r] = 1.0;
            let w = self.factor.backward(&unit);
            if w.iter().any(|v| !v.is_finite()) {
                return Err(Outcome::NumericalError);
            }
            let mut candidates = Vec::new();
            for j in 0..c.len() {
                if self.basic[j] || self.upper[j] == 0.0 {
                    continue;
                }
                let direction = if self.at_upper[j] { -1.0 } else { 1.0 };
                let alpha: f64 = self.columns[j].iter().map(|&(i, a)| a * w[i]).sum();
                let denom = -t * alpha * direction;
                if denom > PIVOT {
                    let beta = (reduced[j] * direction).max(0.0) / denom;
                    if !beta.is_finite() {
                        return Err(Outcome::NumericalError);
                    }
                    candidates.push((beta, j, direction, denom));
                }
            }
            candidates.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            let mut remaining = t * (target - self.x[leaving]);
            let mut flips = Vec::new();
            let mut enter = None;
            for (_, j, direction, denom) in candidates {
                let coverage = denom * self.upper[j];
                if self.upper[j].is_finite() && coverage < remaining {
                    remaining -= coverage;
                    flips.push((j, direction));
                } else {
                    enter = Some((j, direction));
                    break;
                }
            }
            let Some((j, direction)) = enter else {
                return Err(if self.impossible_row(&w) {
                    Outcome::Infeasible
                } else {
                    Outcome::NumericalError
                });
            };
            if !flips.is_empty() {
                let mut column = vec![0.0; self.rhs.len()];
                for &(k, sign) in &flips {
                    for &(i, a) in &self.columns[k] {
                        column[i] += a * sign * self.upper[k];
                    }
                }
                let change = self.factor.forward(&column);
                for (i, &k) in self.basis.iter().enumerate() {
                    self.x[k] -= change[i];
                }
                for &(k, _) in &flips {
                    self.at_upper[k] = !self.at_upper[k];
                    self.x[k] = if self.at_upper[k] { self.upper[k] } else { 0.0 };
                }
                self.stats.flips += flips.len();
                self.stats.dual_flips += flips.len();
            }
            let mut column = vec![0.0; self.rhs.len()];
            for &(i, a) in &self.columns[j] {
                column[i] += a;
            }
            let d = self.factor.forward(&column);
            let denom = -t * d[r] * direction;
            let remaining = t * (target - self.x[leaving]);
            if !denom.is_finite() || denom <= PIVOT || remaining < -1e-7 {
                return Err(Outcome::NumericalError);
            }
            let theta = remaining.max(0.0) / denom;
            if !theta.is_finite() || theta > self.upper[j] + 1e-7 {
                return Err(Outcome::NumericalError);
            }
            for (i, &k) in self.basis.iter().enumerate() {
                self.x[k] -= d[i] * direction * theta;
            }
            self.x[j] += direction * theta;
            self.x[leaving] = target;
            self.at_upper[leaving] = t < 0.0;
            self.basic[leaving] = false;
            self.basic[j] = true;
            self.basis[r] = j;
            self.factor.update(r, &d);
            self.stats.pivots += 1;
            self.stats.dual_pivots += 1;
            if theta < 1e-10 {
                degenerate += 1;
            } else {
                degenerate = 0;
            }
            if degenerate > 500 {
                return Err(Outcome::NumericalError);
            }
            if self.factor.updates.len() >= 50 {
                self.refactor()?;
            }
        }
    }
    fn run(&mut self, c: &[f64]) -> Result<(f64, f64), Outcome> {
        let mut degenerate = 0usize;
        let mut verified = false;
        'iterations: loop {
            if self.deadline.is_some_and(|d| Instant::now() >= d) {
                return Err(Outcome::TimeLimit);
            }
            if self.stats.pivots + self.stats.flips >= self.limit {
                return Err(Outcome::IterationLimit);
            }
            let y = self.multipliers(c);
            if y.iter().any(|v| !v.is_finite()) {
                return Err(Outcome::NumericalError);
            }
            let mut entering = None;
            let mut most_negative = -1e-9;
            for j in 0..c.len() {
                if self.basic[j] || self.upper[j] == 0.0 {
                    continue;
                }
                let direction = if self.at_upper[j] { -1.0 } else { 1.0 };
                let reduced = direction * self.reduced(c, &y, j);
                if reduced < most_negative {
                    most_negative = reduced;
                    entering = Some((j, direction));
                    if degenerate > 25 {
                        break;
                    }
                }
            }
            let Some((j, direction)) = entering else {
                // A fresh factorization is required before declaring convergence.
                if !verified && !self.factor.updates.is_empty() {
                    self.refactor()?;
                    verified = true;
                    continue;
                }
                return self.certificate(c);
            };
            verified = false;
            let mut column = vec![0.0; self.rhs.len()];
            for &(i, a) in &self.columns[j] {
                column[i] += a;
            }
            let d = self.factor.forward(&column);
            if d.iter().any(|v| !v.is_finite()) {
                return Err(Outcome::NumericalError);
            }
            let mut theta = self.upper[j];
            let mut leaving = None;
            for (i, &k) in self.basis.iter().enumerate() {
                let a = d[i] * direction;
                let ratio = if a > PIVOT {
                    self.x[k] / a
                } else if a < -PIVOT {
                    (self.upper[k] - self.x[k]) / (-a)
                } else {
                    continue;
                };
                if ratio < -FEAS {
                    if self.factor.updates.is_empty() {
                        return Err(Outcome::NumericalError);
                    }
                    self.refactor()?;
                    continue 'iterations;
                }
                let ratio = ratio.max(0.0);
                if ratio < theta - 1e-12
                    || ((ratio - theta).abs() <= 1e-12
                        && leaving.is_none_or(|r| self.basis[i] < self.basis[r]))
                {
                    theta = ratio;
                    leaving = Some(i);
                }
            }
            if !theta.is_finite() {
                // Re-factor before certifying an unbounded ray.
                if !self.factor.updates.is_empty() {
                    self.refactor()?;
                    continue;
                }
                let mut ray = vec![0.0; c.len()];
                ray[j] = direction;
                for (i, &k) in self.basis.iter().enumerate() {
                    ray[k] = -d[i] * direction;
                }
                let mut activity = vec![0.0; self.rhs.len()];
                for k in 0..c.len() {
                    if ray[k] < -1e-8 || (self.upper[k].is_finite() && ray[k].abs() > 1e-8) {
                        return Err(Outcome::NumericalError);
                    }
                    for &(i, a) in &self.columns[k] {
                        activity[i] += a * ray[k];
                    }
                }
                if activity.iter().any(|v| v.abs() > 1e-7)
                    || c.iter().zip(ray).map(|(c, r)| c * r).sum::<f64>() >= -1e-9
                {
                    return Err(Outcome::NumericalError);
                }
                return Err(Outcome::Unbounded);
            }
            if theta <= 1e-10 {
                degenerate += 1;
            } else {
                degenerate = 0;
            }
            for (i, &k) in self.basis.iter().enumerate() {
                self.x[k] -= theta * direction * d[i];
            }
            self.x[j] += theta * direction;
            if let Some(r) = leaving {
                let k = self.basis[r];
                self.at_upper[k] = d[r] * direction < 0.0 && self.upper[k].is_finite();
                self.x[k] = if self.at_upper[k] { self.upper[k] } else { 0.0 };
                self.basic[k] = false;
                self.basic[j] = true;
                self.basis[r] = j;
                self.factor.update(r, &d);
                self.stats.pivots += 1;
                if self.factor.updates.len() >= 50 {
                    self.refactor()?;
                }
            } else {
                self.at_upper[j] = !self.at_upper[j];
                self.x[j] = if self.at_upper[j] { self.upper[j] } else { 0.0 };
                self.stats.flips += 1;
            }
        }
    }
}
pub fn minimize(p: &Problem, limit: usize, deadline: Option<Instant>) -> Answer {
    cold(p, limit, deadline, false)
}
fn valid(p: &Problem) -> bool {
    p.columns.len() == p.costs.len()
        && p.upper.len() == p.costs.len()
        && p.equality.len() == p.rhs.len()
        && p.slack_upper.len() == p.rhs.len()
        && p.slack_upper.iter().all(|v| !v.is_nan() && *v >= 0.0)
        && p.rhs.iter().chain(&p.costs).all(|v| v.is_finite())
        && p.upper.iter().all(|v| !v.is_nan() && *v >= 0.0)
        && p.columns
            .iter()
            .all(|c| c.iter().all(|&(i, a)| i < p.rhs.len() && a.is_finite()))
}
pub fn minimize_warm(
    p: &Problem,
    limit: usize,
    deadline: Option<Instant>,
    warm: Option<&WarmStart>,
) -> Answer {
    if !valid(p) {
        return Answer {
            outcome: Outcome::NumericalError,
            dual: None,
            stats: Stats::default(),
            warm: None,
        };
    }
    let Some(w) = warm else {
        return cold(p, limit, deadline, true);
    };
    if !w.compatible(p) {
        let mut answer = cold(p, limit, deadline, true);
        answer.stats.warm_rejections = 1;
        return answer;
    }
    let a = &w.structure;
    let n = p.costs.len();
    let total = a.columns.len();
    let mut upper = p.upper.clone();
    upper.extend_from_slice(&a.extra_upper);
    let mut s = Simplex {
        columns: a.columns.clone(),
        rhs: p.rhs.clone(),
        upper,
        basis: w.basis.clone(),
        basic: vec![false; total],
        at_upper: w.at_upper.clone(),
        x: vec![0.0; total],
        factor: w.factor.clone(),
        stats: Stats::default(),
        limit,
        deadline,
        artificial: a.artificial.clone(),
        original: n,
    };
    for &j in &s.basis {
        s.basic[j] = true;
    }
    for j in 0..total {
        if !s.upper[j].is_finite() || s.upper[j] == 0.0 {
            s.at_upper[j] = false;
        }
    }
    let mut c = vec![0.0; total];
    c[..n].copy_from_slice(&p.costs);
    s.stats.warm_attempts = 1;
    let outcome = s.recompute().and_then(|_| s.dual_run(&c));
    match outcome {
        Ok((objective, dual)) => {
            s.stats.warm_successes = 1;
            Answer {
                outcome: Outcome::Optimal {
                    objective,
                    x: s.x[..n].to_vec(),
                },
                dual: Some(dual),
                warm: Some(snapshot(p, &s, Some(a.clone()))),
                stats: s.stats,
            }
        }
        Err(Outcome::Infeasible) => {
            s.stats.warm_successes = 1;
            Answer {
                outcome: Outcome::Infeasible,
                dual: None,
                warm: None,
                stats: s.stats,
            }
        }
        Err(outcome @ (Outcome::TimeLimit | Outcome::IterationLimit)) => Answer {
            outcome,
            dual: None,
            warm: None,
            stats: s.stats,
        },
        Err(_) => {
            let used = s.stats.pivots + s.stats.flips;
            let mut answer = cold(p, limit.saturating_sub(used), deadline, true);
            s.stats.warm_fallbacks = 1;
            answer.stats.add(&s.stats);
            answer
        }
    }
}
impl Stats {
    pub fn add(&mut self, other: &Self) {
        self.pivots += other.pivots;
        self.flips += other.flips;
        self.refactors += other.refactors;
        self.warm_attempts += other.warm_attempts;
        self.warm_successes += other.warm_successes;
        self.warm_fallbacks += other.warm_fallbacks;
        self.warm_rejections += other.warm_rejections;
        self.dual_pivots += other.dual_pivots;
        self.dual_flips += other.dual_flips;
    }
}
fn cold(p: &Problem, limit: usize, deadline: Option<Instant>, retain: bool) -> Answer {
    let m = p.rhs.len();
    let n = p.costs.len();
    let error = |outcome| Answer {
        outcome,
        dual: None,
        stats: Stats::default(),
        warm: None,
    };
    if p.columns.len() != n
        || p.upper.len() != n
        || p.equality.len() != m
        || p.slack_upper.len() != m
        || p.slack_upper.iter().any(|v| v.is_nan() || *v < 0.0)
        || p.rhs.iter().chain(&p.costs).any(|v| !v.is_finite())
        || p.upper.iter().any(|v| v.is_nan() || *v < 0.0)
        || p.columns
            .iter()
            .any(|col| col.iter().any(|&(i, a)| i >= m || !a.is_finite()))
    {
        return error(Outcome::NumericalError);
    }
    let mut s = Simplex {
        columns: Arc::new(p.columns.clone()),
        rhs: p.rhs.clone(),
        upper: p.upper.clone(),
        basis: vec![],
        basic: vec![false; n],
        at_upper: vec![false; n],
        x: vec![0.0; n],
        factor: Basis {
            factor: Arc::new(Factor::Diagonal(vec![])),
            updates: vec![],
        },
        stats: Stats::default(),
        limit,
        deadline,
        artificial: vec![],
        original: n,
    };
    let mut diagonal = vec![];
    for i in 0..m {
        let mut slack = None;
        if !p.equality[i] {
            let j = s.columns.len();
            Arc::make_mut(&mut s.columns).push(vec![(i, 1.0)]);
            s.upper.push(p.slack_upper[i]);
            s.basic.push(false);
            s.at_upper.push(false);
            s.x.push(0.0);
            slack = Some(j);
        }
        let slack_value = if p.rhs[i] > p.slack_upper[i] {
            p.slack_upper[i]
        } else {
            0.0
        };
        if let Some(j) = slack {
            s.x[j] = slack_value;
            s.at_upper[j] = slack_value != 0.0;
        }
        let residual = p.rhs[i] - if slack.is_some() { slack_value } else { 0.0 };
        let j = if residual >= 0.0 && p.rhs[i] <= p.slack_upper[i] && slack.is_some() {
            diagonal.push(1.0);
            slack.unwrap()
        } else {
            let j = s.columns.len();
            let sign = if residual < 0.0 { -1.0 } else { 1.0 };
            diagonal.push(sign);
            Arc::make_mut(&mut s.columns).push(vec![(i, sign)]);
            s.upper.push(f64::INFINITY);
            s.basic.push(false);
            s.at_upper.push(false);
            s.x.push(0.0);
            s.artificial.push(j);
            j
        };
        s.basis.push(j);
        s.basic[j] = true;
        s.x[j] = residual.abs();
    }
    s.factor.factor = Arc::new(Factor::Diagonal(diagonal));
    let outcome = (|| {
        if !s.artificial.is_empty() {
            let mut c = vec![0.0; s.columns.len()];
            for &j in &s.artificial {
                c[j] = 1.0;
            }
            let (value, dual) = s.run(&c)?;
            if value > 1e-7 {
                return Err(if dual > 1e-8 {
                    Outcome::Infeasible
                } else {
                    Outcome::NumericalError
                });
            }
            for &j in &s.artificial {
                s.upper[j] = 0.0;
                s.x[j] = 0.0;
                s.at_upper[j] = false;
            }
            s.recompute()?;
        }
        let mut c = vec![0.0; s.columns.len()];
        c[..n].copy_from_slice(&p.costs);
        let (objective, dual) = s.run(&c)?;
        Ok((objective, dual))
    })();
    match outcome {
        Ok((objective, dual)) => Answer {
            warm: retain.then(|| snapshot(p, &s, None)),
            outcome: Outcome::Optimal {
                objective,
                x: s.x[..s.original].to_vec(),
            },
            dual: Some(dual),
            stats: s.stats,
        },
        Err(outcome) => Answer {
            warm: None,
            outcome,
            dual: None,
            stats: s.stats,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eta_forward_and_transpose_agree_with_fresh_lu() {
        let mut basis = Basis {
            factor: Arc::new(Factor::Diagonal(vec![1.; 3])),
            updates: vec![],
        };
        let cols = [vec![2., 3., 1.], vec![1., 5., 2.], vec![3., 2., 4.]];
        for (j, col) in cols.iter().enumerate() {
            let d = basis.forward(col);
            basis.update(j, &d);
        }
        let triples: Vec<_> = cols
            .iter()
            .enumerate()
            .flat_map(|(j, col)| {
                col.iter()
                    .enumerate()
                    .map(move |(i, &v)| Triplet::new(i, j, v))
            })
            .collect();
        let lu = SparseColMat::try_new_from_triplets(3, 3, &triples)
            .unwrap()
            .sp_lu()
            .unwrap();
        for v in [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.], [2., -3., 5.]] {
            let rhs = Col::from_fn(3, |i| v[i]);
            let forward = lu.solve(&rhs);
            let backward = lu.solve_transpose(&rhs);
            for i in 0..3 {
                assert!((basis.forward(&v)[i] - forward[i]).abs() < 1e-10);
                assert!((basis.backward(&v)[i] - backward[i]).abs() < 1e-10);
            }
        }
    }
}
