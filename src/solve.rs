//! Initial depth-first branch-and-bound over the Rust LP reference engine.
use crate::{lp, model::Model};
use serde::Serialize;
use std::time::Instant;
#[derive(Clone)]
pub struct Limits {
    pub seconds: f64,
    pub nodes: usize,
    pub pivots: usize,
    pub dense_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            seconds: 60.0,
            nodes: 100_000,
            pivots: 100_000,
            dense_bytes: 256 * 1024 * 1024,
        }
    }
}
#[derive(Serialize, Debug)]
pub struct Result {
    pub status: String,
    pub objective: Option<f64>,
    pub dual_bound: Option<f64>,
    pub gap: Option<f64>,
    pub nodes: usize,
    pub seconds: f64,
    pub values: Option<std::collections::BTreeMap<String, f64>>,
}
#[derive(Clone)]
struct Node {
    lower: Vec<Option<f64>>,
    upper: Vec<Option<f64>>,
    bound: f64,
}
struct Standard {
    a: Vec<Vec<f64>>,
    b: Vec<f64>,
    c: Vec<f64>,
    shift: Vec<f64>,
    terms: Vec<Vec<(usize, f64)>>,
}
enum Relaxation {
    Ready(Standard),
    Infeasible,
    ResourceLimit,
}
fn standard(model: &Model, node: &Node, limit: usize) -> Relaxation {
    let mut shift = vec![];
    let mut terms = vec![];
    let mut c = vec![];
    let sign = if model.maximize { -1.0 } else { 1.0 };
    for (j, v) in model.variables.iter().enumerate() {
        let lo = node.lower[j];
        let up = node.upper[j];
        if lo.zip(up).is_some_and(|(l, u)| l > u) {
            return Relaxation::Infeasible;
        }
        if let Some(l) = lo {
            shift.push(l);
            terms.push(vec![(c.len(), 1.0)]);
            c.push(sign * v.objective);
        } else if let Some(u) = up {
            shift.push(u);
            terms.push(vec![(c.len(), -1.0)]);
            c.push(-sign * v.objective);
        } else {
            shift.push(0.0);
            terms.push(vec![(c.len(), 1.0), (c.len() + 1, -1.0)]);
            c.push(sign * v.objective);
            c.push(-sign * v.objective);
        }
    }
    let rows = model
        .rows
        .iter()
        .map(|r| usize::from(r.lower.is_some()) + usize::from(r.upper.is_some()))
        .sum::<usize>()
        + node
            .lower
            .iter()
            .zip(&node.upper)
            .filter(|(l, u)| l.is_some() && u.is_some())
            .count();
    let cols = c.len();
    // Includes dense input and the tableau, conservatively capped before allocation.
    let bytes = (rows as u128 + 1) * (cols as u128 + 2 * rows as u128 + 1) * 8
        + rows as u128 * cols as u128 * 8;
    if bytes > limit as u128 {
        return Relaxation::ResourceLimit;
    }
    let mut a = Vec::with_capacity(rows);
    let mut b = Vec::with_capacity(rows);
    for row in &model.rows {
        let mut coeff = vec![0.0; cols];
        let mut offset = 0.0;
        for &(j, v) in &row.coefficients {
            offset += v * shift[j];
            for &(k, s) in &terms[j] {
                coeff[k] += v * s;
            }
        }
        if let Some(u) = row.upper {
            a.push(coeff.clone());
            b.push(u - offset);
        }
        if let Some(l) = row.lower {
            a.push(coeff.iter().map(|x| -x).collect());
            b.push(offset - l);
        }
    }
    for j in 0..model.variables.len() {
        if let (Some(l), Some(u)) = (node.lower[j], node.upper[j]) {
            let mut r = vec![0.0; cols];
            r[terms[j][0].0] = 1.0;
            a.push(r);
            b.push(u - l);
        }
    }
    Relaxation::Ready(Standard {
        a,
        b,
        c,
        shift,
        terms,
    })
}
pub fn solve(model: &Model, limits: &Limits) -> Result {
    let start = Instant::now();
    let deadline = start + std::time::Duration::from_secs_f64(limits.seconds);
    let sign = if model.maximize { -1.0 } else { 1.0 };
    let mut lower: Vec<_> = model.variables.iter().map(|v| v.lower).collect();
    let mut upper: Vec<_> = model.variables.iter().map(|v| v.upper).collect();
    for (j, v) in model.variables.iter().enumerate() {
        if v.integer {
            lower[j] = lower[j].map(f64::ceil);
            upper[j] = upper[j].map(f64::floor);
        }
    }
    let mut stack = vec![Node {
        lower,
        upper,
        bound: f64::NEG_INFINITY,
    }];
    let mut nodes = 0;
    let mut incumbent = f64::INFINITY;
    let mut best = None;
    let mut status = "optimal".to_string();
    let mut terminal_bound = f64::INFINITY;
    while let Some(mut node) = stack.pop() {
        if node.bound >= incumbent {
            continue;
        }
        let interrupt = if Instant::now() >= deadline {
            Some("time_limit")
        } else if nodes >= limits.nodes {
            Some("node_limit")
        } else {
            None
        };
        if let Some(s) = interrupt {
            stack.push(node);
            status = s.into();
            break;
        }
        let p = match standard(model, &node, limits.dense_bytes) {
            Relaxation::Infeasible => continue,
            Relaxation::ResourceLimit => {
                stack.push(node);
                status = "resource_limit".into();
                break;
            }
            Relaxation::Ready(p) => p,
        };
        nodes += 1;
        let answer = lp::minimize_until(&p.a, &p.b, &p.c, limits.pivots, Some(deadline));
        match answer {
            Ok(lp::Outcome::Infeasible) => continue,
            Ok(lp::Outcome::Optimal { x, .. }) => {
                let original: Vec<f64> = p
                    .terms
                    .iter()
                    .enumerate()
                    .map(|(j, t)| p.shift[j] + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>())
                    .collect();
                if model.check(&original, false, 1e-6).is_err() {
                    stack.push(node);
                    status = "numerical_error".into();
                    break;
                }
                let bound = sign * model.objective(&original);
                // Small negative roundoff in a child must not decrease inherited bounds.
                node.bound = node.bound.max(bound);
                if node.bound >= incumbent {
                    continue;
                }
                let branch = model
                    .variables
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| v.integer)
                    .map(|(j, _)| (j, (original[j] - original[j].round()).abs()))
                    .filter(|(_, f)| *f > 1e-7)
                    .max_by(|a, b| a.1.total_cmp(&b.1));
                if let Some((j, _)) = branch {
                    let mut up = node.clone();
                    up.upper[j] = Some(
                        up.upper[j].map_or(original[j].floor(), |v| v.min(original[j].floor())),
                    );
                    let mut down = node;
                    down.lower[j] = Some(
                        down.lower[j].map_or(original[j].ceil(), |v| v.max(original[j].ceil())),
                    );
                    stack.push(down);
                    stack.push(up);
                } else {
                    let mut rounded = original;
                    for (j, v) in model.variables.iter().enumerate() {
                        if v.integer {
                            rounded[j] = rounded[j].round();
                        }
                    }
                    if model.check(&rounded, true, 1e-6).is_err() {
                        stack.push(node);
                        status = "numerical_error".into();
                        break;
                    }
                    let value = sign * model.objective(&rounded);
                    terminal_bound = terminal_bound.min(node.bound.min(value));
                    if value < incumbent {
                        incumbent = value;
                        best = Some(rounded);
                    }
                }
            }
            other => {
                status = match other {
                    Ok(lp::Outcome::Unbounded) => "unbounded_relaxation",
                    Ok(lp::Outcome::TimeLimit) => "time_limit",
                    Ok(lp::Outcome::IterationLimit) => "iteration_limit",
                    _ => "numerical_error",
                }
                .into();
                // An unbounded LP does not imply an unbounded MILP.
                stack.push(node);
                break;
            }
        }
    }
    let internal_dual = stack
        .iter()
        .map(|n| n.bound)
        .fold(terminal_bound.min(incumbent), f64::min);
    if status == "optimal" && best.is_none() {
        status = "infeasible".into();
    }
    let objective = best.as_ref().map(|x| model.objective(x));
    let dual_bound = if internal_dual.is_finite() {
        Some(sign * internal_dual)
    } else {
        None
    };
    let gap = objective
        .zip(dual_bound)
        .map(|(p, d)| (p - d).abs() / p.abs().max(1.0));
    if status == "optimal" && gap.is_some_and(|g| g > 1e-7) {
        status = "numerical_error".into();
    }
    Result {
        status,
        objective,
        dual_bound,
        gap,
        nodes,
        seconds: start.elapsed().as_secs_f64(),
        values: best.map(|x| {
            model
                .variables
                .iter()
                .zip(x)
                .map(|(v, x)| (v.name.clone(), x))
                .collect()
        }),
    }
}
