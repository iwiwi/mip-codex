//! Bounded-variable, two-phase primal revised simplex.
//! Only general sparse LU is delegated to faer; optimization is implemented here.
use crate::model::Model;
use faer::{
    Col,
    prelude::Solve,
    sparse::{SparseColMat, Triplet, linalg::solvers::Lu},
};
use std::time::Instant;

const FEAS: f64 = 1e-8;
const DUAL: f64 = 1e-9;
const PIVOT: f64 = 1e-10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LpStatus {
    Optimal,
    Infeasible,
    Unbounded,
    Limit,
    Numerical,
}

#[derive(Debug)]
pub struct LpResult {
    pub status: LpStatus,
    pub objective: f64,
    pub values: Vec<f64>,
    pub iterations: usize,
}

#[derive(Clone)]
struct Column {
    entries: Vec<(usize, f64)>,
    upper: f64,
    cost: f64,
    artificial: bool,
}

struct Standard {
    cols: Vec<Column>,
    rhs: Vec<f64>,
    basis: Vec<usize>,
    // Original x[j] = shift[j] + sum(sign * z[k]).
    shift: Vec<f64>,
    mapping: Vec<Vec<(usize, f64)>>,
}

impl Standard {
    fn from_model(model: &Model, lower: &[f64], upper: &[f64]) -> Result<Self, LpStatus> {
        let mut s = Self {
            cols: vec![],
            rhs: vec![],
            basis: vec![],
            shift: vec![],
            mapping: vec![],
        };
        for (j, v) in model.variables.iter().enumerate() {
            if lower[j] > upper[j] {
                return Err(LpStatus::Infeasible);
            }
            let mut map = vec![];
            let shift;
            if lower[j].is_finite() {
                shift = lower[j];
                if upper[j] > lower[j] {
                    map.push((s.cols.len(), 1.0));
                    s.cols.push(Column {
                        entries: vec![],
                        upper: upper[j] - lower[j],
                        cost: v.cost * model.sense,
                        artificial: false,
                    });
                }
            } else if upper[j].is_finite() {
                shift = upper[j];
                map.push((s.cols.len(), -1.0));
                s.cols.push(Column {
                    entries: vec![],
                    upper: f64::INFINITY,
                    cost: -v.cost * model.sense,
                    artificial: false,
                });
            } else {
                shift = 0.0;
                for sign in [1.0, -1.0] {
                    map.push((s.cols.len(), sign));
                    s.cols.push(Column {
                        entries: vec![],
                        upper: f64::INFINITY,
                        cost: sign * v.cost * model.sense,
                        artificial: false,
                    });
                }
            }
            s.shift.push(shift);
            s.mapping.push(map);
        }
        for row in &model.rows {
            let offset: f64 = row.terms.iter().map(|&(j, a)| a * s.shift[j]).sum();
            let lo = row.lower - offset;
            let up = row.upper - offset;
            if lo > up {
                return Err(LpStatus::Infeasible);
            }
            let mut entries = vec![];
            for &(j, a) in &row.terms {
                for &(k, sign) in &s.mapping[j] {
                    entries.push((k, a * sign));
                }
            }
            if entries.is_empty() {
                if lo > FEAS || up < -FEAS {
                    return Err(LpStatus::Infeasible);
                }
                continue;
            }
            if !lo.is_finite() && !up.is_finite() {
                continue;
            }
            let scale = entries
                .iter()
                .fold(0.0_f64, |m, &(_, a)| m.max(a.abs()))
                .max(1e-12);
            let i = s.rhs.len();
            for &(k, a) in &entries {
                s.cols[k].entries.push((i, a / scale));
            }
            let (b, slack_sign, width) = if up.is_finite() {
                (up / scale, 1.0, (up - lo) / scale)
            } else {
                (lo / scale, -1.0, f64::INFINITY)
            };
            s.rhs.push(b);
            let slack = s.cols.len();
            s.cols.push(Column {
                entries: vec![(i, slack_sign)],
                upper: width,
                cost: 0.0,
                artificial: false,
            });
            let slack_value = b / slack_sign;
            if slack_value >= 0.0 && slack_value <= width {
                s.basis.push(slack);
            } else {
                let art = s.cols.len();
                s.cols.push(Column {
                    entries: vec![(i, if b >= 0.0 { 1.0 } else { -1.0 })],
                    upper: f64::INFINITY,
                    cost: 0.0,
                    artificial: true,
                });
                s.basis.push(art);
            }
        }
        Ok(s)
    }
}

struct Eta {
    row: usize,
    entries: Vec<(usize, f64)>,
    pivot: f64,
}

struct Factor {
    lu: Option<Lu<usize, f64>>,
    diagonal: Vec<f64>,
    etas: Vec<Eta>,
}

impl Factor {
    fn new(s: &Standard, basis: &[usize]) -> Result<Self, LpStatus> {
        let n = basis.len();
        let diagonal = basis
            .iter()
            .enumerate()
            .all(|(i, &j)| s.cols[j].entries.len() == 1 && s.cols[j].entries[0].0 == i);
        if diagonal {
            return Ok(Self {
                lu: None,
                diagonal: basis.iter().map(|&j| s.cols[j].entries[0].1).collect(),
                etas: vec![],
            });
        }
        let mut triplets = Vec::new();
        for (k, &j) in basis.iter().enumerate() {
            for &(i, a) in &s.cols[j].entries {
                triplets.push(Triplet::new(i, k, a));
            }
        }
        let a = SparseColMat::<usize, f64>::try_new_from_triplets(n, n, &triplets)
            .map_err(|_| LpStatus::Numerical)?;
        let lu = a.sp_lu().map_err(|_| LpStatus::Numerical)?;
        Ok(Self {
            lu: Some(lu),
            diagonal: vec![],
            etas: vec![],
        })
    }

    fn ftran(&self, rhs: &[f64]) -> Vec<f64> {
        let mut x = if let Some(lu) = &self.lu {
            let rhs = Col::from_fn(rhs.len(), |i| rhs[i]);
            let out = lu.solve(&rhs);
            (0..rhs.nrows()).map(|i| out[i]).collect::<Vec<_>>()
        } else {
            rhs.iter().zip(&self.diagonal).map(|(b, a)| b / a).collect()
        };
        for eta in &self.etas {
            let t = x[eta.row] / eta.pivot;
            for &(i, a) in &eta.entries {
                if i != eta.row {
                    x[i] -= a * t;
                }
            }
            x[eta.row] = t;
        }
        x
    }

    fn btran(&self, rhs: &[f64]) -> Vec<f64> {
        let mut x = rhs.to_vec();
        for eta in self.etas.iter().rev() {
            let sum: f64 = eta
                .entries
                .iter()
                .filter(|&&(i, _)| i != eta.row)
                .map(|&(i, a)| a * x[i])
                .sum();
            x[eta.row] = (x[eta.row] - sum) / eta.pivot;
        }
        if let Some(lu) = &self.lu {
            let rhs = Col::from_fn(x.len(), |i| x[i]);
            let out = lu.solve_transpose(&rhs);
            (0..x.len()).map(|i| out[i]).collect()
        } else {
            x.iter().zip(&self.diagonal).map(|(b, a)| b / a).collect()
        }
    }

    fn update(&mut self, row: usize, direction: &[f64]) {
        self.etas.push(Eta {
            row,
            pivot: direction[row],
            entries: direction
                .iter()
                .enumerate()
                .filter(|&(_, a)| *a != 0.0)
                .map(|(i, &a)| (i, a))
                .collect(),
        });
    }
}

struct State {
    basis: Vec<usize>,
    basic: Vec<bool>,
    at_upper: Vec<bool>,
    x: Vec<f64>,
    factor: Factor,
    iterations: usize,
}

impl State {
    fn new(s: &Standard) -> Result<Self, LpStatus> {
        let factor = Factor::new(s, &s.basis)?;
        let mut state = Self {
            basis: s.basis.clone(),
            basic: vec![false; s.cols.len()],
            at_upper: vec![false; s.cols.len()],
            x: vec![0.0; s.cols.len()],
            factor,
            iterations: 0,
        };
        for &j in &state.basis {
            state.basic[j] = true;
        }
        state.recompute(s)?;
        Ok(state)
    }

    fn recompute(&mut self, s: &Standard) -> Result<(), LpStatus> {
        let mut rhs = s.rhs.clone();
        for (j, col) in s.cols.iter().enumerate() {
            if self.basic[j] {
                continue;
            }
            self.x[j] = if self.at_upper[j] { col.upper } else { 0.0 };
            if self.x[j] != 0.0 {
                for &(i, a) in &col.entries {
                    rhs[i] -= a * self.x[j];
                }
            }
        }
        let xb = self.factor.ftran(&rhs);
        for (i, &j) in self.basis.iter().enumerate() {
            if !xb[i].is_finite() || xb[i] < -1e-6 || xb[i] > s.cols[j].upper + 1e-6 {
                return Err(LpStatus::Numerical);
            }
            self.x[j] = xb[i].max(0.0).min(s.cols[j].upper);
        }
        Ok(())
    }

    fn simplex(&mut self, s: &Standard, costs: &[f64], deadline: Instant) -> LpStatus {
        let m = s.rhs.len();
        let mut stalled = 0usize;
        loop {
            if Instant::now() >= deadline || self.iterations >= 1_000_000 {
                return LpStatus::Limit;
            }
            let cb: Vec<f64> = self.basis.iter().map(|&j| costs[j]).collect();
            let dual = self.factor.btran(&cb);
            if dual.iter().any(|a| !a.is_finite()) {
                return LpStatus::Numerical;
            }
            let mut enter = None;
            let mut best = DUAL;
            for (j, col) in s.cols.iter().enumerate() {
                if self.basic[j] || col.upper == 0.0 {
                    continue;
                }
                let rc = costs[j] - col.entries.iter().map(|&(i, a)| a * dual[i]).sum::<f64>();
                let improvement = if self.at_upper[j] { rc } else { -rc };
                if improvement > best {
                    best = improvement;
                    enter = Some(j);
                    if stalled > 200 {
                        break;
                    } // Bland pricing after prolonged degeneracy.
                }
            }
            let Some(enter) = enter else {
                // Always refactor and recheck optimality before returning.
                if !self.factor.etas.is_empty() {
                    match Factor::new(s, &self.basis) {
                        Ok(f) => self.factor = f,
                        Err(e) => return e,
                    }
                    if let Err(e) = self.recompute(s) {
                        return e;
                    }
                    continue;
                }
                return LpStatus::Optimal;
            };
            let sign = if self.at_upper[enter] { -1.0 } else { 1.0 };
            let mut a = vec![0.0; m];
            for &(i, v) in &s.cols[enter].entries {
                a[i] = v;
            }
            let d = self.factor.ftran(&a);
            if d.iter().any(|a| !a.is_finite()) {
                return LpStatus::Numerical;
            }
            // Scale the pivot threshold to the transformed column. An entire
            // small column can be well conditioned (e.g. a 1x1 basis of 1e-12).
            let direction_norm = d.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
            let pivot_tolerance = PIVOT * direction_norm;
            let mut relaxed = s.cols[enter].upper;
            for (i, &j) in self.basis.iter().enumerate() {
                let v = sign * d[i];
                if v > pivot_tolerance {
                    relaxed = relaxed.min((self.x[j] + FEAS) / v);
                } else if v < -pivot_tolerance && s.cols[j].upper.is_finite() {
                    relaxed = relaxed.min((s.cols[j].upper - self.x[j] + FEAS) / (-v));
                }
            }
            let mut leave = None;
            let mut theta = s.cols[enter].upper;
            let mut largest = 0.0_f64;
            for (i, &j) in self.basis.iter().enumerate() {
                let v = sign * d[i];
                let ratio = if v > pivot_tolerance {
                    self.x[j] / v
                } else if v < -pivot_tolerance && s.cols[j].upper.is_finite() {
                    (s.cols[j].upper - self.x[j]) / (-v)
                } else {
                    continue;
                };
                if ratio <= relaxed && ratio <= s.cols[enter].upper && d[i].abs() > largest {
                    largest = d[i].abs();
                    leave = Some(i);
                    theta = ratio.max(0.0);
                }
            }
            if !theta.is_finite() {
                // Ignoring a tiny, finite blocking coefficient must never turn
                // a bounded LP into an unbounded declaration.
                let tiny_blocker = self.basis.iter().enumerate().any(|(i, &j)| {
                    let v = sign * d[i];
                    (v > 0.0 && v <= pivot_tolerance)
                        || (v < 0.0 && v >= -pivot_tolerance && s.cols[j].upper.is_finite())
                });
                return if tiny_blocker {
                    LpStatus::Numerical
                } else {
                    LpStatus::Unbounded
                };
            }
            if theta < 1e-10 {
                stalled += 1;
            } else {
                stalled = 0;
            }
            for (i, &j) in self.basis.iter().enumerate() {
                self.x[j] -= sign * d[i] * theta;
                if self.x[j] < -1e-5 || self.x[j] > s.cols[j].upper + 1e-5 {
                    return LpStatus::Numerical;
                }
                self.x[j] = self.x[j].max(0.0).min(s.cols[j].upper);
            }
            self.x[enter] += sign * theta;
            if let Some(row) = leave {
                let old = self.basis[row];
                self.at_upper[old] = sign * d[row] < 0.0;
                self.x[old] = if self.at_upper[old] {
                    s.cols[old].upper
                } else {
                    0.0
                };
                self.basic[old] = false;
                self.basic[enter] = true;
                self.basis[row] = enter;
                self.factor.update(row, &d);
            } else {
                self.at_upper[enter] = !self.at_upper[enter];
                self.x[enter] = if self.at_upper[enter] {
                    s.cols[enter].upper
                } else {
                    0.0
                };
            }
            self.iterations += 1;
            if self.factor.etas.len() >= 64 || (self.iterations.is_multiple_of(128)) {
                match Factor::new(s, &self.basis) {
                    Ok(f) => self.factor = f,
                    Err(e) => return e,
                }
                if let Err(e) = self.recompute(s) {
                    return e;
                }
            }
        }
    }
}

pub fn solve(model: &Model, lower: &[f64], upper: &[f64], deadline: Instant) -> LpResult {
    let mut result = LpResult {
        status: LpStatus::Numerical,
        objective: f64::NAN,
        values: vec![],
        iterations: 0,
    };
    let mut s = match Standard::from_model(model, lower, upper) {
        Ok(s) => s,
        Err(e) => {
            result.status = e;
            return result;
        }
    };
    let mut state = match State::new(&s) {
        Ok(s) => s,
        Err(e) => {
            result.status = e;
            return result;
        }
    };
    let phase_one: Vec<f64> = s
        .cols
        .iter()
        .map(|col| if col.artificial { 1.0 } else { 0.0 })
        .collect();
    if s.cols.iter().any(|c| c.artificial) {
        let status = state.simplex(&s, &phase_one, deadline);
        result.iterations = state.iterations;
        if status != LpStatus::Optimal {
            result.status = if status == LpStatus::Unbounded {
                LpStatus::Numerical
            } else {
                status
            };
            return result;
        }
        let violation: f64 = state.x.iter().zip(&phase_one).map(|(a, b)| a * b).sum();
        if violation > 1e-7 {
            result.status = LpStatus::Infeasible;
            return result;
        }
        for (j, col) in s.cols.iter_mut().enumerate() {
            if col.artificial {
                col.upper = 0.0;
                state.x[j] = 0.0;
                state.at_upper[j] = false;
            }
        }
    }
    let costs: Vec<f64> = s.cols.iter().map(|c| c.cost).collect();
    result.status = state.simplex(&s, &costs, deadline);
    result.iterations = state.iterations;
    if result.status == LpStatus::Optimal {
        result.values = s
            .shift
            .iter()
            .zip(&s.mapping)
            .map(|(&shift, map)| shift + map.iter().map(|&(j, a)| a * state.x[j]).sum::<f64>())
            .collect();
        result.objective = model.objective(&result.values);
        // Check original, unscaled constraints even for a continuous relaxation.
        let mut continuous = model.clone();
        for v in &mut continuous.variables {
            v.integer = false;
        }
        if !continuous.check(&result.values, 1e-6).feasible
            || result
                .values
                .iter()
                .enumerate()
                .any(|(j, &x)| x < lower[j] - 1e-7 || x > upper[j] + 1e-7)
        {
            result.status = LpStatus::Numerical;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::parse_mps;
    use std::time::Duration;
    #[test]
    fn tiny_blocking_coefficient_is_not_declared_unbounded() {
        let m=parse_mps("NAME tiny\nROWS\n N o\n L r\nCOLUMNS\n x o -1 r 1e-12\n y r 1\nRHS\n rhs r 1\nENDATA\n").unwrap();
        let r = solve(
            &m,
            &[0.0, 0.0],
            &[f64::INFINITY, f64::INFINITY],
            Instant::now() + Duration::from_secs(5),
        );
        assert_eq!(r.status, LpStatus::Optimal);
        assert!((r.objective + 1e12).abs() < 1.0);
    }
    #[test]
    fn eta_updates_match_fresh_sparse_lu() {
        let n = 80;
        let mut s = Standard {
            cols: vec![],
            rhs: vec![0.0; n],
            basis: (0..n).collect(),
            shift: vec![],
            mapping: vec![],
        };
        for i in 0..n {
            s.cols.push(Column {
                entries: vec![(i, 1.0)],
                upper: f64::INFINITY,
                cost: 0.0,
                artificial: false,
            });
        }
        let mut basis = s.basis.clone();
        let mut factor = Factor::new(&s, &basis).unwrap();
        let mut seed = 4711u64;
        for step in 0..100 {
            let row = step % n;
            // Small off-diagonal updates keep the constructed bases well conditioned.
            let entries: Vec<(usize, f64)> = (0..n)
                .filter_map(|i| {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    if i == row {
                        Some((i, 1.0))
                    } else if seed % 8 == 0 {
                        Some((i, ((seed >> 32) % 101) as f64 * 0.0001))
                    } else {
                        None
                    }
                })
                .collect();
            let mut a = vec![0.0; n];
            for &(i, v) in &entries {
                a[i] = v;
            }
            let d = factor.ftran(&a);
            s.cols.push(Column {
                entries,
                upper: f64::INFINITY,
                cost: 0.0,
                artificial: false,
            });
            basis[row] = s.cols.len() - 1;
            factor.update(row, &d);
        }
        let fresh = Factor::new(&s, &basis).unwrap();
        let rhs: Vec<f64> = (0..n).map(|i| (i as f64 * 0.7).sin()).collect();
        for (a, b) in factor.ftran(&rhs).iter().zip(fresh.ftran(&rhs)) {
            assert!((a - b).abs() < 1e-10);
        }
        for (a, b) in factor.btran(&rhs).iter().zip(fresh.btran(&rhs)) {
            assert!((a - b).abs() < 1e-10);
        }
    }
    #[test]
    fn bounded_two_phase_and_free() {
        let m = parse_mps("NAME test\nROWS\n N obj\n G r\nCOLUMNS\n x obj 1 r 1\n y obj 2 r 1\nRHS\n rhs r 3\nBOUNDS\n UP b x 2\n FR b y\nENDATA\n").unwrap();
        let r = solve(
            &m,
            &[0.0, f64::NEG_INFINITY],
            &[2.0, f64::INFINITY],
            Instant::now() + Duration::from_secs(5),
        );
        assert_eq!(r.status, LpStatus::Optimal);
        assert!((r.objective - 4.0).abs() < 1e-7);
    }
    #[test]
    fn infeasible_and_unbounded() {
        let m = parse_mps("NAME t\nROWS\n N o\n G r\nCOLUMNS\n x o -1 r 1\nRHS\n rhs r 2\nBOUNDS\n UP b x 1\nENDATA\n").unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        assert_eq!(
            solve(&m, &[0.0], &[1.0], deadline).status,
            LpStatus::Infeasible
        );
        assert_eq!(
            solve(&m, &[0.0], &[f64::INFINITY], deadline).status,
            LpStatus::Unbounded
        );
    }
}
