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
