use mip_codex::{
    mps,
    solve::{Limits, solve},
};
fn run(text: &str, enabled: bool, nodes: usize) -> mip_codex::solve::Result {
    let model = mps::parse(std::io::Cursor::new(text)).unwrap();
    let result = solve(
        &model,
        &Limits {
            objective_rounding: false,
            rins: false,
            strong_branching: false,
            cover_cuts: false,
            lp_diving: enabled,
            primal_repair: false,
            primal_probe: false,
            nodes,
            ..Limits::default()
        },
    );
    if let Some(values) = &result.values {
        let x: Vec<_> = model.variables.iter().map(|v| values[&v.name]).collect();
        model.check(&x, true, 1e-7).unwrap();
    }
    result
}
#[test]
fn diving_finds_cover_without_claiming_root_bound_is_integer_optimum() {
    let text = "NAME cover\nROWS\n N c\n G a\n G b\n G d\nCOLUMNS\n x c 1 a 1\n x b 1\n y c 1 a 1\n y d 1\n z c 1 b 1\n z d 1\nRHS\n rhs a 1 b 1\n rhs d 1\nBOUNDS\n BV bounds x\n BV bounds y\n BV bounds z\nENDATA\n";
    let off = run(text, false, 1);
    let on = run(text, true, 1);
    assert_eq!(off.objective, None);
    assert_eq!(on.objective, Some(2.));
    assert_eq!(on.dual_bound, off.dual_bound);
    assert_eq!(on.dual_bound, Some(1.5));
    assert_eq!(on.status, "node_limit");
    assert_eq!(on.nodes, 1);
    assert_eq!(on.diving.improvements, 1);
    assert!(on.lp_solves > on.nodes);
}
#[test]
fn local_infeasibility_reverses_direction_without_pruning_parent() {
    let text = "NAME reverse\nROWS\n N c\n G r\nCOLUMNS\n x c 1 r 1\nRHS\n rhs r 0.4\nBOUNDS\n BV b x\nENDATA\n";
    let on = run(text, true, 1);
    assert_eq!(on.diving.reversals, 1);
    assert_eq!(on.diving.infeasible, 1);
    assert_eq!(on.objective, Some(1.));
    assert_eq!(on.status, "node_limit");
    assert!((on.dual_bound.unwrap() - 0.4).abs() < 1e-12);
}
#[test]
fn two_failed_sides_do_not_prove_mip_infeasibility() {
    let text = "NAME infeasible\nROWS\n N c\n E r\nCOLUMNS\n x c 1 r 1\nRHS\n rhs r 0.4\nBOUNDS\n BV b x\nENDATA\n";
    let result = run(text, true, 1);
    assert!(result.diving.infeasible >= 2);
    assert_eq!(result.objective, None);
    assert_eq!(result.status, "node_limit");
    assert_eq!(run(text, true, usize::MAX).status, "infeasible");
}
#[test]
fn expired_global_deadline_skips_all_local_solves() {
    let model = mps::parse(std::io::Cursor::new("NAME zero\nROWS\n N c\n G r\nCOLUMNS\n x c 1 r 1\nRHS\n rhs r 0.4\nBOUNDS\n BV b x\nENDATA\n")).unwrap();
    let result = solve(
        &model,
        &Limits {
            objective_rounding: false,
            rins: false,
            strong_branching: false,
            cover_cuts: false,
            seconds: 0.,
            lp_diving: true,
            ..Limits::default()
        },
    );
    assert_eq!(result.diving.lp_solves, 0);
    assert_eq!(result.diving.candidates, 0);
    assert_eq!(result.status, "time_limit");
}
#[test]
fn failed_integral_fixing_does_not_discard_a_feasible_integer_solution() {
    // 根LPはx=.5,y=0。局所的にy=0を固定すると両側が失敗するが、x=0,y=1は実行可能。
    let text = "NAME localfix\nROWS\n N c\n E r\nCOLUMNS\n x r 1\n y c 1 r 0.5\nRHS\n rhs r 0.5\nBOUNDS\n BV b x\n BV b y\nENDATA\n";
    let root = run(text, true, 1);
    assert!(root.diving.infeasible >= 2);
    assert_eq!(root.objective, None);
    assert_eq!(root.status, "node_limit");
    let complete = run(text, true, usize::MAX);
    assert_eq!(complete.status, "optimal");
    assert_eq!(complete.objective, Some(1.));
}
