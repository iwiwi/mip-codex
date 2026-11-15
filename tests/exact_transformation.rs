use mip_codex::{
    mps,
    solve::{Engine, Limits, solve},
};
fn limits(engine: Engine) -> Limits {
    Limits {
        engine,
        objective_rounding: false,
        primal_repair: false,
        primal_probe: false,
        lp_diving: false,
        rins: false,
        strong_branching: false,
        cover_cuts: false,
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
        "product-overflow",
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
