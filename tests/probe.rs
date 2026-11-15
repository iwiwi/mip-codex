use mip_codex::{
    mps,
    solve::{Limits, solve},
};
#[test]
fn verified_candidate_does_not_replace_open_node_bound() {
    for sense in [" MIN", " MAX"] {
        let text = format!(
            "NAME probe\nOBJSENSE\n{sense}\nROWS\n N c\n G r\nCOLUMNS\n x c {} r 1\nRHS\n rhs c -7 r 0.4\nBOUNDS\n BV b x\nENDATA\n",
            if sense == " MIN" { 1 } else { -1 }
        );
        let model = mps::parse(std::io::Cursor::new(text)).unwrap();
        let result = solve(
            &model,
            &Limits {
                box_primal: false,
                objective_rounding: false,
                rins: false,
                strong_branching: false,
                cover_cuts: false,
                nodes: 1,
                primal_probe: true,
                primal_repair: false,
                lp_diving: false,
                ..Limits::default()
            },
        );
        assert_eq!(result.status, "node_limit");
        assert_eq!(result.probe.calls, 1);
        assert_eq!(result.probe.improvements, 1);
        assert_eq!(
            result.objective,
            Some(if sense == " MIN" { 8. } else { 6. })
        );
        assert!(
            (result.dual_bound.unwrap() - if sense == " MIN" { 7.4 } else { 6.6 }).abs() < 1e-10
        );
        assert_eq!(result.nodes, 1);
        assert!(result.probe.nodes > 1 && result.lp_solves > result.nodes);
    }
}
#[test]
fn local_proof_is_not_promoted_to_global_infeasibility() {
    let model=mps::parse(std::io::Cursor::new("NAME probe\nROWS\n N c\n E r\nCOLUMNS\n x c 1 r 1\nRHS\n rhs r 0.4\nBOUNDS\n BV b x\nENDATA\n")).unwrap();
    let result = solve(
        &model,
        &Limits {
            box_primal: false,
            objective_rounding: false,
            rins: false,
            strong_branching: false,
            cover_cuts: false,
            nodes: 1,
            primal_probe: true,
            primal_repair: false,
            lp_diving: false,
            ..Limits::default()
        },
    );
    assert_eq!(result.probe.status.as_deref(), Some("infeasible"));
    assert_eq!(result.status, "node_limit");
    assert_eq!(result.objective, None);
}
#[test]
fn zero_deadline_does_not_start_a_probe() {
    let model=mps::parse(std::io::Cursor::new("NAME probe\nROWS\n N c\n G r\nCOLUMNS\n x c 1 r 1\nRHS\n rhs r 0.4\nBOUNDS\n BV b x\nENDATA\n")).unwrap();
    let result = solve(
        &model,
        &Limits {
            box_primal: false,
            objective_rounding: false,
            rins: false,
            strong_branching: false,
            cover_cuts: false,
            seconds: 0.,
            primal_probe: true,
            ..Limits::default()
        },
    );
    assert_eq!(result.probe.calls, 0);
    assert_eq!(result.status, "time_limit");
}
