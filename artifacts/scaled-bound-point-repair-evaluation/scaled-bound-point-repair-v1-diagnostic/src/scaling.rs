//! LP列・行の単位を2の冪で正規化し、元のProblemへ戻して検査する。
use crate::{
    lp::Outcome,
    revised::{self, Answer, Problem, Stats, WarmStart},
};
use std::time::Instant;
fn error() -> Answer {
    Answer {
        outcome: Outcome::NumericalError,
        dual: None,
        stats: Stats::default(),
        warm: None,
        witness: None,
        witness_point: None,
    }
}
fn validation_diagnostic(p: &Problem, answer: &Answer, stage: &str) {
    let trace = std::env::var_os("MIP_LP_TRACE").is_some();
    let dump = std::env::var_os("MIP_LP_NUMERICAL_DUMP");
    if !trace && dump.is_none() {
        return;
    }
    let (kind, x) = match &answer.outcome {
        Outcome::Optimal { x, .. } => ("optimal", Some(x)),
        Outcome::Infeasible => ("infeasible", None),
        Outcome::Unbounded => ("unbounded", None),
        _ => ("numerical", None),
    };
    let mut bad_rows = Vec::new();
    if let Some(x) = x {
        let mut residual = p.rhs.clone();
        let mut rows = vec![Vec::new(); p.rhs.len()];
        for (j, col) in p.columns.iter().enumerate() {
            for &(i, a) in col {
                residual[i] -= a * x[j];
                rows[i].push((j, a));
            }
        }
        for (i, &v) in residual.iter().enumerate() {
            if !v.is_finite()
                || if p.equality[i] {
                    v.abs() > 1e-6
                } else {
                    v < -1e-6 || v > p.slack_upper[i] + 1e-6
                }
            {
                let exact = crate::exact::dot_enclosure(
                    std::iter::once((p.rhs[i], 1.)).chain(rows[i].iter().map(|&(j, a)| (-a, x[j]))),
                );
                bad_rows.push(serde_json::json!({"row":i,"residual":v,"exact_residual":exact}));
            }
        }
    }
    let event = serde_json::json!({"stage":stage,"outcome":kind,"variables":p.columns.len(),"rows":p.rhs.len(),"point_valid":x.map(|x|point_valid(p,x)),"bad_rows":bad_rows});
    if trace {
        eprintln!("LP validation diagnostic {event}");
    }
    if let Some(mut path) = dump {
        static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        path.push(format!(
            ".{}.{}.json",
            std::process::id(),
            SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        if let Ok(file) = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            let _ = serde_json::to_writer(
                file,
                &serde_json::json!({"event":event,"problem":{"columns":p.columns,"rhs":p.rhs,"costs":p.costs,"upper":p.upper.iter().map(|v|v.is_finite().then_some(*v)).collect::<Vec<_>>(),"equality":p.equality,"slack_upper":p.slack_upper.iter().map(|v|v.is_finite().then_some(*v)).collect::<Vec<_>>()},"point":x,"witness":answer.witness,"dual":answer.dual}),
            );
        }
    }
}
pub(crate) fn power(value: f64) -> f64 {
    let bits = value.to_bits();
    let exponent = bits & (0x7ffu64 << 52);
    if exponent != 0 {
        f64::from_bits(exponent)
    } else {
        f64::from_bits(1u64 << (63 - bits.leading_zeros()))
    }
}
fn scale(p: &Problem) -> Result<Option<(Problem, Vec<f64>, Vec<f64>)>, ()> {
    let factors: Vec<_> = p
        .columns
        .iter()
        .map(|c| {
            let largest = c.iter().map(|(_, a)| a.abs()).fold(0., f64::max);
            if largest > 0. && !(1e-4..=1e4).contains(&largest) {
                power(largest)
            } else {
                1.
            }
        })
        .collect();
    let mut row_max = vec![0.0f64; p.rhs.len()];
    for (j, col) in p.columns.iter().enumerate() {
        for &(i, a) in col {
            row_max[i] = row_max[i].max((a / factors[j]).abs());
        }
    }
    let rows: Vec<_> = row_max
        .into_iter()
        .map(|a| {
            if a > 0. && !(1e-4..=1e4).contains(&a) {
                power(a)
            } else {
                1.
            }
        })
        .collect();
    if factors.iter().chain(&rows).all(|s| *s == 1.) {
        return Ok(None);
    }
    let mut q = p.clone();
    for (j, &s) in factors.iter().enumerate() {
        for ((i, a), (_, original)) in q.columns[j].iter_mut().zip(&p.columns[j]) {
            let col_value = *a / s;
            if !col_value.is_finite() || col_value * s != *original {
                return Err(());
            }
            *a = col_value / rows[*i];
            if !a.is_finite() || *a * rows[*i] != col_value {
                return Err(());
            }
        }
        q.costs[j] /= s;
        if !q.costs[j].is_finite() || q.costs[j] * s != p.costs[j] {
            return Err(());
        }
        if p.upper[j].is_finite() {
            q.upper[j] *= s;
            if !q.upper[j].is_finite() || q.upper[j] / s != p.upper[j] {
                return Err(());
            }
        }
    }
    for (i, &r) in rows.iter().enumerate() {
        q.rhs[i] /= r;
        if !q.rhs[i].is_finite() || q.rhs[i] * r != p.rhs[i] {
            return Err(());
        }
        if p.slack_upper[i].is_finite() {
            q.slack_upper[i] /= r;
            if !q.slack_upper[i].is_finite() || q.slack_upper[i] * r != p.slack_upper[i] {
                return Err(());
            }
        }
    }
    Ok(Some((q, factors, rows)))
}
// 外向き丸め。厳密にゼロの積以外はunderflowした積も区間に含める。
fn dot_bounds(terms: impl Iterator<Item = (f64, f64)>) -> (f64, f64) {
    let (mut lo, mut hi) = (0.0f64, 0.0f64);
    for (a, b) in terms {
        if a == 0. || b == 0. {
            continue;
        }
        let t = a * b;
        lo = (lo + t.next_down()).next_down();
        hi = (hi + t.next_up()).next_up();
    }
    (lo, hi)
}
fn contradiction(p: &Problem, y: &[f64]) -> bool {
    if y.len() != p.rhs.len() || y.iter().any(|v| !v.is_finite()) {
        return false;
    }
    let lower = dot_bounds(y.iter().copied().zip(p.rhs.iter().copied())).0;
    let mut support = 0.0f64;
    for (j, col) in p.columns.iter().enumerate() {
        let alpha = dot_bounds(col.iter().map(|&(i, a)| (a, y[i]))).1;
        if !alpha.is_finite() {
            return false;
        }
        if alpha > 0. {
            if !p.upper[j].is_finite() {
                if crate::exact::dot_sign(col.iter().map(|&(i, a)| (a, y[i])))
                    .is_some_and(|sign| sign != std::cmp::Ordering::Greater)
                {
                    continue;
                }
                return false;
            }
            if p.upper[j] != 0. {
                support = (support + (alpha * p.upper[j]).next_up()).next_up();
            }
        }
    }
    for (i, &v) in y.iter().enumerate() {
        if !p.equality[i] && v > 0. {
            if !p.slack_upper[i].is_finite() {
                return false;
            }
            if p.slack_upper[i] != 0. {
                support = (support + (v * p.slack_upper[i]).next_up()).next_up();
            }
        }
    }
    lower > support
}
fn point_valid(p: &Problem, x: &[f64]) -> bool {
    if x.len() != p.costs.len() || x.iter().any(|v| !v.is_finite()) {
        return false;
    }
    let mut residual = p.rhs.clone();
    for (j, col) in p.columns.iter().enumerate() {
        if x[j] < -1e-7 || x[j] > p.upper[j] + 1e-7 {
            return false;
        }
        for &(i, a) in col {
            residual[i] -= a * x[j];
        }
    }
    for (i, &v) in residual.iter().enumerate() {
        if !v.is_finite()
            || if p.equality[i] {
                v.abs() > 1e-6
            } else {
                v < -1e-6 || v > p.slack_upper[i] + 1e-6
            }
        {
            return false;
        }
    }
    true
}
fn primal_dual(p: &Problem, x: &[f64], y: &[f64]) -> Option<(f64, f64)> {
    if !point_valid(p, x) || y.len() != p.rhs.len() || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let primal: f64 = p.costs.iter().zip(x).map(|(c, v)| c * v).sum();
    let mut dual: f64 = y.iter().zip(&p.rhs).map(|(a, b)| a * b).sum();
    for (j, col) in p.columns.iter().enumerate() {
        let r = p.costs[j] - col.iter().map(|&(i, a)| a * y[i]).sum::<f64>();
        if !r.is_finite() {
            return None;
        }
        if p.upper[j].is_finite() {
            dual += r.min(0.) * p.upper[j];
        } else if r < -1e-7 {
            return None;
        }
    }
    for (i, &v) in y.iter().enumerate() {
        if !p.equality[i] {
            if p.slack_upper[i].is_finite() {
                dual += (-v).min(0.) * p.slack_upper[i];
            } else if v > 1e-7 {
                return None;
            }
        }
    }
    if !primal.is_finite()
        || !dual.is_finite()
        || (primal - dual).abs() > 1e-6 * (1. + primal.abs())
    {
        None
    } else {
        Some((primal, dual.min(primal)))
    }
}
fn ray_valid(p: &Problem, ray: &[f64]) -> bool {
    if ray.len() != p.costs.len() || ray.iter().any(|v| !v.is_finite()) {
        return false;
    }
    let mut rows = vec![vec![]; p.rhs.len()];
    for (j, col) in p.columns.iter().enumerate() {
        if ray[j] < 0. || (p.upper[j].is_finite() && ray[j] != 0.) {
            return false;
        }
        if ray[j] == 0. {
            continue;
        }
        for &(i, a) in col {
            rows[i].push((a, ray[j]));
        }
    }
    for (i, row) in rows.into_iter().enumerate() {
        let Some(sign) = crate::exact::dot_sign(row.into_iter()) else {
            return false;
        };
        if p.equality[i] || p.slack_upper[i].is_finite() {
            if sign != std::cmp::Ordering::Equal {
                return false;
            }
        } else if sign == std::cmp::Ordering::Greater {
            return false;
        }
    }
    crate::exact::dot_sign(p.costs.iter().copied().zip(ray.iter().copied()))
        == Some(std::cmp::Ordering::Less)
}
pub(crate) fn solve(
    p: &Problem,
    limit: usize,
    deadline: Option<Instant>,
    warm: Option<&WarmStart>,
    retain: bool,
) -> Answer {
    if !revised::valid(p) {
        return error();
    }
    let normalized = match scale(p) {
        Ok(q) => q,
        Err(()) => return error(),
    };
    let Some((q, factors, rows)) = normalized else {
        let mut answer = revised::raw(p, limit, deadline, warm, retain);
        if matches!(answer.outcome, Outcome::Unbounded)
            && !(answer
                .witness
                .as_deref()
                .is_some_and(|ray| ray_valid(p, ray))
                && answer
                    .witness_point
                    .as_deref()
                    .is_some_and(|x| point_valid(p, x)))
        {
            answer.outcome = Outcome::NumericalError;
            answer.dual = None;
            answer.warm = None;
            answer.witness = None;
            answer.witness_point = None;
        }
        return answer;
    };
    let mut answer = revised::raw(&q, limit, deadline, warm, retain);
    if matches!(answer.outcome, Outcome::NumericalError) {
        validation_diagnostic(p, &answer, "raw_scaled");
    }
    answer.stats.scaled_columns += factors.iter().filter(|s| **s != 1.).count();
    answer.stats.scaled_rows += rows.iter().filter(|r| **r != 1.).count();
    if matches!(
        answer.outcome,
        Outcome::Optimal { .. } | Outcome::Infeasible
    ) {
        if let Some(y) = answer.witness.as_mut() {
            for (v, &r) in y.iter_mut().zip(&rows) {
                *v /= r;
            }
        }
    }
    let valid = match &mut answer.outcome {
        Outcome::Optimal { objective, x } => {
            for (v, &s) in x.iter_mut().zip(&factors) {
                *v /= s;
            }
            if let Some((obj, dual)) = answer.witness.as_deref().and_then(|y| primal_dual(p, x, y))
            {
                *objective = obj;
                answer.dual = Some(dual);
                true
            } else if std::env::var("MIP_LP_BOUND_POINT_REPAIR").is_ok_and(|v|v=="1") && x.len()==p.upper.len() && x.iter().all(|v|v.is_finite()) {
                answer.stats.bound_point_trials+=1;
                let projected:Vec<_>=x.iter().zip(&p.upper).map(|(&v,&upper)|v.max(0.).min(upper)).collect();
                if projected!=*x {
                    if let Some((obj,dual))=answer.witness.as_deref().and_then(|y|primal_dual(p,&projected,y)){
                        *x=projected;*objective=obj;answer.dual=Some(dual);answer.stats.bound_point_repairs+=1;true
                    }else{false}
                }else{false}
            } else {
                false
            }
        }
        Outcome::Infeasible => {
            if let Some(y) = answer.witness.as_deref() {
                if contradiction(p, y)
                    || contradiction(p, &y.iter().map(|v| -v).collect::<Vec<_>>())
                {
                    true
                } else {
                    let mut repaired = None;
                    for cap in [16, 256, 4096, 65536] {
                        if deadline.is_some_and(|d| Instant::now() >= d) {
                            break;
                        }
                        if let Some(mut candidate) = crate::certificate::integer_weights(y, cap) {
                            for _ in 0..2 {
                                answer.stats.certificate_repair_trials += 1;
                                if contradiction(p, &candidate) {
                                    repaired = Some(candidate);
                                    break;
                                }
                                for v in &mut candidate {
                                    *v = -*v;
                                }
                            }
                            if repaired.is_some() {
                                break;
                            }
                        }
                    }
                    if let Some(candidate) = repaired {
                        answer.witness = Some(candidate);
                        answer.stats.certificate_repairs += 1;
                        true
                    } else {
                        false
                    }
                }
            } else {
                false
            }
        }
        Outcome::Unbounded => {
            let base = if let Some(point) = answer.witness_point.as_mut() {
                for (v, &s) in point.iter_mut().zip(&factors) {
                    *v /= s;
                }
                point_valid(p, point)
            } else {
                false
            };
            if let Some(ray) = answer.witness.as_mut() {
                for (v, &s) in ray.iter_mut().zip(&factors) {
                    *v /= s;
                }
                base && ray_valid(p, ray)
            } else {
                false
            }
        }
        _ => true,
    };
    if !valid {
        validation_diagnostic(p, &answer, "scaled_post_validation");
        answer.outcome = Outcome::NumericalError;
        answer.dual = None;
        answer.warm = None;
        answer.witness = None;
        answer.witness_point = None;
    }
    answer
}

#[cfg(test)]
mod saved_validation_diagnostic {
    use super::*;
    #[test]
    fn integer_candidate_cancels_an_unbounded_flow_column_and_rejects_feasible_rhs() {
        let mut p = Problem {
            columns: vec![vec![(0, 1.), (1, -2.5)], vec![(0, -1.), (1, 2.5)]],
            rhs: vec![-1., 0.],
            costs: vec![0., 0.],
            upper: vec![f64::INFINITY; 2],
            equality: vec![false; 2],
            slack_upper: vec![f64::INFINITY; 2],
        };
        let approximate = [-1., -0.4];
        assert!(!contradiction(&p, &approximate));
        let integer = crate::certificate::integer_weights(&approximate, 16).unwrap();
        assert_eq!(integer, vec![-5., -2.]);
        assert!(contradiction(&p, &integer));
        p.rhs[0] = 1.;
        assert!(!contradiction(&p, &integer));
    }
    #[test]
    #[ignore = "saved validation failures via MIP_LP_DUMP_DIR"]
    fn inspect_saved_finite_box_certificates() {
        use crate::model::{Model, Row, Variable};
        for entry in std::fs::read_dir(std::env::var("MIP_LP_DUMP_DIR").unwrap()).unwrap() {
            let path = entry.unwrap().path();
            let d: serde_json::Value =
                serde_json::from_reader(std::fs::File::open(&path).unwrap()).unwrap();
            let p = &d["problem"];
            let columns: Vec<Vec<(usize, f64)>> =
                serde_json::from_value(p["columns"].clone()).unwrap();
            let rhs: Vec<f64> = serde_json::from_value(p["rhs"].clone()).unwrap();
            let upper: Vec<Option<f64>> = serde_json::from_value(p["upper"].clone()).unwrap();
            let slack: Vec<Option<f64>> = serde_json::from_value(p["slack_upper"].clone()).unwrap();
            let eq: Vec<bool> = serde_json::from_value(p["equality"].clone()).unwrap();
            let variables = upper
                .iter()
                .enumerate()
                .map(|(j, &u)| Variable {
                    name: format!("x{j}"),
                    lower: Some(0.),
                    upper: u,
                    integer: false,
                    objective: 0.,
                    objective_enclosure: Some([0., 0.]),
                    lower_enclosure: Some([0., 0.]),
                    upper_enclosure: u.map(|u| [u, u]),
                })
                .collect();
            let mut rows: Vec<_> = rhs
                .iter()
                .enumerate()
                .map(|(i, &b)| {
                    let (lower, lower_enclosure) = if eq[i] {
                        (Some(b), Some([b, b]))
                    } else if let Some(w) = slack[i] {
                        let enclosure =
                            crate::exact::dot_enclosure([(b, 1.), (-w, 1.)].into_iter());
                        if (b - w).is_finite() {
                            (Some(b - w), enclosure)
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    };
                    Row {
                        name: format!("r{i}"),
                        lower,
                        upper: Some(b),
                        lower_enclosure,
                        upper_enclosure: Some([b, b]),
                        coefficients: vec![],
                        exact_coefficients: true,
                        exact_integral: false,
                        exact_coefficient_flags: vec![],
                    }
                })
                .collect();
            for (j, col) in columns.iter().enumerate() {
                for &(i, a) in col {
                    rows[i].coefficients.push((j, a));
                }
            }
            let model = Model {
                name: "saved_standard_lp".into(),
                maximize: false,
                offset: 0.,
                offset_enclosure: Some([0., 0.]),
                original_decimal: false,
                variables,
                rows,
            };
            let y: Vec<f64> = serde_json::from_value(d["witness"].clone()).unwrap();
            let inverse: Vec<_> = y.iter().map(|v| -v).collect();
            let lower = vec![Some(0.); upper.len()];
            let standard = Problem {
                columns: columns.clone(),
                rhs: rhs.clone(),
                costs: vec![0.; upper.len()],
                upper: upper.iter().map(|v| v.unwrap_or(f64::INFINITY)).collect(),
                equality: eq.clone(),
                slack_upper: slack.iter().map(|v| v.unwrap_or(f64::INFINITY)).collect(),
            };
            let mut certified = false;
            for cap in [16, 256, 4096, 65536] {
                if let Some(weights) = crate::certificate::integer_weights(&y, cap) {
                    if contradiction(&standard, &weights)
                        || contradiction(&standard, &weights.iter().map(|v| -v).collect::<Vec<_>>())
                    {
                        certified = true;
                        break;
                    }
                }
            }
            assert!(certified, "{}", path.display());
            let a = crate::objective_bound::Certifier::feasibility(&model).unwrap();
            let b = crate::objective_bound::Certifier::feasibility_with_aggregation(&model, true)
                .unwrap();
            if let Some(candidates) = d["rational_candidates"].as_array() {
                for candidate in candidates {
                    let weights: Vec<f64> =
                        serde_json::from_value(candidate["weights"].clone()).unwrap();
                    let inverse: Vec<_> = weights.iter().map(|v| -v).collect();
                    println!(
                        "{}",
                        serde_json::json!({"file":path.file_name().unwrap().to_string_lossy(),"cap":candidate["cap"],"rational":[b.bound(&lower,&upper,&weights),b.bound(&lower,&upper,&inverse)]})
                    );
                }
            }
            println!(
                "{}",
                serde_json::json!({"file":path.file_name().unwrap().to_string_lossy(),"ordinary":[a.bound(&lower,&upper,&y),a.bound(&lower,&upper,&inverse)],"aggregate":[b.bound(&lower,&upper,&y),b.bound(&lower,&upper,&inverse)]})
            );
        }
    }
}
