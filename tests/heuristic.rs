use mip_codex::{
    heuristic::Repair,
    mps,
    solve::{Limits, solve},
};
use std::time::{Duration, Instant};
#[test]
fn rounds_and_repairs_continuous_equality() {
    let model = mps::parse(std::io::Cursor::new("NAME repair\nROWS\n N c\n E r\nCOLUMNS\n x r 1\n y r 1\nRHS\n rhs r 1\nBOUNDS\n BV b x\n UP b y 2\nENDATA\n")).unwrap();
    let lower = vec![Some(0.), Some(0.)];
    let upper = vec![Some(1.), Some(2.)];
    let x = Repair::new(&model)
        .search(
            &[0.49, 0.51],
            &lower,
            &upper,
            Instant::now() + Duration::from_secs(1),
            10000,
            1,
        )
        .unwrap();
    assert!(model.check(&x, true, 1e-7).is_ok());
    assert_eq!(x[0], 0.);
    assert!((x[1] - 1.).abs() < 1e-12);
}
#[test]
fn incumbent_does_not_change_open_node_lower_bound() {
    let model = mps::parse(std::io::Cursor::new("NAME root\nROWS\n N c\n G r\nCOLUMNS\n x c 1 r 1\nRHS\n rhs r 0.5\nBOUNDS\n BV b x\nENDATA\n")).unwrap();
    let cold = solve(
        &model,
        &Limits {
            nodes: 1,
            primal_repair: false,
            ..Limits::default()
        },
    );
    let repair = solve(
        &model,
        &Limits {
            nodes: 1,
            primal_repair: true,
            ..Limits::default()
        },
    );
    assert_eq!(cold.objective, None);
    assert_eq!(repair.status, "node_limit");
    assert_eq!(repair.objective, Some(1.));
    assert_eq!(repair.dual_bound, cold.dual_bound);
    assert_eq!(repair.repair_improvements, 1);
}
#[test]
fn cancellation_cannot_create_false_incumbent() {
    let model = mps::parse(std::io::Cursor::new("NAME cancellation\nROWS\n N c\n L r\nCOLUMNS\n x r 100000000\n y r 1\nRHS\n rhs r 10000000000000000\nBOUNDS\n FX b x 100000000\n BV b y\nENDATA\n")).unwrap();
    let mut repair = Repair::new(&model);
    let x = repair.search(
        &[1e8, 1.],
        &[Some(1e8), Some(0.)],
        &[Some(1e8), Some(1.)],
        Instant::now() + Duration::from_secs(1),
        10000,
        1,
    );
    assert!(x.is_none());
}
#[test]
fn expired_deadline_produces_no_candidate() {
    let model = mps::parse(std::io::Cursor::new(
        "NAME limit\nROWS\n N c\nCOLUMNS\n x c 1\nBOUNDS\n BV b x\nENDATA\n",
    ))
    .unwrap();
    assert!(
        Repair::new(&model)
            .search(&[0.5], &[Some(0.)], &[Some(1.)], Instant::now(), 10000, 1)
            .is_none()
    );
}
