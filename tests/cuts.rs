use mip_codex::{
    cuts::{Pool, verify},
    mps,
};
use std::time::{Duration, Instant};
fn read(coeff: &str, rhs: &str) -> mip_codex::model::Model {
    mps::parse(std::io::Cursor::new(format!("NAME cover\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1 r {coeff}\n y obj -1 r 2\nRHS\n rhs r {rhs}\nBOUNDS\n BV b x\n BV b y\nENDATA\n"))).unwrap()
}
#[test]
fn literal_integer_provenance_does_not_trust_f64_rounding() {
    for (a, b, expected) in [
        ("2", "3", true),
        ("2.000D+0", "3000e-3", true),
        ("2.00000000000000000000001", "3", false),
        ("2", "3.00000000000000000000001", false),
    ] {
        let model = read(a, b);
        assert_eq!(model.rows[0].exact_integral, expected);
        let cuts = Pool::default().separate(
            &model,
            &[1., 0.5],
            32,
            Instant::now() + Duration::from_secs(1),
        );
        assert_eq!(!cuts.is_empty(), expected);
    }
    for value in ["1e-400", "-1D-9999"] {
        let text = format!("NAME underflow\nROWS\n N obj\nCOLUMNS\n x obj {value}\nENDATA\n");
        assert!(
            mps::parse(std::io::Cursor::new(text))
                .unwrap_err()
                .contains("underflow")
        );
    }
    let zero = read("0D-9999", "3");
    assert!(zero.rows[0].exact_integral);
}
#[test]
fn signed_cover_is_valid_and_rejects_a_changed_certificate() {
    let model=mps::parse(std::io::Cursor::new("NAME signed\nROWS\n N c\n G r\nCOLUMNS\n x r -2\n y r 2\nRHS\n rhs r -1\nBOUNDS\n BV b x\n BV b y\nENDATA\n")).unwrap();
    let cuts = Pool::default().separate(
        &model,
        &[1., 0.5],
        32,
        Instant::now() + Duration::from_secs(1),
    );
    assert_eq!(cuts.len(), 1);
    assert!(verify(&model, &cuts[0]));
    assert_eq!(cuts[0].row.coefficients, vec![(0, 1.), (1, -1.)]);
    assert_eq!(cuts[0].row.upper, Some(0.));
    let mut bad = cuts[0].clone();
    bad.row.upper = Some(-1.);
    assert!(!verify(&model, &bad));
    let mut bad = cuts[0].clone();
    bad.proof.cover.push(1);
    assert!(!verify(&model, &bad));
}
#[test]
fn no_cover_excludes_any_binary_feasible_point() {
    let mut seed = 2026111514u64;
    for _ in 0..250 {
        let mut draw = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            (seed >> 32) as i64
        };
        let weights: Vec<_> = (0..8).map(|_| draw() % 19 - 9).collect();
        let rhs = draw() % 20 - 5;
        let mut text = String::from("NAME enum\nROWS\n N c\n L r\nCOLUMNS\n");
        for (j, a) in weights.iter().enumerate() {
            text += &format!(" x{j} r {a}\n");
        }
        text += &format!("RHS\n rhs r {rhs}\nBOUNDS\n");
        for j in 0..8 {
            text += &format!(" BV b x{j}\n");
        }
        text += "ENDATA\n";
        let model = mps::parse(std::io::Cursor::new(text)).unwrap();
        let point: Vec<_> = (0..8).map(|_| (draw() % 1000) as f64 / 1000.).collect();
        let cuts =
            Pool::default().separate(&model, &point, 32, Instant::now() + Duration::from_secs(1));
        for cut in cuts {
            assert!(verify(&model, &cut));
            for mask in 0..256 {
                let lhs: i64 = (0..8)
                    .filter(|j| mask & (1 << j) != 0)
                    .map(|j| weights[j])
                    .sum();
                if lhs <= rhs {
                    let activity: f64 = cut
                        .row
                        .coefficients
                        .iter()
                        .filter(|(j, _)| mask & (1 << j) != 0)
                        .map(|(_, a)| a)
                        .sum();
                    assert!(
                        activity <= cut.row.upper.unwrap(),
                        "{weights:?} {rhs} {mask} {cut:?}"
                    );
                }
            }
        }
    }
}
#[test]
fn continuous_columns_and_unknown_provenance_are_skipped() {
    let mut model = read("2", "3");
    model.variables[0].integer = false;
    assert!(
        Pool::default()
            .separate(
                &model,
                &[1., 0.5],
                32,
                Instant::now() + Duration::from_secs(1)
            )
            .is_empty()
    );
    model.variables[0].integer = true;
    model.rows[0].exact_integral = false;
    assert!(
        Pool::default()
            .separate(
                &model,
                &[1., 0.5],
                32,
                Instant::now() + Duration::from_secs(1)
            )
            .is_empty()
    );
}
#[test]
fn cover_changes_root_bound_without_excluding_integer_optimum() {
    use mip_codex::solve::{Limits, solve};
    let model = read("2", "3");
    let limits = Limits {
        nodes: 1,
        primal_repair: false,
        lp_diving: false,
        primal_probe: false,
        ..Limits::default()
    };
    let before = solve(&model, &limits);
    let after = solve(
        &model,
        &Limits {
            cover_cuts: true,
            ..limits
        },
    );
    assert_eq!(before.status, "node_limit");
    assert_eq!(before.dual_bound, Some(-1.5));
    assert_eq!(after.status, "optimal");
    assert_eq!(after.objective, Some(-1.));
    assert_eq!(after.dual_bound, Some(-1.));
    assert_eq!(after.nodes, 1);
    assert!(after.cut_lp_solves >= 2);
    assert!(!after.cut_proofs.is_empty());
    assert!(after.cut_proofs.iter().all(|cut| verify(&model, cut)));
}
#[test]
fn ranged_lower_side_cuts_keep_the_maximization_offset() {
    use mip_codex::solve::{Limits, solve};
    let model=mps::parse(std::io::Cursor::new("NAME range\nOBJSENSE MAX\nROWS\n N c\n L r\nCOLUMNS\n x c -1 r 2\n y c -1 r 2\nRHS\n rhs c -7 r 5\nRANGES\n rg r 2\nBOUNDS\n BV b x\n BV b y\nENDATA\n")).unwrap();
    let result = solve(
        &model,
        &Limits {
            nodes: 1,
            cover_cuts: true,
            primal_repair: false,
            lp_diving: false,
            primal_probe: false,
            ..Limits::default()
        },
    );
    assert_eq!(result.status, "optimal");
    assert_eq!(result.objective, Some(5.));
    assert_eq!(result.dual_bound, Some(5.));
    assert!(result.cut_proofs.len() >= 2);
    assert!(result.cut_proofs.iter().all(|cut| verify(&model, cut)));
}
#[test]
fn duplicate_terms_are_summed_before_rounding_in_rows_and_objective() {
    use mip_codex::solve::{Limits, solve};
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        for objective in [false, true] {
            let values = if objective {
                [-10000000000000000i64, -1, 10000000000000000]
            } else {
                [10000000000000000i64, 1, -10000000000000000]
            };
            let mut text = String::from("NAME duplicates\nROWS\n N obj\n L r\nCOLUMNS\n");
            if !objective {
                text += " x obj -1\n";
            }
            for k in order {
                text += &format!(" x {} {}\n", if objective { "obj" } else { "r" }, values[k]);
            }
            text += "RHS\n rhs r 1\nBOUNDS\n UP b x 2\nENDATA\n";
            let model = mps::parse(std::io::Cursor::new(text)).unwrap();
            let result = solve(&model, &Limits::default());
            assert_eq!(result.status, "optimal");
            assert_eq!(result.objective, Some(if objective { -2. } else { -1. }));
        }
    }
}
#[test]
fn decimal_duplicate_cancellation_is_not_a_spurious_unbounded_objective() {
    use mip_codex::solve::{Limits, solve};
    let model = mps::parse(std::io::Cursor::new(
        "NAME decimal\nROWS\n N c\nCOLUMNS\n x c -0.1\n x c -0.2\n x c 0.3\nENDATA\n",
    ))
    .unwrap();
    let result = solve(&model, &Limits::default());
    assert_eq!(model.variables[0].objective, 0.);
    assert_eq!(result.status, "optimal");
    assert_eq!(result.objective, Some(0.));
    let model=mps::parse(std::io::Cursor::new("NAME decimal\nROWS\n N c\n L r\nCOLUMNS\n x c -1.000000000000000000001 r 1.000000000000000000001\n x c 1 r -1\nRHS\n rhs r 1\nENDATA\n")).unwrap();
    assert_eq!(model.variables[0].objective, -1e-21);
    assert_eq!(model.rows[0].coefficients, vec![(0, 1e-21)]);
}
