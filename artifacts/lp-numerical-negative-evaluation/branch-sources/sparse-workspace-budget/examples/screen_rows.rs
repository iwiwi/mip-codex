//! Continuous root-LP row generation using only this repository's Rust optimizer.
use mip_codex::{
    mps,
    solve::{self, Limits},
};
use std::{collections::BTreeSet, time::Instant};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let began = Instant::now();
    let original = mps::read(&args[1]).unwrap();
    let seconds: f64 = args.get(2).map_or(60., |s| s.parse().unwrap());
    let batch: usize = args.get(3).map_or(256, |s| s.parse().unwrap());
    let parse_seconds = began.elapsed().as_secs_f64();
    let mut continuous = original.clone();
    for v in &mut continuous.variables {
        v.integer = false;
    }
    let mut active: Vec<usize> = Vec::new();
    let mut selected = BTreeSet::new();
    let mut records = Vec::new();
    let (mut last_point, mut last_objective, mut last_dual) = (None, None, None);
    let mut status = "time_limit";
    while began.elapsed().as_secs_f64() < seconds {
        let mut relaxation = continuous.clone();
        relaxation.rows = active.iter().map(|&i| original.rows[i].clone()).collect();
        let mut limits = Limits::default();
        limits.seconds = (seconds - began.elapsed().as_secs_f64()).max(0.);
        limits.nodes = 1;
        limits.aggregate_cuts = false;
        limits.clique_cuts = false;
        limits.mir_cuts = false;
        limits.cover_cuts = false;
        limits.rounding_cuts = false;
        limits.binary_graph = false;
        limits.objective_rounding = false;
        limits.objective_rounding_all = false;
        limits.box_primal = false;
        limits.primal_repair = false;
        limits.primal_probe = false;
        limits.rins = false;
        limits.strong_branching = false;
        limits.lp_diving = false;
        limits.feasibility_pump = false;
        limits.propagation = false;
        let answer = solve::solve(&relaxation, &limits);
        records.push(serde_json::json!({"active_rows":active.len(),"status":answer.status,"objective":answer.objective,"dual":answer.dual_bound,"seconds":answer.seconds,"lp_pivots":answer.lp_pivots}));
        let mut add = Vec::new();
        if answer.status == "optimal"
            && let Some(values) = &answer.values
        {
            let point: Vec<_> = original.variables.iter().map(|v| values[&v.name]).collect();
            if original.check(&point, false, 1e-6).is_ok() {
                last_point = Some(point);
                last_objective = answer.objective;
                last_dual = answer.dual_bound;
                status = "validated_continuous_optimum";
                break;
            }
            let mut violations = Vec::new();
            for (i, row) in original.rows.iter().enumerate() {
                if selected.contains(&i) {
                    continue;
                }
                let activity = row
                    .coefficients
                    .iter()
                    .map(|&(j, a)| a * point[j])
                    .sum::<f64>();
                let violation = row
                    .lower
                    .map_or(0., |b| (b - activity).max(0.))
                    .max(row.upper.map_or(0., |b| (activity - b).max(0.)));
                if violation > 1e-8 {
                    let norm = row
                        .coefficients
                        .iter()
                        .map(|&(_, a)| a.abs())
                        .sum::<f64>()
                        .max(1.);
                    violations.push((violation / norm, i));
                }
            }
            violations.sort_unstable_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            add.extend(violations.into_iter().take(batch).map(|(_, i)| i));
        } else if ["time_limit", "iteration_limit", "resource_limit"]
            .contains(&answer.status.as_str())
        {
            status = "unresolved_subset";
            break;
        }
        // Unbounded/numerically unresolved subsets are never claims about the original model.
        if add.is_empty() {
            add.extend(
                (0..original.rows.len())
                    .filter(|i| !selected.contains(i))
                    .take(batch),
            );
        }
        if add.is_empty() {
            status = "unresolved_full_model";
            break;
        }
        for i in add {
            if selected.insert(i) {
                active.push(i);
            }
        }
    }
    let names: Vec<_> = original.variables.iter().map(|v| &v.name).collect();
    println!(
        "{}",
        serde_json::json!({"purpose":"continuous relaxation only; no integer-feasibility or MIP-optimality claim","status":status,"objective":last_objective,"dual_bound":last_dual,"names":names,"point":last_point,"rows":original.rows.len(),"active_rows":active.len(),"parse_seconds":parse_seconds,"wall_seconds":began.elapsed().as_secs_f64(),"records":records,"library_source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT,"source_directory":mip_codex::BUILD_SOURCE_DIRECTORY})
    );
}
