use mip_codex::{
    mps,
    solve::{Limits, solve},
};
fn limits() -> Limits {
    Limits {
        feasibility_pump: true,
        primal_repair: false,
        primal_probe: false,
        lp_diving: false,
        rins: false,
        strong_branching: false,
        cover_cuts: false,
        rounding_cuts: false,
        objective_rounding: false,
        nodes: 1,
        seconds: 5.,
        ..Limits::default()
    }
}
fn run(text: &str) -> mip_codex::solve::Result {
    let m = mps::parse(std::io::Cursor::new(text)).unwrap();
    let r = solve(&m, &limits());
    if let Some(values) = &r.values {
        let x = m
            .variables
            .iter()
            .map(|v| values[&v.name])
            .collect::<Vec<_>>();
        m.check(&x, true, 1e-7).unwrap();
    }
    r
}
#[test]
fn binary_cost_projection_uses_warm_and_only_returns_primal() {
    let r = run(
        "NAME pump\nROWS\n N c\n G r\nCOLUMNS\n x c 2 r 1\nRHS\n rhs r .4\nBOUNDS\n BV b x\nENDATA\n",
    );
    assert_eq!(r.status, "node_limit");
    assert_eq!(r.objective, Some(2.));
    assert!((r.dual_bound.unwrap() - 0.8).abs() < 1e-7);
    assert!(r.pump.lp_solves >= 1);
    assert!(r.lp_warm_cost_successes.unwrap() > 0);
    assert_eq!(r.pump.improvements, 1);
    assert_eq!(r.pump.binary, 1);
}
#[test]
fn general_integer_distance_projection_and_continuous_completion() {
    let r = run(
        "NAME pump\nROWS\n N c\n E r\nCOLUMNS\n x c 2 r 1\n y r 1\nRHS\n rhs r .4\nBOUNDS\n LI b x 0\n UP b x 3\n LO b y -1\n UP b y 0\nENDATA\n",
    );
    assert_eq!(r.status, "node_limit");
    assert_eq!(r.objective, Some(2.));
    assert!((r.dual_bound.unwrap() - 0.8).abs() < 1e-7);
    assert!(r.pump.lp_solves >= 2);
    assert_eq!(r.pump.general, 1);
    assert_eq!(r.pump.improvements, 1);
}
#[test]
fn cycles_and_impossible_integer_problem_do_not_prune_parent() {
    let r = run(
        "NAME pump\nROWS\n N c\n E r\nCOLUMNS\n x c 2 r 1\nRHS\n rhs r .5\nBOUNDS\n BV b x\nENDATA\n",
    );
    assert_eq!(r.status, "node_limit");
    assert_eq!(r.objective, None);
    assert_eq!(r.dual_bound, Some(1.));
    assert_eq!(r.nodes, 1);
    assert!(r.pump.cycles > 0);
    assert!(r.pump.perturbations > 0);
    assert!(r.pump.limited > 0);
}
#[test]
fn maximum_uses_distance_minimization_and_keeps_original_bound() {
    let r = run(
        "NAME pump\nOBJSENSE MAX\nROWS\n N c\n L r\nCOLUMNS\n x c 2 r 1\nRHS\n rhs r .6\nBOUNDS\n BV b x\nENDATA\n",
    );
    assert_eq!(r.status, "node_limit");
    assert_eq!(r.objective, Some(0.));
    assert!((r.dual_bound.unwrap() - 1.2).abs() < 1e-7);
    assert!(r.pump.lp_solves > 0);
}
#[test]
fn zero_node_or_time_budget_does_not_call_pump() {
    let m = mps::parse(std::io::Cursor::new(
        "NAME p\nROWS\n N c\nCOLUMNS\n x c 1\nBOUNDS\n BV b x\nENDATA\n",
    ))
    .unwrap();
    for l in [
        Limits {
            nodes: 0,
            ..limits()
        },
        Limits {
            seconds: 0.,
            ..limits()
        },
    ] {
        let r = solve(&m, &l);
        assert_eq!(r.pump.calls, 0);
        assert_eq!(r.dual_bound, None);
    }
}
