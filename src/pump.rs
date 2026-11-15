//! 主解専用L1射影モデル。局所状態は元Modelや探索木へ書き戻さない。
use crate::model::{Model, Row, Variable};
use std::collections::HashSet;
pub(crate) struct Projection {
    pub model: Model,
    pub original_len: usize,
    pub binary: usize,
    pub general: usize,
    integers: Vec<usize>,
    sides: Vec<Option<(usize, usize)>>,
    seen: HashSet<Vec<i64>>,
}
impl Projection {
    pub fn new(model: &Model, lower: &[Option<f64>], upper: &[Option<f64>]) -> Option<Self> {
        if lower.len() != model.variables.len() || upper.len() != lower.len() {
            return None;
        }
        let mut work = model.clone();
        work.maximize = false;
        work.offset = 0.;
        work.offset_enclosure = None;
        for v in &mut work.variables {
            v.objective = 0.;
            v.objective_enclosure = None;
            v.integer = false;
        }
        let mut integers = Vec::new();
        let mut sides = Vec::new();
        let mut binary = 0;
        let mut general = 0;
        for (j, v) in model.variables.iter().enumerate() {
            if !v.integer || lower[j].zip(upper[j]).is_some_and(|(l, u)| l == u) {
                continue;
            }
            integers.push(j);
            if lower[j]
                .zip(upper[j])
                .is_some_and(|(l, u)| l.fract() == 0. && u - l == 1.)
            {
                binary += 1;
                sides.push(None);
                continue;
            }
            general += 1;
            let d = work.variables.len();
            work.variables.push(Variable {
                name: format!("__pump_d{j}"),
                lower: Some(0.),
                upper: None,
                integer: false,
                objective: 1.,
                objective_enclosure: None,
                lower_enclosure: None,
                upper_enclosure: None,
            });
            let i = work.rows.len();
            for sign in [1., -1.] {
                work.rows.push(Row {
                    name: format!("__pump_r{j}_{sign}"),
                    lower: None,
                    upper: Some(0.),
                    coefficients: vec![(j, sign), (d, -1.)],
                    exact_integral: false,
                    lower_enclosure: None,
                    upper_enclosure: None,
                });
            }
            sides.push(Some((i, i + 1)));
        }
        if integers.is_empty() {
            return None;
        }
        Some(Self {
            model: work,
            original_len: model.variables.len(),
            binary,
            general,
            integers,
            sides,
            seen: HashSet::new(),
        })
    }
    pub fn target(
        &mut self,
        point: &[f64],
        lower: &[Option<f64>],
        upper: &[Option<f64>],
        iteration: usize,
    ) -> Option<(Vec<f64>, bool, usize)> {
        if point.len() != self.original_len {
            return None;
        }
        let mut target = Vec::new();
        for &j in &self.integers {
            let mut value = point[j].round();
            if let Some(l) = lower[j] {
                value = value.max(l.ceil());
            }
            if let Some(u) = upper[j] {
                value = value.min(u.floor());
            }
            if !value.is_finite() || value.abs() > ((1u64 << 50) as f64) {
                return None;
            }
            target.push(value as i64);
        }
        let cycle = self.seen.contains(&target);
        let mut changed = 0;
        if cycle {
            let mut order: Vec<_> = (0..target.len()).collect();
            let mut seed = (iteration as u64).wrapping_add(2026111515);
            for k in (1..order.len()).rev() {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                order.swap(k, ((seed >> 32) as usize) % (k + 1));
            }
            for k in order.into_iter().take((1 + target.len() / 20).min(32)) {
                let j = self.integers[k];
                let value = target[k] as f64;
                let up = upper[j].is_none_or(|u| value + 1. <= u.floor());
                let down = lower[j].is_none_or(|l| value - 1. >= l.ceil());
                let step = if up && (!down || (iteration + k) % 2 == 0) {
                    1
                } else if down {
                    -1
                } else {
                    0
                };
                target[k] += step;
                changed += usize::from(step != 0);
            }
        }
        self.seen.insert(target.clone());
        let mut candidate = point.to_vec();
        for ((&j, side), &value) in self.integers.iter().zip(&self.sides).zip(&target) {
            candidate[j] = value as f64;
            if let Some((a, b)) = side {
                self.model.rows[*a].upper = Some(value as f64);
                self.model.rows[*b].upper = Some(-(value as f64));
            } else {
                self.model.variables[j].objective = if lower[j].is_some_and(|l| value as f64 == l) {
                    1.
                } else {
                    -1.
                };
            }
        }
        Some((candidate, cycle, changed))
    }
}
