//! 有界変数の改訂単体法。最適化の処理は本リポジトリのRustで実装。
//! faerは汎用の疎LU分解と線形方程式の解法だけに使用する。
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
    pub warm_cost_attempts: usize,
    pub warm_cost_successes: usize,
    pub dual_pivots: usize,
    pub dual_flips: usize,
    pub scaled_columns: usize,
    pub scaled_rows: usize,
    pub multiplier_cache_hits: usize,
    pub multiplier_cache_misses: usize,
    pub certificate_repair_trials: usize,
    pub certificate_repairs: usize,
}
#[derive(Debug)]
pub struct Answer {
    pub outcome: Outcome,
    pub dual: Option<f64>,
    pub stats: Stats,
    pub warm: Option<Arc<WarmStart>>,
    pub(crate) witness: Option<Vec<f64>>,
    pub(crate) witness_point: Option<Vec<f64>>,
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
#[derive(Debug)]
struct MultiplierCache {
    input: Vec<f64>,
    values: Vec<f64>,
}
#[derive(Clone, Debug)]
struct Basis {
    factor: Arc<Factor>,
    updates: Vec<Arc<Eta>>,
    multiplier_cache: Option<Arc<MultiplierCache>>,
}
impl Basis {
    // The basis owns the cache; every update/refactor invalidates it, and siblings
    // share only immutable snapshots. RHS and variable bounds do not enter B^-T c_B.
    fn multipliers(
        &mut self,
        input: impl ExactSizeIterator<Item = f64> + Clone,
    ) -> (Arc<MultiplierCache>, bool) {
        if let Some(cache) = &self.multiplier_cache {
            if cache.input.len() == input.len()
                && cache
                    .input
                    .iter()
                    .zip(input.clone())
                    .all(|(a, b)| a.to_bits() == b.to_bits())
            {
                return (cache.clone(), true);
            }
        }
        let input: Vec<_> = input.collect();
        let values = self.backward(&input);
        let cache = Arc::new(MultiplierCache { input, values });
        self.multiplier_cache = Some(cache.clone());
        (cache, false)
    }

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
        self.multiplier_cache = None;
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
        a.costs.len() == p.costs.len()
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
    witness: Option<Vec<f64>>,
    witness_point: Option<Vec<f64>>,
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
            .map_err(|_| numerical())?;
        let lu = matrix.sp_lu().map_err(|_| numerical())?;
        self.factor = Basis {
            factor: Arc::new(Factor::Sparse(lu)),
            updates: vec![],
            multiplier_cache: None,
        };
        self.stats.refactors += 1;
        self.recompute()
    }
    fn ratio_diagnostic(&self, d: &[f64], direction: f64, entering: usize) {
        let trace = std::env::var_os("MIP_LP_TRACE").is_some();
        let dump = std::env::var_os("MIP_LP_RATIO_DUMP");
        if !trace && dump.is_none() {
            return;
        }
        let bad:Vec<_>=self.basis.iter().enumerate().filter_map(|(i,&j)|{
            let a=d[i]*direction;
            let available=if a>PIVOT{self.x[j]}else if a < -PIVOT{self.upper[j]-self.x[j]}else{return None};
            (available < -FEAS).then(||serde_json::json!({"row":i,"variable":j,"value":self.x[j],"upper":self.upper[j].is_finite().then_some(self.upper[j]),"direction_coefficient":a,"available":available}))
        }).collect();
        if trace {
            eprintln!(
                "LP fresh-factor ratio failure {}",
                serde_json::json!({"entering":entering,"bad":bad})
            );
        }
        if let Some(mut path) = dump {
            static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            path.push(format!(
                ".{}.{}.json",
                std::process::id(),
                SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            if let Ok(file) = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
            {
                let _ = serde_json::to_writer(
                    file,
                    &serde_json::json!({"columns":self.columns.as_ref(),"rhs":self.rhs,"upper":self.upper.iter().map(|v|v.is_finite().then_some(*v)).collect::<Vec<_>>(),"basis":self.basis,"at_upper":self.at_upper,"point":self.x,"d":d,"direction":direction,"entering":entering,"bad":bad}),
                );
            }
        }
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
            return Err(numerical());
        }
        Ok(())
    }
    fn multipliers(&mut self, c: &[f64]) -> Arc<MultiplierCache> {
        let input = self.basis.iter().map(|&j| c[j]);
        let (cache, hit) = self.factor.multipliers(input);
        self.stats.multiplier_cache_hits += usize::from(hit);
        self.stats.multiplier_cache_misses += usize::from(!hit);
        cache
    }
    fn reduced_with_tolerance(&self, c: &[f64], y: &[f64], j: usize) -> (f64, f64) {
        let mut dot = 0.0;
        let mut scale = c[j].abs();
        for &(i, a) in &self.columns[j] {
            let product = a * y[i];
            dot += product;
            scale += product.abs();
        }
        (c[j] - dot, 1e-12 * scale)
    }
    fn reduced(&self, c: &[f64], y: &[f64], j: usize) -> f64 {
        self.reduced_with_tolerance(c, y, j).0
    }
    /// Validate primal and bounded-variable dual residuals independently of pricing.
    fn certificate(&mut self, c: &[f64]) -> Result<(f64, f64), Outcome> {
        let mut residual = self.rhs.clone();
        for j in 0..self.columns.len() {
            if !self.x[j].is_finite() || self.x[j] < -1e-7 || self.x[j] > self.upper[j] + 1e-7 {
                return Err(numerical());
            }
            for &(i, a) in &self.columns[j] {
                residual[i] -= a * self.x[j];
            }
        }
        if residual.iter().any(|r| !r.is_finite() || r.abs() > 1e-6) {
            return Err(numerical());
        }
        let pricing = self.multipliers(c);
        let y = &pricing.values;
        let mut dual_terms: Vec<_> = y.iter().zip(&self.rhs).map(|(&y, &b)| (y, b)).collect();
        for j in 0..self.columns.len() {
            let r = self.reduced(c, &y, j);
            if !r.is_finite() {
                return Err(numerical());
            }
            if self.upper[j].is_finite() {
                dual_terms.push((r.min(0.), self.upper[j]));
            } else if r < -1e-7 {
                return Err(numerical());
            }
        }
        let dual = crate::exact::dot_lower(dual_terms.into_iter()).ok_or_else(numerical)?;
        let primal = crate::exact::dot(c.iter().zip(&self.x).map(|(&c, &x)| (c, x)))
            .ok_or_else(numerical)?;
        if !primal.is_finite()
            || !dual.is_finite()
            || (primal - dual).abs() > 1e-6 * (1.0 + primal.abs())
        {
            return Err(numerical());
        }
        self.witness = Some(y.clone());
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
            let pricing = self.multipliers(c);
            let y = &pricing.values;
            if y.iter().any(|v| !v.is_finite()) {
                return Err(numerical());
            }
            let mut reduced = vec![0.0; c.len()];
            for j in 0..c.len() {
                if self.basic[j] || self.upper[j] == 0.0 {
                    continue;
                }
                let r = self.reduced(c, &y, j);
                let q = if self.at_upper[j] { -r } else { r };
                if !r.is_finite() || q < -1e-8 {
                    return Err(numerical());
                }
                reduced[j] = r;
            }
            let mut row = None;
            let mut largest = FEAS;
            for (i, &j) in self.basis.iter().enumerate() {
                let violation = (-self.x[j]).max(self.x[j] - self.upper[j]);
                if violation > FEAS
                    && (if degenerate > 0 {
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
                return Err(numerical());
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
                        return Err(numerical());
                    }
                    candidates.push((beta, j, direction, denom));
                }
            }
            candidates.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            let mut remaining = t * (target - self.x[leaving]);
            let mut flips = Vec::new();
            let mut enter = None;
            for (beta, j, direction, denom) in candidates {
                let coverage = denom * self.upper[j];
                if self.upper[j].is_finite() && coverage < remaining {
                    remaining -= coverage;
                    flips.push((j, direction));
                } else {
                    enter = Some((j, direction, beta));
                    break;
                }
            }
            let Some((j, direction, beta)) = enter else {
                return Err(if self.impossible_row(&w) {
                    self.witness = Some(w);
                    Outcome::Infeasible
                } else {
                    numerical()
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
                return Err(numerical());
            }
            let theta = remaining.max(0.0) / denom;
            if !theta.is_finite() || theta > self.upper[j] + 1e-7 {
                return Err(numerical());
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
            // dual退化は主変数の移動量ではなく双対ステップで判断する。
            if beta < 1e-10 {
                degenerate += 1;
            } else {
                degenerate = 0;
            }
            if self.stats.dual_pivots > 64.max(4 * self.rhs.len()) || degenerate > 500 {
                return Err(numerical());
            }
            if self.factor.updates.len() >= 50 {
                self.refactor()?;
            }
        }
    }
    fn run(&mut self, c: &[f64]) -> Result<(f64, f64), Outcome> {
        let minimum_cost = c
            .iter()
            .map(|v| v.abs())
            .filter(|v| *v > 0.)
            .fold(f64::INFINITY, f64::min);
        let noise_floor = if minimum_cost.is_finite() {
            1e-10 * minimum_cost
        } else {
            0.
        };
        let mut degenerate = 0usize;
        let mut verified = false;
        'iterations: loop {
            if self.deadline.is_some_and(|d| Instant::now() >= d) {
                return Err(Outcome::TimeLimit);
            }
            if self.stats.pivots + self.stats.flips >= self.limit {
                return Err(Outcome::IterationLimit);
            }
            let pricing = self.multipliers(c);
            let y = &pricing.values;
            if y.iter().any(|v| !v.is_finite()) {
                return Err(numerical());
            }
            let mut entering = None;
            let mut most_negative = 0.0;
            for j in 0..c.len() {
                if self.basic[j] || self.upper[j] == 0.0 {
                    continue;
                }
                let direction = if self.at_upper[j] { -1.0 } else { 1.0 };
                let (raw, tolerance) = self.reduced_with_tolerance(c, &y, j);
                let reduced = direction * raw;
                if reduced < -tolerance.max(noise_floor) && reduced < most_negative {
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
                return Err(numerical());
            }
            let (theta, leaving) = match primal_ratio(
                &self.basis,
                &self.x,
                &self.upper,
                &d,
                direction,
                self.upper[j],
            ) {
                Ok(step) => step,
                Err(()) => {
                    if self.factor.updates.is_empty() {
                        self.ratio_diagnostic(&d, direction, j);
                        return Err(numerical());
                    }
                    self.refactor()?;
                    continue 'iterations;
                }
            };
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
                        return Err(numerical());
                    }
                    for &(i, a) in &self.columns[k] {
                        activity[i] += a * ray[k];
                    }
                }
                let slope: f64 = c.iter().zip(&ray).map(|(c, r)| c * r).sum();
                let slope_scale: f64 = c.iter().zip(&ray).map(|(c, r)| (c * r).abs()).sum();
                if activity.iter().any(|v| !v.is_finite() || v.abs() > 1e-7)
                    || !slope.is_finite()
                    || slope >= -1e-12 * slope_scale
                {
                    return Err(numerical());
                }
                self.witness = Some(ray[..self.original].to_vec());
                self.witness_point = Some(self.x[..self.original].to_vec());
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
    crate::scaling::solve(p, limit, deadline, None, false)
}
pub(crate) fn valid(p: &Problem) -> bool {
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
fn warm_impl(
    p: &Problem,
    limit: usize,
    deadline: Option<Instant>,
    warm: Option<&WarmStart>,
) -> Answer {
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
        witness: None,
        witness_point: None,
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
    let cost_changed = a.costs != p.costs;
    s.stats.warm_cost_attempts = usize::from(cost_changed);
    let outcome = s.recompute().and_then(|_| {
        if cost_changed {
            // 新目的ではdual feasibleを仮定できない。元の基底がprimal feasibleな場合だけ使う。
            if s.x
                .iter()
                .zip(&s.upper)
                .any(|(&x, &u)| x < -FEAS || x > u + FEAS)
            {
                return Err(numerical());
            }
            s.run(&c)
        } else {
            s.dual_run(&c)
        }
    });
    match outcome {
        Ok((objective, dual)) => {
            s.stats.warm_successes = 1;
            s.stats.warm_cost_successes = usize::from(cost_changed);
            Answer {
                outcome: Outcome::Optimal {
                    objective,
                    x: s.x[..n].to_vec(),
                },
                dual: Some(dual),
                warm: Some(snapshot(
                    p,
                    &s,
                    if cost_changed { None } else { Some(a.clone()) },
                )),
                stats: s.stats,
                witness: s.witness,
                witness_point: s.witness_point,
            }
        }
        Err(Outcome::Infeasible) => {
            s.stats.warm_successes = 1;
            Answer {
                outcome: Outcome::Infeasible,
                dual: None,
                warm: None,
                stats: s.stats,
                witness: s.witness,
                witness_point: s.witness_point,
            }
        }
        Err(outcome @ (Outcome::TimeLimit | Outcome::IterationLimit)) => Answer {
            outcome,
            dual: None,
            warm: None,
            stats: s.stats,
            witness: s.witness,
            witness_point: s.witness_point,
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
        self.warm_cost_attempts += other.warm_cost_attempts;
        self.warm_cost_successes += other.warm_cost_successes;
        self.dual_pivots += other.dual_pivots;
        self.dual_flips += other.dual_flips;
        self.scaled_columns += other.scaled_columns;
        self.scaled_rows += other.scaled_rows;
        self.multiplier_cache_hits += other.multiplier_cache_hits;
        self.multiplier_cache_misses += other.multiplier_cache_misses;
        self.certificate_repair_trials += other.certificate_repair_trials;
        self.certificate_repairs += other.certificate_repairs;
    }
}
fn cold(p: &Problem, limit: usize, deadline: Option<Instant>, retain: bool) -> Answer {
    let m = p.rhs.len();
    let n = p.costs.len();
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
            multiplier_cache: None,
        },
        stats: Stats::default(),
        limit,
        deadline,
        artificial: vec![],
        original: n,
        witness: None,
        witness_point: None,
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
    s.factor.multiplier_cache = None;
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
                    numerical()
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
            witness: s.witness,
            witness_point: s.witness_point,
        },
        Err(outcome) => Answer {
            warm: None,
            outcome,
            dual: None,
            stats: s.stats,
            witness: s.witness,
            witness_point: s.witness_point,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_multipliers_preserve_bits_cost_keys_updates_and_siblings() {
        let mut parent = Basis {
            factor: Arc::new(Factor::Diagonal(vec![2., 3.])),
            updates: vec![],
            multiplier_cache: None,
        };
        let input = vec![2., 6.];
        let (first, hit) = parent.multipliers(input.iter().copied());
        assert!(!hit);
        assert_eq!(first.values, parent.backward(&input));
        let (same, hit) = parent.multipliers(input.iter().copied());
        assert!(hit);
        assert!(Arc::ptr_eq(&first, &same));
        let mut child = parent.clone();
        child.update(0, &[1., 1.]);
        let (changed, hit) = child.multipliers(input.iter().copied());
        assert!(!hit);
        assert_eq!(changed.values, child.backward(&input));
        assert!(!Arc::ptr_eq(&first, &changed));
        let (unchanged, hit) = parent.multipliers(input.iter().copied());
        assert!(hit);
        assert!(Arc::ptr_eq(&first, &unchanged));
        let (_, hit) = parent.multipliers(vec![4., 6.].into_iter());
        assert!(!hit);
        let (minus, hit) = parent.multipliers(vec![-0., 0.].into_iter());
        assert!(!hit);
        let (plus, hit) = parent.multipliers(vec![0., 0.].into_iter());
        assert!(!hit);
        assert!(!Arc::ptr_eq(&minus, &plus));
        assert_eq!(minus.values[0].to_bits(), (-0f64).to_bits());
        assert_eq!(plus.values[0].to_bits(), 0f64.to_bits());
    }
    #[test]
    fn eta_forward_and_transpose_agree_with_fresh_lu() {
        let mut basis = Basis {
            factor: Arc::new(Factor::Diagonal(vec![1.; 3])),
            updates: vec![],
            multiplier_cache: None,
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

/// Harrisの二段階比率検定。許容誤差内の候補から大きいpivotを選ぶ。
fn primal_ratio(
    basis: &[usize],
    x: &[f64],
    upper: &[f64],
    d: &[f64],
    direction: f64,
    width: f64,
) -> Result<(f64, Option<usize>), ()> {
    let step = primal_ratio_threshold(basis, x, upper, d, direction, width, PIVOT)?;
    if step.0.is_infinite() {
        primal_ratio_threshold(basis, x, upper, d, direction, width, 0.)
    } else {
        Ok(step)
    }
}
fn primal_ratio_threshold(
    basis: &[usize],
    x: &[f64],
    upper: &[f64],
    d: &[f64],
    direction: f64,
    width: f64,
    pivot_threshold: f64,
) -> Result<(f64, Option<usize>), ()> {
    let mut relaxed = width;
    for (i, &k) in basis.iter().enumerate() {
        let a = d[i] * direction;
        let available = if a > pivot_threshold {
            x[k]
        } else if a < -pivot_threshold {
            upper[k] - x[k]
        } else {
            continue;
        };
        if available < -FEAS {
            return Err(());
        }
        relaxed = relaxed.min((available.max(0.) + FEAS) / a.abs());
    }
    let mut theta = width;
    let mut leaving = None;
    let mut largest = 0.;
    for (i, &k) in basis.iter().enumerate() {
        let a = d[i] * direction;
        let available = if a > pivot_threshold {
            x[k]
        } else if a < -pivot_threshold {
            upper[k] - x[k]
        } else {
            continue;
        };
        let ratio = available.max(0.) / a.abs();
        if ratio <= relaxed
            && (a.abs() > largest || (a.abs() == largest && leaving.is_none_or(|r| k < basis[r])))
        {
            theta = ratio;
            largest = a.abs();
            leaving = Some(i);
        }
    }
    Ok((theta, leaving))
}

#[track_caller]
fn numerical() -> Outcome {
    if std::env::var_os("MIP_LP_TRACE").is_some() {
        eprintln!("LP numerical failure at {}", std::panic::Location::caller());
    }
    Outcome::NumericalError
}

#[cfg(test)]
mod ratio_regressions {
    use super::*;
    #[test]
    fn tiny_bound_roundoff_is_not_amplified_by_small_pivot() {
        let step = primal_ratio(&[0], &[-1.1e-13], &[1.], &[4.15e-6], 1., f64::INFINITY);
        assert_eq!(step, Ok((0., Some(0))));
        assert!(primal_ratio(&[0], &[-1e-5], &[1.], &[4.15e-6], 1., f64::INFINITY).is_err());
    }
    #[test]
    fn near_tie_chooses_stable_pivot_and_preserves_tolerance() {
        let x = [1e-8, 1. + 1e-10];
        let d = [1e-8, 1.];
        let (theta, row) =
            primal_ratio(&[0, 1], &x, &[f64::INFINITY; 2], &d, 1., f64::INFINITY).unwrap();
        assert_eq!(row, Some(1));
        assert_eq!(theta, x[1]);
        for i in 0..2 {
            assert!(x[i] - theta * d[i] >= -FEAS);
        }
    }
    #[test]
    fn entering_width_and_upper_bound_are_respected() {
        assert_eq!(
            primal_ratio(&[0], &[2.], &[10.], &[1.], 1., 1.),
            Ok((1., None))
        );
        assert_eq!(
            primal_ratio(&[0], &[2.], &[3.], &[-2.], 1., 10.),
            Ok((0.5, Some(0)))
        );
    }
    #[test]
    fn stable_pivot_ties_use_basic_variable_order() {
        assert_eq!(
            primal_ratio(&[1, 0], &[1., 1.], &[2., 2.], &[1., 1.], 1., 10.),
            Ok((1., Some(1)))
        );
    }
}

pub fn minimize_warm(
    p: &Problem,
    limit: usize,
    deadline: Option<Instant>,
    warm: Option<&WarmStart>,
) -> Answer {
    crate::scaling::solve(p, limit, deadline, warm, true)
}
pub(crate) fn raw(
    p: &Problem,
    limit: usize,
    deadline: Option<Instant>,
    warm: Option<&WarmStart>,
    retain: bool,
) -> Answer {
    if retain {
        warm_impl(p, limit, deadline, warm)
    } else {
        cold(p, limit, deadline, false)
    }
}

#[cfg(test)]
mod saved_ratio_diagnostic {
    use super::*;
    #[test]
    #[ignore = "saved bases via MIP_RATIO_DUMP_DIR"]
    fn replay_saved_basis_residual_refinement() {
        for entry in std::fs::read_dir(std::env::var("MIP_RATIO_DUMP_DIR").unwrap()).unwrap() {
            let path = entry.unwrap().path();
            let saved: serde_json::Value =
                serde_json::from_reader(std::fs::File::open(&path).unwrap()).unwrap();
            let columns: Vec<Vec<(usize, f64)>> =
                serde_json::from_value(saved["columns"].clone()).unwrap();
            let rhs: Vec<f64> = serde_json::from_value(saved["rhs"].clone()).unwrap();
            let basis: Vec<usize> = serde_json::from_value(saved["basis"].clone()).unwrap();
            let mut x: Vec<f64> = serde_json::from_value(saved["point"].clone()).unwrap();
            let upper: Vec<Option<f64>> = serde_json::from_value(saved["upper"].clone()).unwrap();
            let upper: Vec<f64> = upper
                .into_iter()
                .map(|v| v.unwrap_or(f64::INFINITY))
                .collect();
            let d: Vec<f64> = serde_json::from_value(saved["d"].clone()).unwrap();
            let direction = saved["direction"].as_f64().unwrap();
            let entering = saved["entering"].as_u64().unwrap() as usize;
            let triples: Vec<_> = basis
                .iter()
                .enumerate()
                .flat_map(|(k, &j)| columns[j].iter().map(move |&(i, a)| Triplet::new(i, k, a)))
                .collect();
            let matrix =
                SparseColMat::try_new_from_triplets(rhs.len(), rhs.len(), &triples).unwrap();
            let lu = matrix.sp_lu().unwrap();
            let factor = Basis {
                factor: Arc::new(Factor::Sparse(lu)),
                updates: vec![],
                multiplier_cache: None,
            };
            let mut rows = vec![Vec::new(); rhs.len()];
            for (j, col) in columns.iter().enumerate() {
                for &(i, a) in col {
                    rows[i].push((j, a));
                }
            }
            for iteration in 0..=4 {
                let residual: Vec<_> = rows
                    .iter()
                    .enumerate()
                    .map(|(i, row)| {
                        crate::exact::dot(
                            std::iter::once((rhs[i], 1.))
                                .chain(row.iter().map(|&(j, a)| (-a, x[j]))),
                        )
                        .unwrap()
                    })
                    .collect();
                let norm = residual.iter().copied().map(f64::abs).fold(0., f64::max);
                let backward = rows
                    .iter()
                    .enumerate()
                    .map(|(i, row)| {
                        let scale = crate::exact::dot(
                            std::iter::once((rhs[i].abs(), 1.))
                                .chain(row.iter().map(|&(j, a)| (a.abs(), x[j].abs()))),
                        )
                        .unwrap_or(f64::INFINITY);
                        if residual[i] == 0. {
                            0.
                        } else {
                            residual[i].abs() / scale
                        }
                    })
                    .fold(0., f64::max);

                let violation = basis
                    .iter()
                    .map(|&j| (-x[j]).max(x[j] - upper[j]).max(0.))
                    .fold(0., f64::max);
                println!(
                    "{}",
                    serde_json::json!({"file":path.file_name().unwrap().to_string_lossy(),"iteration":iteration,"residual_inf":norm,"backward_error":backward,"basic_violation":violation,"ratio_ok":primal_ratio(&basis,&x,&upper,&d,direction,upper[entering]).is_ok()})
                );
                if iteration == 4 {
                    break;
                }
                let correction = factor.forward(&residual);
                for (i, &j) in basis.iter().enumerate() {
                    x[j] += correction[i];
                }
            }
        }
    }
}
