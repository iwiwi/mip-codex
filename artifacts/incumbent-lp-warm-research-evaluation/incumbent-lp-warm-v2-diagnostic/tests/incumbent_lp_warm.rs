use mip_codex::{
    lp::Outcome,
    revised::{self, Problem},
};
use std::time::Instant;
fn covering(scale: f64) -> Problem {
    Problem {
        columns: vec![vec![(0, -scale)], vec![(0, -scale)]],
        rhs: vec![-scale],
        costs: vec![scale, scale],
        upper: vec![1., 1.],
        equality: vec![false],
        slack_upper: vec![f64::INFINITY],
    }
}
fn objective(answer: &revised::Answer) -> f64 {
    match &answer.outcome {
        Outcome::Optimal { objective, .. } => *objective,
        other => panic!("{other:?}"),
    }
}
#[test]
fn feasible_endpoint_basis_matches_cold_across_scaling() {
    for scale in [2f64.powi(-16), 1., 1e6] {
        let p = covering(scale);
        let cold = revised::minimize_warm(&p, 1000, None, None);
        let warm = revised::minimize_from_point(&p, 1000, None, &[1., 1.]);
        assert!((objective(&cold) - scale).abs() <= 1e-8 * scale.abs().max(1.));
        assert!((objective(&warm) - objective(&cold)).abs() <= 1e-8 * scale.abs().max(1.));
        assert_eq!(warm.stats.feasibility_pricing_calls, 0);
        assert_eq!(warm.stats.warm_attempts, 1);
        assert_eq!(warm.stats.warm_successes, 1);
        assert_eq!(warm.stats.warm_rejections, 0);
        assert_eq!(warm.stats.warm_fallbacks, 0);
        if let Outcome::Optimal { x, .. } = &warm.outcome {
            assert!(x[0] + x[1] >= 1. - 1e-8);
        }
    }
}
#[test]
fn equality_and_ranged_slack_bases_are_valid_and_reusable() {
    let p = Problem {
        columns: vec![vec![(0, 1.)], vec![(0, 1.)]],
        rhs: vec![1.],
        costs: vec![1., 2.],
        upper: vec![1., 1.],
        equality: vec![true],
        slack_upper: vec![0.],
    };
    let warm = revised::minimize_from_point(&p, 1000, None, &[1., 0.]);
    assert_eq!(objective(&warm), 1.);
    let p = Problem {
        rhs: vec![2.],
        costs: vec![1., 1.],
        equality: vec![false],
        slack_upper: vec![1.],
        ..p
    };
    let warm = revised::minimize_from_point(&p, 1000, None, &[1., 1.]);
    assert_eq!(objective(&warm), 1.);
    let child = Problem { rhs: vec![3.], ..p };
    let child = revised::minimize_warm(&child, 1000, None, warm.warm.as_deref());
    assert_eq!(objective(&child), 2.);
    assert_eq!(child.stats.warm_successes, 1);
}
#[test]
fn invalid_interior_and_wrong_length_points_fall_back_and_expiry_is_preserved() {
    let p = covering(1e6);
    for point in [
        vec![0., 0.],
        vec![0.5, 0.5],
        vec![1.],
        vec![1., 1., 0.],
        vec![f64::NAN, 1.],
    ] {
        let r = revised::minimize_from_point(&p, 1000, None, &point);
        assert_eq!(r.stats.warm_rejections, 1);
        assert!((objective(&r) - 1e6).abs() < 1e-6);
    }
    let r = revised::minimize_from_point(&p, 1000, Some(Instant::now()), &[1., 1.]);
    assert!(matches!(r.outcome, Outcome::TimeLimit));
}
