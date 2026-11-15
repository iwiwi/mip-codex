use mip_codex::{
    lp::Outcome,
    revised::{Problem, minimize, minimize_warm},
};
fn problem(costs: Vec<f64>, upper: Vec<f64>, a: Vec<f64>, rhs: f64) -> Problem {
    Problem {
        columns: a.into_iter().map(|v| vec![(0, v)]).collect(),
        rhs: vec![rhs],
        costs,
        upper,
        equality: vec![true],
        slack_upper: vec![0.],
    }
}
fn value(answer: &mip_codex::revised::Answer) -> f64 {
    match &answer.outcome {
        Outcome::Optimal { objective, .. } => *objective,
        other => panic!("{other:?}"),
    }
}
#[test]
fn dual_bound_flips_and_siblings_keep_parent_immutable() {
    let parent = problem(
        vec![0., 1., 2., 3.],
        vec![5., 1., 1., f64::INFINITY],
        vec![1.; 4],
        5.,
    );
    let root = minimize_warm(&parent, 1000, None, None);
    assert_eq!(value(&root), 0.);
    let state = root.warm.as_deref().unwrap();
    for (bound, expected) in [(1., 9.), (0., 12.), (2., 6.), (1., 9.)] {
        let mut child = parent.clone();
        child.upper[0] = bound;
        let warm = minimize_warm(&child, 1000, None, Some(state));
        let cold = minimize(&child, 1000, None);
        assert!((value(&warm) - expected).abs() < 1e-7);
        assert!((value(&cold) - expected).abs() < 1e-7);
        assert_eq!(warm.stats.warm_successes, 1);
        assert_eq!(warm.stats.warm_fallbacks, 0);
        assert!(warm.stats.dual_flips >= 1);
        assert_eq!(warm.stats.dual_pivots, 1);
    }
    assert_eq!(value(&root), 0.);
}
#[test]
fn lower_violation_after_variable_shift() {
    let parent = problem(vec![1., 0.], vec![10., 10.], vec![1., -1.], 0.5);
    let root = minimize_warm(&parent, 1000, None, None);
    let mut child = parent.clone();
    child.rhs[0] = -0.5;
    child.upper[0] = 9.;
    let warm = minimize_warm(&child, 1000, None, root.warm.as_deref());
    assert_eq!(value(&warm), 0.);
    assert_eq!(warm.stats.dual_pivots, 1);
    assert_eq!(warm.stats.warm_fallbacks, 0);
}
#[test]
fn entering_from_upper_bound() {
    let parent = problem(
        vec![0., -2., -1.],
        vec![f64::INFINITY, 1., 1.],
        vec![1.; 3],
        5.,
    );
    let root = minimize_warm(&parent, 1000, None, None);
    assert_eq!(value(&root), -3.);
    let mut child = parent.clone();
    child.rhs[0] = 1.;
    let warm = minimize_warm(&child, 1000, None, root.warm.as_deref());
    assert_eq!(value(&warm), -2.);
    assert_eq!(warm.stats.dual_pivots, 1);
    assert_eq!(warm.stats.warm_fallbacks, 0);
}
#[test]
fn impossible_box_row_requires_certificate() {
    let parent = problem(vec![0., 1.], vec![10., 10.], vec![1., 1.], 2.);
    let root = minimize_warm(&parent, 1000, None, None);
    let mut child = parent.clone();
    child.rhs[0] = -1.;
    let warm = minimize_warm(&child, 1000, None, root.warm.as_deref());
    assert_eq!(warm.outcome, Outcome::Infeasible);
    assert_eq!(warm.stats.warm_successes, 1);
    assert_eq!(minimize(&child, 1000, None).outcome, Outcome::Infeasible);
}
#[test]
fn cost_changes_use_primal_warm_and_structure_changes_reject_basis() {
    let parent = problem(vec![0., 1.], vec![10., 10.], vec![1., 1.], 2.);
    let root = minimize_warm(&parent, 1000, None, None);
    let mut child = parent.clone();
    child.costs = vec![2., 1.];
    let warm = minimize_warm(&child, 1000, None, root.warm.as_deref());
    assert_eq!(value(&warm), 2.);
    assert_eq!(warm.stats.warm_rejections, 0);
    assert_eq!(warm.stats.warm_cost_successes, 1);
    child.columns[1][0].1 = 2.;
    let warm = minimize_warm(&child, 1000, None, root.warm.as_deref());
    assert_eq!(value(&warm), 1.);
    assert_eq!(warm.stats.warm_rejections, 1);
}
#[test]
fn box_only_bounds_and_zero_width_reactivation_fallback() {
    let p = Problem {
        columns: vec![vec![]],
        rhs: vec![],
        costs: vec![-1.],
        upper: vec![0.],
        equality: vec![],
        slack_upper: vec![],
    };
    let root = minimize_warm(&p, 1000, None, None);
    let mut child = p.clone();
    child.upper[0] = 2.;
    let warm = minimize_warm(&child, 1000, None, root.warm.as_deref());
    assert_eq!(value(&warm), -2.);
    assert_eq!(warm.stats.warm_fallbacks, 1);
}

#[test]
fn repeated_random_rhs_and_bound_changes_match_cold_solver() {
    let mut seed = 2026111503u64;
    fn random(seed: &mut u64, n: u64) -> u64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*seed >> 32) % n
    }
    let mut successful = 0;
    let mut dual_pivots = 0;
    let mut dual_flips = 0;
    for _case in 0..100 {
        let n = 8 + random(&mut seed, 9) as usize;
        let m = 3 + random(&mut seed, 6) as usize;
        let mut costs = (0..n)
            .map(|_| random(&mut seed, 11) as f64 - 5.)
            .collect::<Vec<_>>();
        // 目的が一定の面でdual beta=0が続く退化も含める。
        if _case % 3 == 0 {
            costs.fill(0.0);
        }
        let upper = (0..n)
            .map(|_| 2. + random(&mut seed, 4) as f64)
            .collect::<Vec<_>>();
        let equality = (0..m)
            .map(|_| random(&mut seed, 4) == 0)
            .collect::<Vec<_>>();
        let mut columns = vec![vec![]; n];
        let mut rhs = vec![0.; m];
        for j in 0..n {
            for i in 0..m {
                let a = random(&mut seed, 7) as f64 - 3.;
                if a != 0. {
                    columns[j].push((i, a));
                    rhs[i] += a * upper[j] * 0.5;
                }
            }
        }
        for i in 0..m {
            if !equality[i] {
                rhs[i] += random(&mut seed, 3) as f64;
            }
        }
        let root = Problem {
            columns,
            rhs,
            costs,
            upper,
            equality,
            slack_upper: vec![f64::INFINITY; m],
        };
        let original = minimize_warm(&root, 100000, None, None);
        assert!(matches!(original.outcome, Outcome::Optimal { .. }));
        let mut parent = root.clone();
        let mut state = original.warm.clone();
        for step in 0..30 {
            let mut child = parent.clone();
            let j = random(&mut seed, n as u64) as usize;
            child.upper[j] *= random(&mut seed, 5) as f64 / 4.;
            let i = random(&mut seed, m as u64) as usize;
            child.rhs[i] += (random(&mut seed, 5) as f64 - 2.) * 0.25;
            let warm = minimize_warm(&child, 100000, None, state.as_deref());
            let cold = minimize(&child, 100000, None);
            assert!(warm.stats.dual_pivots <= 64.max(4 * m) + 1);
            successful += warm.stats.warm_successes;
            dual_pivots += warm.stats.dual_pivots;
            dual_flips += warm.stats.dual_flips;
            match (&warm.outcome, &cold.outcome) {
                (Outcome::Optimal { objective: a, x }, Outcome::Optimal { objective: b, .. }) => {
                    assert!((a - b).abs() < 1e-6 * (1. + b.abs()), "warm={a}, cold={b}");
                    for j in 0..n {
                        assert!(x[j] >= -1e-7 && x[j] <= child.upper[j] + 1e-7);
                    }
                    let mut activity = vec![0.; m];
                    for j in 0..n {
                        for &(i, a) in &child.columns[j] {
                            activity[i] += a * x[j];
                        }
                    }
                    for i in 0..m {
                        assert!(activity[i] <= child.rhs[i] + 1e-6);
                        if child.equality[i] {
                            assert!((activity[i] - child.rhs[i]).abs() < 1e-6);
                        }
                    }
                    state = warm.warm;
                    parent = child;
                }
                (Outcome::Infeasible, Outcome::Infeasible) => {
                    state = original.warm.clone();
                    parent = root.clone();
                }
                (a, b) => panic!("step {step}: warm {a:?}, cold {b:?}"),
            }
        }
    }
    assert!(successful > 1000);
    assert!(dual_pivots > 100);
    assert!(dual_flips > 10);
}

#[test]
fn ranged_slack_and_parent_artificial_layout_are_reused() {
    let parent = Problem {
        columns: vec![vec![(0, 1.)]],
        rhs: vec![5.],
        costs: vec![1.],
        upper: vec![10.],
        equality: vec![false],
        slack_upper: vec![2.],
    };
    let root = minimize_warm(&parent, 1000, None, None);
    assert_eq!(value(&root), 3.);
    for (rhs, expected) in [(6., 4.), (1., 0.)] {
        let mut child = parent.clone();
        child.rhs[0] = rhs;
        let warm = minimize_warm(&child, 1000, None, root.warm.as_deref());
        assert_eq!(value(&warm), expected);
        assert_eq!(warm.stats.warm_successes, 1);
        assert_eq!(warm.stats.warm_fallbacks, 0);
    }
    let mut child = parent.clone();
    child.rhs[0] = -3.;
    let warm = minimize_warm(&child, 1000, None, root.warm.as_deref());
    assert_eq!(warm.outcome, Outcome::Infeasible);
}
#[test]
fn expired_budget_does_not_restart_cold_phase_one() {
    let parent = problem(vec![0., 1.], vec![5., 5.], vec![1., 1.], 2.);
    let root = minimize_warm(&parent, 1000, None, None);
    let answer = minimize_warm(
        &parent,
        1000,
        Some(std::time::Instant::now()),
        root.warm.as_deref(),
    );
    assert_eq!(answer.outcome, Outcome::TimeLimit);
    assert_eq!(answer.stats.warm_fallbacks, 0);
}

#[test]
fn thousand_cost_changes_match_cold_and_keep_original_basis_reusable() {
    let mut seed = 2026111512u64;
    fn random(seed: &mut u64, n: u64) -> u64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*seed >> 32) % n
    }
    let mut successful = 0;
    for case in 0..100 {
        let n = 4 + random(&mut seed, 9) as usize;
        let m = 2 + random(&mut seed, 5) as usize;
        let mut columns = vec![vec![]; n];
        let mut rhs = vec![0.; m];
        for col in &mut columns {
            for (i, b) in rhs.iter_mut().enumerate() {
                let a = random(&mut seed, 7) as f64 - 3.;
                if a != 0. {
                    col.push((i, a));
                    *b += a;
                }
            }
        }
        let parent = Problem {
            columns,
            rhs,
            costs: (0..n).map(|_| random(&mut seed, 13) as f64 - 6.).collect(),
            upper: vec![2.; n],
            equality: (0..m).map(|i| (case + i) % 3 == 0).collect(),
            slack_upper: vec![f64::INFINITY; m],
        };
        let root = minimize_warm(&parent, 100000, None, None);
        value(&root);
        let mut state = root.warm.clone();
        for step in 0..10 {
            let mut child = parent.clone();
            child.costs = (0..n).map(|_| random(&mut seed, 21) as f64 - 10.).collect();
            if step % 4 == 0 {
                child.costs.fill(0.);
            }
            let warm = minimize_warm(&child, 100000, None, state.as_deref());
            let cold = minimize(&child, 100000, None);
            assert!((value(&warm) - value(&cold)).abs() < 1e-7 * (1. + value(&cold).abs()));
            let mut dense_a = Vec::new();
            let mut dense_b = Vec::new();
            for i in 0..m {
                let row = (0..n)
                    .map(|j| {
                        child.columns[j]
                            .iter()
                            .find(|&&(k, _)| k == i)
                            .map_or(0., |&(_, a)| a)
                    })
                    .collect::<Vec<_>>();
                dense_a.push(row.clone());
                dense_b.push(child.rhs[i]);
                if child.equality[i] {
                    dense_a.push(row.iter().map(|a| -a).collect());
                    dense_b.push(-child.rhs[i]);
                }
            }
            for j in 0..n {
                let mut row = vec![0.; n];
                row[j] = 1.;
                dense_a.push(row);
                dense_b.push(child.upper[j]);
            }
            let dense = mip_codex::lp::minimize(&dense_a, &dense_b, &child.costs, 100000).unwrap();
            let Outcome::Optimal {
                objective: dense_value,
                ..
            } = dense
            else {
                panic!("{dense:?}")
            };
            assert!((value(&warm) - dense_value).abs() < 1e-7 * (1. + dense_value.abs()));
            if let Outcome::Optimal { x, .. } = &warm.outcome {
                for (a, &b) in dense_a.iter().zip(&dense_b) {
                    assert!(a.iter().zip(x).map(|(a, x)| a * x).sum::<f64>() <= b + 1e-7);
                }
            }
            successful += warm.stats.warm_cost_successes;
            assert_eq!(warm.stats.warm_rejections, 0);
            if let Outcome::Optimal { x, .. } = &warm.outcome {
                assert!(x.iter().all(|&v| v >= -1e-7 && v <= 2. + 1e-7));
            }
            state = warm.warm;
        }
        let again = minimize_warm(&parent, 100000, None, root.warm.as_deref());
        assert!((value(&again) - value(&root)).abs() < 1e-8);
    }
    assert!(successful > 900, "{successful}");
}

#[test]
fn changed_cost_and_infeasible_basis_falls_back_to_phase_one() {
    let p = problem(vec![0., 1.], vec![10., 10.], vec![1., 1.], 2.);
    let root = minimize_warm(&p, 1000, None, None);
    let mut child = p.clone();
    child.costs = vec![2., 1.];
    child.upper[0] = 0.;
    let warm = minimize_warm(&child, 1000, None, root.warm.as_deref());
    assert_eq!(value(&warm), 2.);
    assert_eq!(warm.stats.warm_fallbacks, 1);
    assert_eq!(warm.stats.warm_cost_successes, 0);
    child.rhs[0] = -1.;
    let warm = minimize_warm(&child, 1000, None, root.warm.as_deref());
    assert_eq!(warm.outcome, Outcome::Infeasible);
    assert_eq!(warm.stats.warm_fallbacks, 1);
}

#[test]
fn cost_changes_preserve_scaled_units_and_unbounded_ray_checks() {
    for scale in [1e-8, 1., 1e8] {
        let p = problem(vec![0., scale], vec![10. / scale; 2], vec![scale; 2], 2.);
        let root = minimize_warm(&p, 10000, None, None);
        let mut child = p.clone();
        child.costs = vec![2. * scale, scale];
        let warm = minimize_warm(&child, 10000, None, root.warm.as_deref());
        assert!((value(&warm) - 2.).abs() < 1e-7);
        assert_eq!(warm.stats.warm_cost_successes, 1);
        child.costs = vec![scale, -3. * scale];
        let second = minimize_warm(&child, 10000, None, warm.warm.as_deref());
        assert!((value(&second) + 6.).abs() < 1e-7);
        let original = minimize_warm(&p, 10000, None, root.warm.as_deref());
        assert!(value(&original).abs() < 1e-7);
    }
    let p = problem(vec![1., 1.], vec![f64::INFINITY; 2], vec![1., -1.], 0.);
    let root = minimize_warm(&p, 10000, None, None);
    let mut child = p.clone();
    child.costs = vec![-1., 0.];
    let warm = minimize_warm(&child, 10000, None, root.warm.as_deref());
    assert_eq!(warm.outcome, Outcome::Unbounded);
    assert_eq!(warm.stats.warm_cost_attempts, 1);
}
