use mip_codex::{
    mps,
    solve::{Limits, solve},
};
#[test]
fn three_original_rows_improve_the_root_with_a_verified_integer_point() {
    let model = mps::parse(std::io::Cursor::new("NAME three\nROWS\n N obj\n L a\n L b\n L c\nCOLUMNS\n x obj -1 a 3\n z obj -1 c 3\n t a 2 b -1\n u b 3 c -6\nRHS\n rhs a 3 b 0\n rhs c 1\nBOUNDS\n BV bd x\n BV bd z\n FR bd t\n FR bd u\nENDATA\n")).unwrap();
    let mut limits = Limits {
        three_row_cuts: false,
        binary_graph: false,
        box_primal: false,
        primal_repair: false,
        feasibility_pump: false,
        lp_diving: false,
        primal_probe: false,
        rins: false,
        strong_branching: false,
        objective_rounding: false,
        cover_cuts: false,
        clique_cuts: false,
        mir_cuts: false,
        aggregate_cuts: false,
        rounding_cuts: false,
        nodes: 1,
        seconds: 5.,
        ..Limits::default()
    };
    let before = solve(&model, &limits);
    assert_eq!(before.status, "node_limit");
    assert!((before.dual_bound.unwrap() + 4. / 3.).abs() < 1e-9);
    limits.three_row_cuts = true;
    let after = solve(&model, &limits);
    assert_eq!(after.status, "node_limit");
    assert_eq!(after.objective, None);
    assert_eq!(after.dual_bound, Some(-1.));
    assert!(after.cut_stats.three_added > 0);
    assert!(after.cut_stats.three_lp_solves > 0);
    assert!(after.root_lp_reused);
    assert!(
        after
            .cut_proofs
            .iter()
            .any(|c| c.proof.aggregation.len() == 3)
    );
    limits.nodes = 100;
    let complete = solve(&model, &limits);
    // 連続点の微小な残差を許容する点検査と、最適性の承認を分離する。
    assert_eq!(complete.status, "numerical_error");
    assert!(complete.optimal_original_conflict);
    assert_eq!(complete.dual_bound,None);
    assert_eq!(complete.gap,None);
    assert_eq!(complete.objective, Some(-1.));
    let values = complete.values.unwrap();
    let point: Vec<_> = model.variables.iter().map(|v| values[&v.name]).collect();
    assert!(model.check(&point, true, 1e-9).is_ok());
}

#[test]
fn precut_probe_returns_only_a_point_and_keeps_the_stronger_global_lp_bound() {
    let model = mps::parse(std::io::Cursor::new("NAME precut\nROWS\n N obj\n L a\n L b\n L c\nCOLUMNS\n x obj -1 a 3\n z obj -1 c 6\n t a 2 b -1\n u b 3 c -6\nRHS\n rhs a 3 b 0\n rhs c 4\nBOUNDS\n BV bd x\n BV bd z\n FR bd t\n FR bd u\nENDATA\n")).unwrap();
    let result = solve(
        &model,
        &Limits {
            binary_graph: false,
            box_primal: false,
            primal_repair: false,
            feasibility_pump: false,
            lp_diving: false,
            primal_probe: true,
            rins: false,
            strong_branching: false,
            objective_rounding: false,
            cover_cuts: false,
            clique_cuts: false,
            mir_cuts: false,
            aggregate_cuts: false,
            rounding_cuts: false,
            three_row_cuts: true,
            nodes: 1,
            seconds: 5.,
            ..Limits::default()
        },
    );
    assert_eq!(result.status, "node_limit");
    assert_eq!(result.objective, Some(-1.));
    assert!((result.dual_bound.unwrap() + 1.5).abs() < 1e-9);
    assert_eq!(result.probe.precut_roots, 1);
    assert_eq!(result.probe.status.as_deref(), Some("optimal"));
    let values = result.values.unwrap();
    let point: Vec<_> = model.variables.iter().map(|v| values[&v.name]).collect();
    assert!(model.check(&point, true, 1e-9).is_ok());
}
