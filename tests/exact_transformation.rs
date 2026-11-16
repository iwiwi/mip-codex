use mip_codex::{
    mps,
    solve::{Engine, Limits, solve},
};
fn limits(engine: Engine) -> Limits {
    Limits {
        engine,
        fixed_columns: true,
        objective_rounding: false,
        primal_repair: false,
        feasibility_pump: false,
        primal_probe: false,
        lp_diving: false,
        rins: false,
        strong_branching: false,
        cover_cuts: false,
        clique_cuts: false,
        rounding_cuts: false,
        ..Limits::default()
    }
}
#[test]
fn objective_offset_row_shift_and_overflow_cancel_before_rounding() {
    for fixture in [
        "fixed-objective",
        "constrained-objective",
        "offset-objective",
        "fixed-row",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/exact-transformation/{fixture}.mps"));
        let mut model = mps::read(path).unwrap();
        for maximize in [false, true] {
            model.maximize = maximize;
            for engine in [Engine::Dense, Engine::Revised] {
                let r = solve(&model, &limits(engine));
                assert_eq!(r.status, "optimal", "{fixture}/{engine:?}: {r:?}");
                assert_eq!(r.objective, Some(1.));
                assert_eq!(r.dual_bound, Some(1.));
                let values = r.values.unwrap();
                let x = model
                    .variables
                    .iter()
                    .map(|v| values[&v.name])
                    .collect::<Vec<_>>();
                model.check(&x, true, 1e-7).unwrap();
            }
        }
    }
}
#[test]
fn a_shift_that_cannot_preserve_absolute_row_precision_is_unresolved() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/exact-transformation/unrepresentable-shift.mps");
    let model = mps::read(path).unwrap();
    for engine in [Engine::Dense, Engine::Revised] {
        let r = solve(&model, &limits(engine));
        assert_eq!(r.status, "numerical_error", "{engine:?}: {r:?}");
        assert_eq!(r.objective, None);
    }
}

#[test]
fn original_decimal_ill_conditioning_is_unresolved() {
    let prefix = "NAME test\nROWS\n N c\n";
    for body in [
        "COLUMNS\n x c 0\nRHS\n rhs c 0\nBOUNDS\n FX b x 10000000000000001\nENDATA\n",
        "COLUMNS\n x c 0.1\n z c -100000000000\nRHS\n rhs c 0\nBOUNDS\n FX b x 1000000000000\n FX b z 1\nENDATA\n",
        " E forced\n E r\nCOLUMNS\n x c 0 forced 1\n x r 0.1\nRHS\n rhs forced 1000000000000 r 100000000000\nENDATA\n",
        " E r\nCOLUMNS\n x c 0 r 0.1\nRHS\n rhs r 100000000000\nBOUNDS\n FX b x 1000000000000\nENDATA\n",
        "COLUMNS\n x c 10000000000000000\nRHS\n rhs c 3000000000000000\nBOUNDS\n FX b x 0.3\nENDATA\n",
        "COLUMNS\n x c 10000000000000000\n y c 1\n z c -10000000000000001\nRHS\n rhs c 0\nBOUNDS\n FX b x 1\n FX b y 1\n FX b z 1\nENDATA\n",
    ] {
        let model = mps::parse(std::io::Cursor::new(format!("{prefix}{body}"))).unwrap();
        for engine in [Engine::Dense, Engine::Revised] {
            let r = solve(&model, &limits(engine));
            assert_eq!(r.status, "numerical_error", "{engine:?}: {r:?}");
            assert_eq!(r.objective, None);
        }
    }
}
