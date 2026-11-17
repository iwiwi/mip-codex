use mip_codex::{
    mps,
    solve::{Limits, solve},
};
fn run(body: &str) -> mip_codex::solve::Result {
    let model = mps::parse(std::io::Cursor::new(body)).unwrap();
    solve(&model, &Limits::default())
}
#[test]
fn branching_and_dual_bound() {
    let model = mps::parse(std::io::Cursor::new(
        "NAME branch\nROWS\n N c\n L r\nCOLUMNS\n x c -1 r 2\nRHS\n rhs r 3\nBOUNDS\n LI b x 0\n UP b x 3\nENDATA\n",
    )).unwrap();
    let r = solve(
        &model,
        &Limits {
            mir_cuts: false,
            ..Limits::default()
        },
    );
    assert_eq!(r.status, "optimal");
    assert_eq!(r.objective, Some(-1.));
    assert_eq!(r.dual_bound, Some(-1.));
    assert_eq!(r.nodes, 1);
    assert_eq!(r.objective_bound_stats.rounded, 1);
}
#[test]
fn free_and_upper_only_and_offset() {
    let r = run(
        "NAME free\nOBJSENSE MAX\nROWS\n N c\n L r\n G s\nCOLUMNS\n x c 2 r 1\n y c 1 s 1\nRHS\n rhs c -7 r -1\n rhs s -3\nBOUNDS\n FR b x\n MI b y\n UP b y -2\nENDATA\n",
    );
    assert_eq!(r.status, "optimal");
    assert_eq!(r.objective, Some(3.));
}
#[test]
fn integer_infeasible_from_domain() {
    let r =
        run("NAME domain\nROWS\n N c\nCOLUMNS\n x c 1\nBOUNDS\n LI b x 0.2\n UP b x 0.8\nENDATA\n");
    assert_eq!(r.status, "infeasible");
}
#[test]
fn unbounded_relaxation_is_not_mip_claim() {
    let r = run(
        "NAME unbounded\nROWS\n N c\n E r\nCOLUMNS\n x c -1\n y r 1\nRHS\n rhs r 0.5\nBOUNDS\n LI b y 0\nENDATA\n",
    );
    assert_eq!(r.status, "unbounded_relaxation");
    assert_eq!(r.objective, None);
}
#[test]
fn limits_do_not_claim_infeasibility() {
    let model = mps::parse(std::io::Cursor::new(
        "NAME limit\nROWS\n N c\nCOLUMNS\n x c 1\nENDATA\n",
    ))
    .unwrap();
    for (limits, status) in [
        (
            Limits {
                objective_box_report: false,
                rins: false,
                strong_branching: false,
                cover_cuts: false,
                mir_cuts: false,
                clique_cuts: false,
                seconds: 0.,
                ..Limits::default()
            },
            "time_limit",
        ),
        (
            Limits {
                objective_box_report: false,
                rins: false,
                strong_branching: false,
                cover_cuts: false,
                mir_cuts: false,
                clique_cuts: false,
                nodes: 0,
                ..Limits::default()
            },
            "node_limit",
        ),
        (
            Limits {
                objective_box_report: false,
                rins: false,
                strong_branching: false,
                cover_cuts: false,
                mir_cuts: false,
                clique_cuts: false,
                dense_bytes: 0,
                ..Limits::default()
            },
            "resource_limit",
        ),
    ] {
        let r = solve(&model, &limits);
        assert_eq!(r.status, status);
        assert_eq!(r.dual_bound, None);
    }
}
#[test]
fn propagation_preserves_optimum_and_saves_lp_calls() {
    let model = mps::parse(std::io::Cursor::new(
        "NAME prop\nROWS\n N c\n L r\nCOLUMNS\n x c -1 r 2\n y c -1 r 2\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n",
    )).unwrap();
    let cold = solve(
        &model,
        &Limits {
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            strong_branching: false,
            cover_cuts: false,
            mir_cuts: false,
            clique_cuts: false,
            propagation: false,
            primal_repair: false,
            feasibility_pump: false,
            primal_probe: false,
            lp_diving: false,
            ..Limits::default()
        },
    );
    let prop = solve(
        &model,
        &Limits {
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            strong_branching: false,
            cover_cuts: false,
            mir_cuts: false,
            clique_cuts: false,
            propagation: true,
            primal_repair: false,
            feasibility_pump: false,
            primal_probe: false,
            lp_diving: false,
            ..Limits::default()
        },
    );
    assert_eq!(cold.status, "optimal");
    assert_eq!(prop.status, "optimal");
    assert_eq!(cold.objective, Some(-1.));
    assert_eq!(prop.objective, cold.objective);
    assert_eq!(prop.dual_bound, cold.dual_bound);
    assert!(prop.lp_solves < cold.lp_solves);
}

#[test]
fn dense_reference_handles_row_and_objective_scales() {
    for (fixture, expected, scale) in [
        ("scaled-lp-0.mps", 2.6200000000000005e-9f64, 1e-10f64),
        ("scaled-lp-8.mps", -5643204709.798559, 1e8),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(fixture);
        let model = mps::read(path).unwrap();
        let result = solve(
            &model,
            &Limits {
                binary_graph: false,
                rins: false,
                strong_branching: false,
                cover_cuts: false,
                mir_cuts: false,
                clique_cuts: false,
                engine: mip_codex::solve::Engine::Dense,
                primal_repair: false,
                feasibility_pump: false,
                primal_probe: false,
                lp_diving: false,
                ..Limits::default()
            },
        );
        assert_eq!(result.status, "optimal", "{fixture}: {result:?}");
        assert!((result.objective.unwrap() - expected).abs() <= 1e-7 * expected.abs().max(scale));
    }
}
