//! Bounded-variable revised primal simplex. Optimization logic is original Rust;
//! faer is used solely for general-purpose sparse LU factorizations/solves.
use crate::lp::Outcome;
use faer::{
    Col,
    linalg::solvers::Solve,
    sparse::{SparseColMat, Triplet, linalg::solvers::Lu},
};
use std::time::Instant;
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
}
#[derive(Debug)]
pub struct Answer {
    pub outcome: Outcome,
    pub dual: Option<f64>,
    pub stats: Stats,
}
struct Eta {
    row: usize,
    column: Vec<(usize, f64)>,
    pivot: f64,
}
enum Factor {
    Diagonal(Vec<f64>),
    Sparse(Lu<usize, f64>),
}
struct Basis {
    factor: Factor,
    updates: Vec<Eta>,
}
impl Basis {
    fn forward(&self, v: &[f64]) -> Vec<f64> {
        let mut x = match &self.factor {
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
        match &self.factor {
            Factor::Diagonal(d) => x.iter().zip(d).map(|(v, d)| v / d).collect(),
            Factor::Sparse(lu) => {
                let rhs = Col::from_fn(v.len(), |i| x[i]);
                let x = lu.solve_transpose(&rhs);
                (0..v.len()).map(|i| x[i]).collect()
            }
        }
    }
    fn update(&mut self, row: usize, d: &[f64]) {
        self.updates.push(Eta {
            row,
            column: d
                .iter()
                .enumerate()
                .filter(|(_, v)| **v != 0.0)
                .map(|(i, &v)| (i, v))
                .collect(),
            pivot: d[row],
        });
    }
}
struct Simplex {
    columns: Vec<Vec<(usize, f64)>>,
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
            factor: Factor::Sparse(lu),
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
    let m = p.rhs.len();
    let n = p.costs.len();
    let error = |outcome| Answer {
        outcome,
        dual: None,
        stats: Stats::default(),
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
        columns: p.columns.clone(),
        rhs: p.rhs.clone(),
        upper: p.upper.clone(),
        basis: vec![],
        basic: vec![false; n],
        at_upper: vec![false; n],
        x: vec![0.0; n],
        factor: Basis {
            factor: Factor::Diagonal(vec![]),
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
            s.columns.push(vec![(i, 1.0)]);
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
            s.columns.push(vec![(i, sign)]);
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
    s.factor.factor = Factor::Diagonal(diagonal);
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
            outcome: Outcome::Optimal {
                objective,
                x: s.x[..s.original].to_vec(),
            },
            dual: Some(dual),
            stats: s.stats,
        },
        Err(outcome) => Answer {
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
            factor: Factor::Diagonal(vec![1.; 3]),
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
