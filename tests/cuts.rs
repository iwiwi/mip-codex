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
        assert!(!cuts.is_empty());
        assert!(cuts.iter().all(|cut| verify(&model, cut)));
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
    model.rows[0].lower_enclosure = None;
    model.rows[0].upper_enclosure = None;
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
        gomory_cuts: false,
        box_primal: false,
        binary_graph: false,
        objective_rounding: false,
        rins: false,
        strong_branching: false,
        cover_cuts: false,
        clique_cuts: false,
        mir_cuts: false,
        nodes: 1,
        primal_repair: false,
        feasibility_pump: false,
        lp_diving: false,
        primal_probe: false,
        ..Limits::default()
    };
    let before = solve(&model, &limits);
    let after = solve(
        &model,
        &Limits {
            box_primal: false,
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            strong_branching: false,
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
            box_primal: false,
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            strong_branching: false,
            nodes: 1,
            cover_cuts: true,
            primal_repair: false,
            feasibility_pump: false,
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
#[test]
fn fractional_coefficients_project_a_continuous_lower_bound_safely() {
    use mip_codex::solve::{Limits, solve};
    let model=mps::parse(std::io::Cursor::new("NAME mixed\nROWS\n N c\n L r\nCOLUMNS\n x c -1 r 0.2\n y c -1 r 0.2\n z r 1\nRHS\n rhs r 0.4\nBOUNDS\n BV b x\n BV b y\n LO b z 0.1\n UP b z 1\nENDATA\n")).unwrap();
    let result = solve(
        &model,
        &Limits {
            box_primal: false,
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            strong_branching: false,
            nodes: 1,
            cover_cuts: true,
            primal_probe: false,
            primal_repair: false,
            feasibility_pump: false,
            lp_diving: false,
            ..Limits::default()
        },
    );
    assert_eq!(result.status, "optimal");
    assert_eq!(result.objective, Some(-1.));
    assert!(result.cut_stats.interval > 0 && result.cut_stats.projected > 0);
    assert!(result.cut_proofs.iter().all(|cut| verify(&model, cut)));
    // x=1,y=0,z=.1は元の制約を満たし、単独変数を排除するcutは不可。
    assert!(result.cut_proofs.iter().all(|cut| {
        cut.row
            .coefficients
            .iter()
            .filter(|(j, _)| *j == 0)
            .map(|(_, a)| a)
            .sum::<f64>()
            <= cut.row.upper.unwrap()
    }));
}
#[test]
fn a_negative_continuous_term_needs_its_finite_upper_bound() {
    let model=mps::parse(std::io::Cursor::new("NAME upper\nROWS\n N c\n L r\nCOLUMNS\n x r 0.2\n y r 0.2\n z r -1\nRHS\n rhs r 0.1\nBOUNDS\n BV b x\n BV b y\n UP b z 0.2\nENDATA\n")).unwrap();
    let mut pool = Pool::default();
    let cuts = pool.separate(
        &model,
        &[1., 0.5, 0.2],
        32,
        Instant::now() + Duration::from_secs(1),
    );
    assert!(!cuts.is_empty());
    assert!(cuts.iter().all(|cut| verify(&model, cut)));
    let mut unbounded = model.clone();
    unbounded.variables[2].upper = None;
    unbounded.variables[2].upper_enclosure = None;
    assert!(!verify(&unbounded, &cuts[0]));
}
#[test]
fn mixed_decimal_cuts_preserve_all_box_feasible_binary_assignments() {
    let mut seed = 2026111515u64;
    let mut checked = 0;
    for case in 0..400 {
        let mut draw = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            (seed >> 32) as i64
        };
        let weights: Vec<_> = (0..8).map(|_| draw() % 19 - 9).collect();
        let rhs = draw() % 21 - 10;
        let sign = if case % 2 == 0 { 1 } else { -1 };
        let mut text = format!(
            "NAME mixed\nROWS\n N c\n {} r\nCOLUMNS\n",
            if sign == 1 { "L" } else { "G" }
        );
        for (j, a) in weights.iter().enumerate() {
            text += &format!(" x{j} c 0 r {}\n", *a as f64 / 10.);
        }
        text += &format!("RHS\n rhs r {}\nBOUNDS\n", rhs as f64 / 10.);
        for j in 0..6 {
            text += &format!(" BV b x{j}\n");
        }
        text += " LO b x6 -0.2\n UP b x6 0.7\n LO b x7 -0.6\n UP b x7 1.3\nENDATA\n";
        let model = mps::parse(std::io::Cursor::new(text)).unwrap();
        let mut point: Vec<_> = (0..6).map(|_| (draw() % 1000) as f64 / 1000.).collect();
        point.extend([0.2, 0.1]);
        let cuts =
            Pool::default().separate(&model, &point, 32, Instant::now() + Duration::from_secs(1));
        for cut in cuts {
            assert!(verify(&model, &cut));
            checked += 1;
            for mask in 0..64 {
                let mut minimum = 10
                    * (0..6)
                        .filter(|j| mask & (1 << j) != 0)
                        .map(|j| sign * weights[j])
                        .sum::<i64>();
                minimum += (sign * weights[6] * -2).min(sign * weights[6] * 7)
                    + (sign * weights[7] * -6).min(sign * weights[7] * 13);
                if minimum <= sign * 10 * rhs {
                    let activity: f64 = cut
                        .row
                        .coefficients
                        .iter()
                        .filter(|(j, _)| mask & (1 << j) != 0)
                        .map(|(_, a)| a)
                        .sum();
                    assert!(activity <= cut.row.upper.unwrap(), "case{case} {cut:?}");
                }
            }
        }
    }
    assert!(checked > 20, "only {checked} cuts");
}
#[test]
fn ranges_are_combined_as_original_decimal_values_before_rounding() {
    let model=mps::parse(std::io::Cursor::new("NAME ranged\nROWS\n N c\n E r\nCOLUMNS\n x c 1 r 1\nRHS\n rhs r 10000000000000001\nRANGES\n range r -10000000000000000\nENDATA\n")).unwrap();
    assert_eq!(model.rows[0].lower, Some(1.));
    let interval = model.rows[0].lower_enclosure.unwrap();
    assert!(interval[0] <= 1. && interval[1] >= 1.);
    let result = mip_codex::solve::solve(&model, &mip_codex::solve::Limits::default());
    assert_eq!(result.status, "optimal");
    assert_eq!(result.objective, Some(1.));
}
#[test]
fn root_cut_incumbent_does_not_replace_the_fractional_global_bound() {
    use mip_codex::solve::{Limits, solve};
    let model=mps::parse(std::io::Cursor::new("NAME triangle\nROWS\n N c\n L r\nCOLUMNS\n x c -1 r 2\n y c -1 r 2\n z c -1 r 2\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\n BV b z\nENDATA\n")).unwrap();
    for (rounds, status, bound) in [(2, "node_limit", -1.5), (4, "optimal", -1.)] {
        let result = solve(
            &model,
            &Limits {
                box_primal: false,
                binary_graph: false,
                objective_rounding: false,
                rins: false,
                strong_branching: false,
                nodes: 1,
                cover_cuts: true,
                cover_rounds: rounds,
                mir_cuts: false,
                primal_probe: false,
                primal_repair: false,
                feasibility_pump: false,
                lp_diving: true,
                ..Limits::default()
            },
        );
        assert_eq!(result.status, status);
        assert_eq!(result.objective, Some(-1.));
        assert_eq!(result.dual_bound, Some(bound));
        assert!(!result.cut_proofs.is_empty());
        assert!(result.cut_proofs.iter().all(|c| verify(&model, c)));
    }
}

#[test]
fn mir_alone_strengthens_a_mixed_integer_root_and_has_an_original_row_proof() {
    use mip_codex::solve::{Limits, solve};
    let model=mps::parse(std::io::Cursor::new("NAME mixed\nROWS\n N c\n L r\nCOLUMNS\n x c -1 r 3\n t c 1 r -1\nRHS\n rhs r 1\nBOUNDS\n LI b x 0\nENDATA\n")).unwrap();
    let limits = Limits {
        gomory_cuts: false,
        cover_cuts: false,
        clique_cuts: false,
        aggregate_cuts: false,
        rounding_cuts: false,
        mir_cuts: false,
        box_primal: false,
        binary_graph: false,
        objective_rounding: false,
        rins: false,
        strong_branching: false,
        primal_repair: false,
        feasibility_pump: false,
        lp_diving: false,
        primal_probe: false,
        nodes: 1,
        ..Limits::default()
    };
    let before = solve(&model, &limits);
    let after = solve(
        &model,
        &Limits {
            mir_cuts: true,
            ..limits
        },
    );
    assert!(before.dual_bound.unwrap() < -0.3);
    assert_eq!(after.status, "optimal");
    assert_eq!(after.objective, Some(0.));
    assert_eq!(after.dual_bound, Some(0.));
    assert!(after.cut_proofs.iter().any(|c| c.proof.mir.is_some()));
    assert!(after.cut_proofs.iter().all(|c| verify(&model, c)));
}

fn mir_only_limits() -> mip_codex::solve::Limits {
    use mip_codex::solve::Limits;
    Limits {
        cover_cuts: false,
        clique_cuts: false,
        aggregate_cuts: false,
        rounding_cuts: false,
        mir_cuts: true,
        box_primal: false,
        binary_graph: false,
        objective_rounding: false,
        rins: false,
        strong_branching: false,
        primal_repair: false,
        feasibility_pump: false,
        lp_diving: false,
        primal_probe: false,
        nodes: 1,
        ..Limits::default()
    }
}
#[test]
fn mir_trial_can_finish_zero_objective_feasibility() {
    use mip_codex::solve::{Limits, solve};
    let m=mps::parse(std::io::Cursor::new("NAME feasibility\nROWS\n N c\n E r\nCOLUMNS\n x r 3\n t r 3\nRHS\n rhs r 1\nBOUNDS\n LI b x 0\nENDATA\n")).unwrap();
    let limits = mir_only_limits();
    let before = solve(
        &m,
        &Limits {
            mir_cuts: false,
            ..limits.clone()
        },
    );
    assert_eq!(before.status, "node_limit");
    assert_eq!(before.objective, None);
    let after = solve(&m, &limits);
    assert_eq!(after.status, "optimal");
    assert_eq!(after.objective, Some(0.));
    assert_eq!(after.dual_bound, Some(0.));
    assert!(after.cut_stats.mir_added > 0);
    assert!(after.cut_proofs.iter().all(|c| verify(&m, c)));
}
#[test]
fn rejected_mir_trials_preserve_the_original_root_bound_and_status() {
    use mip_codex::solve::{Limits, solve};
    for (coefficient, rhs) in [("1", "0.5"), ("2", "1")] {
        let text = format!(
            "NAME rollback\nROWS\n N c\n E r\n E s\nCOLUMNS\n x r 3\n t r 3\n z s {coefficient}\nRHS\n rhs r 1 s {rhs}\nBOUNDS\n LI b x 0\n BV b z\nENDATA\n"
        );
        let m = mps::parse(std::io::Cursor::new(text)).unwrap();
        let limits = mir_only_limits();
        let before = solve(
            &m,
            &Limits {
                mir_cuts: false,
                ..limits.clone()
            },
        );
        let after = solve(&m, &limits);
        assert_eq!(before.status, "node_limit");
        assert_eq!(after.status, before.status);
        assert_eq!(after.objective, before.objective);
        assert_eq!(after.dual_bound, before.dual_bound);
        assert!(after.cut_proofs.is_empty());
        assert_eq!(after.cut_stats.mir_added, 0);
        assert!(after.cut_stats.mir_lp_solves > 0);
        assert!(after.cut_stats.mir_rejected_cuts > 0);
        assert!(after.root_lp_reused);
    }
}
