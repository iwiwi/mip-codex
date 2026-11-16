use mip_codex::{
    mps,
    solve::{Limits, solve},
};
fn model(rhs: &str) -> mip_codex::model::Model {
    mps::parse(std::io::Cursor::new(format!("NAME cache\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1 r 2\n y obj -1 r 2\nRHS\n rhs r {rhs}\nBOUNDS\n BV b x\n BV b y\nENDATA\n"))).unwrap()
}
fn limits() -> Limits {
    Limits {
        binary_graph: false,
        box_primal: false,
        primal_repair: false,
        feasibility_pump: false,
        lp_diving: false,
        primal_probe: false,
        rins: false,
        strong_branching: false,
        objective_rounding: false,
        rounding_cuts: false,
        aggregate_cuts: false,
        cover_cuts: true,
        ..Limits::default()
    }
}
#[test]
fn root_without_new_cuts_is_solved_once_even_without_warm_start() {
    for warm_start in [false, true] {
        let r = solve(
            &model("2"),
            &Limits {
                warm_start,
                ..limits()
            },
        );
        assert_eq!(r.status, "optimal");
        assert_eq!(r.objective, Some(-1.));
        assert!(r.root_lp_reused);
        assert_eq!(r.lp_solves, 1);
        assert_eq!(r.cut_lp_solves, 1);
        assert!(r.cut_initial_lp_seconds > 0.);
    }
}
#[test]
fn last_cut_result_is_reused_only_when_no_later_row_is_added() {
    for rounds in [1, 2] {
        let r = solve(
            &model("3"),
            &Limits {
                cover_rounds: rounds,
                ..limits()
            },
        );
        assert_eq!(r.status, "optimal");
        assert_eq!(r.objective, Some(-1.));
        assert!(!r.cut_proofs.is_empty());
        assert_eq!(r.root_lp_reused, rounds == 2);
        assert_eq!(r.lp_solves, 2);
        assert_eq!(r.lp_warm_row_extensions, Some(1));
    }
}
#[test]
fn propagation_and_disabled_cuts_do_not_reuse_an_old_root() {
    for l in [
        Limits {
            propagation: true,
            ..limits()
        },
        Limits {
            cover_cuts: false,
            ..limits()
        },
    ] {
        let r = solve(&model("3"), &l);
        assert_eq!(r.status, "optimal");
        assert_eq!(r.objective, Some(-1.));
        assert!(!r.root_lp_reused);
        assert_eq!(r.cut_initial_lp_seconds, 0.);
    }
}
#[test]
fn zero_limits_and_cached_pivot_limit_preserve_status() {
    for (l, status) in [
        (
            Limits {
                seconds: 0.,
                ..limits()
            },
            "time_limit",
        ),
        (
            Limits {
                nodes: 0,
                ..limits()
            },
            "node_limit",
        ),
    ] {
        let r = solve(&model("3"), &l);
        assert_eq!(r.status, status);
        assert_eq!(r.lp_solves, 0);
        assert!(!r.root_lp_reused);
        assert_eq!(r.objective, None);
        assert_eq!(r.dual_bound, None);
    }
    let r = solve(
        &model("3"),
        &Limits {
            pivots: 0,
            ..limits()
        },
    );
    assert_eq!(r.status, "iteration_limit");
    assert!(r.root_lp_reused);
    assert_eq!(r.lp_solves, 1);
    assert_eq!(r.objective, None);
}
#[test]
fn cached_infeasibility_still_uses_original_row_certification() {
    let r = solve(&model("-1.00000000000000000001"), &limits());
    assert_eq!(r.status, "infeasible");
    assert!(r.root_lp_reused);
    assert_eq!(r.lp_solves, 1);
    assert_eq!(r.infeasibility_proof_failures, 0);
    assert!(r.infeasibility_proof_calls > 0);
}
