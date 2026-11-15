use mip_codex::{
    mps,
    solve::{Engine, Limits, solve},
};
fn limits(engine: Engine) -> Limits {
    Limits {
        engine,
        fixed_columns: true,
        objective_rounding: false,
        cover_cuts: false,
        rounding_cuts: false,
        primal_repair: false,
        feasibility_pump: false,
        lp_diving: false,
        primal_probe: false,
        rins: false,
        strong_branching: false,
        ..Limits::default()
    }
}
#[test]
fn fixed_large_integers_and_reported_bounds_are_exact_json_numbers() {
    for name in [
        "max-zero",
        "max-cost",
        "negative-max-cost",
        "two60",
        "two100",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/serialization/{name}.mps"));
        let raw = std::fs::read_to_string(&path).unwrap();
        let expected = raw
            .lines()
            .find(|l| l.trim_start().starts_with("FX "))
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap();
        let mut model = mps::read(path).unwrap();
        for maximize in [false, true] {
            model.maximize = maximize;
            for engine in [Engine::Dense, Engine::Revised] {
                let r = solve(&model, &limits(engine));
                assert_eq!(r.status, "optimal", "{name}/{engine:?}: {r:?}");
                let json = serde_json::to_value(&r).unwrap();
                assert!(json["values"]["x"].is_number());
                assert_eq!(json["values"]["x"].to_string(), expected);
                if model.variables[0].objective == 1. {
                    assert_eq!(json["objective"].to_string(), expected);
                    assert_eq!(json["dual_bound"].to_string(), expected);
                } else {
                    assert_eq!(r.objective, Some(0.));
                }
            }
        }
    }
}
#[derive(serde::Deserialize)]
struct Boundary {
    bound: String,
    value_bits: u64,
    serialized: String,
    valid: bool,
}
#[test]
fn rounded_bound_plus_tolerance_never_accepts_an_invalid_decimal_point() {
    let cases: Vec<Boundary> = serde_json::from_str(include_str!(
        "fixtures/serialization/tolerance-boundaries.json"
    ))
    .unwrap();
    let mut rejected = 0;
    for c in cases {
        let model=mps::parse(std::io::Cursor::new(format!("NAME tolerance\nROWS\n N c\nCOLUMNS\n x c 0\nRHS\n rhs c 0\nBOUNDS\n FX b x {}\nENDATA\n",c.bound))).unwrap();
        let value = f64::from_bits(c.value_bits);
        assert_eq!(serde_json::to_string(&value).unwrap(), c.serialized);
        if !c.valid {
            assert!(
                model.check(&[value], false, 1e-6).is_err(),
                "{} {}",
                c.bound,
                c.serialized
            );
            rejected += 1;
        }
    }
    assert!(rejected > 50);
}
