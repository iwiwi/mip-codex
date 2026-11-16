//! Candidate integer row multipliers from continued fractions.
//! These routines never certify anything: the LP's outward verifier must accept.
const MAX_INTEGER: u128 = 1 << 52;
fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}
fn approximate(value: f64, max_den: u128) -> Option<(u128, u128)> {
    if !value.is_finite() || value < 0. || value > MAX_INTEGER as f64 || max_den == 0 {
        return None;
    }
    let (mut p0, mut q0, mut p1, mut q1) = (0u128, 1u128, 1u128, 0u128);
    let mut x = value;
    for _ in 0..64 {
        let whole = x.floor();
        if q1 != 0 && whole > ((max_den - q0) / q1) as f64 {
            let k = (max_den - q0) / q1;
            let pn = p0.checked_add(k.checked_mul(p1)?)?;
            let qn = q0 + k * q1;
            let old_error = (p1 as f64 / q1 as f64 - value).abs();
            let new_error = (pn as f64 / qn as f64 - value).abs();
            return Some(if new_error < old_error {
                (pn, qn)
            } else {
                (p1, q1)
            });
        }
        let a = whole as u128;
        let p2 = a.checked_mul(p1)?.checked_add(p0)?;
        let q2 = a.checked_mul(q1)?.checked_add(q0)?;
        (p0, q0, p1, q1) = (p1, q1, p2, q2);
        let fraction = x - whole;
        if fraction == 0. {
            return Some((p1, q1));
        }
        x = 1. / fraction;
        if !x.is_finite() {
            return Some((p1, q1));
        }
    }
    (q1 > 0).then_some((p1, q1))
}
pub(crate) fn integer_weights(weights: &[f64], max_den: u128) -> Option<Vec<f64>> {
    if weights.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let positive = weights
        .iter()
        .copied()
        .filter(|v| *v > 0.)
        .fold(0., f64::max);
    let negative = weights
        .iter()
        .copied()
        .filter(|v| *v < 0.)
        .map(f64::abs)
        .fold(0., f64::max);
    let anchor = match (positive > 0., negative > 0.) {
        (true, true) => positive.min(negative),
        (true, false) => positive,
        (false, true) => negative,
        _ => return None,
    };
    let fractions: Vec<_> = weights
        .iter()
        .map(|v| approximate(v.abs() / anchor, max_den))
        .collect::<Option<_>>()?;
    let mut denominator = 1u128;
    for &(_, q) in &fractions {
        denominator = (denominator / gcd(denominator, q)).checked_mul(q)?;
        if denominator > MAX_INTEGER {
            return None;
        }
    }
    weights
        .iter()
        .zip(fractions)
        .map(|(v, (p, q))| {
            let integer = p.checked_mul(denominator / q)?;
            if integer > MAX_INTEGER {
                return None;
            }
            Some(if v.is_sign_negative() {
                -(integer as f64)
            } else {
                integer as f64
            })
        })
        .collect()
}
/// Drops small components only to propose a different rational certificate.
/// The caller must still verify every inequality in the original Problem.
pub(crate) fn clipped_integer_weights(weights: &[f64], max_den: u128) -> Option<Vec<f64>> {
    if max_den == 0 || weights.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let largest = weights.iter().map(|v| v.abs()).fold(0., f64::max);
    if largest == 0. {
        return None;
    }
    let threshold = largest / (2. * max_den as f64);
    let clipped: Vec<_> = weights
        .iter()
        .map(|&v| if v.abs() < threshold { 0. } else { v })
        .collect();
    integer_weights(&clipped, max_den)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clipped_candidates_remove_small_opposite_sign_noise() {
        assert_eq!(
            clipped_integer_weights(&[1. / 27., 1., -1e-13], 32),
            Some(vec![1., 27., 0.])
        );
        assert!(clipped_integer_weights(&[0., 0.], 32).is_none());
        assert!(clipped_integer_weights(&[1., f64::NAN], 32).is_none());
        assert!(clipped_integer_weights(&[1.], 0).is_none());
    }
    #[test]
    fn ratios_clear_roundoff_without_trusting_the_candidate() {
        assert_eq!(
            integer_weights(
                &[-1., -0.4, -0.5, 8388608., 4194304.000000001, 3355443.2],
                16
            )
            .unwrap(),
            vec![-10., -4., -5., 83886080., 41943040., 33554432.]
        );
        assert_eq!(
            integer_weights(&[2., 4., 6.], 16).unwrap(),
            vec![1., 2., 3.]
        );
        assert_eq!(
            integer_weights(&[-2., -4., -6.], 16).unwrap(),
            vec![-1., -2., -3.]
        );
    }
    #[test]
    fn continued_fractions_match_independent_denominator_enumeration() {
        let mut seed = 87391u64;
        for _ in 0..1000 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let value = (seed % 100000) as f64 / 10000.;
            let cap = (seed % 64 + 1) as u128;
            let (p, q) = approximate(value, cap).unwrap();
            assert!(q >= 1 && q <= cap);
            let best = (1..=cap)
                .map(|d| ((value * d as f64).round() / d as f64 - value).abs())
                .fold(f64::INFINITY, f64::min);
            assert!(
                (p as f64 / q as f64 - value).abs() <= best + 1e-14,
                "{value} {cap} {p}/{q} {best}"
            );
        }
    }
    #[test]
    fn unsupported_inputs_and_excessive_integer_sizes_fail_closed() {
        assert!(integer_weights(&[0., 0.], 16).is_none());
        assert!(integer_weights(&[f64::NAN, 1.], 16).is_none());
        assert!(integer_weights(&[f64::INFINITY, -1.], 16).is_none());
        assert!(integer_weights(&[1e300, -1.], 16).is_none());
        assert!(
            integer_weights(
                &[
                    1.,
                    1. / 53.,
                    1. / 59.,
                    1. / 61.,
                    1. / 67.,
                    1. / 71.,
                    1. / 73.,
                    1. / 79.,
                    1. / 83.,
                    1. / 89.
                ],
                65536
            )
            .is_none()
        );
    }
}
