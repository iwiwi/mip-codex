use crate::{
    lp::{self, LpStatus},
    model::Model,
};
use serde::Serialize;
use std::time::{Duration, Instant};
#[derive(Serialize, Debug)]
pub struct Solution {
    pub status: String,
    pub objective: Option<f64>,
    pub dual_bound: Option<f64>,
    pub values: Option<Vec<f64>>,
    pub nodes: u64,
    pub seconds: f64,
}
struct Node {
    lower: Vec<f64>,
    upper: Vec<f64>,
    bound: f64,
}
enum LpResult {
    Optimal { objective: f64, values: Vec<f64> },
    Infeasible,
    Unbounded,
    Limit,
    NumericalFailure,
}
fn relaxation(m: &Model, lo: &[f64], up: &[f64], deadline: Instant) -> LpResult {
    let result = lp::solve(m, lo, up, deadline);
    match result.status {
        LpStatus::Optimal => LpResult::Optimal {
            objective: m.sense * result.objective,
            values: result.values,
        },
        LpStatus::Infeasible => LpResult::Infeasible,
        LpStatus::Unbounded => LpResult::Unbounded,
        LpStatus::Limit => LpResult::Limit,
        LpStatus::Numerical => LpResult::NumericalFailure,
    }
}
pub fn feasible(m: &Model, x: &[f64]) -> bool {
    if x.len() != m.variables.len() || x.iter().any(|v| !v.is_finite()) {
        return false;
    }
    for (v, &x) in m.variables.iter().zip(x) {
        if x < v.lower - 1e-7 || x > v.upper + 1e-7 || v.integer && (x - x.round()).abs() > 1e-7 {
            return false;
        }
    }
    m.rows.iter().all(|r| {
        let activity = r.terms.iter().map(|&(j, v)| v * x[j]).sum::<f64>();
        activity >= r.lower - 1e-7 && activity <= r.upper + 1e-7
    })
}
pub fn solve(m: &Model, seconds: f64) -> Solution {
    let start = Instant::now();
    let deadline = start + Duration::from_secs_f64(seconds);
    let sense = m.sense;
    let mut stack = vec![Node {
        lower: m.variables.iter().map(|v| v.lower).collect(),
        upper: m.variables.iter().map(|v| v.upper).collect(),
        bound: f64::NEG_INFINITY,
    }];
    let (mut best, mut incumbent, mut nodes) = (f64::INFINITY, None, 0);
    let mut status = "optimal";
    while let Some(node) = stack.pop() {
        if Instant::now() >= deadline {
            stack.push(node);
            status = "time_limit";
            break;
        }
        if node.bound >= best {
            continue;
        }
        nodes += 1;
        match relaxation(m, &node.lower, &node.upper, deadline) {
            LpResult::Optimal {
                objective,
                mut values,
            } => {
                if objective >= best {
                    continue;
                }
                let branch = m
                    .variables
                    .iter()
                    .zip(&values)
                    .enumerate()
                    .filter(|(_, (v, _))| v.integer)
                    .map(|(j, (_, x))| (j, (x - x.round()).abs()))
                    .filter(|(_, f)| *f > 1e-7)
                    .max_by(|a, b| a.1.total_cmp(&b.1));
                if let Some((j, _)) = branch {
                    let mut down = Node {
                        lower: node.lower.clone(),
                        upper: node.upper.clone(),
                        bound: objective,
                    };
                    let mut up = Node {
                        lower: node.lower,
                        upper: node.upper,
                        bound: objective,
                    };
                    down.upper[j] = down.upper[j].min(values[j].floor());
                    up.lower[j] = up.lower[j].max(values[j].ceil());
                    stack.push(up);
                    stack.push(down);
                } else {
                    for (v, x) in m.variables.iter().zip(&mut values) {
                        if v.integer {
                            *x = x.round();
                        }
                    }
                    if !feasible(m, &values) {
                        stack.push(Node {
                            bound: objective,
                            ..node
                        });
                        status = "numerical_failure";
                        break;
                    }
                    best = sense
                        * (m.offset
                            + m.variables
                                .iter()
                                .zip(&values)
                                .map(|(v, x)| v.cost * x)
                                .sum::<f64>());
                    incumbent = Some(values);
                }
            }
            LpResult::Infeasible => {}
            other => {
                status = match other {
                    LpResult::Limit => "resource_limit",
                    LpResult::Unbounded => "unbounded_relaxation",
                    _ => "numerical_failure",
                };
                stack.push(node);
                break;
            }
        }
    }
    if stack.is_empty() && incumbent.is_none() {
        status = "infeasible";
    }
    let dual = stack.iter().map(|n| n.bound).fold(best, f64::min);
    Solution {
        status: status.into(),
        objective: best.is_finite().then_some(sense * best),
        dual_bound: dual.is_finite().then_some(sense * dual),
        values: incumbent,
        nodes,
        seconds: start.elapsed().as_secs_f64(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binary_knapsack() {
        let m=crate::model::parse_mps("NAME toy\nROWS\n N obj\n L cap\nCOLUMNS\n x obj -3 cap 2\n y obj -2 cap 2\nRHS\n r cap 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n").unwrap();
        let s = solve(&m, 1.);
        assert_eq!(s.status, "optimal");
        assert_eq!(s.objective, Some(-3.));
        assert!(feasible(&m, s.values.as_ref().unwrap()));
    }
}
