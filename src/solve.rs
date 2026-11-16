//! Initial depth-first branch-and-bound over the Rust LP reference engine.
use crate::objective_bound::{Certifier, Grid};
use crate::{branching, cuts, heuristic, lp, model::Model, propagate, revised, rins, search};
use serde::Serialize;
use std::{sync::Arc, time::Instant};
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
    pub warm_start: bool,
    pub fixed_columns: bool,
    pub propagation: bool,
    pub primal_repair: bool,
    pub box_primal: bool,
    pub root_seed_repair: bool,
    pub binary_graph: bool,
    pub parity_equalities: bool,
    pub graph_local_search: bool,
    pub graph_root_coloring: bool,
    pub graph_root_prefix: bool,
    pub graph_local_diversify: bool,
    pub graph_adaptive_local: bool,
    pub graph_cover_local: bool,
    pub aggregate_cuts: bool,
    pub clique_cuts: bool,
    pub mir_cuts: bool,
    pub three_row_cuts: bool,
    pub graph_fractional_cover: bool,
    pub defer_infeasibility: bool,
    pub joint_infeasibility: bool,
    pub aggregate_infeasibility: bool,
    pub lp_diving: bool,
    pub feasibility_pump: bool,
    pub primal_probe: bool,
    pub cover_cuts: bool,
    pub cover_rounds: usize,
    pub rounding_cuts: bool,
    pub strong_branching: bool,
    pub strong_objective: bool,
    pub strong_infeasible_hint: bool,
    pub rins: bool,
    pub terminal_neighborhood: bool,
    pub objective_rounding: bool,
    pub objective_rounding_all: bool,
    pub branching: branching::Rule,
    pub child_order: branching::Order,
    pub search: search::Strategy,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            seconds: 60.0,
            nodes: usize::MAX,
            pivots: usize::MAX,
            dense_bytes: 512 * 1024 * 1024,
            engine: Engine::Revised,
            warm_start: true,
            fixed_columns: true,
            propagation: false,
            primal_repair: true,
            box_primal: true,
            root_seed_repair: true,
            binary_graph: true,
            parity_equalities: true,
            graph_local_search: true,
            graph_root_coloring: true,
            graph_root_prefix: true,
            graph_local_diversify: true,
            graph_adaptive_local: true,
            graph_cover_local: true,
            aggregate_cuts: true,
            clique_cuts: true,
            mir_cuts: true,
            three_row_cuts: true,
            graph_fractional_cover: true,
            defer_infeasibility: true,
            joint_infeasibility: true,
            aggregate_infeasibility: true,
            lp_diving: true,
            feasibility_pump: true,
            primal_probe: true,
            cover_cuts: true,
            cover_rounds: 4,
            rounding_cuts: false,
            strong_branching: true,
            strong_objective: true,
            strong_infeasible_hint: true,
            rins: true,
            terminal_neighborhood: true,
            objective_rounding: true,
            objective_rounding_all: false,
            branching: branching::Rule::Pseudocost,
            child_order: branching::Order::Down,
            search: search::Strategy::DepthFirst,
        }
    }
}
#[derive(Serialize, Debug, Default)]
pub struct Result {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parity_proof: Option<crate::parity::Proof>,
    pub binary_graph_enabled: bool,
    pub graph_cover_local_enabled: bool,
    pub aggregate_cuts_enabled: bool,
    pub clique_cuts_enabled: bool,
    pub mir_cuts_enabled: bool,
    pub three_row_cuts_enabled: bool,
    pub graph_fractional_cover_enabled: bool,
    pub binary_graph: Option<crate::binary_graph::Stats>,
    pub branching: String,
    pub child_order: String,
    pub search_strategy: String,
    pub search_pops: usize,
    pub search_best_pops: usize,
    pub search_transfers: usize,
    pub max_open_nodes: usize,
    pub pseudocost_updates: usize,
    pub pseudocost_selections: usize,
    pub primal_repair: bool,
    pub box_primal_enabled: bool,
    pub box_primal: heuristic::BoxStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_seed_repair: Option<heuristic::BoxStats>,
    pub root_incumbent_conflict: bool,
    pub lp_diving: bool,
    pub primal_probe: bool,
    pub cover_cuts_enabled: bool,
    pub cover_round_limit: usize,
    pub rounding_cuts_enabled: bool,
    pub strong_branching: bool,
    pub strong_objective: bool,
    pub strong_infeasible_hint: bool,
    pub strong: StrongStats,
    pub rins_enabled: bool,
    pub rins: RinsStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_neighborhood: Option<RinsStats>,
    pub objective_rounding: bool,
    pub objective_rounding_all: bool,
    pub objective_grid: Option<Grid>,
    pub objective_bound_stats: ObjectiveBoundStats,
    pub infeasibility_proof_calls: usize,
    pub infeasibility_proof_trials: usize,
    pub infeasibility_proof_failures: usize,
    pub infeasibility_proof_seconds: f64,
    pub cut_proofs: Vec<cuts::Cut>,
    pub cut_stats: cuts::Stats,
    pub cut_rounds: usize,
    pub cut_lp_solves: usize,
    pub cut_seconds: f64,
    pub cut_initial_lp_seconds: f64,
    pub root_lp_reused: bool,
    pub probe: ProbeStats,
    pub diving: DivingStats,
    pub feasibility_pump_enabled: bool,
    pub pump: PumpStats,
    pub repair_calls: usize,
    pub repair_candidates: usize,
    pub repair_improvements: usize,
    pub repair_moves: usize,
    pub first_primal_seconds: Option<f64>,
    pub repair_lp_solves: usize,
    pub repair_lp_candidates: usize,
    pub repair_work: usize,
    pub lp_engine: String,
    pub propagation: bool,
    pub propagation_calls: usize,
    pub propagation_rows: usize,
    pub propagation_tightenings: usize,
    pub propagation_pruned: usize,
    pub propagation_budget_hits: usize,
    pub fixed_leaves: usize,
    pub lp_solves: usize,
    pub defer_infeasibility_enabled: bool,
    pub joint_infeasibility_enabled: bool,
    pub aggregate_infeasibility_enabled: bool,
    pub deferred_infeasibilities: usize,
    pub deferred_infeasibilities_pending: usize,
    pub deferred_infeasibilities_pruned: usize,
    pub deferred_infeasibility_bound: Option<f64>,
    pub deferred_infeasibility_unbounded: bool,
    pub lp_search_termination: Option<String>,
    pub fixed_columns_enabled: bool,
    pub lp_workspace_bytes: usize,
    pub root_fixed_columns: usize,
    pub lp_scaled_columns: Option<usize>,
    pub lp_scaled_rows: Option<usize>,
    pub lp_warm_start: bool,
    pub lp_warm_attempts: Option<usize>,
    pub lp_warm_successes: Option<usize>,
    pub lp_warm_fallbacks: Option<usize>,
    pub lp_warm_rejections: Option<usize>,
    pub lp_warm_row_extensions: Option<usize>,
    pub lp_warm_cost_attempts: Option<usize>,
    pub lp_warm_cost_successes: Option<usize>,
    pub lp_multiplier_cache_hits: Option<usize>,
    pub lp_multiplier_cache_misses: Option<usize>,
    pub lp_certificate_repair_trials: Option<usize>,
    pub lp_certificate_repairs: Option<usize>,
    pub lp_restoration_attempts: Option<usize>,
    pub lp_restoration_restarts: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lp_restoration_extended_attempts: Option<usize>,
    pub lp_restoration_successes: Option<usize>,
    pub lp_restoration_infeasible_candidates: Option<usize>,
    pub lp_restoration_pivots: Option<usize>,
    pub lp_refinement_attempts: Option<usize>,
    pub lp_refinement_steps: Option<usize>,
    pub lp_refinement_successes: Option<usize>,
    pub lp_dual_pivots: Option<usize>,
    pub lp_dual_flips: Option<usize>,
    pub lp_pivots: Option<usize>,
    pub lp_feasibility_pricing_calls: Option<usize>,
    pub lp_pricing_cutoffs: Option<usize>,
    pub lp_pricing_fallbacks: Option<usize>,
    pub lp_flips: Option<usize>,
    pub lp_refactors: Option<usize>,
    pub status: String,
    #[serde(serialize_with = "crate::model::serialize_optional_solution_number")]
    pub objective: Option<f64>,
    #[serde(serialize_with = "crate::model::serialize_optional_solution_number")]
    pub dual_bound: Option<f64>,
    pub gap: Option<f64>,
    pub nodes: usize,
    pub seconds: f64,
    #[serde(serialize_with = "crate::model::serialize_solution_values")]
    pub values: Option<std::collections::BTreeMap<String, f64>>,
}
#[derive(Clone)]
struct Node {
    certified_bound: Option<i64>,
    lower: Vec<Option<f64>>,
    upper: Vec<Option<f64>>,
    bound: f64,
    warm: Option<Arc<revised::WarmStart>>,
    changed: Option<usize>,
    edge: Option<branching::Edge>,
}
struct Standard {
    row_sources: Vec<(usize, f64)>,
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
/// Only constants proved by the original box; child fixings keep their LP columns for warm compatibility.
fn root_fixed_value(v: &crate::model::Variable, original_decimal: bool) -> Option<f64> {
    let (lower, upper) = (v.lower?, v.upper?);
    if !lower.is_finite() || !upper.is_finite() {
        return None;
    }
    let (l, u) = if v.integer {
        (lower.ceil(), upper.floor())
    } else {
        (lower, upper)
    };
    if l != u {
        return None;
    }
    if original_decimal {
        let (lo, up) = (v.lower_enclosure?, v.upper_enclosure?);
        let (l0, l1, u0, u1) = if v.integer {
            (lo[0].ceil(), lo[1].ceil(), up[0].floor(), up[1].floor())
        } else {
            (lo[0], lo[1], up[0], up[1])
        };
        if l0 != l || l1 != l || u0 != u || u1 != u {
            return None;
        }
    }
    Some(l)
}
fn checked_difference(a: f64, b: f64) -> f64 {
    crate::exact::dot_error([(a, 1.), (-b, 1.)].into_iter())
        .filter(|&(_, e)| e <= 1e-9)
        .map_or(f64::NAN, |(v, _)| v)
}
fn shifted_bound(
    row: &crate::model::Row,
    shift: &[f64],
    upper: bool,
    original_decimal: bool,
    exact_shift: bool,
) -> f64 {
    let (mut bound, sign) = if upper {
        (row.upper.unwrap(), 1.)
    } else {
        (row.lower.unwrap(), -1.)
    };
    if original_decimal {
        let enclosure = if upper {
            row.upper_enclosure
        } else {
            row.lower_enclosure
        };
        let Some([lo, hi]) = enclosure else {
            return f64::NAN;
        };
        bound = if upper { hi } else { lo };
        let error = if exact_shift && row.exact_coefficients {
            0.
        } else if exact_shift {
            row.activity_error_exact_point(shift)
        } else {
            row.activity_error(shift)
        };
        if !error.is_finite() || error > 1e-9 {
            return f64::NAN;
        }
    }
    crate::exact::dot_error(
        std::iter::once((sign * bound, 1.))
            .chain(row.coefficients.iter().map(|&(j, a)| (-sign * a, shift[j]))),
    )
    .filter(|&(_, error)| error <= 1e-9)
    .map_or(f64::NAN, |(value, _)| value)
}
fn standard_dense(model: &Model, node: &Node, limit: usize, fixed_columns: bool) -> Relaxation {
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
        if fixed_columns && let Some(value) = root_fixed_value(v, model.original_decimal) {
            if lo.is_some_and(|l| value < l) || up.is_some_and(|u| value > u) {
                return Relaxation::Infeasible;
            }
            shift.push(value);
            terms.push(vec![]);
            continue;
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
    let exact_shift = shift.iter().all(|&x| crate::model::point_error(x) == 0.);
    let rows = model
        .rows
        .iter()
        .map(|r| usize::from(r.lower.is_some()) + usize::from(r.upper.is_some()))
        .sum::<usize>()
        + node
            .lower
            .iter()
            .zip(&node.upper)
            .enumerate()
            .filter(|(j, (l, u))| !terms[*j].is_empty() && l.is_some() && u.is_some())
            .count();
    let cols = c.len();
    // Dense input and both current/rebuilt tableaux are estimated before allocation.
    let bytes = 2 * (rows as u128 + 1) * (cols as u128 + 2 * rows as u128 + 1) * 8
        + rows as u128 * cols as u128 * 8;
    if bytes > limit as u128 {
        return Relaxation::ResourceLimit;
    }
    let mut a = Vec::with_capacity(rows);
    let mut b = Vec::with_capacity(rows);
    for row in &model.rows {
        let mut coeff = vec![0.0; cols];
        for &(j, v) in &row.coefficients {
            for &(k, s) in &terms[j] {
                coeff[k] += v * s;
            }
        }
        if row.upper.is_some() {
            a.push(coeff.clone());
            b.push(shifted_bound(
                row,
                &shift,
                true,
                model.original_decimal,
                exact_shift,
            ));
        }
        if row.lower.is_some() {
            a.push(coeff.iter().map(|x| -x).collect());
            b.push(shifted_bound(
                row,
                &shift,
                false,
                model.original_decimal,
                exact_shift,
            ));
        }
    }
    for j in 0..model.variables.len() {
        if let (Some(l), Some(u)) = (node.lower[j], node.upper[j])
            && !terms[j].is_empty()
        {
            let mut r = vec![0.0; cols];
            r[terms[j][0].0] = 1.0;
            a.push(r);
            b.push(checked_difference(u, l));
        }
    }
    Relaxation::Ready(Standard {
        row_sources: vec![],
        sparse: None,
        a,
        b,
        c,
        shift,
        terms,
    })
}
fn split_range(row: &crate::model::Row) -> bool {
    row.lower.zip(row.upper).is_some_and(|(l, u)| {
        let width = u - l;
        l < u && (!width.is_finite() || u - width != l || l + width != u)
    })
}
fn standard_sparse(
    model: &Model,
    node: &Node,
    limit: usize,
    track_rows: bool,
    fixed_columns: bool,
) -> Relaxation {
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
        if fixed_columns && let Some(value) = root_fixed_value(v, model.original_decimal) {
            if lo.is_some_and(|l| value < l) || up.is_some_and(|u| value > u) {
                return Relaxation::Infeasible;
            }
            shift.push(value);
            terms.push(vec![]);
            continue;
        }
        if let Some(l) = lo {
            shift.push(l);
            terms.push(vec![(costs.len(), 1.0)]);
            costs.push(sign * v.objective);
            upper.push(up.map_or(f64::INFINITY, |u| checked_difference(u, l)));
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
        .map(|r| r.coefficients.len() as u128 * if split_range(r) { 2 } else { 1 })
        .sum::<u128>();
    let rows = model
        .rows
        .iter()
        .map(|r| if split_range(r) { 2u128 } else { 1 })
        .sum::<u128>();
    let estimate = nonzeros * 48 + rows * 2048 + (costs.len() as u128) * 128;
    if estimate > limit as u128 {
        return Relaxation::ResourceLimit;
    }
    let mut columns = vec![vec![]; costs.len()];
    let mut rhs = Vec::new();
    let mut equality = Vec::new();
    let mut slack_upper = Vec::new();
    let exact_shift = shift.iter().all(|&x| crate::model::point_error(x) == 0.);
    let mut row_sources = Vec::new();
    for (source, row) in model.rows.iter().enumerate() {
        if row.lower.is_none() && row.upper.is_none() {
            continue;
        }
        if row.lower.zip(row.upper).is_some_and(|(l, u)| l > u) {
            return Relaxation::Infeasible;
        }
        let split = split_range(row);
        let orientations: &[f64] = if split {
            &[1., -1.]
        } else if row.upper.is_some() {
            &[1.]
        } else {
            &[-1.]
        };
        for &row_sign in orientations {
            let i = rhs.len();
            for &(j, a) in &row.coefficients {
                for &(k, s) in &terms[j] {
                    columns[k].push((i, row_sign * a * s));
                }
            }
            let bound = shifted_bound(
                row,
                &shift,
                row_sign > 0.,
                model.original_decimal,
                exact_shift,
            );
            rhs.push(bound);
            if track_rows {
                row_sources.push((source, row_sign));
            }
            equality.push(!split && row.lower.zip(row.upper).is_some_and(|(l, u)| l == u));
            slack_upper.push(if split {
                f64::INFINITY
            } else {
                row.lower
                    .zip(row.upper)
                    .map_or(f64::INFINITY, |(l, u)| u - l)
            });
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
        row_sources,
        sparse: Some(p),
        a: vec![],
        b: vec![],
        c: vec![],
        shift,
        terms,
    })
}
#[derive(Default, Debug, Serialize)]
pub struct PumpStats {
    pub calls: usize,
    pub lp_solves: usize,
    pub perturbations: usize,
    pub candidates: usize,
    pub improvements: usize,
    pub numerical_errors: usize,
    pub limited: usize,
    pub seconds: f64,
}
fn binary_pump(
    model: &Model,
    root: &Node,
    standard: &Standard,
    point: &[f64],
    limits: &Limits,
    deadline: Instant,
    stats: &mut PumpStats,
    lp_stats: &mut revised::Stats,
) -> Option<Vec<f64>> {
    let mut binary = Vec::new();
    for (j, v) in model
        .variables
        .iter()
        .enumerate()
        .filter(|(_, v)| v.integer)
    {
        let (Some(l), Some(u)) = (v.lower, v.upper) else {
            return None;
        };
        if l.ceil() == u.floor() {
            continue;
        }
        if l < 0. || u > 1. {
            return None;
        }
        binary.push(j);
    }
    if binary.is_empty() {
        return None;
    }
    let mut problem = standard.sparse.as_ref()?.clone();
    stats.calls += 1;
    let checker = heuristic::Repair::new(model);
    let mut current = point.to_vec();
    let mut warm = root.warm.clone();
    let mut history = std::collections::VecDeque::<Vec<bool>>::new();
    let mut seed = 0xace189e3914b5837u64;
    for _ in 0..64 {
        if Instant::now() >= deadline {
            stats.limited += 1;
            return None;
        }
        let mut rounded = current.clone();
        for (j, v) in model.variables.iter().enumerate() {
            if v.integer {
                rounded[j] = rounded[j].round();
            }
        }
        if checker.candidate_check(&rounded) {
            stats.candidates += 1;
            if let Some(path) = std::env::var_os("MIP_PUMP_CANDIDATE_DUMP") {
                #[derive(Serialize)]
                struct Candidate {
                    #[serde(serialize_with = "crate::model::serialize_optional_solution_number")]
                    objective: Option<f64>,
                    #[serde(serialize_with = "crate::model::serialize_solution_values")]
                    values: Option<std::collections::BTreeMap<String, f64>>,
                }
                let objective = model.objective(&rounded);
                if objective.is_finite() {
                    if let Ok(file) = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(path)
                    {
                        let candidate = Candidate {
                            objective: Some(objective),
                            values: Some(
                                model
                                    .variables
                                    .iter()
                                    .zip(&rounded)
                                    .map(|(v, &x)| (v.name.clone(), x))
                                    .collect(),
                            ),
                        };
                        let _ = serde_json::to_writer(file, &candidate);
                    }
                }
            }
            return Some(rounded);
        }
        let mut target: Vec<_> = binary.iter().map(|&j| current[j] >= 0.5).collect();
        if history.iter().any(|old| *old == target) {
            let mut ranking: Vec<_> = binary
                .iter()
                .enumerate()
                .map(|(k, &j)| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    let noise = (seed >> 11) as f64 / (1u64 << 53) as f64;
                    (k, (current[j] - current[j].round()).abs() + 0.2 * noise)
                })
                .collect();
            ranking.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let count = (binary.len() as f64).sqrt() as usize;
            for &(k, _) in ranking.iter().take(count.clamp(1, 32)) {
                target[k] = !target[k];
            }
            stats.perturbations += 1;
        }
        history.push_back(target.clone());
        if history.len() > 8 {
            history.pop_front();
        }
        problem.costs.fill(0.);
        for (&j, &value) in binary.iter().zip(&target) {
            let coefficient = if value { -1. } else { 1. };
            for &(k, s) in &standard.terms[j] {
                problem.costs[k] += coefficient * s;
            }
        }
        let answer = revised::minimize_warm(
            &problem,
            limits.pivots.min(4096),
            Some(deadline),
            if limits.warm_start {
                warm.as_deref()
            } else {
                None
            },
        );
        stats.lp_solves += 1;
        lp_stats.add(&answer.stats);
        match answer.outcome {
            lp::Outcome::Optimal { x, .. } => {
                current = standard
                    .terms
                    .iter()
                    .enumerate()
                    .map(|(j, t)| standard.shift[j] + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>())
                    .collect();
                if model.check(&current, false, 1e-6).is_err() {
                    stats.numerical_errors += 1;
                    return None;
                }
                warm = answer.warm;
            }
            lp::Outcome::TimeLimit | lp::Outcome::IterationLimit => {
                stats.limited += 1;
                return None;
            }
            _ => {
                stats.numerical_errors += 1;
                return None;
            }
        }
    }
    stats.limited += 1;
    None
}
/// 主解探索の局所LPは探索木・下界・擬似コストには一切反映しない。
#[derive(Default, Debug, Serialize)]
pub struct DivingStats {
    pub calls: usize,
    pub lp_solves: usize,
    pub reversals: usize,
    pub infeasible: usize,
    pub numerical_errors: usize,
    pub limited: usize,
    pub candidates: usize,
    pub improvements: usize,
    pub seconds: f64,
}
fn dive(
    model: &Model,
    root: &Node,
    point: &[f64],
    limits: &Limits,
    deadline: Instant,
    stats: &mut DivingStats,
    lp_stats: &mut revised::Stats,
) -> Option<Vec<f64>> {
    stats.calls += 1;
    let mut current = root.clone();
    let mut point = point.to_vec();
    let checker = heuristic::Repair::new(model);
    let first_solve = stats.lp_solves;
    loop {
        if Instant::now() >= deadline || stats.lp_solves - first_solve >= 256 {
            stats.limited += 1;
            return None;
        }
        // ほぼ整数の変数から順に近い側へ潜る。整数許容幅内の値は分枝しない。
        let choice = model
            .variables
            .iter()
            .enumerate()
            .filter(|(j, v)| v.integer && (point[*j] - point[*j].round()).abs() > 1e-6)
            .min_by(|(j, _), (k, _)| {
                let distance = |i: usize| (point[i] - point[i].round()).abs();
                distance(*j).total_cmp(&distance(*k)).then(j.cmp(k))
            })
            .map(|(j, _)| j);
        let Some(j) = choice else {
            // 整数を丸めた後に全元制約を検査。連続部分はそのまま保持する。
            for (j, v) in model.variables.iter().enumerate() {
                if v.integer {
                    point[j] = point[j].round();
                }
            }
            if checker.candidate_check(&point) {
                stats.candidates += 1;
                return Some(point);
            }
            return None;
        };
        let floor = point[j].floor();
        let ceil = point[j].ceil();
        let prefer_up = ceil - point[j] <= point[j] - floor;
        let mut accepted = false;
        // 局所LPが実行不能なら同じ親から反対側を一度だけ試す。
        for reverse in [false, true] {
            if Instant::now() >= deadline || stats.lp_solves - first_solve >= 256 {
                stats.limited += 1;
                return None;
            }
            if reverse {
                stats.reversals += 1;
            }
            let mut child = current.clone();
            // 現在整数の値は局所探索内で固定し、再最適化で再び分数化するのを抑える。
            // これらの固定が悪くても、失うのはこの主解探索の機会だけ。
            for (k, v) in model.variables.iter().enumerate() {
                if v.integer && (point[k] - point[k].round()).abs() <= 1e-6 {
                    let value = point[k].round();
                    child.lower[k] = Some(value);
                    child.upper[k] = Some(value);
                }
            }
            if prefer_up != reverse {
                child.lower[j] = Some(child.lower[j].map_or(ceil, |l| l.max(ceil)));
            } else {
                child.upper[j] = Some(child.upper[j].map_or(floor, |u| u.min(floor)));
            }
            let q = match standard_sparse(
                model,
                &child,
                limits.dense_bytes,
                false,
                limits.fixed_columns,
            ) {
                Relaxation::Ready(q) => q,
                Relaxation::Infeasible => {
                    stats.infeasible += 1;
                    continue;
                }
                Relaxation::ResourceLimit => {
                    stats.limited += 1;
                    return None;
                }
            };
            let answer = revised::minimize_warm(
                q.sparse.as_ref().unwrap(),
                limits.pivots.min(4096),
                Some(deadline),
                if limits.warm_start {
                    current.warm.as_deref()
                } else {
                    None
                },
            );
            stats.lp_solves += 1;
            lp_stats.add(&answer.stats);
            match answer.outcome {
                lp::Outcome::Optimal { x, .. } => {
                    point = q
                        .terms
                        .iter()
                        .enumerate()
                        .map(|(j, t)| q.shift[j] + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>())
                        .collect();
                    if model.check(&point, false, 1e-6).is_err() {
                        stats.numerical_errors += 1;
                        return None;
                    }
                    child.warm = answer.warm;
                    current = child;
                    accepted = true;
                    break;
                }
                lp::Outcome::Infeasible => stats.infeasible += 1,
                lp::Outcome::IterationLimit | lp::Outcome::TimeLimit => {
                    stats.limited += 1;
                    return None;
                }
                _ => {
                    stats.numerical_errors += 1;
                    return None;
                }
            }
        }
        if !accepted {
            return None;
        }
    }
}
/// 別の分枝順による短い自作探索。主解以外の情報は主探索へ渡さない。
#[derive(Default, Debug, Serialize)]
pub struct ProbeStats {
    pub precut_roots: usize,
    pub calls: usize,
    pub nodes: usize,
    pub lp_solves: usize,
    pub max_open_nodes: usize,
    pub seconds: f64,
    pub status: Option<String>,
    pub candidates: usize,
    pub improvements: usize,
}
struct CachedRoot {
    standard: Standard,
    answer: revised::Answer,
}
#[derive(Default)]
struct CutPreparation {
    cached: Option<CachedRoot>,
    primal_root: Option<(Model, Arc<revised::WarmStart>)>,
    initial_lp_seconds: f64,
    model: Option<Model>,
    cuts: Vec<cuts::Cut>,
    stats: cuts::Stats,
    rounds: usize,
    lp_solves: usize,
    seconds: f64,
}
fn prepare_cuts(
    model: &Model,
    root: &mut Node,
    limits: &Limits,
    deadline: Instant,
    lp_stats: &mut revised::Stats,
) -> CutPreparation {
    let mut prepared = CutPreparation::default();
    if !matches!(limits.engine, Engine::Revised)
        || !((limits.cover_cuts && cuts::applicable(model))
            || (limits.rounding_cuts && cuts::rounding_applicable(model))
            || (limits.aggregate_cuts && cuts::aggregate_applicable(model))
            || (limits.clique_cuts && cuts::applicable(model))
            || (limits.mir_cuts && cuts::mir_applicable(model))
            || (limits.three_row_cuts && cuts::aggregate_applicable(model)))
        || limits.nodes == 0
        || Instant::now() >= deadline
    {
        return prepared;
    }
    let started = Instant::now();
    let cut_budget = std::time::Duration::from_secs_f64((limits.seconds * 0.05).min(0.25));
    let mut cut_deadline = deadline.min(started + cut_budget);
    // Propagation can change the root bounds before the main solve. Keep its
    // original short preparation path rather than caching a different problem.
    let global_first = !limits.propagation;
    let mut work = model.clone();
    let mut pool = cuts::Pool::default();
    for round in 0..limits.cover_rounds {
        let solve_deadline = if round == 0 && global_first {
            deadline
        } else {
            cut_deadline
        };
        if Instant::now() >= solve_deadline {
            break;
        }
        let Relaxation::Ready(q) = standard_sparse(
            &work,
            root,
            limits.dense_bytes,
            global_first,
            limits.fixed_columns,
        ) else {
            break;
        };
        let lp_started = Instant::now();
        let answer = revised::minimize_warm(
            q.sparse.as_ref().unwrap(),
            limits.pivots,
            Some(solve_deadline),
            if limits.warm_start {
                root.warm.as_deref()
            } else {
                None
            },
        );
        if round == 0 && global_first {
            prepared.initial_lp_seconds = lp_started.elapsed().as_secs_f64();
            cut_deadline = deadline.min(Instant::now() + cut_budget);
        }
        prepared.lp_solves += 1;
        prepared.rounds += 1;
        lp_stats.add(&answer.stats);
        let lp::Outcome::Optimal { x, .. } = &answer.outcome else {
            if round == 0 && global_first {
                prepared.cached = Some(CachedRoot {
                    standard: q,
                    answer,
                });
            }
            break;
        };
        let point: Vec<_> = q
            .terms
            .iter()
            .enumerate()
            .map(|(j, t)| q.shift[j] + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>())
            .collect();
        if work.check(&point, false, 1e-6).is_err() {
            if global_first {
                prepared.cached = Some(CachedRoot {
                    standard: q,
                    answer,
                });
            }
            break;
        }
        if limits.warm_start {
            root.warm = answer.warm.clone();
        }
        let mut generated = if limits.clique_cuts {
            pool.separate_clique(model, &point, 32, cut_deadline)
        } else {
            vec![]
        };
        if limits.rounding_cuts && generated.len() < 32 {
            generated.extend(pool.separate_rounding(
                model,
                &point,
                32 - generated.len(),
                cut_deadline,
            ));
        }
        if limits.aggregate_cuts && generated.len() < 32 {
            generated.extend(pool.separate_aggregate(
                model,
                &point,
                32 - generated.len(),
                cut_deadline,
            ));
        }
        if limits.cover_cuts && generated.len() < 32 {
            generated.extend(pool.separate(model, &point, 32 - generated.len(), cut_deadline));
        }
        if generated.is_empty() {
            if global_first {
                prepared.cached = Some(CachedRoot {
                    standard: q,
                    answer,
                });
            }
            break;
        }
        work.rows
            .extend(generated.iter().map(|cut| cut.row.clone()));
        prepared.cuts.extend(generated);
    }
    if limits.mir_cuts && global_first {
        improve_with_mir(
            model,
            root,
            limits,
            cut_deadline,
            lp_stats,
            &mut work,
            &mut pool,
            &mut prepared,
        );
    }
    if limits.three_row_cuts && global_first {
        let old_rows = work.rows.len();
        let old_cuts = prepared.cuts.len();
        let old_warm = if limits.primal_probe && limits.warm_start {
            prepared.cached.as_ref().and_then(|c| c.answer.warm.clone())
        } else {
            None
        };
        let mut three_pool = cuts::Pool::default();
        improve_with_three(
            model,
            root,
            limits,
            cut_deadline,
            lp_stats,
            &mut work,
            &mut three_pool,
            &mut prepared,
        );
        if prepared.cuts.len() > old_cuts
            && let Some(warm) = old_warm
        {
            let mut primal_model = work.clone();
            primal_model.rows.truncate(old_rows);
            prepared.primal_root = Some((primal_model, warm));
        }
        pool.stats.three_calls += three_pool.stats.aggregate_calls;
        pool.stats.three_work += three_pool.stats.aggregate_work;
        pool.stats.three_added += three_pool.stats.aggregate_added;
        pool.stats.three_lp_solves += three_pool.stats.three_lp_solves;
        pool.stats.three_rejected_cuts += three_pool.stats.three_rejected_cuts;
        pool.stats.three_seconds += three_pool.stats.aggregate_seconds;
        pool.stats.three_trial_seconds += three_pool.stats.three_trial_seconds;
        pool.stats.added += three_pool.stats.added;
        pool.stats.candidates += three_pool.stats.candidates;
        pool.stats.rejected += three_pool.stats.rejected;
    }
    if !prepared.cuts.is_empty() {
        prepared.model = Some(work);
    }
    prepared.stats = pool.stats;
    prepared.seconds = started.elapsed().as_secs_f64();
    prepared
}
// MIR is an optional extension of an already solved root. Keep its rows only
// after a timely reoptimization shows useful objective progress. Rejected trials
// leave the original point, model prefix and immutable warm basis available.
fn improve_with_mir(
    model: &Model,
    root: &mut Node,
    limits: &Limits,
    deadline: Instant,
    lp_stats: &mut revised::Stats,
    work: &mut Model,
    pool: &mut cuts::Pool,
    prepared: &mut CutPreparation,
) {
    fn point(q: &Standard, answer: &revised::Answer) -> Option<Vec<f64>> {
        let lp::Outcome::Optimal { x, .. } = &answer.outcome else {
            return None;
        };
        Some(
            q.terms
                .iter()
                .enumerate()
                .map(|(j, t)| q.shift[j] + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>())
                .collect(),
        )
    }
    let Some(mut baseline) = prepared.cached.take() else {
        return;
    };
    for _ in 0..limits.cover_rounds {
        if Instant::now() >= deadline {
            break;
        }
        let Some(old_point) = point(&baseline.standard, &baseline.answer) else {
            break;
        };
        let old_objective = model.objective(&old_point);
        if !old_objective.is_finite() || work.check(&old_point, false, 1e-6).is_err() {
            break;
        }
        let proposed = pool.separate_mir(model, &old_point, 8, deadline);
        if proposed.is_empty() {
            break;
        }
        let old_rows = work.rows.len();
        work.rows.extend(proposed.iter().map(|c| c.row.clone()));
        let started = Instant::now();
        let trial = if let Relaxation::Ready(q) =
            standard_sparse(work, root, limits.dense_bytes, true, limits.fixed_columns)
        {
            let answer = revised::minimize_warm(
                q.sparse.as_ref().unwrap(),
                limits.pivots,
                Some(deadline),
                if limits.warm_start {
                    baseline.answer.warm.as_deref()
                } else {
                    None
                },
            );
            prepared.lp_solves += 1;
            prepared.rounds += 1;
            pool.stats.mir_lp_solves += 1;
            lp_stats.add(&answer.stats);
            Some(CachedRoot {
                standard: q,
                answer,
            })
        } else {
            None
        };
        pool.stats.mir_trial_seconds += started.elapsed().as_secs_f64();
        let accepted = trial
            .as_ref()
            .and_then(|t| point(&t.standard, &t.answer))
            .is_some_and(|p| {
                let new_objective = model.objective(&p);
                let sign = if model.maximize { -1. } else { 1. };
                let gain = sign * (new_objective - old_objective);
                Instant::now() < deadline
                    && new_objective.is_finite()
                    && work.check(&p, false, 1e-6).is_ok()
                    && (gain >= 1e-3 * old_objective.abs().max(1.)
                        || model.check(&p, true, 1e-6).is_ok())
            });
        if !accepted {
            work.rows.truncate(old_rows);
            pool.stats.mir_added -= proposed.len();
            pool.stats.added -= proposed.len();
            pool.stats.mir_rejected_cuts += proposed.len();
            break;
        }
        prepared.cuts.extend(proposed);
        baseline = trial.unwrap();
        if limits.warm_start {
            root.warm = baseline.answer.warm.clone();
        }
    }
    prepared.cached = Some(baseline);
}
// Experimental three-row trials obey the same root-cache retention policy.
fn improve_with_three(
    model: &Model,
    root: &mut Node,
    limits: &Limits,
    deadline: Instant,
    lp_stats: &mut revised::Stats,
    work: &mut Model,
    pool: &mut cuts::Pool,
    prepared: &mut CutPreparation,
) {
    fn point(q: &Standard, answer: &revised::Answer) -> Option<Vec<f64>> {
        let lp::Outcome::Optimal { x, .. } = &answer.outcome else {
            return None;
        };
        Some(
            q.terms
                .iter()
                .enumerate()
                .map(|(j, t)| q.shift[j] + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>())
                .collect(),
        )
    }
    let Some(mut baseline) = prepared.cached.take() else {
        return;
    };
    for _ in 0..limits.cover_rounds {
        if Instant::now() >= deadline {
            break;
        }
        let Some(old_point) = point(&baseline.standard, &baseline.answer) else {
            break;
        };
        let old_objective = model.objective(&old_point);
        if !old_objective.is_finite() || work.check(&old_point, false, 1e-6).is_err() {
            break;
        }
        let proposed = pool.separate_three_aggregate(model, &old_point, 8, deadline);
        if proposed.is_empty() {
            break;
        }
        let old_rows = work.rows.len();
        work.rows.extend(proposed.iter().map(|c| c.row.clone()));
        let started = Instant::now();
        let trial = if let Relaxation::Ready(q) =
            standard_sparse(work, root, limits.dense_bytes, true, limits.fixed_columns)
        {
            let answer = revised::minimize_warm(
                q.sparse.as_ref().unwrap(),
                limits.pivots,
                Some(deadline),
                if limits.warm_start {
                    baseline.answer.warm.as_deref()
                } else {
                    None
                },
            );
            prepared.lp_solves += 1;
            prepared.rounds += 1;
            pool.stats.three_lp_solves += 1;
            lp_stats.add(&answer.stats);
            Some(CachedRoot {
                standard: q,
                answer,
            })
        } else {
            None
        };
        pool.stats.three_trial_seconds += started.elapsed().as_secs_f64();
        let accepted = trial
            .as_ref()
            .and_then(|t| point(&t.standard, &t.answer))
            .is_some_and(|p| {
                let new_objective = model.objective(&p);
                let sign = if model.maximize { -1. } else { 1. };
                let gain = sign * (new_objective - old_objective);
                Instant::now() < deadline
                    && new_objective.is_finite()
                    && work.check(&p, false, 1e-6).is_ok()
                    && (gain >= 1e-3 * old_objective.abs().max(1.)
                        || model.check(&p, true, 1e-6).is_ok())
            });
        if !accepted {
            work.rows.truncate(old_rows);
            pool.stats.aggregate_added -= proposed.len();
            pool.stats.added -= proposed.len();
            pool.stats.three_rejected_cuts += proposed.len();
            break;
        }
        prepared.cuts.extend(proposed);
        baseline = trial.unwrap();
        if limits.warm_start {
            root.warm = baseline.answer.warm.clone();
        }
    }
    prepared.cached = Some(baseline);
}
pub fn solve(model: &Model, limits: &Limits) -> Result {
    solve_seeded(model, limits, None)
}
fn solve_seeded(model: &Model, limits: &Limits, seed: Option<&[f64]>) -> Result {
    solve_seeded_warm(model, limits, seed, None)
}
fn solve_seeded_warm(
    model: &Model,
    limits: &Limits,
    seed: Option<&[f64]>,
    root_warm: Option<Arc<revised::WarmStart>>,
) -> Result {
    let start = Instant::now();
    if seed.is_none() && limits.nodes > 0 && limits.seconds > 0. && limits.parity_equalities {
        let deadline = start + std::time::Duration::from_secs_f64(limits.seconds.min(0.05));
        if let Some(answer) = crate::parity::solve(model, deadline) {
            let sign = if model.maximize { -1. } else { 1. };
            let objective = model.objective(&answer.point);
            let fixed: Vec<_> = answer.point.iter().map(|&x| Some(x)).collect();
            if let Some(bound) = Certifier::new(model)
                .and_then(|c| c.bound(&fixed, &fixed, &vec![0.; model.rows.len()]))
                .filter(|_| objective.is_finite())
            {
                let dual = sign * bound;
                return Result {
                    parity_proof: Some(answer.proof),
                    status: "optimal".into(),
                    objective: Some(objective),
                    dual_bound: Some(dual),
                    gap: Some((objective - dual).abs() / objective.abs().max(1.)),
                    values: Some(
                        model
                            .variables
                            .iter()
                            .zip(answer.point)
                            .map(|(v, x)| (v.name.clone(), x))
                            .collect(),
                    ),
                    seconds: start.elapsed().as_secs_f64(),
                    first_primal_seconds: Some(start.elapsed().as_secs_f64()),
                    lp_search_termination: Some("parity_unique".into()),
                    ..Result::default()
                };
            }
        }
    }
    if limits.binary_graph && seed.is_none() && matches!(limits.engine, Engine::Revised) {
        let deadline = start + std::time::Duration::from_secs_f64(limits.seconds);
        if Instant::now() < deadline {
            if let Some(answer) = crate::binary_graph::solve_with_fractional(
                model,
                limits.dense_bytes,
                deadline,
                limits.nodes,
                limits.graph_local_search,
                limits.graph_root_coloring,
                limits.graph_root_prefix,
                limits.graph_local_diversify,
                limits.graph_adaptive_local,
                limits.graph_cover_local,
                limits.graph_fractional_cover,
            ) {
                let sign = if model.maximize { -1. } else { 1. };
                let objective = model.objective(&answer.point);
                let bound = crate::exact::dot_lower(
                    [
                        (sign * model.offset, 1.),
                        (
                            sign * model.variables[0].objective,
                            answer.stats.upper as f64,
                        ),
                    ]
                    .into_iter(),
                )
                .map(|v| if v.fract() == 0. { v } else { v.next_down() });
                let valid = objective.is_finite()
                    && model.check(&answer.point, true, 1e-7).is_ok()
                    && bound.is_some();
                let dual_bound = valid.then(|| sign * bound.unwrap());
                let gap = dual_bound.map(|b| (objective - b).abs() / objective.abs().max(1.));
                return Result {
                    binary_graph_enabled: true,
                    graph_cover_local_enabled: limits.graph_cover_local,
                    aggregate_cuts_enabled: limits.aggregate_cuts,
                    clique_cuts_enabled: limits.clique_cuts,
                    mir_cuts_enabled: limits.mir_cuts,
                    three_row_cuts_enabled: limits.three_row_cuts,
                    graph_fractional_cover_enabled: limits.graph_fractional_cover,
                    nodes: answer.stats.nodes,
                    first_primal_seconds: answer.stats.first_seconds,
                    binary_graph: Some(answer.stats),
                    status: if valid {
                        answer.status.into()
                    } else {
                        "numerical_error".into()
                    },
                    objective: valid.then_some(objective),
                    dual_bound,
                    gap,
                    values: valid.then(|| {
                        model
                            .variables
                            .iter()
                            .zip(answer.point)
                            .map(|(v, x)| (v.name.clone(), x))
                            .collect()
                    }),
                    seconds: start.elapsed().as_secs_f64(),
                    lp_engine: "BinaryGraph".into(),
                    branching: "GraphColoring".into(),
                    child_order: "ColorReverse".into(),
                    search_strategy: "DepthFirst".into(),
                    lp_workspace_bytes: limits.dense_bytes,
                    box_primal_enabled: limits.box_primal,
                    ..Result::default()
                };
            }
        }
    }
    let requested_fixed_columns = limits.fixed_columns;
    let root_fixed_columns = if requested_fixed_columns {
        model
            .variables
            .iter()
            .filter(|v| root_fixed_value(v, model.original_decimal).is_some())
            .count()
    } else {
        0
    };
    let mut effective_limits = limits.clone();
    effective_limits.fixed_columns = root_fixed_columns != 0;
    let limits = &effective_limits;
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
    let mut root = Node {
        certified_bound: None,
        lower,
        upper,
        bound: f64::NEG_INFINITY,
        warm: root_warm,
        changed: None,
        edge: None,
    };
    let mut box_primal = heuristic::BoxStats::default();
    let mut box_candidate = None;
    let mut early_repair = None;
    let mut root_seed_repair = None;
    if limits.box_primal && seed.is_none() && Instant::now() < deadline {
        let started = Instant::now();
        let box_deadline = deadline
            .min(started + std::time::Duration::from_secs_f64((limits.seconds * 0.05).min(0.25)));
        let mut repair = heuristic::Repair::new(model);
        let offset = start.elapsed().as_secs_f64();
        (box_candidate, box_primal) = repair.box_search(box_deadline);
        if let Some(t) = &mut box_primal.first_seconds {
            *t += offset;
        }
        box_primal.seconds = started.elapsed().as_secs_f64();
        if box_candidate.is_none()
            && limits.root_seed_repair && limits.primal_repair
            && matches!(limits.engine, Engine::Revised)
            && (4096..=100_000).contains(&model.variables.len())
            && model.variables.iter().all(|v| v.integer
                && matches!((v.lower,v.upper),(Some(l),Some(u)) if (l==0. || l==1.) && (u==0. || u==1.)))
            && model.rows.iter().map(|r|r.coefficients.len()).sum::<usize>() <= 10_000_000
            && Instant::now() < deadline
        {
            let began = Instant::now();
            let local_deadline = deadline.min(began + std::time::Duration::from_secs_f64((limits.seconds*0.1).min(0.5)));
            let point: Vec<_> = root.lower.iter().zip(&root.upper).map(|(l,u)|0f64.max(l.unwrap_or(f64::NEG_INFINITY)).min(u.unwrap_or(f64::INFINITY))).collect();
            let before = repair.stats.clone();
            let candidate = repair.search(&point,&root.lower,&root.upper,local_deadline,50_000_000,0)
                .filter(|x|model.objective(x).is_finite());
            let found = candidate.is_some();
            root_seed_repair = Some(heuristic::BoxStats {
                calls: repair.stats.calls-before.calls,
                candidates: usize::from(found), improvements: usize::from(found),
                moves: repair.stats.moves-before.moves, work: repair.stats.work-before.work,
                seconds: began.elapsed().as_secs_f64(),
                first_seconds: found.then(||start.elapsed().as_secs_f64()),
            });
            box_candidate = candidate;
        }
        early_repair = Some(repair);
    }
    let mut lp_stats = revised::Stats::default();
    let mut prepared = prepare_cuts(model, &mut root, limits, deadline, &mut lp_stats);
    let mut cached_root = prepared.cached.take();
    let mut primal_root = prepared.primal_root.take();
    let root_lp_reused = cached_root.is_some();
    let lp_model = prepared.model.as_ref().unwrap_or(model);
    let objective_grid = if limits.objective_rounding
        && !limits.propagation
        && matches!(limits.engine, Engine::Revised)
        && Grid::supported_bounds(model)
        && (lp_model
            .rows
            .iter()
            .map(|r| r.coefficients.len() as u128)
            .sum::<u128>()
            + lp_model.rows.len() as u128
            + lp_model.variables.len() as u128)
            * 64
            <= limits.dense_bytes as u128
    {
        Grid::from_model(model)
    } else {
        None
    };
    let mut objective_certifier = objective_grid
        .as_ref()
        .and_then(|_| Certifier::new(lp_model));
    let check_original_infeasibility = lp_model.needs_original_feasibility_check();
    let mut infeasibility_certifier = None;
    let mut infeasibility_proof_calls = 0;
    let mut infeasibility_proof_trials = 0;
    let mut infeasibility_proof_failures = 0;
    let mut infeasibility_proof_seconds = 0.;
    let mut objective_bound_stats = ObjectiveBoundStats::default();
    let mut stack = search::Queue::new(limits.search, |n: &Node| n.bound);
    stack.push(root);
    let mut costs = branching::Costs::new(model.variables.len());
    let mut nodes = 0;
    let mut lp_solves = prepared.lp_solves;
    let mut fixed_leaves = 0;
    let mut propagator = limits
        .propagation
        .then(|| propagate::Propagator::new(model));
    let mut repair = limits.primal_repair.then(|| {
        early_repair
            .take()
            .unwrap_or_else(|| heuristic::Repair::new(model))
    });
    drop(early_repair);
    let mut diving = DivingStats::default();
    let mut pump = PumpStats::default();
    let mut probe = ProbeStats::default();
    let mut strong = StrongStats::default();
    let mut rins = RinsStats::default();
    let mut repair_improvements = 0;
    let mut repair_lp_solves = 0;
    let mut repair_lp_candidates = 0;
    let mut incumbent = f64::INFINITY;
    let mut best = None;
    let mut first_primal_seconds = None;
    let mut root_incumbent_conflict = false;
    if let Some(x) = box_candidate {
        incumbent = sign * model.objective(&x);
        first_primal_seconds = box_primal
            .first_seconds
            .or_else(|| root_seed_repair.as_ref().and_then(|s| s.first_seconds));
        best = Some(x);
    }
    if let Some(x) = seed {
        if heuristic::Repair::new(model).candidate_check(x) {
            let value = sign * model.objective(x);
            if value.is_finite() {
                incumbent = value;
                best = Some(x.to_vec());
                first_primal_seconds = Some(start.elapsed().as_secs_f64());
            }
        }
    }
    let mut status = "optimal".to_string();
    let mut terminal_bound = f64::INFINITY;
    let mut deferred = search::DeferredBounds::default();
    while let Some(mut node) = stack.pop() {
        if node.bound >= incumbent {
            continue;
        }
        if let (Some(grid), Some(bound)) = (&objective_grid, node.certified_bound) {
            if grid
                .upper_box(model, &node.lower, &node.upper)
                .is_some_and(|upper| upper < bound as i128)
            {
                objective_bound_stats.box_pruned += 1;
                continue;
            }
        }
        // An already computed root result still needs the normal point/proof
        // checks. This permits no new LP solve after the global deadline.
        let interrupt = if Instant::now() >= deadline && !(nodes == 0 && cached_root.is_some()) {
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
        nodes += 1;
        if let Some(p) = &mut propagator {
            match p.propagate(
                &mut node.lower,
                &mut node.upper,
                node.changed,
                Some(deadline),
            ) {
                propagate::Status::Infeasible => continue,
                propagate::Status::TimeLimit => {
                    stack.push(node);
                    status = "time_limit".into();
                    break;
                }
                propagate::Status::Stable | propagate::Status::Limited => {}
            }
            if node
                .lower
                .iter()
                .zip(&node.upper)
                .all(|(l, u)| l.zip(*u).is_some_and(|(l, u)| l == u))
            {
                let x: Vec<_> = node.lower.iter().map(|v| v.unwrap()).collect();
                if model.check(&x, true, 1e-6).is_ok() {
                    let value = sign * model.objective(&x);
                    if !value.is_finite() {
                        stack.push(node);
                        status = "numerical_error".into();
                        break;
                    }
                    fixed_leaves += 1;
                    terminal_bound = terminal_bound.min(value);
                    if value < incumbent {
                        incumbent = value;
                        first_primal_seconds.get_or_insert_with(|| start.elapsed().as_secs_f64());
                        best = Some(x);
                    }
                    continue;
                }
            }
        }
        let cached = if nodes == 1 { cached_root.take() } else { None };
        let (relaxation, cached_answer) = if let Some(cached) = cached {
            (Relaxation::Ready(cached.standard), Some(cached.answer))
        } else {
            (
                match limits.engine {
                    Engine::Dense => {
                        standard_dense(lp_model, &node, limits.dense_bytes, limits.fixed_columns)
                    }
                    Engine::Revised => standard_sparse(
                        lp_model,
                        &node,
                        limits.dense_bytes,
                        objective_certifier.is_some() || check_original_infeasibility,
                        limits.fixed_columns,
                    ),
                },
                None,
            )
        };
        let p = match relaxation {
            Relaxation::Infeasible => {
                if nodes == 1 && best.is_some() {
                    root_incumbent_conflict = true;
                    stack.push(node);
                    status = "numerical_error".into();
                    break;
                }
                continue;
            }
            Relaxation::ResourceLimit => {
                stack.push(node);
                status = "resource_limit".into();
                break;
            }
            Relaxation::Ready(p) => p,
        };
        let reused_root = cached_answer.is_some();
        lp_solves += usize::from(!reused_root);
        let mut next_warm = None;
        let (answer, relaxation_dual, relaxation_witness) = if let Some(sparse) = &p.sparse {
            let answer = cached_answer.unwrap_or_else(|| {
                if limits.warm_start {
                    revised::minimize_warm(
                        sparse,
                        limits.pivots,
                        Some(deadline),
                        node.warm.as_deref(),
                    )
                } else {
                    revised::minimize(sparse, limits.pivots, Some(deadline))
                }
            });
            if !reused_root {
                lp_stats.add(&answer.stats);
            }
            if limits.warm_start {
                next_warm = answer.warm;
            }
            (Ok(answer.outcome), answer.dual, answer.witness)
        } else {
            (
                lp::minimize_until(&p.a, &p.b, &p.c, limits.pivots, Some(deadline)),
                None,
                None,
            )
        };
        match answer {
            Ok(lp::Outcome::Infeasible) => {
                if nodes == 1 && best.is_some() {
                    root_incumbent_conflict = true;
                    stack.push(node);
                    status = "numerical_error".into();
                    break;
                }
                if check_original_infeasibility {
                    let proof_started = Instant::now();
                    infeasibility_proof_calls += 1;
                    let proof = (|| {
                        let y = relaxation_witness.as_deref()?;
                        if y.len() != p.row_sources.len() {
                            return None;
                        }
                        if infeasibility_certifier.is_none() {
                            let entries = lp_model
                                .rows
                                .iter()
                                .map(|r| r.coefficients.len() as u128)
                                .sum::<u128>()
                                + lp_model.rows.len() as u128
                                + lp_model.variables.len() as u128;
                            if entries * 64 > limits.dense_bytes as u128 {
                                return None;
                            }
                            infeasibility_certifier = Certifier::feasibility_with_aggregation(
                                lp_model,
                                limits.aggregate_infeasibility,
                            );
                        }
                        let certifier = infeasibility_certifier.as_ref()?;
                        let mut multipliers = vec![0.; lp_model.rows.len()];
                        for (&value, &(i, sign)) in y.iter().zip(&p.row_sources) {
                            multipliers[i] += value * sign;
                        }
                        let mut lower = node.lower.clone();
                        let mut upper = node.upper.clone();
                        for (j, v) in lp_model.variables.iter().enumerate() {
                            if lower[j] == v.lower && v.lower.is_some() {
                                lower[j] = Some(v.lower_enclosure?[0]);
                            }
                            if upper[j] == v.upper && v.upper.is_some() {
                                upper[j] = Some(v.upper_enclosure?[1]);
                            }
                        }
                        let mut active: Vec<_> = multipliers
                            .iter()
                            .enumerate()
                            .filter(|(_, v)| **v != 0.)
                            .map(|(j, v)| (j, v.abs()))
                            .collect();
                        active.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
                        for sign in [1., -1.] {
                            let mut scaled: Vec<_> = multipliers.iter().map(|v| sign * v).collect();
                            infeasibility_proof_trials += 1;
                            if certifier
                                .bound(&lower, &upper, &scaled)
                                .is_some_and(|b| b > 0.)
                            {
                                return Some(());
                            }
                            // Multiplier perturbations only propose certificates; every trial is independently enclosed.
                            for &(j, _) in active.iter().take(8) {
                                let old = scaled[j];
                                for change in [-2f64.powi(-32), 2f64.powi(-32)] {
                                    scaled[j] = old * (1. + change);
                                    infeasibility_proof_trials += 1;
                                    if certifier
                                        .bound(&lower, &upper, &scaled)
                                        .is_some_and(|b| b > 0.)
                                    {
                                        return Some(());
                                    }
                                }
                                scaled[j] = old;
                            }
                        }
                        if limits.joint_infeasibility {
                            let (bound, trials) = certifier.joint_positive_bound(
                                &lower,
                                &upper,
                                &multipliers,
                                Some(deadline),
                            );
                            infeasibility_proof_trials += trials;
                            if bound.is_some() {
                                return Some(());
                            }
                        }
                        None
                    })();
                    infeasibility_proof_seconds += proof_started.elapsed().as_secs_f64();
                    if proof.is_none() {
                        infeasibility_proof_failures += 1;
                        if let Some(path) = std::env::var_os("MIP_INFEASIBILITY_DUMP") {
                            if let Some(y) = relaxation_witness.as_deref() {
                                let mut multipliers = vec![0.; lp_model.rows.len()];
                                for (&v, &(i, s)) in y.iter().zip(&p.row_sources) {
                                    multipliers[i] += s * v;
                                }
                                let path = if limits.defer_infeasibility {
                                    static SERIAL: std::sync::atomic::AtomicU64 =
                                        std::sync::atomic::AtomicU64::new(0);
                                    let serial =
                                        SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    let mut path = path;
                                    path.push(format!(".node-{nodes}.failure-{serial}.json"));
                                    path
                                } else {
                                    path
                                };
                                if let Ok(file) = std::fs::File::create(path) {
                                    let _ = serde_json::to_writer(
                                        file,
                                        &serde_json::json!({"model":lp_model,"lower":node.lower,"upper":node.upper,"multipliers":multipliers}),
                                    );
                                }
                            }
                            if std::env::var_os("MIP_LP_TRACE").is_some() {
                                eprintln!("Original infeasibility certificate unavailable");
                            }
                        }
                        if limits.defer_infeasibility {
                            deferred.record(node.bound);
                            continue;
                        }
                        stack.push(node);
                        status = "numerical_error".into();
                        break;
                    }
                }
                continue;
            }
            Ok(lp::Outcome::Optimal { x, .. }) => {
                node.warm = next_warm;
                let original: Vec<f64> = p
                    .terms
                    .iter()
                    .enumerate()
                    .map(|(j, t)| p.shift[j] + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>())
                    .collect();
                if let Err(error) = model.check(&original, false, 1e-6) {
                    if std::env::var_os("MIP_LP_TRACE").is_some() {
                        eprintln!("Original LP check failed: {error}");
                    }
                    stack.push(node);
                    status = "numerical_error".into();
                    break;
                }
                if nodes == 1 {
                    if let Some(path) = std::env::var_os("MIP_ROOT_LP_DUMP") {
                        if let Ok(file) = std::fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(path)
                        {
                            let names: Vec<_> = model.variables.iter().map(|v| &v.name).collect();
                            let _ = serde_json::to_writer(
                                file,
                                &serde_json::json!({
                                    "purpose": "diagnostic LP relaxation point, not an integer solution",
                                    "names": names, "point": &original,
                                    "objective": model.objective(&original),
                                    "standard_dual": relaxation_dual,
                                    "multipliers": relaxation_witness.as_ref(), "row_sources": &p.row_sources
                                }),
                            );
                        }
                    }
                }
                let objective = sign * model.objective(&original);
                if !objective.is_finite() {
                    stack.push(node);
                    status = "numerical_error".into();
                    break;
                }
                if let Some(edge) = node.edge {
                    costs.observe(edge, objective);
                }
                let bound = if let Some(dual) = relaxation_dual {
                    let value = crate::exact::dot_lower(
                        std::iter::once((dual, 1.))
                            .chain(std::iter::once((sign * model.offset, 1.)))
                            .chain(
                                model
                                    .variables
                                    .iter()
                                    .zip(&p.shift)
                                    .map(|(v, &s)| (sign * v.objective, s)),
                            ),
                    );
                    let Some(value) = value else {
                        stack.push(node);
                        status = "numerical_error".into();
                        break;
                    };
                    value.min(objective)
                } else {
                    objective
                };
                // Small negative roundoff in a child must not decrease inherited bounds.
                node.bound = node.bound.max(bound);
                if let (Some(grid), Some(certifier), Some(y)) = (
                    &objective_grid,
                    &objective_certifier,
                    relaxation_witness.as_deref(),
                ) {
                    let profitable = (nodes == 1 || limits.objective_rounding_all)
                        && node.bound < incumbent
                        && grid
                            .round_lower(node.bound, model.maximize)
                            .is_some_and(|candidate| {
                                candidate > node.bound + 1e-9 * (1. + node.bound.abs())
                                    && (nodes == 1
                                        || candidate + 1e-8 * (1. + candidate.abs()) >= incumbent)
                            });
                    // 近似値は計算を省く判定だけに使い、採用するboundは必ず別に証明する。
                    if profitable {
                        let started = Instant::now();
                        objective_bound_stats.calls += 1;
                        if y.len() == p.row_sources.len() {
                            let mut original_y = vec![0.; lp_model.rows.len()];
                            for (&value, &(i, s)) in y.iter().zip(&p.row_sources) {
                                original_y[i] += s * value;
                            }
                            for factor in [1., 1. - 2f64.powi(-40), 1. + 2f64.powi(-40)] {
                                objective_bound_stats.trials += 1;
                                let multipliers: Vec<_> =
                                    original_y.iter().map(|y| y * factor).collect();
                                if let Some(rigorous) =
                                    certifier.bound(&node.lower, &node.upper, &multipliers)
                                {
                                    objective_bound_stats.valid += 1;
                                    if let Some(rounded) =
                                        grid.round_lower(rigorous, model.maximize)
                                    {
                                        let certified = rounded as i64;
                                        node.certified_bound = Some(
                                            node.certified_bound
                                                .map_or(certified, |old| old.max(certified)),
                                        );
                                        if rounded > node.bound {
                                            node.bound = rounded;
                                            objective_bound_stats.rounded += 1;
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                        objective_bound_stats.seconds += started.elapsed().as_secs_f64();
                    }
                }
                if nodes == 1 && !limits.objective_rounding_all {
                    objective_certifier = None;
                }
                if nodes == 1
                    && node
                        .certified_bound
                        .is_some_and(|b| incumbent < b as f64 - 1e-6)
                {
                    objective_bound_stats.rejected_candidates += 1;
                    incumbent = f64::INFINITY;
                    best = None;
                    first_primal_seconds = None;
                }
                if node.bound >= incumbent {
                    continue;
                }
                let mut branch = costs.select(model, &original, limits.branching);
                if branch.is_none()
                    && node
                        .certified_bound
                        .is_some_and(|b| objective < b as f64 - 1e-6)
                {
                    branch = model
                        .variables
                        .iter()
                        .enumerate()
                        .filter(|(_, v)| v.integer)
                        .filter_map(|(j, _)| {
                            let f = original[j] - original[j].floor();
                            let d = f.min(1. - f);
                            (d > 0.).then_some((j, d))
                        })
                        .max_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
                        .map(|(j, _)| j);
                }
                if branch.is_some() && (nodes == 1 || nodes % 256 == 0) {
                    if let Some(h) = &mut repair {
                        let allocation = if nodes == 1 { 0.05 } else { 0.001 };
                        let repair_deadline = deadline
                            .min(Instant::now() + std::time::Duration::from_secs_f64(allocation));
                        // 整数値を固定した連続部分の修復。結果は主解だけに使う。
                        if matches!(limits.engine, Engine::Revised)
                            && Instant::now() < repair_deadline
                        {
                            let mut fixed = node.clone();
                            for (j, v) in model.variables.iter().enumerate() {
                                if v.integer {
                                    let mut value = original[j].round();
                                    if let Some(l) = node.lower[j] {
                                        value = value.max(l.ceil());
                                    }
                                    if let Some(u) = node.upper[j] {
                                        value = value.min(u.floor());
                                    }
                                    fixed.lower[j] = Some(value);
                                    fixed.upper[j] = Some(value);
                                }
                            }
                            if let Relaxation::Ready(q) = standard_sparse(
                                lp_model,
                                &fixed,
                                limits.dense_bytes,
                                false,
                                limits.fixed_columns,
                            ) {
                                let answer = revised::minimize_warm(
                                    q.sparse.as_ref().unwrap(),
                                    limits.pivots.min(256),
                                    Some(repair_deadline),
                                    node.warm.as_deref(),
                                );
                                lp_solves += 1;
                                repair_lp_solves += 1;
                                lp_stats.add(&answer.stats);
                                if let lp::Outcome::Optimal { x, .. } = answer.outcome {
                                    let x: Vec<_> = q
                                        .terms
                                        .iter()
                                        .enumerate()
                                        .map(|(j, t)| {
                                            q.shift[j]
                                                + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>()
                                        })
                                        .collect();
                                    if h.candidate_check(&x) {
                                        repair_lp_candidates += 1;
                                        let value = sign * model.objective(&x);
                                        if value.is_finite()
                                            && value < incumbent
                                            && candidate_respects_bound(
                                                value,
                                                node.certified_bound,
                                                &mut objective_bound_stats,
                                            )
                                        {
                                            repair_improvements += 1;
                                            incumbent = value;
                                            first_primal_seconds.get_or_insert_with(|| {
                                                start.elapsed().as_secs_f64()
                                            });
                                            best = Some(x);
                                        }
                                    }
                                }
                            }
                        }
                        let budget = model
                            .rows
                            .iter()
                            .map(|r| r.coefficients.len() + 1)
                            .sum::<usize>()
                            .saturating_mul(256)
                            .clamp(4096, 2_000_000);
                        if let Some(x) = h.search(
                            &original,
                            &node.lower,
                            &node.upper,
                            repair_deadline,
                            budget,
                            nodes as u64,
                        ) {
                            let value = sign * model.objective(&x);
                            if value.is_finite()
                                && value < incumbent
                                && candidate_respects_bound(
                                    value,
                                    node.certified_bound,
                                    &mut objective_bound_stats,
                                )
                            {
                                repair_improvements += 1;
                                incumbent = value;
                                first_primal_seconds
                                    .get_or_insert_with(|| start.elapsed().as_secs_f64());
                                best = Some(x);
                            }
                        }
                    }
                    if node.bound >= incumbent {
                        continue;
                    }
                }
                if limits.feasibility_pump
                    && !incumbent.is_finite()
                    && nodes == 1
                    && branch.is_some()
                    && matches!(limits.engine, Engine::Revised)
                    && Instant::now() < deadline
                {
                    let started = Instant::now();
                    let pump_deadline = deadline.min(
                        started
                            + std::time::Duration::from_secs_f64((limits.seconds * 0.1).min(1.)),
                    );
                    let old_solves = pump.lp_solves;
                    if let Some(x) = binary_pump(
                        model,
                        &node,
                        &p,
                        &original,
                        limits,
                        pump_deadline,
                        &mut pump,
                        &mut lp_stats,
                    ) {
                        let value = sign * model.objective(&x);
                        if value.is_finite()
                            && value < incumbent
                            && candidate_respects_bound(
                                value,
                                node.certified_bound,
                                &mut objective_bound_stats,
                            )
                        {
                            pump.improvements += 1;
                            incumbent = value;
                            first_primal_seconds
                                .get_or_insert_with(|| start.elapsed().as_secs_f64());
                            best = Some(x);
                        }
                    }
                    lp_solves += pump.lp_solves - old_solves;
                    pump.seconds += started.elapsed().as_secs_f64();
                }
                if limits.lp_diving
                    && nodes == 1
                    && branch.is_some()
                    && matches!(limits.engine, Engine::Revised)
                {
                    let started = Instant::now();
                    let diving_deadline = deadline.min(
                        started
                            + std::time::Duration::from_secs_f64((limits.seconds * 0.05).min(0.25)),
                    );
                    let old_solves = diving.lp_solves;
                    let candidate = dive(
                        lp_model,
                        &node,
                        &original,
                        limits,
                        diving_deadline,
                        &mut diving,
                        &mut lp_stats,
                    );
                    if let Some(x) = candidate {
                        let value = sign * model.objective(&x);
                        if value.is_finite()
                            && value < incumbent
                            && candidate_respects_bound(
                                value,
                                node.certified_bound,
                                &mut objective_bound_stats,
                            )
                        {
                            diving.improvements += 1;
                            incumbent = value;
                            first_primal_seconds
                                .get_or_insert_with(|| start.elapsed().as_secs_f64());
                            best = Some(x);
                        }
                    }
                    lp_solves += diving.lp_solves - old_solves;
                    diving.seconds += started.elapsed().as_secs_f64();
                }
                if limits.primal_probe
                    && node.bound < incumbent
                    && nodes == 1
                    && branch.is_some()
                    && matches!(limits.engine, Engine::Revised)
                    && Instant::now() < deadline
                {
                    let probe_started = Instant::now();
                    let mut probe_limits = limits.clone();
                    probe_limits.cover_cuts = false;
                    probe_limits.mir_cuts = false;
                    probe_limits.three_row_cuts = false;
                    probe_limits.box_primal = false;
                    probe_limits.rounding_cuts = false;
                    probe_limits.strong_branching = false;
                    probe_limits.rins = false;
                    probe_limits.objective_rounding = false;
                    probe_limits.objective_rounding_all = false;
                    probe_limits.primal_probe = false; // 再帰は一段で必ず止める。
                    probe_limits.primal_repair = false;
                    probe_limits.lp_diving = false;
                    probe_limits.feasibility_pump = false;
                    probe_limits.search = search::Strategy::DepthFirst;
                    probe_limits.branching = branching::Rule::Pseudocost;
                    probe_limits.child_order = branching::Order::Estimate;
                    probe_limits.nodes = 4096;
                    probe_limits.seconds = (limits.seconds * 0.2).min(1.0).min(
                        deadline
                            .saturating_duration_since(Instant::now())
                            .as_secs_f64(),
                    );
                    let result = if let Some((primal_model, warm)) = primal_root.take() {
                        probe.precut_roots += 1;
                        probe_limits.binary_graph = false;
                        probe_limits.aggregate_cuts = false;
                        probe_limits.clique_cuts = false;
                        probe_limits.branching = limits.branching;
                        probe_limits.child_order = limits.child_order;
                        probe_limits.strong_branching = limits.strong_branching;
                        probe_limits.objective_rounding = limits.objective_rounding;
                        probe_limits.objective_rounding_all = limits.objective_rounding_all;
                        solve_seeded_warm(&primal_model, &probe_limits, None, Some(warm))
                    } else if node.warm.is_some()
                        && prepared.initial_lp_seconds >= probe_limits.seconds
                    {
                        solve_seeded_warm(lp_model, &probe_limits, None, node.warm.clone())
                    } else {
                        solve(model, &probe_limits)
                    };
                    probe.calls += 1;
                    probe.nodes += result.nodes;
                    probe.lp_solves += result.lp_solves;
                    probe.max_open_nodes = probe.max_open_nodes.max(result.max_open_nodes);
                    probe.status = Some(result.status.clone());
                    lp_solves += result.lp_solves;
                    add_lp_result(&mut lp_stats, &result);
                    if let Some(values) = result.values {
                        let x: Option<Vec<_>> = model
                            .variables
                            .iter()
                            .map(|v| values.get(&v.name).copied())
                            .collect();
                        if let Some(x) = x {
                            if heuristic::Repair::new(model).candidate_check(&x) {
                                probe.candidates += 1;
                                let value = sign * model.objective(&x);
                                if value.is_finite()
                                    && value < incumbent
                                    && candidate_respects_bound(
                                        value,
                                        node.certified_bound,
                                        &mut objective_bound_stats,
                                    )
                                {
                                    probe.improvements += 1;
                                    incumbent = value;
                                    first_primal_seconds
                                        .get_or_insert_with(|| start.elapsed().as_secs_f64());
                                    best = Some(x);
                                }
                            }
                        }
                    }
                    probe.seconds += probe_started.elapsed().as_secs_f64();
                }
                if node.bound >= incumbent {
                    continue;
                }
                if limits.rins
                    && nodes == 1
                    && rins.attempts == 0
                    && branch.is_some()
                    && best.is_some()
                    && matches!(limits.engine, Engine::Revised)
                    && Instant::now() < deadline
                {
                    let started = Instant::now();
                    rins.attempts += 1;
                    rins.at_node = Some(nodes);
                    let local_deadline = deadline.min(
                        started
                            + std::time::Duration::from_secs_f64((limits.seconds * 0.1).min(0.5)),
                    );
                    if let Some(neighborhood) =
                        rins::neighborhood(model, &original, best.as_ref().unwrap())
                    {
                        rins.fixed = neighborhood.fixed;
                        rins.eligible = neighborhood.eligible;
                        if Instant::now() < local_deadline {
                            let mut local = limits.clone();
                            local.rins = false;
                            local.objective_rounding = false;
                            local.objective_rounding_all = false;
                            local.primal_probe = false;
                            local.primal_repair = false;
                            local.lp_diving = false;
                            local.feasibility_pump = false;
                            local.strong_branching = false;
                            local.cover_cuts = false;
                            // This neighborhood search seeks a quick incumbent;
                            // root MIR trials must not redirect its limited tree.
                            local.mir_cuts = false;
                            local.three_row_cuts = false;
                            local.box_primal = false;
                            local.rounding_cuts = false;
                            local.propagation = false;
                            local.nodes = 1024;
                            local.pivots = local.pivots.min(4096);
                            local.seconds = local_deadline
                                .saturating_duration_since(Instant::now())
                                .as_secs_f64();
                            local.search = search::Strategy::DepthFirst;
                            local.branching = branching::Rule::Pseudocost;
                            local.child_order = branching::Order::Nearest;
                            let result =
                                solve_seeded(&neighborhood.model, &local, Some(&neighborhood.seed));
                            rins.calls += 1;
                            rins.nodes += result.nodes;
                            rins.lp_solves += result.lp_solves;
                            rins.status = Some(result.status.clone());
                            lp_solves += result.lp_solves;
                            add_lp_result(&mut lp_stats, &result);
                            if let Some(values) = result.values {
                                let x: Option<Vec<_>> = model
                                    .variables
                                    .iter()
                                    .map(|v| values.get(&v.name).copied())
                                    .collect();
                                if let Some(x) = x {
                                    if heuristic::Repair::new(model).candidate_check(&x) {
                                        rins.candidates += 1;
                                        let value = sign * model.objective(&x);
                                        if value.is_finite()
                                            && value < incumbent
                                            && candidate_respects_bound(
                                                value,
                                                node.certified_bound,
                                                &mut objective_bound_stats,
                                            )
                                        {
                                            rins.improvements += 1;
                                            incumbent = value;
                                            best = Some(x);
                                            first_primal_seconds.get_or_insert_with(|| {
                                                start.elapsed().as_secs_f64()
                                            });
                                        }
                                    }
                                }
                            }
                        } else {
                            rins.limited += 1;
                        }
                    }
                    rins.seconds += started.elapsed().as_secs_f64();
                }
                if node.bound >= incumbent {
                    continue;
                }
                let branch = if limits.strong_branching
                    && nodes == 1
                    && branch.is_some()
                    && matches!(limits.engine, Engine::Revised)
                    && Instant::now() < deadline
                {
                    let old_solves = strong.lp_solves;
                    let strong_deadline = deadline.min(
                        Instant::now()
                            + std::time::Duration::from_secs_f64((limits.seconds * 0.05).min(0.1)),
                    );
                    let selected = strong_branch(
                        lp_model,
                        &node,
                        &original,
                        limits,
                        strong_deadline,
                        &mut costs,
                        &mut strong,
                        &mut lp_stats,
                    )
                    .or(branch);
                    lp_solves += strong.lp_solves - old_solves;
                    if selected != branch {
                        strong.branch_changes += 1;
                    }
                    selected
                } else {
                    branch
                };
                if let Some(j) = branch {
                    let prefer_up = costs.prefer_up(j, original[j], limits.child_order);
                    let mut up = node.clone();
                    up.changed = Some(j);
                    up.edge = Some(branching::Edge {
                        variable: j,
                        up: false,
                        distance: original[j] - original[j].floor(),
                        parent_objective: objective,
                    });
                    up.upper[j] = Some(
                        up.upper[j].map_or(original[j].floor(), |v| v.min(original[j].floor())),
                    );
                    let mut down = node;
                    down.changed = Some(j);
                    down.edge = Some(branching::Edge {
                        variable: j,
                        up: true,
                        distance: original[j].ceil() - original[j],
                        parent_objective: objective,
                    });
                    down.lower[j] = Some(
                        down.lower[j].map_or(original[j].ceil(), |v| v.max(original[j].ceil())),
                    );
                    if prefer_up {
                        stack.push(up);
                        stack.push(down);
                    } else {
                        stack.push(down);
                        stack.push(up);
                    }
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
                    if !value.is_finite()
                        || !candidate_respects_bound(
                            value,
                            node.certified_bound,
                            &mut objective_bound_stats,
                        )
                    {
                        stack.push(node);
                        status = "numerical_error".into();
                        break;
                    }
                    terminal_bound = terminal_bound.min(node.bound.min(value));
                    if value < incumbent {
                        incumbent = value;
                        first_primal_seconds.get_or_insert_with(|| start.elapsed().as_secs_f64());
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
    deferred.prune(incumbent);
    let lp_search_termination = Some(if status == "optimal" {
        "active_search_complete".into()
    } else {
        status.clone()
    });
    if deferred.pending != 0 {
        status = "numerical_error".into();
    }
    let mut terminal_neighborhood = None;
    // 数値停止後の主解専用探索。局所の境界・終了判定は大域探索に戻さない。
    if status == "numerical_error"
        && limits.rins
        && matches!(limits.engine, Engine::Revised)
        && limits.terminal_neighborhood
        && best.is_some()
        && model.variables.len() <= 50_000
        && model.rows.len() <= 50_000
        && model.variables.iter().filter(|v| v.integer).count() <= 4096
        && model
            .rows
            .iter()
            .map(|r| r.coefficients.len())
            .sum::<usize>()
            <= 1_000_000
        && Instant::now() < deadline
    {
        let mut terminal = RinsStats::default();
        terminal.at_node = Some(nodes);
        let started = Instant::now();
        let end = deadline.min(started + std::time::Duration::from_secs(10));
        let protected_bound = stack.min_bound(terminal_bound.min(deferred.minimum));
        for attempt in 0..16 {
            if Instant::now() >= end {
                break;
            }
            terminal.attempts += 1;
            let seed = best.as_ref().unwrap();
            let Some(neighborhood) = rins::neighborhood_released(
                model,
                seed,
                seed,
                [20, 40, 60, 80][attempt % 4],
                attempt as u64 + 1,
            ) else {
                continue;
            };
            if Instant::now() >= end {
                break;
            }
            let mut local = limits.clone();
            local.rins = false;
            local.objective_rounding = false;
            local.objective_rounding_all = false;
            local.primal_probe = false;
            local.primal_repair = false;
            local.lp_diving = false;
            local.feasibility_pump = false;
            local.strong_branching = false;
            local.cover_cuts = false;
            local.mir_cuts = false;
            local.three_row_cuts = false;
            local.box_primal = false;
            local.rounding_cuts = false;
            local.propagation = false;
            local.nodes = 1024;
            local.pivots = local.pivots.min(4096);
            local.seconds = end
                .saturating_duration_since(Instant::now())
                .as_secs_f64()
                .min(1.);
            local.search = search::Strategy::DepthFirst;
            local.branching = branching::Rule::Pseudocost;
            local.child_order = branching::Order::Nearest;
            let result = solve_seeded(&neighborhood.model, &local, Some(&neighborhood.seed));
            terminal.calls += 1;
            terminal.fixed = neighborhood.fixed;
            terminal.eligible = neighborhood.eligible;
            terminal.nodes += result.nodes;
            terminal.lp_solves += result.lp_solves;
            terminal.status = Some(format!("terminal:{}", result.status));
            lp_solves += result.lp_solves;
            add_lp_result(&mut lp_stats, &result);
            if let Some(values) = result.values {
                let candidate: Option<Vec<_>> = model
                    .variables
                    .iter()
                    .map(|v| values.get(&v.name).copied())
                    .collect();
                if let Some(x) = candidate {
                    if heuristic::Repair::new(model).candidate_check(&x) {
                        terminal.candidates += 1;
                        let value = sign * model.objective(&x);
                        if value.is_finite()
                            && value < incumbent
                            && (protected_bound == f64::NEG_INFINITY
                                || value + 1e-8 * (1. + value.abs().max(protected_bound.abs()))
                                    >= protected_bound)
                        {
                            incumbent = value;
                            best = Some(x);
                            terminal.improvements += 1;
                        }
                    }
                }
            }
        }
        terminal.seconds += started.elapsed().as_secs_f64();
        terminal_neighborhood = Some(terminal);
    }
    let internal_dual = stack.min_bound(terminal_bound.min(incumbent).min(deferred.minimum));
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
    let propagation_stats = propagator.map_or_else(propagate::Stats::default, |p| p.stats);
    let repair_stats = repair.map_or_else(heuristic::Stats::default, |h| h.stats);
    Result {
        parity_proof: None,
        binary_graph_enabled: limits.binary_graph,
        graph_cover_local_enabled: limits.graph_cover_local,
        aggregate_cuts_enabled: limits.aggregate_cuts,
        clique_cuts_enabled: limits.clique_cuts,
        mir_cuts_enabled: limits.mir_cuts,
        three_row_cuts_enabled: limits.three_row_cuts,
        graph_fractional_cover_enabled: limits.graph_fractional_cover,
        defer_infeasibility_enabled: limits.defer_infeasibility,
        joint_infeasibility_enabled: limits.joint_infeasibility,
        aggregate_infeasibility_enabled: limits.aggregate_infeasibility,
        binary_graph: None,
        deferred_infeasibilities: deferred.observed,
        deferred_infeasibilities_pending: deferred.pending,
        deferred_infeasibilities_pruned: deferred.pruned,
        deferred_infeasibility_bound: deferred
            .minimum
            .is_finite()
            .then_some(sign * deferred.minimum),
        deferred_infeasibility_unbounded: deferred.minimum == f64::NEG_INFINITY,
        lp_search_termination,
        branching: format!("{:?}", limits.branching),
        child_order: format!("{:?}", limits.child_order),
        search_strategy: format!("{:?}", limits.search),
        search_pops: stack.pops,
        search_best_pops: stack.best_pops,
        search_transfers: stack.transfers,
        max_open_nodes: stack.max_open,
        pseudocost_updates: costs.updates,
        pseudocost_selections: costs.selections,
        primal_repair: limits.primal_repair,
        box_primal_enabled: limits.box_primal,
        box_primal,
        root_seed_repair,
        root_incumbent_conflict,
        lp_diving: limits.lp_diving,
        primal_probe: limits.primal_probe,
        cover_cuts_enabled: limits.cover_cuts,
        cover_round_limit: limits.cover_rounds,
        rounding_cuts_enabled: limits.rounding_cuts,
        strong_branching: limits.strong_branching,
        strong_objective: limits.strong_objective,
        strong_infeasible_hint: limits.strong_infeasible_hint,
        strong,
        rins_enabled: limits.rins,
        rins,
        terminal_neighborhood,
        objective_rounding: limits.objective_rounding,
        objective_rounding_all: limits.objective_rounding_all,
        objective_grid,
        objective_bound_stats,
        infeasibility_proof_calls,
        infeasibility_proof_trials,
        infeasibility_proof_failures,
        infeasibility_proof_seconds,
        cut_proofs: prepared.cuts,
        cut_stats: prepared.stats,
        cut_rounds: prepared.rounds,
        cut_lp_solves: prepared.lp_solves,
        cut_seconds: prepared.seconds,
        cut_initial_lp_seconds: prepared.initial_lp_seconds,
        root_lp_reused,
        probe,
        diving,
        feasibility_pump_enabled: limits.feasibility_pump,
        pump,
        repair_calls: repair_stats.calls,
        repair_candidates: repair_stats.candidates,
        repair_improvements,
        repair_moves: repair_stats.moves,
        first_primal_seconds,
        repair_lp_solves,
        repair_lp_candidates,
        repair_work: repair_stats.work,
        propagation: limits.propagation,
        propagation_calls: propagation_stats.calls,
        propagation_rows: propagation_stats.rows,
        propagation_tightenings: propagation_stats.tightenings,
        propagation_pruned: propagation_stats.pruned,
        propagation_budget_hits: propagation_stats.budget_hits,
        fixed_leaves,
        lp_solves,
        fixed_columns_enabled: requested_fixed_columns,
        lp_workspace_bytes: limits.dense_bytes,
        root_fixed_columns,
        lp_scaled_columns: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.scaled_columns),
        lp_scaled_rows: matches!(limits.engine, Engine::Revised).then_some(lp_stats.scaled_rows),
        lp_engine: format!("{:?}", limits.engine),
        lp_warm_start: limits.warm_start && matches!(limits.engine, Engine::Revised),
        lp_warm_attempts: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.warm_attempts),
        lp_warm_successes: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.warm_successes),
        lp_warm_fallbacks: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.warm_fallbacks),
        lp_warm_row_extensions: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.warm_row_extensions),
        lp_warm_rejections: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.warm_rejections),
        lp_warm_cost_attempts: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.warm_cost_attempts),
        lp_warm_cost_successes: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.warm_cost_successes),
        lp_restoration_attempts: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.restoration_attempts),
        lp_restoration_restarts: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.restoration_restarts),
        lp_restoration_extended_attempts: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.restoration_extended_attempts),
        lp_restoration_successes: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.restoration_successes),
        lp_restoration_infeasible_candidates: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.restoration_infeasible_candidates),
        lp_restoration_pivots: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.restoration_pivots),
        lp_refinement_attempts: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.refinement_attempts),
        lp_refinement_steps: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.refinement_steps),
        lp_refinement_successes: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.refinement_successes),
        lp_certificate_repair_trials: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.certificate_repair_trials),
        lp_certificate_repairs: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.certificate_repairs),
        lp_multiplier_cache_hits: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.multiplier_cache_hits),
        lp_multiplier_cache_misses: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.multiplier_cache_misses),
        lp_dual_pivots: matches!(limits.engine, Engine::Revised).then_some(lp_stats.dual_pivots),
        lp_dual_flips: matches!(limits.engine, Engine::Revised).then_some(lp_stats.dual_flips),
        lp_pivots: matches!(limits.engine, Engine::Revised).then_some(lp_stats.pivots),
        lp_feasibility_pricing_calls: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.feasibility_pricing_calls),
        lp_pricing_cutoffs: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.pricing_cutoffs),
        lp_pricing_fallbacks: matches!(limits.engine, Engine::Revised)
            .then_some(lp_stats.pricing_fallbacks),
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

/// 根の候補を試して順位と擬似コストだけを学ぶ。局所status/下界で本体を枝刈りしない。
/// 参考: SCIP branch_relpscost.hの数学的説明。実装コードの移植は行わない。
/// https://scipopt.org/scip/doc/html/branch__relpscost_8h.php
#[derive(Default, Debug, Serialize)]
pub struct StrongStats {
    pub calls: usize,
    pub candidates: usize,
    pub lp_solves: usize,
    pub optimal: usize,
    pub infeasible: usize,
    pub limited: usize,
    pub numerical_errors: usize,
    pub branch_changes: usize,
    pub evaluated_variables: Vec<usize>,
    pub selected_variable: Option<usize>,
    pub infeasible_hint_selections: usize,
    pub seconds: f64,
}
fn strong_branch(
    model: &Model,
    root: &Node,
    point: &[f64],
    limits: &Limits,
    deadline: Instant,
    costs: &mut branching::Costs,
    stats: &mut StrongStats,
    lp_stats: &mut revised::Stats,
) -> Option<usize> {
    let started = Instant::now();
    stats.calls += 1;
    let candidates = branching::strong_candidates(model, point, limits.strong_objective);
    let sign = if model.maximize { -1. } else { 1. };
    let parent = sign * model.objective(point);
    let floor = (parent.abs() * 1e-9).max(1e-8);
    let mut best: Option<(usize, f64, usize)> = None;
    for (j, _) in candidates {
        if Instant::now() >= deadline {
            break;
        }
        stats.candidates += 1;
        stats.evaluated_variables.push(j);
        let mut gains = [None, None];
        let mut infeasible = [false; 2];
        for up in [false, true] {
            if Instant::now() >= deadline {
                stats.limited += 1;
                break;
            }
            let mut child = root.clone();
            child.changed = Some(j);
            let distance = if up {
                point[j].ceil() - point[j]
            } else {
                point[j] - point[j].floor()
            };
            if up {
                child.lower[j] =
                    Some(child.lower[j].map_or(point[j].ceil(), |b| b.max(point[j].ceil())));
            } else {
                child.upper[j] =
                    Some(child.upper[j].map_or(point[j].floor(), |b| b.min(point[j].floor())));
            }
            let relaxation = standard_sparse(
                model,
                &child,
                limits.dense_bytes,
                false,
                limits.fixed_columns,
            );
            let q = match relaxation {
                Relaxation::Ready(q) => q,
                Relaxation::Infeasible => {
                    stats.infeasible += 1;
                    infeasible[usize::from(up)] = true;
                    continue;
                }
                Relaxation::ResourceLimit => {
                    stats.limited += 1;
                    continue;
                }
            };
            stats.lp_solves += 1;
            let answer = revised::minimize_warm(
                q.sparse.as_ref().unwrap(),
                limits.pivots.min(256),
                Some(deadline),
                if limits.warm_start {
                    root.warm.as_deref()
                } else {
                    None
                },
            );
            lp_stats.add(&answer.stats);
            match answer.outcome {
                lp::Outcome::Optimal { x, .. } => {
                    let original: Vec<_> = q
                        .terms
                        .iter()
                        .enumerate()
                        .map(|(j, t)| q.shift[j] + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>())
                        .collect();
                    let objective = sign * model.objective(&original);
                    if !objective.is_finite() || model.check(&original, false, 1e-6).is_err() {
                        stats.numerical_errors += 1;
                        continue;
                    }
                    stats.optimal += 1;
                    gains[usize::from(up)] = Some((objective - parent).max(0.));
                    costs.observe(
                        branching::Edge {
                            variable: j,
                            up,
                            distance,
                            parent_objective: parent,
                        },
                        objective,
                    );
                }
                lp::Outcome::Infeasible => {
                    stats.infeasible += 1;
                    infeasible[usize::from(up)] = true;
                }
                lp::Outcome::TimeLimit | lp::Outcome::IterationLimit => stats.limited += 1,
                _ => stats.numerical_errors += 1,
            }
        }
        let hints = infeasible.iter().filter(|&&v| v).count();
        let score = if limits.strong_infeasible_hint && hints != 0 {
            Some(
                1e6 * hints as f64
                    + gains
                        .iter()
                        .map(|g| g.unwrap_or(floor).max(floor).ln())
                        .sum::<f64>(),
            )
        } else if let [Some(down), Some(up)] = gains {
            Some(down.max(floor).ln() + up.max(floor).ln())
        } else {
            None
        };
        if let Some(score) = score {
            if best.is_none_or(|(_, old, _)| score > old) {
                best = Some((j, score, hints));
            }
        }
    }
    stats.seconds += started.elapsed().as_secs_f64();
    if let Some((j, _, hints)) = best {
        stats.selected_variable = Some(j);
        stats.infeasible_hint_selections +=
            usize::from(limits.strong_infeasible_hint && hints != 0);
        Some(j)
    } else {
        None
    }
}

#[derive(Default, Debug, Serialize)]
pub struct RinsStats {
    pub at_node: Option<usize>,
    pub attempts: usize,
    pub calls: usize,
    pub fixed: usize,
    pub eligible: usize,
    pub nodes: usize,
    pub lp_solves: usize,
    pub candidates: usize,
    pub improvements: usize,
    pub limited: usize,
    pub status: Option<String>,
    pub seconds: f64,
}
fn add_lp_result(stats: &mut revised::Stats, result: &Result) {
    stats.add(&revised::Stats {
        pivots: result.lp_pivots.unwrap_or(0),
        feasibility_pricing_calls: result.lp_feasibility_pricing_calls.unwrap_or(0),
        pricing_cutoffs: result.lp_pricing_cutoffs.unwrap_or(0),
        pricing_fallbacks: result.lp_pricing_fallbacks.unwrap_or(0),
        flips: result.lp_flips.unwrap_or(0),
        refactors: result.lp_refactors.unwrap_or(0),
        warm_attempts: result.lp_warm_attempts.unwrap_or(0),
        warm_successes: result.lp_warm_successes.unwrap_or(0),
        warm_fallbacks: result.lp_warm_fallbacks.unwrap_or(0),
        warm_rejections: result.lp_warm_rejections.unwrap_or(0),
        warm_row_extensions: result.lp_warm_row_extensions.unwrap_or(0),
        warm_cost_attempts: result.lp_warm_cost_attempts.unwrap_or(0),
        warm_cost_successes: result.lp_warm_cost_successes.unwrap_or(0),
        multiplier_cache_hits: result.lp_multiplier_cache_hits.unwrap_or(0),
        multiplier_cache_misses: result.lp_multiplier_cache_misses.unwrap_or(0),
        certificate_repair_trials: result.lp_certificate_repair_trials.unwrap_or(0),
        certificate_repairs: result.lp_certificate_repairs.unwrap_or(0),
        restoration_attempts: result.lp_restoration_attempts.unwrap_or(0),
        restoration_restarts: result.lp_restoration_restarts.unwrap_or(0),
        restoration_extended_attempts: result.lp_restoration_extended_attempts.unwrap_or(0),
        restoration_successes: result.lp_restoration_successes.unwrap_or(0),
        restoration_infeasible_candidates: result.lp_restoration_infeasible_candidates.unwrap_or(0),
        restoration_pivots: result.lp_restoration_pivots.unwrap_or(0),
        refinement_attempts: result.lp_refinement_attempts.unwrap_or(0),
        refinement_steps: result.lp_refinement_steps.unwrap_or(0),
        refinement_successes: result.lp_refinement_successes.unwrap_or(0),
        dual_pivots: result.lp_dual_pivots.unwrap_or(0),
        dual_flips: result.lp_dual_flips.unwrap_or(0),
        scaled_columns: result.lp_scaled_columns.unwrap_or(0),
        scaled_rows: result.lp_scaled_rows.unwrap_or(0),
    });
}

#[cfg(test)]
mod seed_tests {
    use super::*;
    #[test]
    fn seed_is_checked_and_cannot_prove_optimality_at_zero_budget() {
        let m = crate::mps::parse(std::io::Cursor::new(
            "NAME seed\nROWS\n N obj\nCOLUMNS\n x obj 1\nRHS\nBOUNDS\n BV b x\nENDATA\n",
        ))
        .unwrap();
        let limits = Limits {
            seconds: 0.,
            cover_cuts: false,
            ..Limits::default()
        };
        let result = solve_seeded(&m, &limits, Some(&[1.]));
        assert_eq!(result.status, "time_limit");
        assert_eq!(result.objective, Some(1.));
        assert_eq!(result.dual_bound, None);
        let invalid = solve_seeded(&m, &limits, Some(&[2.]));
        assert_eq!(invalid.objective, None);
        let fractional = solve_seeded(&m, &limits, Some(&[0.5]));
        assert_eq!(fractional.objective, None);
    }
}

#[derive(Default, Debug, Serialize)]
pub struct ObjectiveBoundStats {
    pub box_pruned: usize,
    pub rejected_candidates: usize,
    pub calls: usize,
    pub trials: usize,
    pub valid: usize,
    pub rounded: usize,
    pub seconds: f64,
}

fn candidate_respects_bound(
    value: f64,
    bound: Option<i64>,
    stats: &mut ObjectiveBoundStats,
) -> bool {
    if bound.is_some_and(|b| value < b as f64 - 1e-6) {
        stats.rejected_candidates += 1;
        false
    } else {
        true
    }
}

#[cfg(test)]
mod pump_tests {
    use super::*;
    fn prepare(text: &str) -> (Model, Node, Standard, Vec<f64>) {
        let m = crate::mps::parse(std::io::Cursor::new(text)).unwrap();
        let mut node = Node {
            certified_bound: None,
            lower: m.variables.iter().map(|v| v.lower).collect(),
            upper: m.variables.iter().map(|v| v.upper).collect(),
            bound: f64::NEG_INFINITY,
            warm: None,
            changed: None,
            edge: None,
        };
        let Relaxation::Ready(p) = standard_sparse(&m, &node, 1 << 24, false, true) else {
            panic!("no LP")
        };
        let a = revised::minimize(p.sparse.as_ref().unwrap(), 1000, None);
        let lp::Outcome::Optimal { x, .. } = a.outcome else {
            panic!("no optimal root")
        };
        node.warm = a.warm;
        let point = p
            .terms
            .iter()
            .enumerate()
            .map(|(j, t)| p.shift[j] + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>())
            .collect();
        (m, node, p, point)
    }
    const FRACTIONAL: &str = "NAME pump\nOBJSENSE\n MAX\nROWS\n N obj\n L r\nCOLUMNS\n x obj 1 r 1\n y obj 1 r 1\nRHS\n rhs r 1.5\nBOUNDS\n BV b x\n BV b y\nENDATA\n";
    #[test]
    fn local_probe_reuses_root_warm_without_changing_parent_or_bound() {
        let (m, node, p, _) = prepare(FRACTIONAL);
        let limits = Limits {
            binary_graph: false,
            box_primal: false,
            cover_cuts: false,
            aggregate_cuts: false,
            clique_cuts: false,
            mir_cuts: false,
            rounding_cuts: false,
            objective_rounding: false,
            strong_branching: false,
            rins: false,
            primal_repair: false,
            lp_diving: false,
            feasibility_pump: false,
            nodes: 1,
            ..Limits::default()
        };
        let result = solve_seeded_warm(&m, &limits, None, node.warm.clone());
        assert_eq!(result.status, "node_limit");
        assert_eq!(result.objective, Some(1.));
        assert!((result.dual_bound.unwrap() - 1.5).abs() < 1e-9);
        assert_eq!(result.probe.calls, 1);
        assert_eq!(result.probe.improvements, 1);
        assert!(result.lp_warm_successes.unwrap() >= 2);
        let parent =
            revised::minimize_warm(p.sparse.as_ref().unwrap(), 1000, None, node.warm.as_deref());
        assert!(
            matches!(parent.outcome, lp::Outcome::Optimal { objective, .. } if (objective + 1.5).abs() < 1e-9)
        );
    }
    #[test]
    fn pump_projects_and_perturbs_without_changing_parent_lp() {
        let (m, node, p, point) = prepare(FRACTIONAL);
        let mut s = PumpStats::default();
        let mut lp = revised::Stats::default();
        let candidate = binary_pump(
            &m,
            &node,
            &p,
            &point,
            &Limits::default(),
            Instant::now() + std::time::Duration::from_secs(2),
            &mut s,
            &mut lp,
        )
        .unwrap();
        assert!(m.check(&candidate, true, 1e-6).is_ok());
        assert_eq!(m.objective(&candidate), 1.);
        assert!(s.lp_solves > 0);
        assert!(s.perturbations > 0);
        let a =
            revised::minimize_warm(p.sparse.as_ref().unwrap(), 1000, None, node.warm.as_deref());
        let lp::Outcome::Optimal { objective, .. } = a.outcome else {
            panic!("parent corrupted")
        };
        assert!((objective + 1.5).abs() < 1e-9);
    }
    #[test]
    fn expired_pump_and_infeasible_integer_domain_return_no_candidate() {
        let text = "NAME impossible\nROWS\n N obj\n E r\nCOLUMNS\n x obj 0 r 1\nRHS\n rhs r 0.5\nBOUNDS\n BV b x\nENDATA\n";
        let (m, node, p, point) = prepare(text);
        let mut s = PumpStats::default();
        let mut lp = revised::Stats::default();
        assert!(
            binary_pump(
                &m,
                &node,
                &p,
                &point,
                &Limits::default(),
                Instant::now(),
                &mut s,
                &mut lp
            )
            .is_none()
        );
        assert_eq!(s.lp_solves, 0);
        assert!(
            binary_pump(
                &m,
                &node,
                &p,
                &point,
                &Limits::default(),
                Instant::now() + std::time::Duration::from_secs(2),
                &mut s,
                &mut lp
            )
            .is_none()
        );
        assert_eq!(s.candidates, 0);
        assert_eq!(s.lp_solves, 64);
    }
    #[test]
    fn mixed_continuous_projection_keeps_original_constraints() {
        let text = "NAME mixed\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1 r 1\n y obj -1 r 1\n z obj 0 r -1\nRHS\n rhs r 0.5\nBOUNDS\n BV b x\n BV b y\n UP b z 0.4\nENDATA\n";
        let (m, node, p, point) = prepare(text);
        let mut s = PumpStats::default();
        let mut lp = revised::Stats::default();
        let candidate = binary_pump(
            &m,
            &node,
            &p,
            &point,
            &Limits::default(),
            Instant::now() + std::time::Duration::from_secs(2),
            &mut s,
            &mut lp,
        )
        .unwrap();
        assert!(m.check(&candidate, true, 1e-6).is_ok());
        assert!(s.lp_solves > 0);
    }
}
