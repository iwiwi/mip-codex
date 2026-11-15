use mip_codex::mps;
fn read(cost: &str, offset_rhs: &str) -> mip_codex::model::Model {
    mps::parse(std::io::Cursor::new(format!("NAME objective\nROWS\n N obj\n L r\nCOLUMNS\n x obj {cost} r 1\nRHS\n rhs obj {offset_rhs} r 2\nBOUNDS\n BV b x\nENDATA\n"))).unwrap()
}
#[test]
fn objective_provenance_distinguishes_integer_literals_from_f64_rounding() {
    let exact = read("1", "-2");
    assert_eq!(exact.variables[0].objective_enclosure, Some([1., 1.]));
    assert_eq!(exact.offset_enclosure, Some([2., 2.]));
    let rounded = read("1.00000000000000000000000001", "0");
    let b = rounded.variables[0].objective_enclosure.unwrap();
    assert_eq!(rounded.variables[0].objective, 1.);
    assert!(b[0] < 1. && b[1] > 1.);
    let offset = read("1", "10000000000000001");
    let b = offset.offset_enclosure.unwrap();
    assert!(b[0] < -1e16 && b[1] > -1e16);
}
#[test]
fn cost_provenance_is_independent_of_fractional_objective_offset() {
    let m = read("2", "0.1");
    assert_eq!(m.variables[0].objective_enclosure, Some([2., 2.]));
    assert_ne!(m.offset_enclosure, Some([-0.1, -0.1]));
    let tiny = read("1e-12", "0");
    assert_ne!(tiny.variables[0].objective_enclosure, Some([1e-12, 1e-12]));
}
#[test]
fn arbitrary_multipliers_never_exceed_independent_original_decimal_fraction_bounds() {
    #[derive(serde::Deserialize)]
    struct Case {
        mps: String,
        multipliers: Vec<f64>,
        lower: Vec<Option<f64>>,
        upper: Vec<Option<f64>>,
        expected_lower: Option<f64>,
    }
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/original-bound.json")).unwrap();
    let mut certified = 0;
    for (i, c) in cases.iter().enumerate() {
        let model = mps::parse(std::io::Cursor::new(&c.mps)).unwrap();
        let cert = mip_codex::objective_bound::Certifier::new(&model).unwrap();
        if let Some(bound) = cert.bound(&c.lower, &c.upper, &c.multipliers) {
            let expected = c
                .expected_lower
                .unwrap_or_else(|| panic!("case{i}: falsely certified an unbounded Lagrangian"));
            assert!(
                bound <= expected,
                "case{i}: bound={bound:e} > exact floor={expected:e}"
            );
            certified += 1;
        }
    }
    assert!(certified > 200, "too few actual certificates: {certified}");
}
#[test]
fn objective_grid_follows_unit_equalities_without_circular_inference() {
    use mip_codex::objective_bound::Grid;
    let text = "NAME grid\nROWS\n N obj\n E r\n E s\nCOLUMNS\n x obj 0 r 2\n y r -1 s 3\n z obj 6 s -1\nRHS\n rhs r 1 s 2\nBOUNDS\n BV b x\n LO b y 0\n LO b z 0\nENDATA\n";
    let m = mps::parse(std::io::Cursor::new(text)).unwrap();
    let g = Grid::from_model(&m).unwrap();
    assert_eq!(g.step, 6);
    assert_eq!(g.implied.len(), 2);
    assert_eq!(g.implied[0].variable, 1);
    assert_eq!(g.implied[1].variable, 2);
    let circular = text
        .replace(" x obj 0 r 2", " x obj 0 r 2\n x s 1")
        .replace(" BV b x", " LO b x 0");
    assert!(Grid::from_model(&mps::parse(std::io::Cursor::new(circular)).unwrap()).is_none());
    assert!(Grid::from_model(&read("1.00000000000000000000000001", "0")).is_none());
}
#[test]
fn lattice_rounding_is_conservative_at_integer_boundaries_and_for_maximization() {
    use mip_codex::objective_bound::Grid;
    let g = Grid {
        offset: 3,
        step: 2,
        implied: vec![],
    };
    assert_eq!(g.round_lower(3., false), Some(3.));
    assert_eq!(g.round_lower(3.1, false), Some(5.));
    assert_eq!(g.round_lower(3f64.next_up(), false), Some(5.));
    assert_eq!(g.round_lower(-3., true), Some(-3.));
    assert_eq!(g.round_lower(-2.9, true), Some(-1.));
    assert_eq!(g.round_lower(f64::INFINITY, false), None);
}
#[test]
fn a_tiny_negative_reduced_cost_on_an_unbounded_column_is_not_a_bound() {
    use mip_codex::objective_bound::Certifier;
    let m=mps::parse(std::io::Cursor::new("NAME bound\nROWS\n N obj\n E r\nCOLUMNS\n x obj 1 r 1\nRHS\n rhs r 1\nBOUNDS\n LO b x 0\nENDATA\n")).unwrap();
    let cert = Certifier::new(&m).unwrap();
    assert_eq!(cert.bound(&[Some(0.)], &[None], &[1f64.next_up()]), None);
    assert_eq!(cert.bound(&[Some(0.)], &[None], &[1.]), Some(1.));
    assert!(
        cert.bound(&[Some(0.)], &[None], &[1f64.next_down()])
            .unwrap()
            <= 1.
    );
}
