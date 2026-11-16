use mip_codex::{cuts, mps};
use std::{
    io::Cursor,
    time::{Duration, Instant},
};
fn input(coefficient: &str, rhs: &str, range: &str) -> String {
    format!(
        "NAME decimal\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1 r {coefficient}\n y r -0.1\nRHS\n rhs r {rhs}\n{range}BOUNDS\n LI b x 0\n UP b x 3\n LO b y 0\n UP b y 3\nENDATA\n"
    )
}
#[test]
fn original_decimal_row_is_unchanged_and_scaled_mir_is_verified() {
    let text = input("0.3", "0.2", "");
    let a = mps::parse(Cursor::new(&text)).unwrap();
    let b = mps::parse_with_decimal_scaling(Cursor::new(&text)).unwrap();
    assert_eq!(a.rows[0].decimal_scale, None);
    assert_eq!(b.rows[0].decimal_scale, Some(10));
    assert!(!b.rows[0].exact_integral);
    let mut original = serde_json::to_value(&b.rows[0]).unwrap();
    original.as_object_mut().unwrap().remove("decimal_scale");
    assert_eq!(original, serde_json::to_value(&a.rows[0]).unwrap());
    let point = [2. / 3., 0.];
    assert!(
        cuts::Pool::default()
            .separate_mir(&a, &point, 8, Instant::now() + Duration::from_secs(1))
            .is_empty()
    );
    let generated =
        cuts::Pool::default().separate_mir(&b, &point, 8, Instant::now() + Duration::from_secs(1));
    assert!(!generated.is_empty());
    for c in generated {
        assert_eq!(c.proof.mir.as_ref().unwrap().row_scale, 10);
        assert!(cuts::verify(&b, &c));
        for x in 0..=3 {
            for y in 0..=30 {
                if 30 * x - y <= 20 {
                    let p = [x as f64, y as f64 / 10.];
                    assert!(
                        c.row
                            .coefficients
                            .iter()
                            .map(|&(j, a)| a * p[j])
                            .sum::<f64>()
                            <= c.row.upper.unwrap() + 1e-10
                    );
                }
            }
        }
        for bad in [0, 3, 100, u64::MAX] {
            let mut corrupt = c.clone();
            corrupt.proof.mir.as_mut().unwrap().row_scale = bad;
            assert!(!cuts::verify(&b, &corrupt));
        }
    }
}
#[test]
fn ranges_duplicates_precision_and_large_values_are_checked_from_tokens() {
    let text = input("0.3", "0.2", "RANGES\n rng r -0.1\n")
        .replace(" x obj -1 r 0.3", " x obj -1 r 0.1\n x r 0.2");
    let model = mps::parse_with_decimal_scaling(Cursor::new(text)).unwrap();
    assert_eq!(model.rows[0].decimal_scale, Some(10));
    assert_eq!(
        (model.rows[0].lower, model.rows[0].upper),
        (Some(0.1), Some(0.2))
    );
    for (coefficient, rhs) in [
        ("0.3000000000000000001", "0.2"),
        ("3.0000000000000000001", "0.2"),
        ("0.3", "0.2000000000000000001"),
        ("-17014118346046923173168730371588410572.8", "0.2"),
    ] {
        let m = mps::parse_with_decimal_scaling(Cursor::new(input(coefficient, rhs, ""))).unwrap();
        assert_eq!(m.rows[0].decimal_scale, None);
    }
}
