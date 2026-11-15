//! Initial depth-first branch-and-bound over the Rust LP reference engine.
use crate::{lp, model::Model, revised};
use serde::Serialize;
use std::time::Instant;
#[derive(Clone, Copy, Debug)]
pub enum Engine {
    Dense,
    Revised,
}

#[derive(Clone)]
pub struct Limits {
    pub seconds: f64,
    pub nodes: usize,
    pub pivots: usize,
    pub dense_bytes: usize,
    pub engine: Engine,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            seconds: 60.0,
            nodes: usize::MAX,
            pivots: usize::MAX,
            dense_bytes: 256 * 1024 * 1024,
            engine: Engine::Revised,
        }
    }
}
#[derive(Serialize, Debug)]
pub struct Result {
    pub lp_engine: String,
    pub lp_pivots: Option<usize>,
    pub lp_flips: Option<usize>,
    pub lp_refactors: Option<usize>,
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
    sparse: Option<revised::Problem>,
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
fn standard_dense(model: &Model, node: &Node, limit: usize) -> Relaxation {
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
        sparse: None,
        a,
        b,
        c,
        shift,
        terms,
    })
}
fn standard_sparse(model: &Model, node: &Node, limit: usize) -> Relaxation {
    let mut shift = Vec::new();
    let mut terms = Vec::new();
    let mut costs = Vec::new();
    let mut upper = Vec::new();
    let sign = if model.maximize { -1.0 } else { 1.0 };
    for (j, v) in model.variables.iter().enumerate() {
        let lo = node.lower[j];
        let up = node.upper[j];
        if lo.zip(up).is_some_and(|(l, u)| l > u) {
            return Relaxation::Infeasible;
        }
        if let Some(l) = lo {
            shift.push(l);
            terms.push(vec![(costs.len(), 1.0)]);
            costs.push(sign * v.objective);
            upper.push(up.map_or(f64::INFINITY, |u| u - l));
        } else if let Some(u) = up {
            shift.push(u);
            terms.push(vec![(costs.len(), -1.0)]);
            costs.push(-sign * v.objective);
            upper.push(f64::INFINITY);
        } else {
            shift.push(0.0);
            terms.push(vec![(costs.len(), 1.0), (costs.len() + 1, -1.0)]);
            costs.push(sign * v.objective);
            costs.push(-sign * v.objective);
            upper.extend([f64::INFINITY; 2]);
        }
    }
    let nonzeros = model
        .rows
        .iter()
        .map(|r| r.coefficients.len())
        .sum::<usize>();
    let estimate =
        (nonzeros as u128) * 48 + (model.rows.len() as u128) * 2048 + (costs.len() as u128) * 128;
    if estimate > limit as u128 {
        return Relaxation::ResourceLimit;
    }
    let mut columns = vec![vec![]; costs.len()];
    let mut rhs = Vec::new();
    let mut equality = Vec::new();
    let mut slack_upper = Vec::new();
    for row in &model.rows {
        if row.lower.is_none() && row.upper.is_none() {
            continue;
        }
        let i = rhs.len();
        let mut offset = 0.0;
        let row_sign = if row.upper.is_some() { 1.0 } else { -1.0 };
        for &(j, a) in &row.coefficients {
            offset += a * shift[j];
            for &(k, s) in &terms[j] {
                columns[k].push((i, row_sign * a * s));
            }
        }
        let bound = if let Some(u) = row.upper {
            u - offset
        } else {
            offset - row.lower.unwrap()
        };
        rhs.push(bound);
        equality.push(row.lower.zip(row.upper).is_some_and(|(l, u)| l == u));
        slack_upper.push(
            row.lower
                .zip(row.upper)
                .map_or(f64::INFINITY, |(l, u)| u - l),
        );
        if row.lower.zip(row.upper).is_some_and(|(l, u)| l > u) {
            return Relaxation::Infeasible;
        }
    }
    let p = revised::Problem {
        columns,
        rhs,
        costs,
        upper,
        equality,
        slack_upper,
    };
    Relaxation::Ready(Standard {
        sparse: Some(p),
        a: vec![],
        b: vec![],
        c: vec![],
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
    let mut lp_stats = revised::Stats::default();
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
        let relaxation = match limits.engine {
            Engine::Dense => standard_dense(model, &node, limits.dense_bytes),
            Engine::Revised => standard_sparse(model, &node, limits.dense_bytes),
        };
        let p = match relaxation {
            Relaxation::Infeasible => continue,
            Relaxation::ResourceLimit => {
                stack.push(node);
                status = "resource_limit".into();
                break;
            }
            Relaxation::Ready(p) => p,
        };
        nodes += 1;
        let (answer, relaxation_dual) = if let Some(sparse) = &p.sparse {
            let answer = revised::minimize(sparse, limits.pivots, Some(deadline));
            lp_stats.pivots += answer.stats.pivots;
            lp_stats.flips += answer.stats.flips;
            lp_stats.refactors += answer.stats.refactors;
            (Ok(answer.outcome), answer.dual)
        } else {
            (
                lp::minimize_until(&p.a, &p.b, &p.c, limits.pivots, Some(deadline)),
                None,
            )
        };
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
                let objective = sign * model.objective(&original);
                if !objective.is_finite() {
                    stack.push(node);
                    status = "numerical_error".into();
                    break;
                }
                let bound = if let Some(dual) = relaxation_dual {
                    let constant = model.offset
                        + model
                            .variables
                            .iter()
                            .zip(&p.shift)
                            .map(|(v, s)| v.objective * s)
                            .sum::<f64>();
                    (dual + sign * constant).min(objective)
                } else {
                    objective
                };
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
        lp_engine: format!("{:?}", limits.engine),
        lp_pivots: matches!(limits.engine, Engine::Revised).then_some(lp_stats.pivots),
        lp_flips: matches!(limits.engine, Engine::Revised).then_some(lp_stats.flips),
        lp_refactors: matches!(limits.engine, Engine::Revised).then_some(lp_stats.refactors),
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
