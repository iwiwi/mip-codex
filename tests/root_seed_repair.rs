use mip_codex::{
    mps,
    solve::{Limits, solve},
};
fn model(equality: bool) -> mip_codex::model::Model {
    let mut text = format!(
        "NAME initial\nROWS\n N obj\n {} row\nCOLUMNS\n",
        if equality { "E" } else { "L" }
    );
    for j in 0..4096 {
        text.push_str(&format!(" x{j} obj 0\n"));
        if j < 2 {
            text.push_str(&format!(" x{j} row 1\n"));
        }
    }
    text.push_str("RHS\n rhs row 1\nBOUNDS\n");
    for j in 0..4096 {
        text.push_str(&format!(" BV b x{j}\n"));
    }
    text.push_str("ENDATA\n");
    mps::parse(std::io::Cursor::new(text)).unwrap()
}
fn limits() -> Limits {
    Limits {
        nodes: 0,
        seconds: 1.,
        root_seed_repair: true,
        binary_graph: false,
        cover_cuts: false,
        clique_cuts: false,
        aggregate_cuts: false,
        mir_cuts: false,
        three_row_cuts: false,
        rounding_cuts: false,
        objective_rounding: false,
        row_ray_report: false,
        ..Limits::default()
    }
}
#[test]
fn root_seed_is_found_without_an_lp_or_an_optimality_claim() {
    let m = model(true);
    let r = solve(&m, &limits());
    assert_eq!(r.status, "node_limit");
    assert_eq!(r.lp_solves, 0);
    assert_eq!(r.objective, Some(0.));
    let stats = r.root_seed_repair.as_ref().unwrap();
    assert_eq!(stats.candidates, 1);
    assert!(stats.moves > 0);
    assert!(r.first_primal_seconds.is_some());
    let values = r.values.unwrap();
    let x: Vec<_> = m.variables.iter().map(|v| values[&v.name]).collect();
    assert!(m.check(&x, true, 0.).is_ok());
    assert!(r.dual_bound.is_none());
}
#[test]
fn existing_box_point_disabled_flag_and_expired_budget_skip_the_extra_search() {
    let m = model(false);
    let r = solve(&m, &limits());
    assert!(r.values.is_some());
    assert!(r.root_seed_repair.is_none());
    let m = model(true);
    let r = solve(
        &m,
        &Limits {
            root_seed_repair: false,
            ..limits()
        },
    );
    assert!(r.root_seed_repair.is_none());
    assert!(r.values.is_none());
    let r = solve(
        &m,
        &Limits {
            seconds: 0.,
            ..limits()
        },
    );
    assert!(r.root_seed_repair.is_none());
    assert!(r.values.is_none());
    assert_eq!(r.status, "time_limit");
}

#[test]
fn reported_row_bound_does_not_upgrade_a_node_limited_seed(){
 let m=model(true);
 let r=solve(&m,&Limits{row_ray_report:true,..limits()});
 assert_eq!(r.status,"node_limit");assert_eq!(r.lp_solves,0);assert_eq!(r.objective,Some(0.));
 let bound=r.dual_bound.unwrap();assert!(bound<=0.&&bound>=-1e-12);assert!(r.row_ray_proof.is_some());
}
