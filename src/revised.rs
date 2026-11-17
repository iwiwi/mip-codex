//! 有界変数の改訂単体法。最適化の処理は本リポジトリのRustで実装。
//! faerは汎用の疎LU分解と線形方程式の解法だけに使用する。
use crate::lp::Outcome;
#[cfg(test)]
use faer::Col;
use faer::{
    MatMut,
    linalg::solvers::Solve,
    sparse::{SparseColMat, Triplet, linalg::solvers::Lu},
};
use std::{sync::Arc, time::Instant};
const FEAS: f64 = 1e-8;
const PIVOT: f64 = 1e-10;
// Static L1 ranking is restricted to Phase I of zero-cost LPs.
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
    pub zero_phase_one_skips:usize,
    pub pivots: usize,
    pub feasibility_pricing_calls: usize,
    pub pricing_cutoffs: usize,
    pub empty_pricing_calls:usize,
    pub empty_pricing_skips:usize,
    pub pricing_fallbacks: usize,
    pub flips: usize,
    pub refactors: usize,
    pub warm_attempts: usize,
    pub warm_successes: usize,
    pub warm_fallbacks: usize,
    pub warm_rejections: usize,
    pub warm_row_extensions: usize,
    pub warm_cost_attempts: usize,
    pub warm_cost_successes: usize,
    pub dual_pivots: usize,
    pub dual_flips: usize,
    pub dual_refactor_retries: usize,
    pub singleton_crash_columns: usize,
    pub dual_form_attempts: usize,
    pub dual_form_successes: usize,
    pub dual_form_fallbacks: usize,
    pub scaled_columns: usize,
    pub scaled_rows: usize,
    pub multiplier_cache_hits: usize,
    pub multiplier_cache_misses: usize,
    pub certificate_repair_trials: usize,
    pub certificate_repairs: usize,
    pub restoration_attempts: usize,
    pub restoration_restarts: usize,
    pub restoration_extended_attempts: usize,
    pub restoration_successes: usize,
    pub restoration_infeasible_candidates: usize,
    pub restoration_pivots: usize,
    pub bound_point_trials: usize,
    pub bound_point_repairs: usize,
    pub refinement_attempts: usize,
    pub refinement_steps: usize,
    pub refinement_successes: usize,
}
#[derive(Clone, Debug)]
pub struct Answer {
    /// Checked by scaling in the original LP units; never an optimality certificate.
    pub limited_point: Option<Vec<f64>>,
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
                // A Vec-backed matrix retains column stride n, unlike a Col view.
                // Solve in the final output buffer, avoiding a second allocation/copy.
                let mut x = v.to_vec();
                lu.solve_in_place(MatMut::from_column_major_slice_mut(&mut x, v.len(), 1));
                x
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
                lu.solve_transpose_in_place(MatMut::from_column_major_slice_mut(
                    &mut x,
                    v.len(),
                    1,
                ));
                x
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
    // Auxiliary columns may be interleaved with artificial columns. Append new
    // slacks at the end so every old internal index remains unchanged.
    fn extend_rows(&self, p: &Problem, deadline: Option<Instant>) -> Option<Self> {
        let a = &self.structure;
        let old_m = a.equality.len();
        let m = p.rhs.len();
        if deadline.is_some_and(|d| Instant::now() >= d)
            || !valid(p)
            || m <= old_m
            || a.costs != p.costs
            || a.equality != p.equality[..old_m]
            || a.slack_upper != p.slack_upper[..old_m]
            || p.equality[old_m..].iter().any(|&v| v)
        {
            return None;
        }
        for (old, new) in a.columns.iter().zip(&p.columns) {
            if !new.starts_with(old) || new[old.len()..].iter().any(|&(i, _)| i < old_m) {
                return None;
            }
        }
        let n = p.costs.len();
        let mut columns = p.columns.clone();
        columns.extend_from_slice(&a.columns[n..]);
        let mut extra_upper = a.extra_upper.clone();
        let mut basis = self.basis.clone();
        let mut at_upper = self.at_upper.clone();
        for i in old_m..m {
            basis.push(columns.len());
            columns.push(vec![(i, 1.)]);
            extra_upper.push(p.slack_upper[i]);
            at_upper.push(false);
        }
        let triplets: Vec<_> = basis
            .iter()
            .enumerate()
            .flat_map(|(k, &j)| columns[j].iter().map(move |&(i, v)| Triplet::new(i, k, v)))
            .collect();
        let matrix = SparseColMat::try_new_from_triplets(m, m, &triplets).ok()?;
        let lu = matrix.sp_lu().ok()?;
        if deadline.is_some_and(|d| Instant::now() >= d) {
            return None;
        }
        Some(Self {
            structure: Arc::new(Structure {
                columns: Arc::new(columns),
                costs: p.costs.clone(),
                equality: p.equality.clone(),
                slack_upper: p.slack_upper.clone(),
                extra_upper,
                artificial: a.artificial.clone(),
            }),
            basis,
            at_upper,
            factor: Basis {
                factor: Arc::new(Factor::Sparse(lu)),
                updates: vec![],
                multiplier_cache: None,
            },
        })
    }
    pub(crate) fn tableau_rows_raw(&self, p: &Problem, columns: &[usize], deadline: Instant)
        -> Vec<(usize, Vec<f64>)>
    {
        if !self.compatible(p) { return vec![]; }
        let mut result=Vec::new();
        for &column in columns.iter().take(16) {
            if Instant::now()>=deadline {break;}
            if column>=p.columns.len() {continue;}
            let Some(row)=self.basis.iter().position(|&j|j==column) else {continue;};
            let mut unit=vec![0.;p.rhs.len()];unit[row]=1.;
            let weights=self.factor.backward(&unit);
            if weights.iter().all(|v|v.is_finite()) {result.push((column,weights));}
        }
        result
    }
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
#[derive(Clone)]
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
    fn restore_primal(&mut self, since: usize) -> Result<(), Outcome> {
        if self.stats.restoration_attempts.saturating_sub(since) >= 4 {
            return Err(numerical());
        }
        if self.deadline.is_some_and(|d| Instant::now() >= d) {
            return Err(Outcome::TimeLimit);
        }
        self.stats.restoration_extended_attempts +=
            usize::from(self.stats.restoration_attempts >= 4);
        self.stats.restoration_attempts += 1;
        // Zero costs make every basis dual feasible. Existing bounded dual
        // simplex can then repair the basis without changing the LP bounds.
        let zero = vec![0.; self.columns.len()];
        let before = self.stats.dual_pivots;
        let backup = self.clone();
        let mut result = self.dual_run::<true>(&zero);
        if matches!(result, Err(Outcome::NumericalError))
            && self.deadline.is_none_or(|d| Instant::now() < d)
            && self.stats.pivots + self.stats.flips < self.limit
        {
            let spent = std::mem::take(&mut self.stats);
            *self = backup;
            self.stats = spent;
            self.stats.restoration_restarts += 1;
            result = self.dual_run_mode::<true, true>(&zero);
        }
        self.stats.restoration_pivots += self.stats.dual_pivots - before;
        match result {
            Ok(_)
                if self
                    .x
                    .iter()
                    .zip(&self.upper)
                    .all(|(&x, &u)| x.is_finite() && x >= -FEAS && x <= u + FEAS) =>
            {
                self.stats.restoration_successes += 1;
                Ok(())
            }
            Err(Outcome::Infeasible) if self.artificial.iter().all(|&j| self.upper[j] == 0.) => {
                // This is only a candidate: the ordinary original-Problem and
                // original-MPS contradiction checks must still accept it.
                self.stats.restoration_infeasible_candidates += 1;
                Err(Outcome::Infeasible)
            }
            Err(e @ (Outcome::TimeLimit | Outcome::IterationLimit)) => Err(e),
            _ => Err(numerical()),
        }
    }
    fn refine_ratio(
        &mut self,
        d: &[f64],
        direction: f64,
        entering: usize,
    ) -> Option<(f64, Option<usize>)> {
        self.stats.refinement_attempts += 1;
        if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return None;
        }
        let mut rows = vec![Vec::new(); self.rhs.len()];
        for (j, col) in self.columns.iter().enumerate() {
            if j & 255 == 0
                && self
                    .deadline
                    .is_some_and(|deadline| Instant::now() >= deadline)
            {
                return None;
            }
            for &(i, a) in col {
                rows[i].push((j, a));
            }
        }
        let evaluate = |point: &[f64]| -> Option<(Vec<f64>, f64)> {
            let mut residual = Vec::with_capacity(rows.len());
            let mut score = 0f64;
            for (i, row) in rows.iter().enumerate() {
                if i & 63 == 0
                    && self
                        .deadline
                        .is_some_and(|deadline| Instant::now() >= deadline)
                {
                    return None;
                }
                let r = crate::exact::dot(
                    std::iter::once((self.rhs[i], 1.))
                        .chain(row.iter().map(|&(j, a)| (-a, point[j]))),
                )?;
                let scale = crate::exact::dot(
                    std::iter::once((self.rhs[i].abs(), 1.))
                        .chain(row.iter().map(|&(j, a)| (a.abs(), point[j].abs()))),
                )?;
                score = score.max(r.abs() / (1. + scale));
                residual.push(r);
            }
            Some((residual, score))
        };
        let (mut residual, old_score) = evaluate(&self.x)?;
        let mut point = self.x.clone();
        for _ in 0..3 {
            if self
                .deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                return None;
            }
            let correction = self.factor.forward(&residual);
            self.stats.refinement_steps += 1;
            for (i, &j) in self.basis.iter().enumerate() {
                point[j] += correction[i];
            }
            let (next, score) = evaluate(&point)?;
            if score <= old_score
                && self
                    .basis
                    .iter()
                    .all(|&j| point[j] >= -FEAS && point[j] <= self.upper[j] + FEAS)
            {
                if let Ok(step) = primal_ratio(
                    &self.basis,
                    &point,
                    &self.upper,
                    d,
                    direction,
                    self.upper[entering],
                ) {
                    self.x = point;
                    self.stats.refinement_successes += 1;
                    return Some(step);
                }
            }
            residual = next;
        }
        None
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
    fn dual_run<const RESTORATION: bool>(&mut self, c: &[f64]) -> Result<(f64, f64), Outcome> {
        self.dual_run_retry::<RESTORATION>(c,
            std::env::var("MIP_DUAL_REFACTOR_RETRY").map_or(true, |v| v == "1"))
    }
    fn dual_run_retry<const RESTORATION: bool>(
        &mut self, c: &[f64], retry: bool,
    ) -> Result<(f64, f64), Outcome> {
        let result = self.dual_run_mode::<RESTORATION, false>(c);
        if retry && matches!(result, Err(Outcome::NumericalError))
            && !self.factor.updates.is_empty()
            && self.deadline.is_none_or(|d| Instant::now() < d)
            && self.stats.pivots + self.stats.flips < self.limit
        {
            self.stats.dual_refactor_retries += 1;
            self.refactor()?;
            self.witness = None;
            self.witness_point = None;
            self.dual_run_mode::<RESTORATION, false>(c)
        } else { result }
    }
    fn dual_run_mode<const RESTORATION: bool, const STABLE: bool>(
        &mut self,
        c: &[f64],
    ) -> Result<(f64, f64), Outcome> {
        let mut degenerate = 0usize;
        let initial_dual_pivots = self.stats.dual_pivots;
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
            candidates.sort_unstable_by(|a, b| {
                a.0.total_cmp(&b.0)
                    .then_with(|| {
                        if STABLE {
                            b.3.total_cmp(&a.3)
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    })
                    .then(a.1.cmp(&b.1))
            });
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
                if std::env::var_os("MIP_LP_TRACE").is_some() {
                    eprintln!(
                        "dual ratio failure restoration={RESTORATION} denom={denom:e} remaining={remaining:e} t={t} direction={direction} row={r} entering={j} flips={} updates={}",
                        flips.len(),
                        self.factor.updates.len()
                    );
                }
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
            let dual_pivots = if STABLE {
                self.stats.dual_pivots - initial_dual_pivots
            } else {
                self.stats.dual_pivots
            };
            if dual_pivots > 64.max(4 * self.rhs.len()) || (!RESTORATION && degenerate > 500) {
                return Err(numerical());
            }
            if self.factor.updates.len() >= 50 {
                self.refactor()?;
            }
        }
    }
    fn run(&mut self, c: &[f64]) -> Result<(f64, f64), Outcome> {
        self.run_priced::<false>(c)
    }
    fn run_priced<const NORMALIZED: bool>(&mut self, c: &[f64]) -> Result<(f64, f64), Outcome> {
        let enabled=c.len()>=1024&&std::env::var("MIP_LP_EMPTY_PRICING").map_or(true,|v|v=="1");
        self.run_priced_mode::<NORMALIZED>(c,enabled)
    }
    fn run_priced_mode<const NORMALIZED:bool>(&mut self,c:&[f64],enabled:bool)->Result<(f64,f64),Outcome>{
        let eligible=pricing_columns(&self.columns,c,enabled);
        self.stats.empty_pricing_calls+=usize::from(eligible.is_some());
        let weights: Vec<f64> = if NORMALIZED {
            self.columns
                .iter()
                .map(|column| {
                    let weight = 1. + column.iter().map(|&(_, a)| a.abs()).sum::<f64>();
                    if weight.is_finite() && weight > 0. {
                        weight
                    } else {
                        1.
                    }
                })
                .collect()
        } else {
            vec![]
        };
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
        let pricing_budget = self.rhs.len().saturating_mul(8).max(100);
        let normalized_active = NORMALIZED && self.stats.pivots + self.stats.flips < pricing_budget;
        let mut degenerate = 0usize;
        let mut verified = false;
        let mut restoration_epoch = self.stats.restoration_attempts;
        let mut restoration_objective = f64::INFINITY;
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
            if normalized_active && self.stats.pivots + self.stats.flips >= pricing_budget {
                self.stats.pricing_cutoffs += 1;
                return Err(Outcome::IterationLimit);
            }
            let mut entering = None;
            let mut most_negative = 0.0;
            let mut visit = |j:usize| -> bool {
                if self.basic[j] || self.upper[j] == 0.0 {
                    return false;
                }
                let direction = if self.at_upper[j] { -1.0 } else { 1.0 };
                let (raw, tolerance) = self.reduced_with_tolerance(c, &y, j);
                let reduced = direction * raw;
                let score = if normalized_active {
                    reduced / weights[j]
                } else {
                    reduced
                };
                let better = if NORMALIZED {
                    entering.is_none() || score < most_negative
                } else {
                    reduced < most_negative
                };
                if reduced < -tolerance.max(noise_floor) && better {
                    // Keep the first eligible column even if division underflows.
                    most_negative = score;
                    entering = Some((j, direction));
                    if degenerate > 25 {
                        return true;
                    }
                }
                false
            };
            let skipped=if let Some(indices)=&eligible{
                let mut stopped=None;
                for (position,&j) in indices.iter().enumerate(){if visit(j){stopped=Some(j-position);break;}}
                stopped.unwrap_or(c.len()-indices.len())
            }else{for j in 0..c.len(){if visit(j){break;}}0};
            self.stats.empty_pricing_skips=self.stats.empty_pricing_skips.saturating_add(skipped);

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
                        if let Some(step) = self.refine_ratio(&d, direction, j) {
                            step
                        } else {
                            let objective =
                                crate::exact::dot(c.iter().copied().zip(self.x.iter().copied()))
                                    .ok_or_else(numerical)?;
                            if !objective.is_finite() {
                                return Err(numerical());
                            }
                            if !restoration_objective.is_finite()
                                || objective
                                    < restoration_objective
                                        - 1e-7 * (1. + restoration_objective.abs())
                            {
                                restoration_epoch = self.stats.restoration_attempts;
                                restoration_objective = objective;
                            }
                            if std::env::var_os("MIP_LP_TRACE").is_some() {
                                eprintln!(
                                    "restoration progress objective={objective:e} best={restoration_objective:e} epoch={restoration_epoch} attempts={}",
                                    self.stats.restoration_attempts
                                );
                            }
                            self.restore_primal(restoration_epoch)?;
                            continue 'iterations;
                        }
                    } else {
                        self.refactor()?;
                        continue 'iterations;
                    }
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
    let extended = if w.compatible(p) {
        None
    } else {
        w.extend_rows(p, deadline)
    };
    let w = extended.as_ref().unwrap_or(w);
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
    s.stats.warm_row_extensions = usize::from(extended.is_some());
    s.stats.refactors += usize::from(extended.is_some());
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
            s.dual_run::<false>(&c)
        }
    });
    let limited_point = if matches!(&outcome,Err(Outcome::TimeLimit|Outcome::IterationLimit)) && std::env::var("MIP_LP_LIMIT_POINT").map_or(true,|v|v=="1") {Some(s.x[..s.original].to_vec())}else{None};
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
                limited_point,
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
                limited_point,
            }
        }
        Err(outcome @ (Outcome::TimeLimit | Outcome::IterationLimit)) => Answer {
            outcome,
            dual: None,
            warm: None,
            stats: s.stats,
            witness: s.witness,
            witness_point: s.witness_point,
                limited_point,
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
fn pricing_columns(columns:&[Vec<(usize,f64)>],costs:&[f64],enabled:bool)->Option<Vec<usize>>{
 if !enabled||costs.len()<1024{return None;}
 let skipped=costs.iter().enumerate().filter(|&(j,c)|*c==0.&&columns[j].is_empty()).count();
 if skipped<costs.len()/2{return None;}
 Some((0..costs.len()).filter(|&j|costs[j]!=0.||!columns[j].is_empty()).collect())
}
impl Stats {
    pub fn add(&mut self, other: &Self) {
        self.pivots += other.pivots;
        self.feasibility_pricing_calls += other.feasibility_pricing_calls;
        self.pricing_cutoffs += other.pricing_cutoffs;
        self.zero_phase_one_skips+=other.zero_phase_one_skips;
        self.empty_pricing_calls+=other.empty_pricing_calls;
        self.empty_pricing_skips+=other.empty_pricing_skips;
        self.pricing_fallbacks += other.pricing_fallbacks;
        self.flips += other.flips;
        self.refactors += other.refactors;
        self.warm_attempts += other.warm_attempts;
        self.warm_successes += other.warm_successes;
        self.warm_fallbacks += other.warm_fallbacks;
        self.warm_rejections += other.warm_rejections;
        self.warm_row_extensions += other.warm_row_extensions;
        self.warm_cost_attempts += other.warm_cost_attempts;
        self.warm_cost_successes += other.warm_cost_successes;
        self.dual_pivots += other.dual_pivots;
        self.dual_flips += other.dual_flips;
        self.dual_refactor_retries += other.dual_refactor_retries;
        self.singleton_crash_columns += other.singleton_crash_columns;
        self.dual_form_attempts += other.dual_form_attempts;
        self.dual_form_successes += other.dual_form_successes;
        self.dual_form_fallbacks += other.dual_form_fallbacks;
        self.scaled_columns += other.scaled_columns;
        self.scaled_rows += other.scaled_rows;
        self.multiplier_cache_hits += other.multiplier_cache_hits;
        self.multiplier_cache_misses += other.multiplier_cache_misses;
        self.certificate_repair_trials += other.certificate_repair_trials;
        self.certificate_repairs += other.certificate_repairs;
        self.restoration_attempts += other.restoration_attempts;
        self.restoration_restarts += other.restoration_restarts;
        self.restoration_extended_attempts += other.restoration_extended_attempts;
        self.restoration_successes += other.restoration_successes;
        self.restoration_infeasible_candidates += other.restoration_infeasible_candidates;
        self.restoration_pivots += other.restoration_pivots;
        self.bound_point_trials += other.bound_point_trials;
        self.bound_point_repairs += other.bound_point_repairs;
        self.refinement_attempts += other.refinement_attempts;
        self.refinement_steps += other.refinement_steps;
        self.refinement_successes += other.refinement_successes;
    }
}
fn cold(p: &Problem, limit: usize, deadline: Option<Instant>, retain: bool) -> Answer {
    cold_with_probe(
        p,
        limit,
        deadline,
        retain,
        std::time::Duration::from_secs(1),
    )
}
fn cold_with_probe(
    p: &Problem,
    limit: usize,
    deadline: Option<Instant>,
    retain: bool,
    probe: std::time::Duration,
) -> Answer {
    if !p.costs.iter().all(|&c| c == 0.) {
        return cold_priced(p, limit, deadline, retain, false);
    }
    let probe_deadline = Instant::now() + probe;
    let first_deadline = Some(deadline.map_or(probe_deadline, |d| d.min(probe_deadline)));
    let first = cold_priced(p, limit, first_deadline, retain, false);
    if matches!(first.outcome, Outcome::TimeLimit)
        && deadline.is_none_or(|d| probe_deadline < d && Instant::now() < d)
    {
        let remaining = limit.saturating_sub(first.stats.pivots + first.stats.flips);
        if remaining > 0 {
            let mut retry = cold_priced(p, remaining, deadline, retain, true);
            retry.stats.add(&first.stats);
            return retry;
        }
    }
    first
}
fn cold_priced(
    p: &Problem,
    limit: usize,
    deadline: Option<Instant>,
    retain: bool,
    normalized: bool,
) -> Answer {
    cold_crashed(p,limit,deadline,retain,normalized,std::env::var("MIP_LP_SINGLETON_CRASH").map_or(true,|v|v=="1"))
}
fn cold_crashed(p:&Problem,limit:usize,deadline:Option<Instant>,retain:bool,normalized:bool,crash:bool)->Answer {
    cold_initialized(p,limit,deadline,retain,normalized,crash,std::env::var("MIP_LP_ZERO_PHASE_ONE").map_or(true,|v|v=="1"))
}
fn cold_initialized(p:&Problem,limit:usize,deadline:Option<Instant>,retain:bool,normalized:bool,crash:bool,skip_zero:bool)->Answer {
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
    let mut singleton=if crash {vec![[None,None];m]}else{vec![]};
    if crash {
        for (j,column) in p.columns.iter().enumerate() {
            if j%1024==0 && deadline.is_some_and(|d|Instant::now()>=d) {break;}
            if p.costs[j]!=0. || column.len()!=1 {continue;}
            let (row,a)=column[0];let magnitude=a.abs();
            if !(2f64.powi(-30)..=2f64.powi(30)).contains(&magnitude) || magnitude.to_bits()&((1u64<<52)-1)!=0 {continue;}
            let side=usize::from(a<0.);let old:Option<(usize,f64)>=singleton[row][side];
            if old.is_none_or(|(k,b)|magnitude*p.upper[j]>b.abs()*p.upper[k]) {singleton[row][side]=Some((j,a));}
        }
    }
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
        let natural=if crash {singleton[i].iter().flatten().find_map(|&(j,a)| {
            let x=residual/a;
            (x.is_finite() && x>=0. && x<=p.upper[j] && a*x==residual).then_some((j,a,x))
        })}else{None};
        let (j,value) = if residual >= 0.0 && p.rhs[i] <= p.slack_upper[i] && slack.is_some() {
            diagonal.push(1.0);
            (slack.unwrap(),residual.abs())
        } else if let Some((j,a,x))=natural {
            diagonal.push(a);s.stats.singleton_crash_columns+=1;(j,x)
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
            (j,residual.abs())
        };
        s.basis.push(j);
        s.basic[j] = true;
        s.x[j] = value;
    }
    s.factor.factor = Arc::new(Factor::Diagonal(diagonal));
    s.factor.multiplier_cache = None;
    let outcome = (|| {
        if s.stats.singleton_crash_columns>0 {s.recompute()?;}
        if !s.artificial.is_empty() {
            // Phase Iは非負人工変数の和。初期値が全て厳密0なら最小値0が既知。
            // 正の微小値を0とみなす許容幅は使わず、既存Phase II/証明へ進む。
            if skip_zero && s.artificial.iter().all(|&j|s.x[j]==0.) {
                s.stats.zero_phase_one_skips+=1;
            } else {
            s.stats.feasibility_pricing_calls += usize::from(normalized);
            let mut c = vec![0.0; s.columns.len()];
            for &j in &s.artificial {
                c[j] = 1.0;
            }
            let (value, dual) = if normalized {
                s.run_priced::<true>(&c)?
            } else {
                s.run_priced::<false>(&c)?
            };
            if value > 1e-7 {
                return Err(if dual > 1e-8 {
                    Outcome::Infeasible
                } else {
                    numerical()
                });
            }
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
    if normalized
        && (matches!(outcome, Err(Outcome::NumericalError))
            || (matches!(outcome, Err(Outcome::IterationLimit)) && s.stats.pricing_cutoffs > 0))
        && deadline.is_none_or(|d| Instant::now() < d)
    {
        let remaining = limit.saturating_sub(s.stats.pivots + s.stats.flips);
        let mut answer = cold_priced(p, remaining, deadline, retain, false);
        answer.stats.add(&s.stats);
        answer.stats.pricing_fallbacks += 1;
        return answer;
    }
    let limited_point = if matches!(&outcome,Err(Outcome::TimeLimit|Outcome::IterationLimit)) && std::env::var("MIP_LP_LIMIT_POINT").map_or(true,|v|v=="1") {Some(s.x[..s.original].to_vec())}else{None};
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
                limited_point,
        },
        Err(outcome) => Answer {
            warm: None,
            outcome,
            dual: None,
            stats: s.stats,
            witness: s.witness,
            witness_point: s.witness_point,
                limited_point,
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
mod refinement_tests {
    use super::*;
    fn refinement_example(true_negative: bool) -> Simplex {
        let tiny = 2f64.powi(-40);
        let columns = vec![
            vec![(0, 1.), (1, 1.)],
            vec![(0, 1.), (1, 1. + tiny)],
            vec![(0, 1.), (1, 1.)],
        ];
        let triples: Vec<_> = columns[..2]
            .iter()
            .enumerate()
            .flat_map(|(j, col)| col.iter().map(move |&(i, a)| Triplet::new(i, j, a)))
            .collect();
        let matrix = SparseColMat::try_new_from_triplets(2, 2, &triples).unwrap();
        let factor = Basis {
            factor: Arc::new(Factor::Sparse(matrix.sp_lu().unwrap())),
            updates: vec![],
            multiplier_cache: None,
        };
        Simplex {
            columns: Arc::new(columns),
            rhs: vec![1., 1. + if true_negative { 2. * tiny } else { tiny }],
            upper: vec![f64::INFINITY, f64::INFINITY, 1.],
            basis: vec![0, 1],
            basic: vec![true, true, false],
            at_upper: vec![false; 3],
            x: if true_negative {
                vec![-1., 2., 0.]
            } else {
                vec![-0.001, 1.001, 0.]
            },
            factor,
            stats: Stats::default(),
            limit: 1000,
            deadline: None,
            artificial: vec![],
            original: 3,
            witness: None,
            witness_point: None,
        }
    }
    #[test]
    fn refinement_recovers_an_inaccurate_basis_point_without_moving_nonbasics() {
        let mut s = refinement_example(false);
        assert!(primal_ratio(&s.basis, &s.x, &s.upper, &[1., 0.], 1., 1.).is_err());
        assert!(s.refine_ratio(&[1., 0.], 1., 2).is_some());
        assert_eq!(s.x[2], 0.);
        assert!(s.x[0].abs() < FEAS);
        assert!((s.x[1] - 1.).abs() < FEAS);
        assert_eq!(s.stats.refinement_successes, 1);
    }
    #[test]
    fn refinement_does_not_clip_a_truly_infeasible_basis_or_mutate_on_expiry() {
        let mut s = refinement_example(true);
        let point = s.x.clone();
        assert!(s.refine_ratio(&[1., 0.], 1., 2).is_none());
        assert_eq!(s.x, point);
        let mut s = refinement_example(false);
        let point = s.x.clone();
        s.deadline = Some(Instant::now());
        assert!(s.refine_ratio(&[1., 0.], 1., 2).is_none());
        assert_eq!(s.x, point);
        assert_eq!(s.stats.refinement_steps, 0);
    }
}

#[cfg(test)]
mod restoration_tests {
    use super::*;
    fn example(infeasible: bool) -> Simplex {
        Simplex {
            columns: Arc::new(vec![
                vec![(0, 1.)],
                vec![(0, 1.)],
                vec![(0, if infeasible { 1. } else { -1. })],
            ]),
            rhs: vec![-1.],
            upper: vec![2.; 3],
            basis: vec![0],
            basic: vec![true, false, false],
            at_upper: vec![false; 3],
            x: vec![-1., 0., 0.],
            factor: Basis {
                factor: Arc::new(Factor::Diagonal(vec![1.])),
                updates: vec![],
                multiplier_cache: None,
            },
            stats: Stats::default(),
            limit: 100,
            deadline: None,
            artificial: vec![],
            original: 3,
            witness: None,
            witness_point: None,
        }
    }
    fn inaccurate_updated_basis() -> Simplex {
        let mut s = example(false);
        // Simulate a badly inaccurate accumulated inverse. Rebuilding the
        // actual basis [1] must restore the original LP, not clip its bounds.
        s.factor.updates.push(Arc::new(Eta {
            row: 0, column: vec![(0, -1.)], pivot: -1.,
        }));
        s
    }
    #[test]
    fn dual_retry_rebuilds_an_inaccurate_inverse_and_checks_the_original_lp() {
        let costs = [1., 2., 0.];
        let mut off = inaccurate_updated_basis();
        assert!(matches!(off.dual_run_retry::<false>(&costs, false), Err(Outcome::NumericalError)));
        let mut on = inaccurate_updated_basis();
        let (primal, dual) = on.dual_run_retry::<false>(&costs, true).unwrap();
        assert_eq!((primal, dual), (0., 0.));
        assert_eq!(on.x, vec![0., 0., 1.]);
        assert_eq!(on.stats.dual_refactor_retries, 1);
        assert_eq!(on.stats.refactors, 1);
        assert_eq!(on.certificate(&costs).unwrap(), (0., 0.));
    }
    #[test]
    fn dual_retry_preserves_deadlines_and_work_limits() {
        let mut expired = inaccurate_updated_basis();
        expired.deadline = Some(Instant::now());
        assert!(matches!(expired.dual_run_retry::<false>(&[1., 2., 0.], true), Err(Outcome::TimeLimit)));
        assert_eq!(expired.stats.dual_refactor_retries, 0);
        let mut limited = inaccurate_updated_basis();
        limited.limit = 0;
        assert!(matches!(limited.dual_run_retry::<false>(&[1., 2., 0.], true), Err(Outcome::IterationLimit)));
        assert_eq!(limited.stats.dual_refactor_retries, 0);
    }
    #[test]
    fn zero_objective_repairs_a_genuinely_infeasible_basis_then_optimizes() {
        let mut s = example(false);
        assert!(s.refine_ratio(&[1.], 1., 1).is_none());
        let (objective, dual) = s.run(&[0., -1., 0.]).unwrap();
        assert_eq!((objective, dual), (-1., -1.));
        assert_eq!(s.x, vec![0., 1., 2.]);
        assert_eq!(s.stats.restoration_successes, 1);
        assert!(s.stats.restoration_pivots > 0);
    }
    #[test]
    fn progress_epoch_preserves_all_spent_iterations_and_deadline() {
        let mut s = example(false);
        s.stats.restoration_attempts = 4;
        s.stats.pivots = 7;
        // The final certificate requires entering another budget-checked loop.
        s.limit = 9;
        assert!(matches!(s.restore_primal(0), Err(Outcome::NumericalError)));
        s.restore_primal(4).unwrap();
        assert_eq!(s.stats.restoration_attempts, 5);
        assert_eq!(s.stats.restoration_extended_attempts, 1);
        assert_eq!(s.stats.pivots, 8);
        let mut s = example(false);
        s.stats.restoration_attempts = 4;
        s.stats.pivots = 7;
        s.limit = 7;
        assert!(matches!(s.restore_primal(4), Err(Outcome::IterationLimit)));
        assert_eq!(s.stats.pivots, 7);
        let mut s = example(false);
        s.stats.restoration_attempts = 4;
        s.deadline = Some(Instant::now());
        let point = s.x.clone();
        assert!(matches!(s.restore_primal(4), Err(Outcome::TimeLimit)));
        assert_eq!(s.stats.restoration_attempts, 4);
        assert_eq!(s.x, point);
    }
    #[test]
    fn infeasible_candidate_has_a_real_contradiction_and_expiry_preserves_point() {
        let mut s = example(true);
        assert!(matches!(s.restore_primal(0), Err(Outcome::Infeasible)));
        let witness = s.witness.as_ref().unwrap();
        assert!([1., -1.].into_iter().any(|sign| {
            let y = sign * witness[0];
            y * s.rhs[0]
                - s.columns
                    .iter()
                    .zip(&s.upper)
                    .map(|(c, &u)| (y * c[0].1 * u).max(0.))
                    .sum::<f64>()
                > 0.
        }));
        let mut s = example(false);
        s.deadline = Some(Instant::now());
        let point = s.x.clone();
        assert!(matches!(s.restore_primal(0), Err(Outcome::TimeLimit)));
        assert_eq!(s.x, point);
    }
}

#[cfg(test)]
mod long_restoration_test {
    use super::*;
    #[test]
    fn zero_cost_dual_progress_is_not_mistaken_for_stagnation() {
        let n = 600;
        let columns: Vec<_> = (0..2 * n)
            .map(|j| vec![(j % n, if j < n { 1. } else { -1. })])
            .collect();
        let mut s = Simplex {
            columns: Arc::new(columns),
            rhs: vec![-1.; n],
            upper: vec![2.; 2 * n],
            basis: (0..n).collect(),
            basic: (0..2 * n).map(|j| j < n).collect(),
            at_upper: vec![false; 2 * n],
            x: (0..2 * n).map(|j| if j < n { -1. } else { 0. }).collect(),
            factor: Basis {
                factor: Arc::new(Factor::Diagonal(vec![1.; n])),
                updates: vec![],
                multiplier_cache: None,
            },
            stats: Stats::default(),
            limit: 10000,
            deadline: None,
            artificial: vec![],
            original: 2 * n,
            witness: None,
            witness_point: None,
        };
        s.restore_primal(0).unwrap();
        assert_eq!(s.stats.restoration_successes, 1);
        assert_eq!(s.stats.restoration_pivots, n);
        for i in 0..n {
            assert_eq!(s.x[i], 0.);
            assert_eq!(s.x[n + i], 1.);
        }
    }
}

#[cfg(test)]
mod row_extension_tests {
    use super::*;
    fn parent(equality: bool) -> Problem {
        Problem {
            columns: if equality {
                vec![vec![]]
            } else {
                vec![vec![(0, 1.)], vec![(0, 1.)]]
            },
            rhs: vec![if equality { 0. } else { 3. }],
            costs: if equality { vec![-1.] } else { vec![-1.; 2] },
            upper: if equality { vec![2.] } else { vec![2.; 2] },
            equality: vec![equality],
            slack_upper: vec![f64::INFINITY],
        }
    }
    fn append(p: &mut Problem, coeffs: &[f64], rhs: f64) {
        let i = p.rhs.len();
        for (c, &v) in p.columns.iter_mut().zip(coeffs) {
            if v != 0. {
                c.push((i, v));
            }
        }
        p.rhs.push(rhs);
        p.equality.push(false);
        p.slack_upper.push(f64::INFINITY);
    }
    fn objective(a: &Answer) -> f64 {
        match &a.outcome {
            Outcome::Optimal { objective, .. } => *objective,
            o => panic!("{o:?}"),
        }
    }
    #[test]
    fn appended_inequality_reuses_basis_and_preserves_parent_and_sibling() {
        for equality in [false, true] {
            let p = parent(equality);
            let a = warm_impl(&p, 1000, None, None);
            let warm = a.warm.as_deref().unwrap();
            let basis = warm.basis.clone();
            let columns = warm.structure.columns.clone();
            let mut q = p.clone();
            append(&mut q, &vec![1.; p.costs.len()], 1.);
            let b = warm_impl(&q, 1000, None, Some(warm));
            assert!((objective(&b) + 1.).abs() < 1e-9);
            assert_eq!(objective(&b), objective(&cold(&q, 1000, None, true)));
            assert_eq!(b.stats.warm_row_extensions, 1);
            assert_eq!(b.stats.warm_successes, 1);
            assert!(b.stats.dual_pivots > 0);
            assert_eq!(warm.basis, basis);
            assert!(Arc::ptr_eq(&warm.structure.columns, &columns));
            assert_eq!(
                objective(&warm_impl(&p, 1000, None, Some(warm))),
                objective(&a)
            );
            let mut sibling = p.clone();
            append(&mut sibling, &vec![1.; p.costs.len()], 0.5);
            assert!((objective(&warm_impl(&sibling, 1000, None, Some(warm))) + 0.5).abs() < 1e-9);
            // Extend an already extended structure as well.
            append(&mut q, &vec![1.; p.costs.len()], 0.25);
            let c = warm_impl(&q, 1000, None, b.warm.as_deref());
            assert!((objective(&c) + 0.25).abs() < 1e-9);
            assert_eq!(c.stats.warm_row_extensions, 1);
        }
    }
    #[test]
    fn appended_infeasible_row_returns_a_contradiction() {
        let p = parent(false);
        let a = warm_impl(&p, 1000, None, None);
        let mut q = p.clone();
        append(&mut q, &[1., 1.], -1.);
        let b = minimize_warm(&q, 1000, None, a.warm.as_deref());
        assert!(matches!(b.outcome, Outcome::Infeasible));
        assert!(b.witness.is_some());
        assert_eq!(b.stats.warm_row_extensions, 1);
    }
    #[test]
    fn changed_old_structure_cost_or_new_equality_rejects_extension() {
        let p = parent(false);
        let a = warm_impl(&p, 1000, None, None);
        let warm = a.warm.as_deref().unwrap();
        let mut q = p.clone();
        append(&mut q, &[1., 1.], 1.);
        assert!(warm.extend_rows(&q, Some(Instant::now())).is_none());
        for change in 0..4 {
            let mut r = q.clone();
            match change {
                0 => r.costs[0] = -2.,
                1 => r.columns[0][0].1 = 2.,
                2 => r.equality[1] = true,
                _ => r.slack_upper[0] = 4.,
            }
            assert!(warm.extend_rows(&r, None).is_none());
            assert_eq!(
                warm_impl(&r, 1000, None, Some(warm)).stats.warm_rejections,
                1
            );
        }
    }
    #[test]
    fn random_append_sequences_match_cold_solves() {
        let mut seed = 12345u64;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            (seed >> 32) as u32
        };
        for _ in 0..200 {
            let mut p = parent(false);
            p.costs = vec![-((next() % 9 + 1) as f64), -((next() % 9 + 1) as f64)];
            let mut a = warm_impl(&p, 1000, None, None);
            for _ in 0..4 {
                let coeff = [(next() % 9) as f64 - 4., (next() % 9) as f64 - 4.];
                append(&mut p, &coeff, (next() % 12 + 1) as f64);
                let b = warm_impl(&p, 1000, None, a.warm.as_deref());
                let c = cold(&p, 1000, None, true);
                assert!((objective(&b) - objective(&c)).abs() < 1e-7);
                assert_eq!(b.stats.warm_row_extensions, 1);
                a = b;
            }
        }
    }
}

#[cfg(test)]
mod pricing_budget_tests {
    use super::*;
    #[test]
    fn cost_changed_warm_keeps_standard_pricing_and_its_feasible_basis() {
        let mut p = Problem {
            columns: vec![vec![]; 200],
            rhs: vec![],
            costs: vec![1.; 200],
            upper: vec![1.; 200],
            equality: vec![],
            slack_upper: vec![],
        };
        let parent = warm_impl(&p, 1000, None, None);
        p.costs.fill(-1.);
        let a = warm_impl(&p, 1000, None, parent.warm.as_deref());
        assert!(matches!(a.outcome,Outcome::Optimal{objective,..}if objective == -200.));
        assert_eq!(a.stats.flips, 200);
        assert_eq!(a.stats.pricing_fallbacks, 0);
        assert_eq!(a.stats.warm_fallbacks, 0);
        assert_eq!(a.stats.pricing_cutoffs, 0);
        assert_eq!(a.stats.warm_successes, 1);
        let a = warm_impl(&p, 120, None, parent.warm.as_deref());
        assert!(matches!(a.outcome, Outcome::IterationLimit));
        assert_eq!(a.stats.flips, 120);
        assert_eq!(a.stats.warm_fallbacks, 0);
        assert_eq!(a.stats.pricing_fallbacks, 0);
    }

    #[test]
    fn many_bound_flips_switch_pricing_and_keep_the_total_iteration_budget() {
        let mut p = Problem {
            columns: vec![vec![(0, -1.)]; 200],
            rhs: vec![-200.],
            costs: vec![0.; 200],
            upper: vec![1.; 200],
            equality: vec![false],
            slack_upper: vec![f64::INFINITY],
        };
        let fast = cold(&p, 1000, None, true);
        assert!(matches!(fast.outcome,Outcome::Optimal{objective,..}if objective == 0.));
        assert_eq!(fast.stats.flips + fast.stats.pivots, 200);
        assert_eq!(fast.stats.feasibility_pricing_calls, 0);
        let a = cold_with_probe(&p, 1000, None, true, std::time::Duration::ZERO);
        assert!(matches!(a.outcome,Outcome::Optimal{objective,..}if objective == 0.));
        assert_eq!(a.stats.pricing_cutoffs, 1);
        assert_eq!(a.stats.flips + a.stats.pivots, 300);
        assert_eq!(a.stats.pricing_fallbacks, 1);
        let a = cold_with_probe(&p, 120, None, true, std::time::Duration::ZERO);
        assert!(matches!(a.outcome, Outcome::IterationLimit));
        assert_eq!(a.stats.flips + a.stats.pivots, 120);
        assert_eq!(a.stats.pricing_cutoffs, 1);
        assert_eq!(a.stats.feasibility_pricing_calls, 1);
        p.costs.fill(-1.);
        let a = cold(&p, 1000, None, true);
        assert!(matches!(a.outcome,Outcome::Optimal{objective,..}if objective == -200.));
        assert!(a.stats.flips + a.stats.pivots < 300);
        assert_eq!(a.stats.feasibility_pricing_calls, 0);
        assert_eq!(a.stats.pricing_cutoffs, 0);
        assert_eq!(a.stats.pricing_fallbacks, 0);
    }
}

/// Approximate tableau rows for cut proposal only; validity requires separate proof.
pub fn tableau_rows(p: &Problem, warm: &WarmStart, columns: &[usize], deadline: Instant)
    -> Vec<(usize, Vec<f64>)>
{ crate::scaling::tableau_rows(p,warm,columns,deadline) }
#[cfg(test)]
mod tableau_row_tests {
    use super::*;
    #[test]
    fn tableau_rows_return_original_units_after_column_and_row_scaling() {
        let large=2f64.powi(20);
        let p=Problem {columns:vec![vec![(0,large),(1,1.)],vec![(0,large),(1,2.)]],rhs:vec![large,1.5],costs:vec![1.,2.],upper:vec![f64::INFINITY;2],equality:vec![true;2],slack_upper:vec![0.;2]};
        let a=minimize_warm(&p,1000,None,None);assert!(matches!(a.outcome,Outcome::Optimal{..}));let warm=a.warm.unwrap();
        let rows=tableau_rows(&p,&warm,&[0,1],Instant::now()+std::time::Duration::from_secs(1));assert_eq!(rows.len(),2);
        for (k,w) in rows {for j in 0..2 {let value:f64=p.columns[j].iter().map(|&(i,a)|a*w[i]).sum();assert!((value-if j==k{1.}else{0.}).abs()<1e-12);}}
        assert!(tableau_rows(&p,&warm,&[0,1],Instant::now()).is_empty());
        let mut changed=p.clone();changed.columns[0][0].1*=2.;assert!(tableau_rows(&changed,&warm,&[0],Instant::now()+std::time::Duration::from_secs(1)).is_empty());
    }
}

#[cfg(test)]mod limited_point_tests{
 use super::*;
 #[test]fn limited_points_are_candidates_only_and_validated_in_original_scaled_units(){
  if !std::env::var("MIP_LP_LIMIT_POINT").map_or(true,|v|v=="1"){return;}
  for scale in [1.,1e8,1e-8]{let p=Problem{columns:vec![vec![(0,scale)]],rhs:vec![scale],costs:vec![-1.],upper:vec![2.],equality:vec![false],slack_upper:vec![f64::INFINITY]};let a=minimize_warm(&p,0,None,None);assert!(matches!(a.outcome,Outcome::IterationLimit));assert_eq!(a.limited_point,Some(vec![0.]));assert!(a.dual.is_none());let opt=minimize_warm(&p,100,None,None);let a=minimize_warm(&p,0,None,opt.warm.as_deref());assert!(matches!(a.outcome,Outcome::IterationLimit));assert_eq!(a.limited_point,Some(vec![1.]));assert!(a.dual.is_none());}
  let p=Problem{columns:vec![vec![(0,1.)]],rhs:vec![1.],costs:vec![-1.],upper:vec![2.],equality:vec![true],slack_upper:vec![0.]};let a=minimize_warm(&p,0,None,None);assert!(matches!(a.outcome,Outcome::IterationLimit));assert!(a.limited_point.is_none());
 }
}
#[cfg(test)]mod singleton_crash_tests {
 use super::*;
 fn equality(rhs:f64,a:f64,upper:f64)->Problem{Problem{columns:vec![vec![(0,2.)],vec![(0,a)]],rhs:vec![rhs],costs:vec![1.,0.],upper:vec![f64::INFINITY,upper],equality:vec![true],slack_upper:vec![0.]}}
 #[test]fn feasible_zero_cost_singleton_avoids_artificial_pivots_and_keeps_warm_reusable(){
  let p=equality(4.,1.,f64::INFINITY);let old=cold_crashed(&p,1000,None,true,false,false);let new=cold_crashed(&p,1000,None,true,false,true);assert_eq!(new.stats.singleton_crash_columns,1);assert!(new.stats.pivots<old.stats.pivots);assert!(matches!(new.outcome,Outcome::Optimal{objective,x}if objective==0.&&x==vec![0.,4.]));let warm=new.warm.unwrap();
  let mut q=p.clone();q.rhs[0]=5.;let next=minimize_warm(&q,1000,None,Some(&warm));assert!(matches!(next.outcome,Outcome::Optimal{objective,x}if objective==0.&&x[1]==5.));
  let parent=minimize_warm(&p,1000,None,Some(&warm));assert!(matches!(parent.outcome,Outcome::Optimal{objective,x}if objective==0.&&x[1]==4.));
  let mut extended=p.clone();extended.columns[0].push((1,1.));extended.rhs.push(1.);extended.equality.push(false);extended.slack_upper.push(f64::INFINITY);let row=minimize_warm(&extended,1000,None,Some(&warm));assert!(matches!(row.outcome,Outcome::Optimal{objective,..}if objective==0.));assert_eq!(row.stats.warm_row_extensions,1);
 }
 #[test]fn negative_sign_and_ranged_slack_are_checked_against_actual_bounds(){
  let p=equality(-1.,-1.,2.);let new=cold_crashed(&p,1000,None,false,false,true);assert_eq!(new.stats.singleton_crash_columns,1);assert!(matches!(new.outcome,Outcome::Optimal{objective,x}if objective==0.&&x[1]==1.));
  let p=equality(-1.,-1.,0.5);let new=cold_crashed(&p,1000,None,false,false,true);assert_eq!(new.stats.singleton_crash_columns,0);assert!(matches!(new.outcome,Outcome::Infeasible));
  let mut p=equality(5.,1.,2.);p.equality[0]=false;p.slack_upper[0]=4.;let new=cold_crashed(&p,1000,None,false,false,true);assert_eq!(new.stats.singleton_crash_columns,1);assert!(matches!(new.outcome,Outcome::Optimal{objective,x}if objective==0.&&x[1]>=1.&&x[1]<=2.));
 }
 #[test]fn non_power_of_two_and_expired_budget_do_not_force_a_crash(){
  let p=equality(4.,3.,f64::INFINITY);let new=cold_crashed(&p,1000,None,false,false,true);assert_eq!(new.stats.singleton_crash_columns,0);assert!(matches!(new.outcome,Outcome::Optimal{objective,..}if objective==0.));let p=equality(4.,1.,f64::INFINITY);let expired=cold_crashed(&p,1000,Some(Instant::now()),false,false,true);assert!(matches!(expired.outcome,Outcome::TimeLimit));assert_eq!(expired.stats.singleton_crash_columns,0);
 }
}

#[cfg(test)]mod singleton_extension_tests{
 use super::*;
 #[test]fn a_new_row_can_touch_the_original_singleton_basic_column(){
  let p=Problem{columns:vec![vec![(0,2.)],vec![(0,1.)]],rhs:vec![4.],costs:vec![1.,0.],upper:vec![f64::INFINITY;2],equality:vec![true],slack_upper:vec![0.]};let a=cold_crashed(&p,1000,None,true,false,true);assert!(matches!(a.outcome,Outcome::Optimal{objective,..}if objective==0.));assert_eq!(a.stats.singleton_crash_columns,1);let w=a.warm.unwrap();let mut q=p.clone();q.columns[1].push((1,1.));q.rhs.push(3.);q.equality.push(false);q.slack_upper.push(f64::INFINITY);let b=minimize_warm(&q,1000,None,Some(&w));let c=minimize(&q,1000,None);assert_eq!(b.stats.warm_row_extensions,1);assert!(matches!(b.outcome,Outcome::Optimal{objective,x}if objective==0.5&&x==vec![0.5,3.]));assert!(matches!(c.outcome,Outcome::Optimal{objective,..}if objective==0.5));let parent=minimize_warm(&p,1000,None,Some(&w));assert!(matches!(parent.outcome,Outcome::Optimal{objective,x}if objective==0.&&x==vec![0.,4.]));
 }
}
#[cfg(test)]mod empty_pricing_tests{
 use super::*;
 fn initial()->Simplex{let n=1300;let mut columns=vec![vec![];n];columns[0]=vec![(0,1.)];columns.push(vec![(0,1.)]);let mut basic=vec![false;n+1];basic[n]=true;let mut x=vec![0.;n+1];x[n]=1.;let mut upper=vec![3.;n+1];upper[n]=f64::INFINITY;Simplex{columns:Arc::new(columns),rhs:vec![1.],upper,basis:vec![n],basic,at_upper:vec![false;n+1],x,factor:Basis{factor:Arc::new(Factor::Diagonal(vec![1.])),updates:vec![],multiplier_cache:None},stats:Stats::default(),limit:1000,deadline:None,artificial:vec![],original:n,witness:None,witness_point:None}}
 #[test]fn skipped_columns_preserve_all_pivots_points_and_certificates(){for normalized in [false,true]{let mut old=initial();let mut new=old.clone();let mut c=vec![0.;old.columns.len()];c[0]=-1.;let(a,b)=if normalized{(old.run_priced_mode::<true>(&c,false),new.run_priced_mode::<true>(&c,true))}else{(old.run_priced_mode::<false>(&c,false),new.run_priced_mode::<false>(&c,true))};assert_eq!(a.unwrap(),b.unwrap());assert_eq!(old.x,new.x);assert_eq!(old.basis,new.basis);assert_eq!(old.stats.pivots,new.stats.pivots);assert_eq!(old.stats.flips,new.stats.flips);assert_eq!(new.stats.empty_pricing_calls,1);assert!(new.stats.empty_pricing_skips>=1299);assert_eq!(new.certificate(&c).unwrap(),(-1.,-1.));}}
 #[test]fn nonzero_empty_columns_keep_bound_flips_and_tiny_unbounded_objectives(){for cost in [-2.,-1e-19]{for finite in [false,true]{let mut old=initial();let mut new=old.clone();old.upper[1]=if finite{3.}else{f64::INFINITY};new.upper[1]=old.upper[1];let mut c=vec![0.;old.columns.len()];c[1]=cost;let a=old.run_priced_mode::<false>(&c,false);let b=new.run_priced_mode::<false>(&c,true);if finite{assert_eq!(a.unwrap(),b.unwrap());assert_eq!(old.x,new.x);assert_eq!(new.x[1],3.);}else{assert!(matches!(a,Err(Outcome::Unbounded)));assert!(matches!(b,Err(Outcome::Unbounded)));}}}}
 #[test]fn cost_and_matrix_changes_rebuild_the_candidate_set(){let mut s=initial();let mut c=vec![0.;s.columns.len()];assert!(!pricing_columns(&s.columns,&c,true).unwrap().contains(&5));s.run_priced_mode::<false>(&c,true).unwrap();c[5]=-2.;assert!(pricing_columns(&s.columns,&c,true).unwrap().contains(&5));assert_eq!(s.run_priced_mode::<false>(&c,true).unwrap(),(-6.,-6.));let mut columns=(*s.columns).clone();columns[6].push((1,1.));assert!(pricing_columns(&columns,&c,true).unwrap().contains(&6));assert!(s.columns[6].is_empty());}
}
#[cfg(test)]
mod zero_phase_one_tests{
 use super::*;
 fn cycle()->Problem{let n=16;Problem{columns:(0..n).map(|j|vec![(j,2.),((j+n-1)%n,-1.)]).collect(),rhs:vec![0.;n],costs:vec![0.;n],upper:vec![4.;n],equality:vec![true;n],slack_upper:vec![0.;n]}}
 #[test]
 fn zero_artificial_objective_skips_only_phase_one_and_keeps_a_reusable_parent(){
  let p=cycle();let old=cold_initialized(&p,10000,None,true,true,true,false);let new=cold_initialized(&p,10000,None,true,true,true,true);assert!(matches!(old.outcome,Outcome::Optimal{objective,..}if objective==0.));assert!(matches!(new.outcome,Outcome::Optimal{objective,..}if objective==0.));assert_eq!(new.stats.zero_phase_one_skips,1);assert_eq!(new.stats.feasibility_pricing_calls,0);assert!(old.stats.feasibility_pricing_calls>0);assert!(new.stats.pivots<old.stats.pivots);
  let parent=new.warm.unwrap();let mut changed=p.clone();changed.costs[0]=-1.;let a=minimize_warm(&changed,10000,None,Some(&parent));assert!(matches!(a.outcome,Outcome::Optimal{objective,..}if objective==0.));let b=minimize_warm(&p,10000,None,Some(&parent));assert!(matches!(b.outcome,Outcome::Optimal{objective,..}if objective==0.));
  let mut extended=p.clone();let row=extended.rhs.len();extended.rhs.push(0.);extended.equality.push(false);extended.slack_upper.push(f64::INFINITY);extended.columns[0].push((row,-1.));let e=minimize_warm(&extended,10000,None,Some(&parent));assert!(matches!(e.outcome,Outcome::Optimal{objective,..}if objective==0.));assert!(e.stats.warm_row_extensions>0);
 }
 #[test]
 fn tiny_positive_phase_one_values_never_use_the_zero_shortcut(){let mut p=cycle();p.rhs[0]=1e-16;let r=cold_initialized(&p,10000,None,true,true,true,true);assert_eq!(r.stats.zero_phase_one_skips,0);assert_eq!(r.stats.feasibility_pricing_calls,1);}
 #[test]
 fn deadline_and_pivot_budget_remain_in_force(){let p=cycle();let r=cold_initialized(&p,10000,Some(Instant::now()),true,true,true,true);assert!(matches!(r.outcome,Outcome::TimeLimit));let r=cold_initialized(&p,0,None,true,true,true,true);assert!(matches!(r.outcome,Outcome::IterationLimit));}
}
