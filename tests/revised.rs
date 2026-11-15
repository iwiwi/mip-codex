use mip_codex::{
    lp::Outcome,
    revised::{Problem, minimize},
};
fn p(
    a: Vec<Vec<f64>>,
    rhs: Vec<f64>,
    costs: Vec<f64>,
    upper: Vec<f64>,
    equality: Vec<bool>,
    slack_upper: Vec<f64>,
) -> Problem {
    let n = costs.len();
    let mut columns = vec![vec![]; n];
    for (i, row) in a.iter().enumerate() {
        for (j, &v) in row.iter().enumerate() {
            if v != 0.0 {
                columns[j].push((i, v));
            }
        }
    }
    Problem {
        columns,
        rhs,
        costs,
        upper,
        equality,
        slack_upper,
    }
}
fn obj(p: &Problem, expected: f64) {
    let a = minimize(p, 10000, None);
    match a.outcome {
        Outcome::Optimal { objective, .. } => {
            assert!(
                (objective - expected).abs() < 1e-7,
                "{objective} != {expected}"
            );
            assert!((a.dual.unwrap() - expected).abs() < 1e-7);
        }
        s => panic!("{s:?}"),
    }
}
#[test]
fn finite_upper_bound_flips() {
    let p = p(vec![], vec![], vec![-2., -3.], vec![2., 3.], vec![], vec![]);
    obj(&p, -13.);
    assert_eq!(minimize(&p, 100, None).stats.flips, 2);
}
#[test]
fn ranged_slack_starts_at_upper_bound() {
    obj(
        &p(
            vec![vec![1.]],
            vec![5.],
            vec![1.],
            vec![10.],
            vec![false],
            vec![2.],
        ),
        3.,
    );
}
#[test]
fn phase_one_and_fixed_artificial_degeneracy() {
    obj(
        &p(
            vec![vec![1., 1.], vec![2., 2.]],
            vec![1., 2.],
            vec![1., 2.],
            vec![1., 1.],
            vec![true, true],
            vec![0., 0.],
        ),
        1.,
    );
}
#[test]
fn infeasible_bounds_and_equations() {
    let p = p(
        vec![vec![1.]],
        vec![2.],
        vec![1.],
        vec![1.],
        vec![true],
        vec![0.],
    );
    assert_eq!(minimize(&p, 100, None).outcome, Outcome::Infeasible);
}
#[test]
fn negative_rhs() {
    obj(
        &p(
            vec![vec![-1.]],
            vec![-2.],
            vec![1.],
            vec![5.],
            vec![false],
            vec![f64::INFINITY],
        ),
        2.,
    );
}
#[test]
fn unbounded_ray() {
    let p = p(
        vec![vec![-1.]],
        vec![0.],
        vec![-1.],
        vec![f64::INFINITY],
        vec![false],
        vec![f64::INFINITY],
    );
    assert_eq!(minimize(&p, 100, None).outcome, Outcome::Unbounded);
}
#[test]
fn sparse_factorization_after_many_updates() {
    let n = 80;
    let mut rows = vec![vec![0.; n]; n];
    for i in 0..n {
        rows[i][i] = 1.;
        if i > 0 {
            rows[i][i - 1] = 0.25;
        }
    }
    let p = p(
        rows,
        vec![2.; n],
        vec![-1.; n],
        vec![f64::INFINITY; n],
        vec![false; n],
        vec![f64::INFINITY; n],
    );
    let a = minimize(&p, 10000, None);
    let expected: f64 = (0..n)
        .scan(0., |previous, _| {
            let x = 2. - 0.25 * (*previous);
            *previous = x;
            Some(-x)
        })
        .sum();
    match a.outcome {
        Outcome::Optimal { objective, .. } => assert!((objective - expected).abs() < 1e-7),
        s => panic!("{s:?}"),
    }
    assert!(a.stats.pivots >= 50);
    assert!(a.stats.refactors >= 1);
}

#[test]
fn duplicate_sparse_entries_are_summed() {
    let p = Problem {
        columns: vec![vec![(0, 0.5), (0, 0.5)]],
        rhs: vec![2.0],
        costs: vec![-1.0],
        upper: vec![f64::INFINITY],
        equality: vec![false],
        slack_upper: vec![f64::INFINITY],
    };
    obj(&p, -2.0);
}

#[test]
fn tiny_cost_does_not_hide_unbounded_direction() {
    for cost in [1e-12, 1e-18] {
        // x=z is an unbounded improving direction; unrelated y has unit cost.
        let p = Problem {
            columns: vec![vec![(0, 1.)], vec![(0, -1.)], vec![]],
            rhs: vec![0.],
            costs: vec![-cost, 0., 1.],
            upper: vec![f64::INFINITY, f64::INFINITY, 1.],
            equality: vec![true],
            slack_upper: vec![f64::INFINITY],
        };
        assert!(matches!(
            minimize(&p, 10000, None).outcome,
            Outcome::Unbounded
        ));
    }
}
#[test]
fn small_cost_and_large_finite_width_preserve_objective() {
    let p = Problem {
        columns: vec![vec![], vec![]],
        rhs: vec![],
        costs: vec![-1e-12, 1.],
        upper: vec![1e12, 1.],
        equality: vec![],
        slack_upper: vec![],
    };
    obj(&p, -1.);
}
