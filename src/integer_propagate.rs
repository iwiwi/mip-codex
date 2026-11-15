//! 元十進の整数性とpoint境界が分かる行だけをchecked i128で伝播する。
use crate::{model::Model, propagate::Status};
use serde::Serialize;
use std::{collections::VecDeque, time::Instant};
const LIMIT: i128 = 1i128 << 52;
fn integer(x: f64) -> Option<i128> {
    (x.is_finite() && x.fract() == 0. && x.abs() <= LIMIT as f64).then_some(x as i128)
}
fn original_bound(value: Option<f64>, enclosure: Option<[f64; 2]>) -> bool {
    value.is_none_or(|v| integer(v).is_some() && enclosure == Some([v, v]))
}
fn side(value: Option<f64>, enclosure: Option<[f64; 2]>) -> Option<i128> {
    let v = value?;
    (enclosure == Some([v, v])).then_some(())?;
    integer(v)
}
#[derive(Default, Clone, Debug, Serialize)]
pub struct Stats {
    pub calls: usize,
    pub eligible_rows: usize,
    pub inequalities: usize,
    pub rows: usize,
    pub tightenings: usize,
    pub pruned: usize,
    pub budget_hits: usize,
    pub unsupported_domains: usize,
    pub overflows: usize,
}
struct Inequality {
    coefficients: Vec<(usize, i128)>,
    rhs: i128,
}
pub struct IntegerPropagator<'a> {
    model: &'a Model,
    rows: Vec<Inequality>,
    incidence: Vec<Vec<usize>>,
    queue: VecDeque<usize>,
    marks: Vec<u64>,
    epoch: u64,
    root_work: usize,
    pub stats: Stats,
}
impl<'a> IntegerPropagator<'a> {
    pub fn new(model: &'a Model, max_bytes: usize) -> Option<Self> {
        let nnz = model
            .rows
            .iter()
            .try_fold(0usize, |n, r| n.checked_add(r.coefficients.len()))?;
        let bytes = nnz
            .checked_mul(128)?
            .checked_add(model.rows.len().checked_mul(256)?)?
            .checked_add(model.variables.len().checked_mul(32)?)?;
        if bytes > max_bytes {
            return None;
        }
        let allowed: Vec<_> = model
            .variables
            .iter()
            .map(|v| {
                original_bound(v.lower, v.lower_enclosure)
                    && original_bound(v.upper, v.upper_enclosure)
            })
            .collect();
        let mut rows = Vec::new();
        let mut incidence = vec![vec![]; model.variables.len()];
        let mut stats = Stats::default();
        let mut root_work = 0usize;
        for row in &model.rows {
            if !row.exact_integral
                || row
                    .coefficients
                    .iter()
                    .any(|&(j, a)| !allowed[j] || integer(a).is_none())
            {
                continue;
            }
            let first = rows.len();
            for (sign, rhs) in [
                (1, side(row.upper, row.upper_enclosure)),
                (-1, side(row.lower, row.lower_enclosure)),
            ] {
                let Some(rhs) = rhs else { continue };
                let coefficients: Vec<_> = row
                    .coefficients
                    .iter()
                    .filter(|&&(_, a)| a != 0.)
                    .map(|&(j, a)| (j, sign * integer(a).unwrap()))
                    .collect();
                if coefficients.iter().any(|&(j, a)| {
                    !model.variables[j].integer
                        && if a > 0 {
                            model.variables[j].lower.is_none()
                        } else {
                            model.variables[j].upper.is_none()
                        }
                }) {
                    continue;
                }
                let i = rows.len();
                for &(j, _) in &coefficients {
                    incidence[j].push(i);
                }
                root_work = root_work.saturating_add(coefficients.len() + 1);
                rows.push(Inequality {
                    coefficients,
                    rhs: sign * rhs,
                });
            }
            stats.eligible_rows += usize::from(rows.len() != first);
        }
        stats.inequalities = rows.len();
        let marks = vec![0; rows.len()];
        Some(Self {
            model,
            rows,
            incidence,
            queue: VecDeque::new(),
            marks,
            epoch: 0,
            root_work,
            stats,
        })
    }
    fn enqueue(&mut self, i: usize) {
        if self.marks[i] != self.epoch {
            self.marks[i] = self.epoch;
            self.queue.push_back(i);
        }
    }
    fn changed(&mut self, j: usize) {
        for k in 0..self.incidence[j].len() {
            self.enqueue(self.incidence[j][k]);
        }
    }
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
        self.queue.clear();
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.marks.fill(0);
            self.epoch = 1;
        }
        if let Some(j) = changed {
            self.changed(j);
        } else {
            for i in 0..self.rows.len() {
                self.enqueue(i);
            }
        }
        let cap = 64usize.saturating_add(self.root_work.saturating_mul(4));
        let mut work = 0usize;
        while let Some(i) = self.queue.pop_front() {
            self.marks[i] = 0;
            if deadline.is_some_and(|d| Instant::now() >= d) {
                return Status::TimeLimit;
            }
            if work >= cap {
                self.stats.budget_hits += 1;
                return Status::Limited;
            }
            work = work.saturating_add(self.rows[i].coefficients.len() + 1);
            self.stats.rows += 1;
            let row = &self.rows[i];
            let mut sum = 0i128;
            let mut unknown = 0usize;
            let mut terms = Vec::with_capacity(row.coefficients.len());
            let mut unsupported = false;
            for &(j, a) in &row.coefficients {
                let bound = if a > 0 { lower[j] } else { upper[j] };
                let term = if let Some(value) = bound {
                    let Some(value) = integer(value) else {
                        unsupported = true;
                        break;
                    };
                    let Some(t) = a.checked_mul(value) else {
                        self.stats.overflows += 1;
                        unsupported = true;
                        break;
                    };
                    let Some(next) = sum.checked_add(t) else {
                        self.stats.overflows += 1;
                        unsupported = true;
                        break;
                    };
                    sum = next;
                    Some(t)
                } else {
                    unknown += 1;
                    None
                };
                terms.push(term);
            }
            if unsupported {
                self.stats.unsupported_domains += 1;
                continue;
            }
            if unknown == 0 && sum > row.rhs {
                self.stats.pruned += 1;
                return Status::Infeasible;
            }
            let mut updates = Vec::new();
            for (k, &(j, a)) in row.coefficients.iter().enumerate() {
                if !self.model.variables[j].integer {
                    continue;
                }
                let others = if unknown == 0 {
                    sum.checked_sub(terms[k].unwrap())
                } else if unknown == 1 && terms[k].is_none() {
                    Some(sum)
                } else {
                    continue;
                };
                let Some(residual) = others.and_then(|n| row.rhs.checked_sub(n)) else {
                    self.stats.overflows += 1;
                    continue;
                };
                let value = if a > 0 {
                    Some(residual.div_euclid(a))
                } else {
                    residual.div_euclid(-a).checked_neg()
                };
                let Some(value) = value.filter(|v| (-LIMIT..=LIMIT).contains(v)) else {
                    continue;
                };
                let value = value as f64;
                let improves = if a > 0 {
                    upper[j].is_none_or(|old| value < old)
                } else {
                    lower[j].is_none_or(|old| value > old)
                };
                if improves {
                    if a > 0 {
                        upper[j] = Some(value);
                    } else {
                        lower[j] = Some(value);
                    }
                    self.stats.tightenings += 1;
                    updates.push(j);
                }
                if lower[j].zip(upper[j]).is_some_and(|(l, u)| l > u) {
                    self.stats.pruned += 1;
                    return Status::Infeasible;
                }
            }
            for j in updates {
                self.changed(j);
            }
        }
        Status::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_product_overflow_never_tightens_or_prunes() {
        let model=crate::mps::parse(std::io::Cursor::new("NAME overflow\nROWS\n N c\n L r\nCOLUMNS\n x c 0 r 1\nRHS\n rhs r 10\nBOUNDS\n LI b x 0\n UP b x 3\nENDATA\n")).unwrap();
        let mut p = IntegerPropagator::new(&model, 1 << 20).unwrap();
        p.rows[0].coefficients[0].1 = i128::MAX;
        let mut lo = vec![Some(2.)];
        let mut up = vec![Some(3.)];
        assert_eq!(p.propagate(&mut lo, &mut up, None, None), Status::Stable);
        assert_eq!(lo, vec![Some(2.)]);
        assert_eq!(up, vec![Some(3.)]);
        assert_eq!(p.stats.overflows, 1);
        assert_eq!(p.stats.pruned, 0);
        assert_eq!(p.stats.tightenings, 0);
    }
}
