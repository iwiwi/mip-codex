use mip_codex::{
    mps,
    solve::{Limits, solve},
};
use std::io::Cursor;
fn limits() -> Limits {
    Limits {
        objective_box_report: false,
        box_primal: true,
        dense_bytes: 1,
        primal_repair: false,
        feasibility_pump: false,
        cover_cuts: false,
        mir_cuts: false,
        clique_cuts: false,
        lp_diving: false,
        primal_probe: false,
        rins: false,
        strong_branching: false,
        ..Limits::default()
    }
}
#[test]
fn box_primal_preserves_a_greedy_feasible_point_when_lp_cannot_start() {
    for maximize in [false, true] {
        let cost = if maximize { 1 } else { -1 };
        let sense = if maximize { "MAX" } else { "MIN" };
        let model=mps::parse(Cursor::new(format!("NAME packing\nOBJSENSE\n {sense}\nROWS\n N c\n L a\n L b\n L d\nCOLUMNS\n x c {cost}\n x a 1 b 1\n y c {cost}\n y a 1 d 1\n z c {cost}\n z b 1 d 1\nRHS\n rhs a 1 b 1\n rhs d 1 c -7\nBOUNDS\n BV b x\n BV b y\n BV b z\nENDATA\n"))).unwrap();
        let r = solve(&model, &limits());
        assert_eq!(r.status, "resource_limit");
        assert_eq!(r.lp_solves, 0);
        assert_eq!(r.objective, Some(if maximize { 8. } else { 6. }));
        assert!(r.dual_bound.is_none());
        let x: Vec<_> = model
            .variables
            .iter()
            .map(|v| r.values.as_ref().unwrap()[&v.name])
            .collect();
        assert!(model.check(&x, true, 1e-7).is_ok());
        assert!(r.box_primal.moves > 0 && r.box_primal.candidates > 0);
        assert_eq!(r.first_primal_seconds, r.box_primal.first_seconds);
    }
}
#[test]
fn box_primal_has_no_claim_when_all_endpoints_violate_the_model() {
    let model=mps::parse(Cursor::new("NAME impossible\nROWS\n N c\n G one\n L zero\nCOLUMNS\n x c 0 one 1\n x zero 1\nRHS\n rhs one 1 zero 0\nBOUNDS\n BV b x\nENDATA\n")).unwrap();
    let r = solve(&model, &limits());
    assert_eq!(r.status, "resource_limit");
    assert!(r.values.is_none());
    assert_eq!(r.box_primal.candidates, 0);
    let r = solve(
        &model,
        &Limits {
            seconds: 0.,
            ..limits()
        },
    );
    assert_eq!(r.status, "time_limit");
    assert_eq!(r.box_primal.calls, 0);
    assert!(r.values.is_none());
}
#[test]
fn approximate_activity_updates_cannot_accept_a_cancellation_violation() {
    let model=mps::parse(Cursor::new("NAME cancellation\nROWS\n N c\n L row\nCOLUMNS\n x c -1 row 10000000000000000\n y c -1 row 1\nRHS\n rhs row 10000000000000000\nBOUNDS\n BV b x\n BV b y\nENDATA\n")).unwrap();
    let r = solve(&model, &limits());
    assert_eq!(r.status, "resource_limit");
    let x: Vec<_> = model
        .variables
        .iter()
        .map(|v| r.values.as_ref().unwrap()[&v.name])
        .collect();
    assert!(model.check(&x, true, 1e-7).is_ok());
    assert_ne!(x, vec![1., 1.]);
}

#[test]
fn a_tolerance_feasible_box_point_cannot_turn_root_infeasibility_into_optimality() {
    let model=mps::parse(Cursor::new("NAME near_bounds\nROWS\n N c\nCOLUMNS\n x c 0\nRHS\n rhs c 0\nBOUNDS\n LO b x 0.00000005\n UP b x 0\nENDATA\n")).unwrap();
    let r = solve(
        &model,
        &Limits {
            box_primal: false,
            ..limits()
        },
    );
    assert_eq!(r.status, "infeasible");
    let r = solve(&model, &limits());
    assert_eq!(r.status, "numerical_error");
    assert!(r.root_incumbent_conflict);
    assert!(r.values.is_some());
    assert!(r.dual_bound.is_none());
}

#[test]
fn tolerance_feasibility_cannot_override_phase_one_infeasibility() {
    let model=mps::parse(Cursor::new("NAME phase_one\nROWS\n N c\n G a\n G b\n G d\n G e\n L zero\nCOLUMNS\n x c 0 a 1\n x b 1 d 1\n x e 1 zero 1\nRHS\n rhs a 0.00000005 b 0.00000005\n rhs d 0.00000005 e 0.00000005\n rhs zero 0\nBOUNDS\n LO b x 0\n UP b x 1\nENDATA\n")).unwrap();
    let r = solve(
        &model,
        &Limits {
            dense_bytes: 512 * 1024 * 1024,
            ..limits()
        },
    );
    assert_eq!(r.status, "numerical_error");
    assert!(r.root_incumbent_conflict);
    assert!(r.values.is_some());
}
