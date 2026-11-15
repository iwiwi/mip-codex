use mip_codex::{
    cuts::{Pool, verify},
    model::Model,
    mps,
    solve::{Limits, solve},
};
use std::time::{Duration, Instant};
fn model(a: &[i64], rhs: i64, side: &str) -> Model {
    let mut text = format!("NAME rounding\nROWS\n N obj\n {side} r\nCOLUMNS\n");
    for (j, v) in a.iter().enumerate() {
        text += &format!(" x{j} obj -1 r {v}\n");
    }
    text += &format!("RHS\n rhs r {rhs}\nBOUNDS\n");
    for j in 0..a.len() {
        text += &format!(" LI b x{j} 0\n UP b x{j} 3\n");
    }
    text += "ENDATA\n";
    mps::parse(std::io::Cursor::new(text)).unwrap()
}
fn separate(m: &Model, x: &[f64]) -> Vec<mip_codex::cuts::Cut> {
    Pool::default().separate_rounding(m, x, 32, Instant::now() + Duration::from_secs(1))
}
#[test]
fn division_closes_a_root_that_pairwise_covers_cannot_close() {
    let m = model(&[2, 2, 2], 3, "L");
    let limits = Limits {
        nodes: 1,
        cover_cuts: false,
        primal_repair: false,
        lp_diving: false,
        primal_probe: false,
        ..Limits::default()
    };
    let before = solve(&m, &limits);
    let after = solve(
        &m,
        &Limits {
            rounding_cuts: true,
            ..limits
        },
    );
    assert_eq!(before.status, "node_limit");
    assert_eq!(before.dual_bound, Some(-1.5));
    assert_eq!(after.status, "optimal");
    assert_eq!(after.objective, Some(-1.));
    assert_eq!(after.dual_bound, Some(-1.));
    assert!(after.cut_stats.rounding > 0);
}
#[test]
fn negative_coefficients_and_rhs_use_floor_not_truncation() {
    let m = model(&[-3, 2], -1, "L");
    let cuts = separate(&m, &[1. / 3., 0.]);
    let cut = cuts.iter().find(|c| c.proof.divisor == Some(2)).unwrap();
    assert_eq!(cut.row.coefficients, vec![(0, -2.), (1, 1.)]);
    assert_eq!(cut.row.upper, Some(-1.));
    assert!(verify(&m, cut));
    let mut bad = cut.clone();
    bad.row.coefficients[0].1 = -1.;
    assert!(!verify(&m, &bad));
    let mut bad = cut.clone();
    bad.proof.divisor = Some(0);
    assert!(!verify(&m, &bad));
}
#[test]
fn lower_side_is_oriented_before_division() {
    let m = model(&[-2, -2], -3, "G");
    let cuts = separate(&m, &[0.75, 0.75]);
    let c = cuts.iter().find(|c| c.proof.divisor == Some(2)).unwrap();
    assert!(!c.proof.upper_side);
    assert_eq!(c.row.coefficients, vec![(0, 1.), (1, 1.)]);
    assert_eq!(c.row.upper, Some(1.));
    assert!(verify(&m, c));
}
#[test]
fn unknown_noninteger_continuous_and_negative_domains_are_rejected() {
    let m = model(&[2, 2], 3, "L");
    for kind in 0..4 {
        let mut changed = m.clone();
        match kind {
            0 => changed.rows[0].exact_integral = false,
            1 => changed.variables[0].integer = false,
            2 => changed.variables[0].lower_enclosure = None,
            _ => {
                changed.variables[0].lower = Some(-1.);
                changed.variables[0].lower_enclosure = Some([-1., -1.]);
            }
        };
        assert!(separate(&changed, &[0.75, 0.75]).is_empty());
    }
}
#[test]
fn all_small_nonnegative_integer_points_preserved_by_random_rows() {
    let mut seed = 911u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed
    };
    let mut count = 0;
    for _ in 0..250 {
        let a: Vec<i64> = (0..5).map(|_| (next() % 19) as i64 - 9).collect();
        let rhs = (next() % 41) as i64 - 20;
        let side = if next() % 2 == 0 { "L" } else { "G" };
        let m = model(&a, rhs, side);
        let x: Vec<f64> = (0..5).map(|_| (next() % 301) as f64 / 100.).collect();
        let cuts = separate(&m, &x);
        count += cuts.len();
        for cut in cuts {
            assert!(verify(&m, &cut));
            for code in 0..1024 {
                let mut n = code;
                let point: Vec<i64> = (0..5)
                    .map(|_| {
                        let z = n % 4;
                        n /= 4;
                        z
                    })
                    .collect();
                let original: i64 = a.iter().zip(&point).map(|(a, x)| a * x).sum();
                if (side == "L" && original <= rhs) || (side == "G" && original >= rhs) {
                    let activity: i64 = cut
                        .row
                        .coefficients
                        .iter()
                        .map(|&(j, c)| c as i64 * point[j])
                        .sum();
                    assert!(
                        activity <= cut.row.upper.unwrap() as i64,
                        "{a:?} {rhs}, {point:?}, {cut:?}"
                    );
                }
            }
        }
    }
    assert!(
        count > 50,
        "insufficient independently checked cuts: {count}"
    );
}
#[test]
fn continuous_terms_project_from_the_correct_original_endpoint() {
    for (a, rhs, lo, up, z) in [(1, 4, 1, 2, 1.), (-1, 2, 0, 1, 1.)] {
        let mut m = model(&[2, 2, a], rhs, "L");
        m.variables[2].integer = false;
        m.variables[2].lower = Some(lo as f64);
        m.variables[2].upper = Some(up as f64);
        m.variables[2].lower_enclosure = Some([lo as f64; 2]);
        m.variables[2].upper_enclosure = Some([up as f64; 2]);
        let cuts = separate(&m, &[0.75, 0.75, z]);
        let c = cuts.iter().find(|c| c.proof.divisor == Some(2)).unwrap();
        assert_eq!(c.row.coefficients, vec![(0, 1.), (1, 1.)]);
        assert_eq!(c.row.upper, Some(1.));
        assert!(verify(&m, c));
        let mut unknown = m.clone();
        if a > 0 {
            unknown.variables[2].lower_enclosure = None;
        } else {
            unknown.variables[2].upper_enclosure = None;
        }
        assert!(!verify(&unknown, c));
    }
}
#[test]
fn projected_random_rows_preserve_every_integer_point_with_a_continuous_box() {
    let mut seed = 337u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed
    };
    let mut count = 0;
    for _ in 0..250 {
        let a: Vec<i64> = (0..5).map(|_| (next() % 19) as i64 - 9).collect();
        let rhs = (next() % 41) as i64 - 20;
        let side = if next() % 2 == 0 { "L" } else { "G" };
        let mut m = model(&a, rhs, side);
        m.variables[4].integer = false;
        m.variables[4].lower = Some(-2.);
        m.variables[4].upper = Some(3.);
        m.variables[4].lower_enclosure = Some([-2., -2.]);
        m.variables[4].upper_enclosure = Some([3., 3.]);
        let x: Vec<f64> = (0..5).map(|_| (next() % 301) as f64 / 100.).collect();
        let cuts = separate(&m, &x);
        count += cuts.len();
        for cut in cuts {
            assert!(verify(&m, &cut));
            for code in 0..256 {
                let mut n = code;
                let point: Vec<i64> = (0..4)
                    .map(|_| {
                        let z = n % 4;
                        n /= 4;
                        z
                    })
                    .collect();
                let binary: i64 = a[..4].iter().zip(&point).map(|(a, x)| a * x).sum();
                let feasible = if side == "L" {
                    binary + (a[4] * (-2)).min(a[4] * 3) <= rhs
                } else {
                    binary + (a[4] * (-2)).max(a[4] * 3) >= rhs
                };
                if feasible {
                    let activity: i64 = cut
                        .row
                        .coefficients
                        .iter()
                        .map(|&(j, c)| c as i64 * point[j])
                        .sum();
                    assert!(activity <= cut.row.upper.unwrap() as i64);
                }
            }
        }
    }
    assert!(count > 20, "insufficient projected cuts: {count}");
}
