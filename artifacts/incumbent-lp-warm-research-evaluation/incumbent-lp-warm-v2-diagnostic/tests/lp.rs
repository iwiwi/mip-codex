use mip_codex::lp::{Outcome, minimize};
fn optimal(a: &[Vec<f64>], b: &[f64], c: &[f64], expected: f64) {
    match minimize(a, b, c, 10000).unwrap() {
        Outcome::Optimal { objective, .. } => assert!(
            (objective - expected).abs() < 1e-7,
            "{objective} != {expected}"
        ),
        status => panic!("{status:?}"),
    }
}
#[test]
fn bounded() {
    optimal(
        &[vec![1., 1.], vec![1., 0.], vec![0., 1.]],
        &[4., 2., 3.],
        &[-3., -2.],
        -10.,
    );
}
#[test]
fn phase_one() {
    optimal(&[vec![-1.], vec![1.]], &[-2., 5.], &[1.], 2.);
}
#[test]
fn infeasible() {
    assert_eq!(
        minimize(&[vec![1.], vec![-1.]], &[1., -2.], &[1.], 100).unwrap(),
        Outcome::Infeasible
    );
}
#[test]
fn unbounded() {
    assert_eq!(minimize(&[], &[], &[-1.], 100).unwrap(), Outcome::Unbounded);
}
#[test]
fn redundant() {
    optimal(
        &[vec![0.], vec![-1.], vec![-1.]],
        &[0., -2., -2.],
        &[1.],
        2.,
    );
}
#[test]
fn empty() {
    optimal(&[], &[], &[], 0.);
}
#[test]
fn malformed() {
    assert!(minimize(&[vec![1.]], &[], &[1.], 10).is_err());
    assert!(minimize(&[], &[], &[f64::NAN], 10).is_err());
}
#[test]
fn limit() {
    assert_eq!(
        minimize(&[vec![1.]], &[1.], &[-1.], 0).unwrap(),
        Outcome::IterationLimit
    );
}

#[test]
fn tiny_cost_with_large_width_is_not_ignored() {
    match minimize(&[vec![1.]], &[1e12], &[-1e-12], 1000).unwrap() {
        Outcome::Optimal { objective, .. } => assert!((objective + 1.).abs() < 1e-10),
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        minimize(&[], &[], &[-1e-18], 1000).unwrap(),
        Outcome::Unbounded
    ));
}
