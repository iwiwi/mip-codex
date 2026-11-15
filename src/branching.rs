//! 分枝の順位だけを学習する。擬似コストを下界/枝刈りには使わない。
use crate::model::Model;
#[derive(Clone, Copy, Debug)]
pub enum Rule {
    Fractional,
    Pseudocost,
}
#[derive(Clone, Copy, Debug)]
pub enum Order {
    Down,
    Nearest,
    Estimate,
}
#[derive(Clone, Copy, Debug)]
pub struct Edge {
    pub variable: usize,
    pub up: bool,
    pub distance: f64,
    pub parent_objective: f64,
}
#[derive(Clone, Copy, Default)]
struct Mean {
    value: f64,
    count: usize,
}
impl Mean {
    fn add(&mut self, value: f64) {
        self.count += 1;
        self.value += (value - self.value) / self.count as f64;
    }
}
pub struct Costs {
    local: Vec<[Mean; 2]>,
    global: [Mean; 2],
    pub updates: usize,
    pub selections: usize,
}
impl Costs {
    pub fn new(n: usize) -> Self {
        Self {
            local: vec![[Mean::default(); 2]; n],
            global: [Mean::default(); 2],
            updates: 0,
            selections: 0,
        }
    }
    pub fn observe(&mut self, edge: Edge, objective: f64) {
        if !objective.is_finite() || !edge.parent_objective.is_finite() || edge.distance < 1e-7 {
            return;
        }
        let cost = (objective - edge.parent_objective).max(0.) / edge.distance;
        if !cost.is_finite() {
            return;
        }
        let direction = usize::from(edge.up);
        self.local[edge.variable][direction].add(cost);
        self.global[direction].add(cost);
        self.updates += 1;
    }
    fn prior(&self) -> f64 {
        let total = self.global[0].count + self.global[1].count;
        if total == 0 {
            0.
        } else {
            self.global[0].value * (self.global[0].count as f64 / total as f64)
                + self.global[1].value * (self.global[1].count as f64 / total as f64)
        }
    }
    fn mean(&self, j: usize, up: bool) -> f64 {
        let d = usize::from(up);
        let prior = if self.global[d].count > 0 {
            self.global[d].value
        } else {
            self.prior()
        };
        let local = self.local[j][d];
        // 二つ分の全体平均をpriorとし、少数観測の過信を避ける。
        let weight = local.count as f64 / (local.count as f64 + 2.);
        weight * local.value + (1. - weight) * prior
    }
    pub fn select(&mut self, model: &Model, x: &[f64], rule: Rule) -> Option<usize> {
        let use_cost = matches!(rule, Rule::Pseudocost) && self.prior() > 1e-12;
        let floor = self.prior() * 1e-6;
        let result = model
            .variables
            .iter()
            .enumerate()
            .filter(|(_, v)| v.integer)
            .filter_map(|(j, _)| {
                let f = x[j] - x[j].floor();
                let fraction = f.min(1. - f);
                if fraction <= 1e-7 {
                    return None;
                }
                let score = if use_cost {
                    (self.mean(j, false) * f).max(floor).ln()
                        + (self.mean(j, true) * (1. - f)).max(floor).ln()
                } else {
                    fraction
                };
                Some((j, score, fraction))
            })
            .max_by(|a, b| {
                a.1.total_cmp(&b.1)
                    .then(a.2.total_cmp(&b.2))
                    .then(a.0.cmp(&b.0))
            })
            .map(|(j, _, _)| j);
        if result.is_some() && use_cost {
            self.selections += 1;
        }
        result
    }
    pub fn prefer_up(&self, j: usize, value: f64, order: Order) -> bool {
        let f = value - value.floor();
        match order {
            Order::Down => false,
            Order::Nearest => f > 0.5,
            Order::Estimate => {
                if self.prior() <= 1e-12 {
                    f > 0.5
                } else {
                    self.mean(j, true) * (1. - f) < self.mean(j, false) * f
                }
            }
        }
    }
}

/// 強分枝の短い候補列。分数度の多様性を保ち目的影響の上位も早めに試す。
pub(crate) fn strong_candidates(model: &Model, point: &[f64], mixed: bool) -> Vec<(usize, f64)> {
    let mut fractional: Vec<_> = model
        .variables
        .iter()
        .enumerate()
        .filter_map(|(j, v)| {
            let f = point[j] - point[j].floor();
            let d = f.min(1. - f);
            (v.integer && d > 1e-7).then_some((j, d))
        })
        .collect();
    fractional.sort_by(|a, b| b.1.total_cmp(&a.1).then(b.0.cmp(&a.0)));
    if !mixed {
        fractional.truncate(8);
        return fractional;
    }
    let mut objective: Vec<_> = fractional
        .iter()
        .copied()
        .filter(|(j, d)| model.variables[*j].objective.abs() * d > 0.)
        .collect();
    objective.sort_by(|a, b| {
        let impact = |(j, d): (usize, f64)| model.variables[j].objective.abs() * d;
        impact(*b).total_cmp(&impact(*a)).then(b.0.cmp(&a.0))
    });
    objective.truncate(2);
    let mut result = Vec::new();
    let mut add = |candidate: (usize, f64)| {
        if result.len() < 8 && !result.iter().any(|&(j, _)| j == candidate.0) {
            result.push(candidate);
        }
    };
    for k in 0..2 {
        if let Some(&v) = fractional.get(k) {
            add(v);
        }
        if let Some(&v) = objective.get(k) {
            add(v);
        }
    }
    for v in fractional {
        add(v);
    }
    result
}

#[cfg(test)]
mod strong_candidate_tests {
    use super::*;
    #[test]
    fn mixed_candidates_keep_fractional_variables_and_include_rare_cost_variable() {
        let mut text = String::from("NAME s\nROWS\n N c\n L r\nCOLUMNS\n");
        for j in 0..10 {
            text += &format!(" x{j} c {} r 1\n", if j == 9 { 100 } else { 0 });
        }
        text += "RHS\n rhs r 10\nBOUNDS\n";
        for j in 0..10 {
            text += &format!(" BV b x{j}\n");
        }
        text += "ENDATA\n";
        let mut model = crate::mps::parse(std::io::Cursor::new(text)).unwrap();
        let mut point = vec![0.5; 10];
        point[8] = 0.4;
        point[9] = 0.01;
        for max in [false, true] {
            model.maximize = max;
            let classic = strong_candidates(&model, &point, false);
            let mixed = strong_candidates(&model, &point, true);
            assert_eq!(classic.len(), 8);
            assert_eq!(mixed.len(), 8);
            assert!(!classic.iter().any(|&(j, _)| j == 9));
            assert_eq!(mixed[1].0, 9);
            assert!(
                classic
                    .iter()
                    .take(6)
                    .all(|&(j, _)| mixed.iter().any(|&(k, _)| j == k))
            );
        }
    }
}
