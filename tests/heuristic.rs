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
            box_primal: false,
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            strong_branching: false,
            cover_cuts: false,
            mir_cuts: false,
            clique_cuts: false,
            nodes: 1,
            primal_repair: false,
            feasibility_pump: false,
            primal_probe: false,
            lp_diving: false,
            ..Limits::default()
        },
    );
    let repair = solve(
        &model,
        &Limits {
            box_primal: false,
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            strong_branching: false,
            cover_cuts: false,
            mir_cuts: false,
            clique_cuts: false,
            nodes: 1,
            primal_repair: true,
            primal_probe: false,
            lp_diving: false,
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
#[test]
fn fixed_integer_lp_repairs_linked_continuous_variables() {
    let model = mps::parse(std::io::Cursor::new("NAME linked\nROWS\n N c\n E balance\n G a\n G b\nCOLUMNS\n x balance 1\n p balance 1 a -1\n n balance -1 b -1\n z c 1 a 1\n z b 1\nRHS\n rhs balance 0.5\nBOUNDS\n BV bounds x\nENDATA\n")).unwrap();
    let result = solve(
        &model,
        &Limits {
            box_primal: false,
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            strong_branching: false,
            cover_cuts: false,
            mir_cuts: false,
            clique_cuts: false,
            nodes: 1,
            primal_repair: true,
            primal_probe: false,
            lp_diving: false,
            ..Limits::default()
        },
    );
    assert_eq!(result.status, "node_limit");
    assert_eq!(result.objective, Some(0.5));
    assert_eq!(result.dual_bound, Some(0.));
    assert!(result.repair_lp_candidates > 0);
}
