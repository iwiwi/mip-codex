//! 元行の追加のみを行うLP fallback。部分LPの非有界を元LPへ伝えない。
use super::*;
#[derive(Default, Debug, Serialize)]
pub struct Stats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bound_proof: Option<crate::master_bound::Proof>,
    pub report_calls: usize,
    pub report_trials: usize,
    pub report_seconds: f64,
    pub calls: usize,
    pub lp_solves: usize,
    pub rows_added: usize,
    pub max_active: usize,
    pub scans: usize,
    pub unbounded_masters: usize,
    pub seconds: f64,
}
pub(super) struct State {
    indices: Vec<usize>,
    active: Vec<bool>,
}
fn violations(
    model: &Model,
    point: &[f64],
    state: &State,
    deadline: Instant,
) -> Option<Vec<(usize, f64)>> {
    let mut rows = vec![];
    for (i, r) in model.rows.iter().enumerate() {
        if i % 4096 == 0 && Instant::now() >= deadline {
            return None;
        }
        if state.active[i] {
            continue;
        }
        let value = r
            .coefficients
            .iter()
            .map(|&(j, a)| a * point[j])
            .sum::<f64>();
        if !value.is_finite() {
            return None;
        }
        let delta = r
            .lower
            .map_or(0., |b| (b - value).max(0.))
            .max(r.upper.map_or(0., |b| (value - b).max(0.)));
        if delta > 1e-7 {
            let norm = r
                .coefficients
                .iter()
                .map(|&(_, a)| a.abs())
                .sum::<f64>()
                .max(1.);
            rows.push((i, delta / norm));
        }
    }
    rows.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    Some(rows)
}
fn append(state: &mut State, rows: impl Iterator<Item = usize>, stats: &mut Stats) -> usize {
    let mut added = 0;
    for i in rows {
        if !state.active[i] {
            state.active[i] = true;
            state.indices.push(i);
            added += 1;
            if added == 512 {
                break;
            }
        }
    }
    stats.rows_added += added;
    stats.max_active = stats.max_active.max(state.indices.len());
    added
}
fn cancel(q: Standard, outcome: lp::Outcome) -> CachedRoot {
    CachedRoot {
        standard: q,
        answer: revised::Answer {
            outcome,
            dual: None,
            stats: revised::Stats::default(),
            warm: None,
            witness: None,
            witness_point: None,
            limited_point: None,
        },
    }
}
pub(super) fn run(
    model: &Model,
    node: &Node,
    limits: &Limits,
    deadline: Instant,
    state: &mut Option<State>,
    stats: &mut Stats,
    lp_stats: &mut revised::Stats,
    lp_solves: &mut usize,
    report_root: bool,
) -> Option<CachedRoot> {
    let started = Instant::now();
    stats.calls += 1;
    let result = run_inner(
        model,
        node,
        limits,
        deadline,
        state,
        stats,
        lp_stats,
        lp_solves,
        report_root,
    );
    stats.seconds += started.elapsed().as_secs_f64();
    result
}
fn run_inner(
    model: &Model,
    node: &Node,
    limits: &Limits,
    deadline: Instant,
    state: &mut Option<State>,
    stats: &mut Stats,
    lp_stats: &mut revised::Stats,
    lp_solves: &mut usize,
    report_root: bool,
) -> Option<CachedRoot> {
    let state = state.get_or_insert_with(|| State {
        indices: vec![],
        active: vec![false; model.rows.len()],
    });
    if state.active.len() != model.rows.len() {
        return None;
    }
    if state.indices.is_empty() {
        let point: Vec<_> = model
            .variables
            .iter()
            .enumerate()
            .map(|(j, v)| {
                let c = if model.maximize {
                    -v.objective
                } else {
                    v.objective
                };
                if c < 0. {
                    node.upper[j].or(node.lower[j]).unwrap_or(0.)
                } else {
                    node.lower[j].or(node.upper[j]).unwrap_or(0.)
                }
            })
            .collect();
        stats.scans += 1;
        if let Some(rows) = violations(model, &point, state, deadline) {
            append(state, rows.into_iter().map(|(i, _)| i), stats);
        }
    }
    let mut warm = node.warm.clone();
    let mut used = 0usize;
    loop {
        let q = match standard_sparse_selected(
            model,
            node,
            limits.dense_bytes,
            true,
            limits.fixed_columns,
            Some(&state.indices),
        ) {
            Relaxation::Ready(q) => q,
            _ => return None,
        };
        if Instant::now() >= deadline {
            return Some(cancel(q, lp::Outcome::TimeLimit));
        }
        if stats.lp_solves < 2
            && let Some(dir) = std::env::var_os("MIP_ROW_GENERATION_DUMP")
        {
            let path =
                std::path::PathBuf::from(dir).join(format!("master-{}.json", stats.lp_solves));
            if let Ok(file) = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
            {
                let p = q.sparse.as_ref().unwrap();
                let _ = serde_json::to_writer(
                    file,
                    &serde_json::json!({"indices":state.indices,"columns":p.columns,"rhs":p.rhs,"costs":p.costs,"upper":p.upper,"equality":p.equality,"slack_upper":p.slack_upper,"shift":q.shift,"terms":q.terms}),
                );
            }
        }
        let mut a = revised::minimize_warm(
            q.sparse.as_ref().unwrap(),
            limits.pivots.saturating_sub(used),
            Some(deadline),
            if limits.warm_start {
                warm.as_deref()
            } else {
                None
            },
        );
        stats.lp_solves += 1;
        *lp_solves += 1;
        used = used.saturating_add(a.stats.pivots.saturating_add(a.stats.flips));
        lp_stats.add(&a.stats);
        if report_root
            && limits.row_generation_report
            && matches!(&a.outcome, lp::Outcome::Optimal { .. })
            && stats.report_seconds < (limits.seconds * 0.05).min(1.)
            && let Some(y) = a.witness.as_deref()
        {
            let began = Instant::now();
            let allowance = ((limits.seconds * 0.05).min(1.) - stats.report_seconds).max(0.);
            stats.report_calls += 1;
            if let Some(proof) = crate::master_bound::certify(
                model,
                &q.row_sources,
                y,
                limits.dense_bytes,
                deadline.min(began + std::time::Duration::from_secs_f64(allowance)),
                &mut stats.report_trials,
            ) {
                if stats
                    .bound_proof
                    .as_ref()
                    .and_then(|p| p.bound())
                    .is_none_or(|old| proof.bound().unwrap() > old)
                {
                    stats.bound_proof = Some(proof);
                }
            }
            stats.report_seconds += began.elapsed().as_secs_f64();
        }
        match &a.outcome {
            lp::Outcome::Optimal { x, .. } => {
                let point: Vec<_> = q
                    .terms
                    .iter()
                    .enumerate()
                    .map(|(j, t)| q.shift[j] + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>())
                    .collect();
                stats.scans += 1;
                let Some(rows) = violations(model, &point, state, deadline) else {
                    return Some(cancel(
                        q,
                        if Instant::now() >= deadline {
                            lp::Outcome::TimeLimit
                        } else {
                            lp::Outcome::NumericalError
                        },
                    ));
                };
                if rows.is_empty() {
                    if model.check(&point, false, 1e-7).is_ok() {
                        return Some(CachedRoot {
                            standard: q,
                            answer: a,
                        });
                    }
                    return Some(cancel(q, lp::Outcome::NumericalError));
                }
                if append(state, rows.into_iter().map(|(i, _)| i), stats) == 0 {
                    return Some(cancel(q, lp::Outcome::NumericalError));
                }
                warm = a.warm;
            }
            lp::Outcome::Unbounded => {
                stats.unbounded_masters += 1;
                let next: Vec<_> = (0..model.rows.len())
                    .filter(|&i| !state.active[i])
                    .take(512)
                    .collect();
                if next.is_empty() {
                    return Some(CachedRoot {
                        standard: q,
                        answer: a,
                    });
                }
                append(state, next.into_iter(), stats);
                warm = None;
            }
            lp::Outcome::TimeLimit | lp::Outcome::IterationLimit => {
                a.limited_point = None;
                return Some(CachedRoot {
                    standard: q,
                    answer: a,
                });
            }
            _ => {
                return Some(CachedRoot {
                    standard: q,
                    answer: a,
                });
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn node(m: &Model) -> Node {
        Node {
            certified_bound: None,
            lower: m.variables.iter().map(|v| v.lower).collect(),
            upper: m.variables.iter().map(|v| v.upper).collect(),
            bound: f64::NEG_INFINITY,
            warm: None,
            changed: None,
            edge: None,
        }
    }
    fn invoke(m: &Model, n: &Node, l: &Limits, state: &mut Option<State>) -> (CachedRoot, Stats) {
        let mut stats = Stats::default();
        let mut lp = revised::Stats::default();
        let mut solves = 0;
        let r = run(
            m,
            n,
            l,
            Instant::now() + std::time::Duration::from_secs(2),
            state,
            &mut stats,
            &mut lp,
            &mut solves,
            false,
        )
        .unwrap();
        assert_eq!(stats.lp_solves, solves);
        (r, stats)
    }
    fn decode(q: &CachedRoot) -> Vec<f64> {
        let lp::Outcome::Optimal { x, .. } = &q.answer.outcome else {
            panic!("{:?}", q.answer.outcome)
        };
        q.standard
            .terms
            .iter()
            .enumerate()
            .map(|(j, t)| q.standard.shift[j] + t.iter().map(|&(k, s)| s * x[k]).sum::<f64>())
            .collect()
    }
    #[test]
    fn child_adds_an_original_row_and_parent_warm_is_reusable_in_both_senses() {
        for maximize in [false, true] {
            let text = format!(
                "NAME rg\nOBJSENSE\n {}\nROWS\n N c\n L sum\n L diff\nCOLUMNS\n x c {} sum 1\n x diff 1\n y c {} sum 1\n y diff -1\nRHS\n rhs sum 1.5 diff 0.25\n rhs c -100\nBOUNDS\n UP b x 1\n UP b y 1\nENDATA\n",
                if maximize { "MAX" } else { "MIN" },
                if maximize { 1 } else { -1 },
                if maximize { 2 } else { -2 }
            );
            let m = crate::mps::parse(std::io::Cursor::new(text)).unwrap();
            let root = node(&m);
            let mut state = None;
            let limits = Limits::default();
            let (parent, _) = invoke(&m, &root, &limits, &mut state);
            assert_eq!(decode(&parent), [0.5, 1.]);
            assert_eq!(state.as_ref().unwrap().indices, [0]);
            assert_eq!(
                m.objective(&decode(&parent)),
                if maximize { 102.5 } else { 97.5 }
            );
            let mut child = root.clone();
            child.warm = parent.answer.warm.clone();
            child.upper[1] = Some(0.);
            let (result, stats) = invoke(&m, &child, &limits, &mut state);
            assert_eq!(decode(&result), [0.25, 0.]);
            assert_eq!(stats.rows_added, 1);
            assert_eq!(result.standard.row_sources, [(0, 1.), (1, 1.)]);
            let mut retry = root.clone();
            retry.warm = parent.answer.warm.clone();
            let (again, _) = invoke(&m, &retry, &limits, &mut state);
            assert_eq!(decode(&again), [0.5, 1.]);
            assert_eq!(root.upper[1], Some(1.));
        }
    }
    #[test]
    fn unbounded_master_adds_rows_before_returning_a_bounded_original_point() {
        let m = crate::mps::parse(std::io::Cursor::new(
            "NAME unb\nROWS\n N c\n L cap\nCOLUMNS\n x c -1 cap 1\nRHS\n rhs cap 1\nENDATA\n",
        ))
        .unwrap();
        let mut state = None;
        let (r, s) = invoke(&m, &node(&m), &Limits::default(), &mut state);
        assert_eq!(decode(&r), [1.]);
        assert_eq!(s.unbounded_masters, 1);
        assert_eq!(s.rows_added, 1);
    }
    #[test]
    fn a_subset_infeasibility_retains_original_row_sources() {
        let m=crate::mps::parse(std::io::Cursor::new("NAME infeas\nROWS\n N c\n L cap\n G need\nCOLUMNS\n x c 1 cap 1\n x need 1\nRHS\n rhs cap 1 need 2\nBOUNDS\n UP b x 3\nENDATA\n")).unwrap();
        let mut state = None;
        let (r, _) = invoke(&m, &node(&m), &Limits::default(), &mut state);
        assert!(matches!(r.answer.outcome, lp::Outcome::Infeasible));
        assert_eq!(r.standard.row_sources, [(1, -1.), (0, 1.)]);
        assert!(r.answer.witness.is_some());
    }
    #[test]
    fn expired_deadline_never_returns_a_master_optimality_claim() {
        let m = crate::mps::parse(std::io::Cursor::new(
            "NAME time\nROWS\n N c\n L cap\nCOLUMNS\n x c -1 cap 1\nRHS\n rhs cap 1\nENDATA\n",
        ))
        .unwrap();
        let mut state = None;
        let mut stats = Stats::default();
        let mut lp = revised::Stats::default();
        let mut count = 0;
        let r = run(
            &m,
            &node(&m),
            &Limits::default(),
            Instant::now(),
            &mut state,
            &mut stats,
            &mut lp,
            &mut count,
            false,
        )
        .unwrap();
        assert!(matches!(r.answer.outcome, lp::Outcome::TimeLimit));
        assert!(
            r.answer.dual.is_none()
                && r.answer.witness.is_none()
                && r.answer.limited_point.is_none()
        );
        assert_eq!(count, 0);
    }
}
#[cfg(test)]
mod mixed_domain_tests {
    use super::*;
    #[test]
    fn ranged_rows_free_continuous_columns_and_inner_integer_bounds_preserve_both_senses() {
        for maximize in [false, true] {
            let mut text = format!(
                "NAME mixed\nOBJSENSE\n {}\nROWS\n N c\n E balance\n L band\n",
                if maximize { "MAX" } else { "MIN" }
            );
            for i in 0..800 {
                text += &format!(" L noise{i}\n");
            }
            text += &format!(
                "COLUMNS\n x c {} balance -1\n x band 1\n",
                if maximize { 2 } else { -2 }
            );
            for i in 0..800 {
                text += &format!(" x noise{i} 1\n");
            }
            text += &format!(
                " z c {} balance 1\nRHS\n rhs balance 0.5 band 0.75\n rhs c -7\n",
                if maximize { -1 } else { 1 }
            );
            for i in 0..800 {
                text += &format!(" rhs noise{i} {}\n", 1000 + i);
            }
            text += "RANGES\n rng band 1\nBOUNDS\n LI b x -1.2\n UI b x 2.3\n FR b z\nENDATA\n";
            let m = crate::mps::parse(std::io::Cursor::new(text)).unwrap();
            let limits = Limits {
                seconds: 2.,
                dense_bytes: 1 << 20,
                row_generation: true,
                primal_repair: false,
                box_primal: false,
                primal_probe: false,
                lp_diving: false,
                objective_lp_dive: false,
                feasibility_pump: false,
                rins: false,
                terminal_neighborhood: false,
                objective_rounding: false,
                objective_rounding_all: false,
                binary_graph: false,
                cover_cuts: false,
                mir_cuts: false,
                gomory_cuts: false,
                aggregate_cuts: false,
                clique_cuts: false,
                three_row_cuts: false,
                rounding_cuts: false,
                ..Limits::default()
            };
            let r = solve(&m, &limits);
            assert_eq!(r.status, "optimal");
            assert_eq!(r.objective, Some(if maximize { 6.5 } else { 7.5 }));
            assert!(r.row_generation.calls > 0);
            let x: Vec<_> = m
                .variables
                .iter()
                .map(|v| r.values.as_ref().unwrap()[&v.name])
                .collect();
            assert!(m.check(&x, true, 1e-7).is_ok());
            assert_eq!(x, [0., 0.5]);
            assert_eq!(m.variables[0].lower, Some(-1.2));
        }
    }
    #[test]
    fn zero_pivots_does_not_return_a_subset_bound_as_original_optimality() {
        let m=crate::mps::parse(std::io::Cursor::new("NAME piv\nROWS\n N c\n G need\nCOLUMNS\n x c 1 need 1\nRHS\n rhs need 0.5\nBOUNDS\n UP b x 2\nENDATA\n")).unwrap();
        let n = Node {
            certified_bound: None,
            lower: vec![Some(0.)],
            upper: vec![Some(2.)],
            bound: f64::NEG_INFINITY,
            warm: None,
            changed: None,
            edge: None,
        };
        let mut state = None;
        let mut stats = Stats::default();
        let mut lp = revised::Stats::default();
        let mut count = 0;
        let r = run(
            &m,
            &n,
            &Limits {
                pivots: 0,
                ..Limits::default()
            },
            Instant::now() + std::time::Duration::from_secs(2),
            &mut state,
            &mut stats,
            &mut lp,
            &mut count,
            false,
        )
        .unwrap();
        assert!(matches!(r.answer.outcome, lp::Outcome::IterationLimit));
        assert!(r.answer.dual.is_none());
        assert_eq!(lp.pivots + lp.flips, 0);
    }
}
#[cfg(test)]
mod report_tests {
    use super::*;
    #[test]
    fn root_reporting_is_explicit_and_never_borrows_a_child_box() {
        let m=crate::mps::parse(std::io::Cursor::new("NAME report\nROWS\n N c\n G need\nCOLUMNS\n x c 0 need 1\n y c 1 need 1\nRHS\n rhs need 1\nBOUNDS\n UP b x 2\n UP b y 2\nENDATA\n")).unwrap();
        for root in [false, true] {
            let n = Node {
                certified_bound: None,
                lower: vec![Some(0.); 2],
                upper: vec![Some(0.5), Some(2.)],
                bound: f64::NEG_INFINITY,
                warm: None,
                changed: None,
                edge: None,
            };
            let mut state = None;
            let mut stats = Stats::default();
            let mut lp = revised::Stats::default();
            let mut count = 0;
            let result = run(
                &m,
                &n,
                &Limits {
                    row_generation_report: true,
                    ..Limits::default()
                },
                Instant::now() + std::time::Duration::from_secs(2),
                &mut state,
                &mut stats,
                &mut lp,
                &mut count,
                root,
            )
            .unwrap();
            assert!(
                matches!(result.answer.outcome,lp::Outcome::Optimal{objective,..}if objective==0.5)
            );
            if root {
                let p = stats.bound_proof.expect("root certificate");
                assert!(p.bound().unwrap() <= 0.);
                assert!(stats.report_calls > 0);
            } else {
                assert!(stats.bound_proof.is_none());
                assert_eq!(stats.report_calls, 0);
            }
        }
    }
}
