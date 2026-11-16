use mip_codex::{
    lp::Outcome,
    revised::{Problem, minimize, minimize_warm},
};
fn cap(a: f64, b: f64, upper: f64) -> Problem {
    Problem {
        columns: vec![vec![(0, a)]],
        rhs: vec![b],
        costs: vec![-1.],
        upper: vec![upper],
        equality: vec![false],
        slack_upper: vec![f64::INFINITY],
    }
}
fn optimal(p: &Problem, expected: f64) {
    let a = minimize(p, 10000, None);
    match a.outcome {
        Outcome::Optimal { objective, x } => {
            assert!(
                (objective - expected).abs() <= 1e-9 * expected.abs().max(1.),
                "{objective} vs {expected}"
            );
            assert!((p.columns[0][0].1 * x[0] - p.rhs[0]).abs() < 1e-6);
            assert!(a.stats.scaled_columns > 0);
        }
        other => panic!("{other:?}"),
    }
}
#[test]
fn tiny_and_large_columns_have_bounded_optima() {
    for scale in [1e-12, 1e12] {
        optimal(&cap(scale, 1., f64::INFINITY), -1. / scale);
    }
}
#[test]
fn fixed_normal_column_does_not_hide_tiny_column() {
    let mut p = cap(1e-12, 1., f64::INFINITY);
    p.columns.push(vec![(0, 1.)]);
    p.upper.push(0.);
    p.costs.push(0.);
    optimal(&p, -1e12);
}
#[test]
fn scaled_unbounded_ray_is_checked_in_original_units() {
    let p = Problem {
        columns: vec![vec![(0, 1e-12)], vec![(0, -1e-12)]],
        rhs: vec![0.],
        costs: vec![-1., 0.],
        upper: vec![f64::INFINITY; 2],
        equality: vec![true],
        slack_upper: vec![f64::INFINITY],
    };
    assert!(matches!(
        minimize(&p, 10000, None).outcome,
        Outcome::Unbounded
    ));
}
#[test]
fn scaled_box_contradiction_is_checked_in_original_units() {
    let p = cap(-1e-12, -1., 5e11);
    assert!(matches!(
        minimize(&p, 10000, None).outcome,
        Outcome::Infeasible
    ));
}
#[test]
fn warm_bounds_keep_the_same_scaled_structure() {
    let mut p = cap(1e-12, 1., 2e12);
    let a = minimize_warm(&p, 10000, None, None);
    assert!(matches!(a.outcome, Outcome::Optimal { .. }));
    let basis = a.warm.as_deref().unwrap();
    p.upper[0] = 5e11;
    let b = minimize_warm(&p, 10000, None, Some(basis));
    match b.outcome {
        Outcome::Optimal { objective, .. } => assert!((objective + 5e11).abs() < 1.),
        other => panic!("{other:?}"),
    }
    assert_eq!(b.stats.warm_successes, 1);
    p.equality[0] = true;
    let a = minimize_warm(&p, 10000, None, None);
    assert!(matches!(a.outcome, Outcome::Infeasible));
}
#[test]
fn nonrepresentable_transform_is_not_silently_clipped() {
    let mut p = cap(1e300, 1., 1e300);
    assert!(matches!(
        minimize(&p, 10000, None).outcome,
        Outcome::NumericalError
    ));
    p.upper[0] = 1.;
    p.costs[0] = -1e-300;
    assert!(matches!(
        minimize(&p, 10000, None).outcome,
        Outcome::NumericalError
    ));
}
#[test]
fn coupled_normal_columns_do_not_hide_a_tiny_row() {
    let p = Problem {
        columns: vec![vec![(0, 1e-12), (1, 1.)], vec![(1, -1.)]],
        rhs: vec![1., 0.],
        costs: vec![-1., 0.],
        upper: vec![f64::INFINITY; 2],
        equality: vec![false; 2],
        slack_upper: vec![f64::INFINITY; 2],
    };
    let a = minimize(&p, 10000, None);
    match a.outcome {
        Outcome::Optimal { objective, x } => {
            assert!((objective + 1e12).abs() < 1.);
            assert!((x[0] - 1e12).abs() < 1.);
            assert!(x[1] >= x[0] - 1e-6);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(a.stats.scaled_rows, 1);
}
#[test]
fn scaled_warm_equality_contradiction_has_original_witness() {
    let mut p = cap(1e-12, 1., 2e12);
    p.equality[0] = true;
    let a = minimize_warm(&p, 10000, None, None);
    let state = a.warm.as_deref().unwrap();
    p.upper[0] = 5e11;
    let b = minimize_warm(&p, 10000, None, Some(state));
    assert!(matches!(b.outcome, Outcome::Infeasible));
    assert_eq!(b.stats.warm_successes, 1);
}
#[test]
fn known_fractional_knapsack_is_invariant_under_binary_units() {
    let exponents = [-40, -20, 0, 20, 40];
    for case in 0..125 {
        let row = 2f64.powi(exponents[case % 5]);
        let cost = 2f64.powi(exponents[(case / 5) % 5]);
        let units = [
            2f64.powi(exponents[(case / 25) % 5]),
            2f64.powi(exponents[(case + 2) % 5]),
            2f64.powi(exponents[(case + 4) % 5]),
        ];
        let a = [1., 2., 4.];
        let c = [-3., -2., -1.];
        let budget = 1. + (case % 28) as f64;
        let p = Problem {
            columns: (0..3).map(|j| vec![(0, a[j] * units[j] * row)]).collect(),
            rhs: vec![budget * row],
            costs: (0..3).map(|j| c[j] * units[j] * cost).collect(),
            upper: (0..3).map(|j| 4. / units[j]).collect(),
            equality: vec![false],
            slack_upper: vec![f64::INFINITY],
        };
        let mut remaining = budget;
        let mut expected = 0.;
        for j in 0..3 {
            let z = (remaining / a[j]).min(4.);
            remaining -= a[j] * z;
            expected += c[j] * z;
        }
        let result = minimize(&p, 10000, None);
        match result.outcome {
            Outcome::Optimal { objective, .. } => assert!(
                (objective / cost - expected).abs() < 1e-8,
                "case {case}: {} vs {expected}",
                objective / cost
            ),
            other => panic!("case {case}: {other:?}"),
        }
    }
}
#[test]
fn near_parallel_constraints_are_bounded_by_small_nonzero_pivot() {
    let eps = 2f64.powi(-40);
    let p = Problem {
        columns: vec![vec![(0, 1.), (1, -1.)], vec![(0, -1.), (1, 1. + eps)]],
        rhs: vec![0., 1.],
        costs: vec![-1., 0.],
        upper: vec![f64::INFINITY; 2],
        equality: vec![false; 2],
        slack_upper: vec![f64::INFINITY; 2],
    };
    match minimize(&p, 10000, None).outcome {
        Outcome::Optimal { objective, .. } => assert_eq!(objective, -1. / eps),
        other => panic!("revised {other:?}"),
    }
    match mip_codex::lp::minimize(
        &[vec![1., -1.], vec![-1., 1. + eps]],
        &[0., 1.],
        &[-1., 0.],
        10000,
    )
    .unwrap()
    {
        Outcome::Optimal { objective, .. } => assert_eq!(objective, -1. / eps),
        other => panic!("dense {other:?}"),
    }
}
