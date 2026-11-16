//! 元制約での境界伝播。演算を外向きに丸め、不確かな区間は弱いまま使う。
use crate::model::Model;
use std::{collections::VecDeque, time::Instant};
const ROW_TOL: f64 = 1e-6;
#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Stable,
    Limited,
    Infeasible,
    TimeLimit,
}
#[derive(Default, Debug, Clone)]
pub struct Stats {
    pub calls: usize,
    pub rows: usize,
    pub tightenings: usize,
    pub pruned: usize,
    pub budget_hits: usize,
}
pub struct Propagator<'a> {
    model: &'a Model,
    incidence: Vec<Vec<usize>>,
    queue: VecDeque<usize>,
    marks: Vec<u64>,
    epoch: u64,
    prefix_lo: Vec<f64>,
    prefix_hi: Vec<f64>,
    suffix_lo: Vec<f64>,
    suffix_hi: Vec<f64>,
    root_work: usize,
    budget_credit: usize,
    pub stats: Stats,
}
fn add_lo(a: f64, b: f64) -> f64 {
    (a + b).next_down()
}
fn add_hi(a: f64, b: f64) -> f64 {
    (a + b).next_up()
}
fn term(a: f64, lower: Option<f64>, upper: Option<f64>) -> (f64, f64) {
    if a == 0.0 {
        return (0.0, 0.0);
    }
    let lo = lower.unwrap_or(f64::NEG_INFINITY);
    let up = upper.unwrap_or(f64::INFINITY);
    if a > 0.0 {
        ((a * lo).next_down(), (a * up).next_up())
    } else {
        ((a * up).next_down(), (a * lo).next_up())
    }
}
fn improve_lower(candidate: f64, old: Option<f64>, integer: bool) -> bool {
    candidate.is_finite()
        && old.is_none_or(|b| {
            candidate > b
                && (integer || candidate - b > 1e-10 * (1.0 + candidate.abs().max(b.abs())))
        })
}
fn improve_upper(candidate: f64, old: Option<f64>, integer: bool) -> bool {
    candidate.is_finite()
        && old.is_none_or(|b| {
            candidate < b
                && (integer || b - candidate > 1e-10 * (1.0 + candidate.abs().max(b.abs())))
        })
}
impl<'a> Propagator<'a> {
    pub fn new(model: &'a Model) -> Self {
        let mut incidence = vec![vec![]; model.variables.len()];
        let mut root_work = 0usize;
        for (i, row) in model.rows.iter().enumerate() {
            root_work = root_work.saturating_add(row.coefficients.len() + 1);
            for &(j, a) in &row.coefficients {
                if a != 0.0 {
                    incidence[j].push(i);
                }
            }
        }
        Self {
            model,
            incidence,
            queue: VecDeque::new(),
            marks: vec![0; model.rows.len()],
            epoch: 0,
            prefix_lo: vec![],
            prefix_hi: vec![],
            suffix_lo: vec![],
            suffix_hi: vec![],
            root_work,
            budget_credit: 0,
            stats: Stats::default(),
        }
    }
    fn enqueue(&mut self, row: usize) {
        if self.marks[row] != self.epoch {
            self.marks[row] = self.epoch;
            self.queue.push_back(row);
        }
    }
    fn variable_changed(&mut self, j: usize, fixed: bool) {
        for k in 0..self.incidence[j].len() {
            let row = self.incidence[j][k];
            if fixed {
                self.budget_credit = self
                    .budget_credit
                    .saturating_add(4 * (self.model.rows[row].coefficients.len() + 1));
            }
            self.enqueue(row);
        }
    }
    fn tighten(
        &mut self,
        j: usize,
        lo: Option<f64>,
        up: Option<f64>,
        lower: &mut [Option<f64>],
        upper: &mut [Option<f64>],
    ) -> bool {
        let was_fixed = lower[j].zip(upper[j]).is_some_and(|(l, u)| l == u);
        let integer = self.model.variables[j].integer;
        let mut changed = false;
        if let Some(v) = lo {
            let v = if integer { v.ceil() } else { v };
            if improve_lower(v, lower[j], integer) {
                lower[j] = Some(v);
                self.stats.tightenings += 1;
                changed = true;
            }
        }
        if let Some(v) = up {
            let v = if integer { v.floor() } else { v };
            if improve_upper(v, upper[j], integer) {
                upper[j] = Some(v);
                self.stats.tightenings += 1;
                changed = true;
            }
        }
        if changed {
            self.variable_changed(
                j,
                !was_fixed && lower[j].zip(upper[j]).is_some_and(|(l, u)| l == u),
            );
        }
        lower[j].zip(upper[j]).is_some_and(|(l, u)| l > u)
    }
    /// changed=Noneは全行、Some(j)は親からjだけ変更されたlocal domainを表す。
    pub fn propagate(
        &mut self,
        lower: &mut [Option<f64>],
        upper: &mut [Option<f64>],
        changed: Option<usize>,
        deadline: Option<Instant>,
    ) -> Status {
        assert_eq!(lower.len(), self.model.variables.len());
        assert_eq!(upper.len(), lower.len());
        self.stats.calls += 1;
        self.budget_credit = 0;
        self.queue.clear();
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.marks.fill(0);
            self.epoch = 1;
        }
        if let Some(j) = changed {
            if lower[j].zip(upper[j]).is_some_and(|(l, u)| l > u) {
                self.stats.pruned += 1;
                return Status::Infeasible;
            }
            self.variable_changed(j, false);
        } else {
            for j in 0..lower.len() {
                if self.model.variables[j].integer {
                    let lo = lower[j].map(f64::ceil);
                    let up = upper[j].map(f64::floor);
                    if self.tighten(j, lo, up, lower, upper) {
                        self.stats.pruned += 1;
                        return Status::Infeasible;
                    }
                }
                if lower[j].zip(upper[j]).is_some_and(|(l, u)| l > u) {
                    self.stats.pruned += 1;
                    return Status::Infeasible;
                }
            }
            for i in 0..self.model.rows.len() {
                self.enqueue(i);
            }
        }
        let cap = 64usize.saturating_add(self.root_work.saturating_mul(4));
        let budget = if let Some(j) = changed {
            let work = self.incidence[j]
                .iter()
                .map(|&i| self.model.rows[i].coefficients.len() + 1)
                .sum::<usize>();
            64usize.saturating_add(work.saturating_mul(8))
        } else {
            64usize.saturating_add(self.root_work.saturating_mul(4))
        };
        let mut work = 0usize;
        while let Some(i) = self.queue.pop_front() {
            self.marks[i] = 0;
            if deadline.is_some_and(|d| Instant::now() >= d) {
                return Status::TimeLimit;
            }
            if work >= cap.min(budget.saturating_add(self.budget_credit)) {
                self.stats.budget_hits += 1;
                return Status::Limited;
            }
            let row = &self.model.rows[i];
            let n = row.coefficients.len();
            work = work.saturating_add(n + 1);
            self.stats.rows += 1;
            self.prefix_lo.resize(n + 1, 0.0);
            self.prefix_hi.resize(n + 1, 0.0);
            self.suffix_lo.resize(n + 1, 0.0);
            self.suffix_hi.resize(n + 1, 0.0);
            self.prefix_lo[0] = 0.0;
            self.prefix_hi[0] = 0.0;
            self.suffix_lo[n] = 0.0;
            self.suffix_hi[n] = 0.0;
            for k in 0..n {
                let (j, a) = row.coefficients[k];
                let (lo, hi) = term(a, lower[j], upper[j]);
                self.prefix_lo[k + 1] = add_lo(self.prefix_lo[k], lo);
                self.prefix_hi[k + 1] = add_hi(self.prefix_hi[k], hi);
            }
            for k in (0..n).rev() {
                let (j, a) = row.coefficients[k];
                let (lo, hi) = term(a, lower[j], upper[j]);
                self.suffix_lo[k] = add_lo(lo, self.suffix_lo[k + 1]);
                self.suffix_hi[k] = add_hi(hi, self.suffix_hi[k + 1]);
            }
            let rhs_lo = row.lower.map(|v| (v - ROW_TOL).next_down());
            let rhs_hi = row.upper.map(|v| (v + ROW_TOL).next_up());
            if rhs_hi.is_some_and(|v| self.prefix_lo[n] > v)
                || rhs_lo.is_some_and(|v| self.prefix_hi[n] < v)
            {
                self.stats.pruned += 1;
                return Status::Infeasible;
            }
            // 古いdomainのprefix/suffixは、更新後にも外側の安全な区間である。
            for k in 0..n {
                let (j, a) = row.coefficients[k];
                if a == 0.0 {
                    continue;
                }
                let minimum = add_lo(self.prefix_lo[k], self.suffix_lo[k + 1]);
                let maximum = add_hi(self.prefix_hi[k], self.suffix_hi[k + 1]);
                let mut lo = None;
                let mut up = None;
                if let Some(rhs) = rhs_hi {
                    let numerator = (rhs - minimum).next_up();
                    if a > 0.0 {
                        up = Some((numerator / a).next_up());
                    } else {
                        lo = Some((numerator / a).next_down());
                    }
                }
                if let Some(lhs) = rhs_lo {
                    let numerator = (lhs - maximum).next_down();
                    if a > 0.0 {
                        lo = Some((numerator / a).next_down());
                    } else {
                        up = Some((numerator / a).next_up());
                    }
                }
                if self.tighten(j, lo, up, lower, upper) {
                    self.stats.pruned += 1;
                    return Status::Infeasible;
                }
            }
        }
        Status::Stable
    }
}
