use mip_codex::{
    mps,
    rins::neighborhood,
    solve::{Limits, solve},
};
fn model() -> mip_codex::model::Model {
    mps::parse(std::io::Cursor::new("NAME rins\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1 r 2\n y obj -1 r 2\n z obj 1\n w obj 1\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\n BV b z\n BV b w\nENDATA\n")).unwrap()
}
fn limits() -> Limits {
    Limits {
        box_primal: false,
        objective_rounding: false,
        cover_cuts: false,
        rounding_cuts: false,
        strong_branching: false,
        lp_diving: false,
        primal_probe: false,
        rins: true,
        ..Limits::default()
    }
}
#[test]
fn matching_fixes_preserve_the_seed_and_leave_the_original_model_intact() {
    let m = model();
    let n = neighborhood(&m, &[1., 0.5, 0., 0.], &[1., 0., 0., 0.]).unwrap();
    assert_eq!(n.fixed, 3);
    assert_eq!(n.eligible, 4);
    assert_eq!(n.model.variables[0].lower, Some(1.));
    assert_eq!(n.model.variables[0].upper, Some(1.));
    assert_eq!(n.model.variables[1].upper, Some(1.));
    assert_eq!(m.variables[0].lower, Some(0.));
    assert!(n.model.check(&n.seed, true, 1e-7).is_ok());
}
#[test]
fn malformed_infeasible_and_unhelpful_neighborhoods_are_rejected() {
    let m = model();
    assert!(neighborhood(&m, &[0.; 3], &[0.; 4]).is_none());
    assert!(neighborhood(&m, &[1.; 4], &[1.; 4]).is_none());
    assert!(neighborhood(&m, &[0.5; 4], &[1., 0., 0., 0.]).is_none());
    assert!(neighborhood(&m, &[f64::NAN, 0., 0., 0.], &[0.; 4]).is_none());
}
#[test]
fn local_neighborhood_optimal_does_not_become_a_global_proof() {
    let m = model();
    let result = solve(
        &m,
        &Limits {
            box_primal: false,
            objective_rounding: false,
            nodes: 1,
            ..limits()
        },
    );
    assert_eq!(result.status, "node_limit");
    assert_eq!(result.nodes, 1);
    assert_eq!(result.objective, Some(-1.));
    assert_eq!(result.dual_bound, Some(-1.5));
    assert_eq!(result.rins.calls, 1);
    assert_eq!(result.rins.status.as_deref(), Some("optimal"));
    assert!(result.rins.fixed >= 2);
    assert!(result.rins.lp_solves > 0);
    assert!(result.lp_solves > result.rins.lp_solves);
}
#[test]
fn no_incumbent_or_expired_deadline_does_not_start_a_local_search() {
    let m = model();
    let result = solve(
        &m,
        &Limits {
            box_primal: false,
            objective_rounding: false,
            nodes: 1,
            primal_repair: false,
            ..limits()
        },
    );
    assert_eq!(result.objective, None);
    assert_eq!(result.rins.calls, 0);
    assert_eq!(result.rins.attempts, 0);
    let result = solve(
        &m,
        &Limits {
            box_primal: false,
            objective_rounding: false,
            seconds: 0.,
            ..limits()
        },
    );
    assert_eq!(result.status, "time_limit");
    assert_eq!(result.rins.calls, 0);
}
#[test]
fn later_incumbent_does_not_trigger_an_unbudgeted_retry() {
    let m = model();
    let result = solve(
        &m,
        &Limits {
            box_primal: false,
            objective_rounding: false,
            nodes: 3,
            primal_repair: false,
            ..limits()
        },
    );
    assert_eq!(result.status, "node_limit");
    assert_eq!(result.objective, Some(-1.));
    assert_eq!(result.dual_bound, Some(-1.5));
    assert_eq!(result.rins.attempts, 0);
    assert_eq!(result.rins.calls, 0);
}
