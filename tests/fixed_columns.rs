use mip_codex::{
    mps,
    solve::{Engine, Limits, solve},
};
fn limits(engine: Engine, enabled: bool) -> Limits {
    Limits {
        box_primal: false,
        binary_graph: false,
        engine,
        fixed_columns: enabled,
        cover_cuts: false,
        rounding_cuts: false,
        primal_repair: false,
        lp_diving: false,
        primal_probe: false,
        rins: false,
        strong_branching: false,
        ..Limits::default()
    }
}
#[test]
fn deleting_a_static_column_keeps_original_cost_and_child_warm_compatibility() {
    let model=mps::parse(std::io::Cursor::new("NAME static\nROWS\n N c\n L r\nCOLUMNS\n z c 3 r 1\n x c -1 r 2\n y c -1 r 2\nRHS\n rhs r 13\nBOUNDS\n FX b z 10\n BV b x\n BV b y\nENDATA\n")).unwrap();
    for engine in [Engine::Dense, Engine::Revised] {
        for enabled in [false, true] {
            let r = solve(&model, &limits(engine, enabled));
            assert_eq!(r.status, "optimal", "{r:?}");
            assert_eq!(r.objective, Some(29.));
            assert_eq!(r.dual_bound, Some(29.));
            assert_eq!(r.root_fixed_columns, usize::from(enabled));
            assert_eq!(r.values.as_ref().unwrap()["z"], 10.);
            if enabled && matches!(engine, Engine::Revised) {
                assert!(r.lp_warm_successes.unwrap() > 0);
            }
        }
    }
}
#[test]
fn an_integer_singleton_is_proved_from_the_original_bound_intervals() {
    for bound in ["0.5", "0.2"] {
        let model=mps::parse(std::io::Cursor::new(format!("NAME integer\nROWS\n N c\nCOLUMNS\n x c 1\nRHS\n rhs c 0\nBOUNDS\n LI b x {bound}\n UP b x 1.5\nENDATA\n"))).unwrap();
        for engine in [Engine::Dense, Engine::Revised] {
            let r = solve(&model, &limits(engine, true));
            assert_eq!(r.status, "optimal");
            assert_eq!(r.objective, Some(1.));
            assert_eq!(r.root_fixed_columns, 1);
        }
    }
    let model = mps::parse(std::io::Cursor::new(
        "NAME uncertain\nROWS\n N c\nCOLUMNS\n x c 0\nRHS\n rhs c 0\nBOUNDS\n FX b x 0.3\nENDATA\n",
    ))
    .unwrap();
    let r = solve(&model, &limits(Engine::Revised, true));
    assert_eq!(r.status, "optimal");
    assert_eq!(r.root_fixed_columns, 0);
}
#[test]
fn an_empty_lp_column_set_preserves_constant_row_infeasibility() {
    let model=mps::parse(std::io::Cursor::new("NAME impossible\nROWS\n N c\n E r\nCOLUMNS\n x c 1 r 1\nRHS\n rhs r 2\nBOUNDS\n FX b x 1\nENDATA\n")).unwrap();
    for engine in [Engine::Dense, Engine::Revised] {
        let r = solve(&model, &limits(engine, true));
        assert_eq!(r.status, "infeasible", "{r:?}");
        assert!(r.values.is_none());
    }
}
