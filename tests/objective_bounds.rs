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
        min_domain: None,
        max_domain: None,
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
fn optimization_limits() -> mip_codex::solve::Limits {
    mip_codex::solve::Limits {
        nodes: 1,
        objective_rounding: true,
        cover_cuts: false,
        rounding_cuts: false,
        strong_branching: false,
        rins: false,
        primal_repair: false,
        lp_diving: false,
        primal_probe: false,
        ..mip_codex::solve::Limits::default()
    }
}
#[test]
fn certified_lattice_bound_strengthens_the_root_without_a_primal_or_extra_lp() {
    use mip_codex::solve::{Limits, solve};
    let text = "NAME grid\nROWS\n N obj\n L r\nCOLUMNS\n x obj -2 r 2\n y obj -2 r 2\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n";
    let m = mps::parse(std::io::Cursor::new(text)).unwrap();
    let limits = optimization_limits();
    let before = solve(
        &m,
        &Limits {
            objective_rounding: false,
            ..limits.clone()
        },
    );
    let after = solve(&m, &limits);
    assert_eq!(before.dual_bound, Some(-3.));
    assert_eq!(after.dual_bound, Some(-2.));
    assert_eq!(after.status, "node_limit");
    assert_eq!(after.objective, None);
    assert_eq!(after.lp_solves, before.lp_solves);
    assert!(after.objective_bound_stats.valid > 0);
    assert_eq!(after.objective_bound_stats.rounded, 1);
}
#[test]
fn maximization_offset_lower_side_and_bounded_range_use_original_row_multipliers() {
    use mip_codex::solve::{Limits, solve};
    for (sense, kind, rhs, range, expected) in [
        ("MAX", "L", 3, "", 5.),
        ("MIN", "G", 1, "", 5.),
        ("MIN", "L", 5, "RANGES\n rng r 4\n", 5.),
    ] {
        let text = format!(
            "NAME grid\nOBJSENSE\n {sense}\nROWS\n N obj\n {kind} r\nCOLUMNS\n x obj 2 r 2\n y obj 2 r 2\nRHS\n rhs obj -3 r {rhs}\n{range}BOUNDS\n BV b x\n BV b y\nENDATA\n"
        );
        let m = mps::parse(std::io::Cursor::new(text)).unwrap();
        let result = solve(&m, &optimization_limits());
        assert_eq!(result.status, "node_limit");
        assert_eq!(result.dual_bound, Some(expected), "{sense} {kind} {range}");
        assert!(result.objective_bound_stats.valid > 0);
        let full = solve(
            &m,
            &Limits {
                nodes: usize::MAX,
                ..optimization_limits()
            },
        );
        assert_eq!(full.status, "optimal");
        assert_eq!(full.objective, Some(expected));
    }
}
#[test]
fn unsupported_bounds_and_propagation_disable_additional_rounding() {
    use mip_codex::solve::{Limits, solve};
    let text = "NAME grid\nROWS\n N obj\n L r\nCOLUMNS\n x obj -2 r 2\n y obj -2 r 2\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n";
    let m = mps::parse(std::io::Cursor::new(text)).unwrap();
    let p = solve(
        &m,
        &Limits {
            propagation: true,
            ..optimization_limits()
        },
    );
    assert!(p.objective_grid.is_none());
    let uncertain = mps::parse(std::io::Cursor::new(
        text.replace(" BV b x", " LI b x 0.1\n UP b x 1"),
    ))
    .unwrap();
    let result = solve(&uncertain, &optimization_limits());
    assert!(result.objective_grid.is_none());
    assert_eq!(result.objective_bound_stats.calls, 0);
}
#[test]
fn certified_pruning_preserves_exhaustive_min_max_optima_with_offsets_and_gcds() {
    use mip_codex::solve::{Limits, solve};
    let mut seed = 44521u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed
    };
    let mut rounded = 0;
    for case in 0..160 {
        let step = 1 + (next() % 4) as i64;
        let offset = (next() % 11) as i64 - 5;
        let cost: Vec<i64> = (0..6).map(|_| ((next() % 9) as i64 - 4) * step).collect();
        let a: Vec<Vec<i64>> = (0..5)
            .map(|_| (0..6).map(|_| (next() % 9) as i64 - 4).collect())
            .collect();
        let plant: Vec<i64> = (0..6).map(|_| (next() % 2) as i64).collect();
        let rhs: Vec<i64> = a
            .iter()
            .map(|a| {
                if case % 2 == 0 {
                    a.iter().zip(&plant).map(|(a, x)| a * x).sum::<i64>() + (next() % 4) as i64
                } else {
                    (next() % 11) as i64 - 5
                }
            })
            .collect();
        let maximize = case % 3 == 0;
        let mut text = format!(
            "NAME grid\nOBJSENSE\n {}\nROWS\n N obj\n",
            if maximize { "MAX" } else { "MIN" }
        );
        for i in 0..5 {
            text += &format!(" L r{i}\n");
        }
        text += "COLUMNS\n";
        for j in 0..6 {
            text += &format!(" x{j} obj {}\n", cost[j]);
            for i in 0..5 {
                text += &format!(" x{j} r{i} {}\n", a[i][j]);
            }
        }
        text += &format!("RHS\n rhs obj {}\n", -offset);
        for i in 0..5 {
            text += &format!(" rhs r{i} {}\n", rhs[i]);
        }
        text += "BOUNDS\n";
        for j in 0..6 {
            text += &format!(" BV b x{j}\n");
        }
        text += "ENDATA\n";
        let model = mps::parse(std::io::Cursor::new(text)).unwrap();
        let mut expected: Option<i64> = None;
        for mask in 0..64 {
            let x: Vec<i64> = (0..6).map(|j| (mask >> j) & 1).collect();
            if a.iter()
                .zip(&rhs)
                .all(|(a, b)| a.iter().zip(&x).map(|(a, x)| a * x).sum::<i64>() <= *b)
            {
                let value = offset + cost.iter().zip(&x).map(|(c, x)| c * x).sum::<i64>();
                expected = Some(expected.map_or(value, |old| {
                    if maximize {
                        old.max(value)
                    } else {
                        old.min(value)
                    }
                }));
            }
        }
        let result = solve(
            &model,
            &Limits {
                nodes: usize::MAX,
                ..optimization_limits()
            },
        );
        rounded += result.objective_bound_stats.rounded;
        match expected {
            None => assert_eq!(result.status, "infeasible", "case{case}"),
            Some(value) => {
                assert_eq!(result.status, "optimal", "case{case}");
                assert!(
                    (result.objective.unwrap() - value as f64).abs() < 1e-6,
                    "case{case}"
                );
            }
        }
    }
    assert!(
        rounded > 40,
        "too few actual bound strengthenings: {rounded}"
    );
}
#[test]
fn original_free_and_lossy_ranged_rows_receive_correct_multipliers() {
    use mip_codex::solve::{Limits, solve};
    let free = "NAME free\nROWS\n N obj\n E r\nCOLUMNS\n x obj 1 r 1\nRHS\n rhs r -2\nBOUNDS\n FR b x\nENDATA\n";
    let m = mps::parse(std::io::Cursor::new(free)).unwrap();
    let result = solve(
        &m,
        &Limits {
            nodes: usize::MAX,
            ..optimization_limits()
        },
    );
    assert_eq!(result.status, "optimal");
    assert_eq!(result.objective, Some(-2.));
    assert_eq!(result.objective_bound_stats.calls, 0);
    assert_eq!(result.objective_grid.unwrap().implied.len(), 1);
    let ranged = "NAME ranged\nROWS\n N obj\n E r\nCOLUMNS\n x obj 2 r 1\nRHS\n rhs r 10000000000000001\nRANGES\n rng r -9999999999999999.5\nBOUNDS\n LI b x 0\nENDATA\n";
    let m = mps::parse(std::io::Cursor::new(ranged)).unwrap();
    let result = solve(
        &m,
        &Limits {
            nodes: usize::MAX,
            ..optimization_limits()
        },
    );
    assert_eq!(result.status, "optimal");
    assert_eq!(result.objective, Some(4.));
    assert!(result.objective_bound_stats.valid > 0);
}
#[test]
fn first_positive_objective_gap_and_complemented_maximum_are_proved_from_costs() {
    use mip_codex::objective_bound::Grid;
    let text = "NAME gap\nROWS\n N obj\n G r\nCOLUMNS\n x obj 100 r 1\n y obj 51 r 1\nRHS\n rhs r 0.1\nBOUNDS\n BV b x\n BV b y\nENDATA\n";
    let m = mps::parse(std::io::Cursor::new(text)).unwrap();
    let g = Grid::from_model(&m).unwrap();
    assert_eq!(g.round_lower(0., false), Some(0.));
    assert_eq!(g.round_lower(1e-100, false), Some(51.));
    assert_eq!(g.round_lower(-120.4, true), Some(-100.));
    let result = mip_codex::solve::solve(&m, &optimization_limits());
    assert_eq!(result.dual_bound, Some(51.));
    let max_text = text
        .replace("NAME gap", "NAME gap\nOBJSENSE\n MAX")
        .replace(" G r", " L r")
        .replace("rhs r 0.1", "rhs r 1.4");
    let max = mps::parse(std::io::Cursor::new(max_text)).unwrap();
    assert_eq!(
        mip_codex::solve::solve(&max, &optimization_limits()).dual_bound,
        Some(100.)
    );
}
#[test]
fn certified_positive_gap_rejects_tolerance_only_zero_and_prunes_its_box() {
    use mip_codex::solve::{Limits, solve};
    let text = "NAME tiny\nROWS\n N obj\n G r\nCOLUMNS\n x obj 1 r 1\nRHS\n rhs r 1e-16\nBOUNDS\n LI b x 0\nENDATA\n";
    let m = mps::parse(std::io::Cursor::new(text)).unwrap();
    let result = solve(
        &m,
        &Limits {
            nodes: usize::MAX,
            ..optimization_limits()
        },
    );
    assert_eq!(result.status, "optimal");
    assert_eq!(result.objective, Some(1.));
    assert_eq!(result.dual_bound, Some(1.));
    assert!(result.objective_bound_stats.box_pruned > 0);
}
#[test]
fn objective_gap_rounding_cannot_skip_any_feasible_integer_box_value() {
    use mip_codex::objective_bound::Grid;
    let mut seed = 22903u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed
    };
    for _ in 0..160 {
        let costs: Vec<i64> = (0..4).map(|_| (next() % 15) as i64 - 7).collect();
        if costs.iter().all(|&c| c == 0) {
            continue;
        }
        let offset = (next() % 11) as i64 - 5;
        let mut text = String::from("NAME domain\nROWS\n N obj\nCOLUMNS\n");
        for j in 0..4 {
            text += &format!(" x{j} obj {}\n", costs[j]);
        }
        text += &format!("RHS\n rhs obj {}\nBOUNDS\n", -offset);
        for j in 0..4 {
            text += &format!(" LI b x{j} -2\n UP b x{j} 3\n");
        }
        text += "ENDATA\n";
        let m = mps::parse(std::io::Cursor::new(text)).unwrap();
        let grid = Grid::from_model(&m).unwrap();
        for code in 0..1296 {
            let mut n = code;
            let x: Vec<i64> = (0..4)
                .map(|_| {
                    let z = (n % 6) as i64 - 2;
                    n /= 6;
                    z
                })
                .collect();
            let value = offset + costs.iter().zip(&x).map(|(c, x)| c * x).sum::<i64>();
            for maximize in [false, true] {
                let signed = if maximize { -value } else { value };
                let bound = grid.round_lower(signed as f64 - 0.3, maximize).unwrap();
                assert!(bound <= signed as f64);
            }
        }
    }
}
