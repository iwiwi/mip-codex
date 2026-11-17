use mip_codex::{
    mps,
    solve::{Limits, solve},
};
fn limits() -> Limits {
    Limits {
        objective_box_report: false,
        nodes: 0,
        seconds: 1.,
        objective_domain_report: true,
        binary_graph: false,
        cover_cuts: false,
        clique_cuts: false,
        aggregate_cuts: false,
        mir_cuts: false,
        three_row_cuts: false,
        rounding_cuts: false,
        objective_rounding: true,
        ..Limits::default()
    }
}
fn model(maximize: bool, upper: bool) -> mip_codex::model::Model {
    let text = format!(
        "NAME domain\nOBJSENSE\n {}\nROWS\n N obj\nCOLUMNS\n x obj 2\nRHS\n rhs obj -3\nBOUNDS\n LI bounds x -3\n{}ENDATA\n",
        if maximize { "MAX" } else { "MIN" },
        if upper { " UP bounds x 4\n" } else { "" }
    );
    mps::parse(std::io::Cursor::new(text)).unwrap()
}
#[test]
fn minimum_domain_bound_is_reported_without_search_or_optimality() {
    let m = model(false, true);
    let r = solve(&m, &limits());
    assert_eq!(r.status, "node_limit");
    assert_eq!(r.lp_solves, 0);
    assert_eq!(r.objective_domain_bound, Some(-3.));
    assert_eq!(r.dual_bound, Some(-3.));
    assert_eq!(r.gap, Some(0.));
    let old = solve(
        &m,
        &Limits {
            objective_domain_report: false,
            ..limits()
        },
    );
    assert_eq!(old.status, r.status);
    assert_eq!(old.objective, r.objective);
    assert_eq!(old.dual_bound, None);
}
#[test]
fn maximum_uses_the_opposite_domain_and_an_absent_side_stays_unknown() {
    let m = model(true, true);
    let r = solve(&m, &limits());
    assert_eq!(r.objective_domain_bound, Some(11.));
    assert_eq!(r.dual_bound, Some(11.));
    assert_eq!(r.status, "node_limit");
    let m = model(true, false);
    let r = solve(&m, &limits());
    assert_eq!(r.objective_domain_bound, None);
    assert_eq!(r.dual_bound, None);
    assert_eq!(r.status, "node_limit");
}
#[test]
fn disabled_grid_does_not_invent_a_domain_proof() {
    let m = model(false, true);
    let r = solve(
        &m,
        &Limits {
            objective_rounding: false,
            ..limits()
        },
    );
    assert!(r.objective_grid.is_none());
    assert!(r.objective_domain_bound.is_none());
    assert!(r.dual_bound.is_none());
}
