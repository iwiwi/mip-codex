//! 有限f64の積和の符号。各値を整数mantissa×2^exponentとして累算する。
use std::cmp::Ordering;
const WORDS: usize = 68; // 64-bit usize個のf64積でも4352bitで全指数域とcarryを保持。
struct Sum {
    positive: [u64; WORDS],
    negative: [u64; WORDS],
}
impl Default for Sum {
    fn default() -> Self {
        Self {
            positive: [0; WORDS],
            negative: [0; WORDS],
        }
    }
}
fn parts(value: f64) -> Option<(bool, u64, i32)> {
    let bits = value.to_bits();
    let e = ((bits >> 52) & 0x7ff) as i32;
    if e == 0x7ff {
        return None;
    }
    let fraction = bits & ((1u64 << 52) - 1);
    Some((
        bits >> 63 != 0,
        if e == 0 {
            fraction
        } else {
            fraction | (1u64 << 52)
        },
        if e == 0 { -1074 } else { e - 1075 },
    ))
}
fn word(sum: &mut [u64; WORDS], mut index: usize, mut value: u64) -> Option<()> {
    while value != 0 {
        let slot = sum.get_mut(index)?;
        let (next, carry) = slot.overflowing_add(value);
        *slot = next;
        value = u64::from(carry);
        index += 1;
    }
    Some(())
}
impl Sum {
    fn product(&mut self, a: f64, b: f64) -> Option<()> {
        let (sa, ma, ea) = parts(a)?;
        let (sb, mb, eb) = parts(b)?;
        if ma == 0 || mb == 0 {
            return Some(());
        }
        let product = (ma as u128) * (mb as u128);
        let shift = (ea + eb + 2148) as usize;
        let index = shift / 64;
        let offset = shift % 64;
        let sum = if sa ^ sb {
            &mut self.negative
        } else {
            &mut self.positive
        };
        let lo = product as u64;
        let hi = (product >> 64) as u64;
        word(sum, index, lo << offset)?;
        word(sum, index + 1, hi << offset)?;
        if offset != 0 {
            word(sum, index + 1, lo >> (64 - offset))?;
            word(sum, index + 2, hi >> (64 - offset))?;
        }
        Some(())
    }
    fn sign(&self) -> Ordering {
        self.positive.iter().rev().cmp(self.negative.iter().rev())
    }
}
// 最後の一度だけ最近接・偶数丸め。中間のoverflowや相殺で項を失わない。
impl Sum {
    fn rounded(&self) -> Option<f64> {
        let (large, small, negative) = match self.sign() {
            Ordering::Equal => return Some(0.),
            Ordering::Greater => (&self.positive, &self.negative, false),
            Ordering::Less => (&self.negative, &self.positive, true),
        };
        let mut magnitude = [0u64; WORDS];
        let mut borrow = false;
        for i in 0..WORDS {
            let (a, b) = large[i].overflowing_sub(small[i]);
            let (a, c) = a.overflowing_sub(u64::from(borrow));
            magnitude[i] = a;
            borrow = b || c;
        }
        if borrow {
            return None;
        }
        let last = magnitude.iter().rposition(|&w| w != 0)?;
        let mut top = last * 64 + 63 - magnitude[last].leading_zeros() as usize;
        let normal = top >= 1126; // -1022 + 2148
        let shift = if normal { top - 52 } else { 1074 }; // 最小subnormalの単位
        let bit = |i: usize| {
            magnitude
                .get(i / 64)
                .is_some_and(|w| (w >> (i % 64)) & 1 != 0)
        };
        let mut mantissa = 0u64;
        for k in 0..53 {
            if bit(shift + k) {
                mantissa |= 1u64 << k;
            }
        }
        let guard = shift - 1;
        let sticky = magnitude[..guard / 64].iter().any(|&w| w != 0)
            || magnitude[guard / 64] & ((1u64 << (guard % 64)) - 1) != 0;
        if bit(guard) && (sticky || mantissa & 1 != 0) {
            mantissa += 1;
        }
        let sign = u64::from(negative) << 63;
        if !normal {
            return Some(f64::from_bits(sign | mantissa));
        }
        if mantissa == 1u64 << 53 {
            mantissa >>= 1;
            top += 1;
        }
        let exponent = top as i32 - 2148 + 1023;
        if exponent >= 2047 {
            return None;
        }
        Some(f64::from_bits(
            sign | ((exponent as u64) << 52) | (mantissa - (1u64 << 52)),
        ))
    }
}
/// 有限f64積和の正確な符号と一度丸めから、隣接値による包含区間を作る。
pub(crate) fn dot_enclosure(terms: impl Iterator<Item = (f64, f64)>) -> Option<[f64; 2]> {
    let mut sum = Sum::default();
    for (a, b) in terms {
        sum.product(a, b)?;
    }
    let sign = sum.sign();
    if sign == Ordering::Equal {
        return Some([0., 0.]);
    }
    let value = sum.rounded()?;
    let lo = if sign == Ordering::Greater {
        value.next_down().max(0.)
    } else {
        value.next_down()
    };
    let hi = if sign == Ordering::Less {
        value.next_up().min(0.)
    } else {
        value.next_up()
    };
    (lo.is_finite() && hi.is_finite()).then_some([lo, hi])
}
#[cfg(test)]
pub(crate) fn sum(terms: impl Iterator<Item = f64>) -> Option<f64> {
    let mut total = Sum::default();
    for x in terms {
        total.product(x, 1.)?;
    }
    total.rounded()
}
pub(crate) fn dot_sign(terms: impl Iterator<Item = (f64, f64)>) -> Option<Ordering> {
    let mut sum = Sum::default();
    for (a, b) in terms {
        sum.product(a, b)?;
    }
    Some(sum.sign())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_preserves_smallest_products_and_overflowing_products() {
        let tiny = f64::from_bits(1);
        assert_eq!(
            dot_sign([(f64::MAX, f64::MAX), (f64::MAX, -f64::MAX)].into_iter()),
            Some(Ordering::Equal)
        );
        assert_eq!(
            dot_sign([(f64::MAX, f64::MAX), (f64::MAX, -f64::MAX), (tiny, tiny)].into_iter()),
            Some(Ordering::Greater)
        );
        assert_eq!(
            dot_sign([(1., 1.), (tiny, 1.), (-1., 1.)].into_iter()),
            Some(Ordering::Greater)
        );
        assert_eq!(dot_sign([(tiny, -tiny)].into_iter()), Some(Ordering::Less));
    }
    #[test]
    fn nonfinite_values_are_not_certified() {
        assert_eq!(dot_sign([(f64::INFINITY, 0.)].into_iter()), None);
        assert_eq!(dot_sign([(f64::NAN, 1.)].into_iter()), None);
    }
    #[test]
    fn correctly_rounded_sums_match_python_fraction() {
        #[derive(serde::Deserialize)]
        struct Case {
            terms: Vec<u64>,
            expected: Option<u64>,
        }
        let cases: Vec<Case> =
            serde_json::from_str(include_str!("../tests/fixtures/exact-sum.json")).unwrap();
        for (i, case) in cases.into_iter().enumerate() {
            assert_eq!(
                sum(case.terms.into_iter().map(f64::from_bits)).map(f64::to_bits),
                case.expected,
                "case{i}"
            );
        }
    }
    #[test]
    fn arbitrary_exponents_match_independent_python_fraction_fixtures() {
        #[derive(serde::Deserialize)]
        struct Case {
            terms: Vec<[u64; 2]>,
            sign: i8,
        }
        let cases: Vec<Case> =
            serde_json::from_str(include_str!("../tests/fixtures/exact-dot-sign.json")).unwrap();
        for (i, case) in cases.into_iter().enumerate() {
            let actual = dot_sign(
                case.terms
                    .into_iter()
                    .map(|[a, b]| (f64::from_bits(a), f64::from_bits(b))),
            );
            assert_eq!(actual, Some(case.sign.cmp(&0)), "fixture {i}");
        }
    }
}

#[cfg(test)]
mod enclosure_tests {
    use super::*;
    #[test]
    fn exact_zero_and_products_below_smallest_subnormal_are_enclosed() {
        let tiny = f64::from_bits(1);
        assert_eq!(dot_enclosure([(tiny, tiny)].into_iter()), Some([0., tiny]));
        assert_eq!(
            dot_enclosure([(-tiny, tiny)].into_iter()),
            Some([-tiny, 0.])
        );
        assert_eq!(
            dot_enclosure([(f64::MAX, f64::MAX), (-f64::MAX, f64::MAX)].into_iter()),
            Some([0., 0.])
        );
        assert_eq!(dot_enclosure([(f64::MAX, f64::MAX)].into_iter()), None);
    }
}

enum Accumulated {
    Point(f64),
    Wide(Sum),
}
fn accumulated(terms: impl Iterator<Item = (f64, f64)>) -> Option<Accumulated> {
    let mut point = 0.;
    let mut wide: Option<Sum> = None;
    for (a, b) in terms {
        if a == 0. || b == 0. {
            if !a.is_finite() || !b.is_finite() {
                return None;
            }
            continue;
        }
        if let Some(sum) = &mut wide {
            sum.product(a, b)?;
            continue;
        }
        let p = a * b;
        let exact_product = p.is_finite()
            && (a.abs() == 1.
                || b.abs() == 1.
                || (a.fract() == 0.
                    && b.fract() == 0.
                    && a.abs() <= (1u64 << 52) as f64
                    && b.abs() <= (1u64 << 52) as f64
                    && p.abs() <= (1u64 << 52) as f64));
        if exact_product {
            let next = point + p;
            let z = next - point;
            let residue = (point - (next - z)) + (p - z);
            if next.is_finite() && residue == 0. {
                point = next;
                continue;
            }
        }
        // TwoSumで無損失と証明できたprefixだけpointに保持し、以後は元の積を累算する。
        let mut sum = Sum::default();
        sum.product(point, 1.)?;
        sum.product(a, b)?;
        wide = Some(sum);
    }
    Some(wide.map_or(Accumulated::Point(point), Accumulated::Wide))
}
/// 各積も丸めずに累算し、最後の一度だけ最近接へ丸める。
pub(crate) fn dot(terms: impl Iterator<Item = (f64, f64)>) -> Option<f64> {
    match accumulated(terms)? {
        Accumulated::Point(v) => Some(v),
        Accumulated::Wide(s) => s.rounded(),
    }
}
/// f64で表せる最も狭い上下界。表現可能な積和はpointになる。
pub(crate) fn dot_bounds(terms: impl Iterator<Item = (f64, f64)>) -> Option<[f64; 2]> {
    let mut sum = match accumulated(terms)? {
        Accumulated::Point(v) => return Some([v, v]),
        Accumulated::Wide(s) => s,
    };
    let value = sum.rounded()?;
    sum.product(-value, 1.)?;
    let direction = sum.sign();
    let lo = if direction == Ordering::Less {
        value.next_down()
    } else {
        value
    };
    let hi = if direction == Ordering::Greater {
        value.next_up()
    } else {
        value
    };
    (lo.is_finite() && hi.is_finite()).then_some([lo, hi])
}
pub(crate) fn dot_lower(terms: impl Iterator<Item = (f64, f64)>) -> Option<f64> {
    dot_bounds(terms).map(|b| b[0])
}
/// 最近接値と、丸め残差の絶対値の上界。
pub(crate) fn dot_error(terms: impl Iterator<Item = (f64, f64)>) -> Option<(f64, f64)> {
    let mut sum = match accumulated(terms)? {
        Accumulated::Point(v) => return Some((v, 0.)),
        Accumulated::Wide(s) => s,
    };
    let value = sum.rounded()?;
    sum.product(-value, 1.)?;
    let error = if sum.sign() == Ordering::Equal {
        0.
    } else {
        sum.rounded()?.abs().next_up()
    };
    error.is_finite().then_some((value, error))
}
/// 通常の有限dot用の保守的な前方誤差区間。不確か/overflowなら厳密累算へ戻す。
pub(crate) fn fast_dot_bounds(terms: impl Iterator<Item = (f64, f64)>) -> Option<[f64; 2]> {
    let mut value = 0.;
    let mut magnitude = 0.;
    let mut count = 0usize;
    for (a, b) in terms {
        let p = a * b;
        value += p;
        magnitude += p.abs();
        count = count.checked_add(1)?;
    }
    let n = count as f64;
    if !value.is_finite() || !magnitude.is_finite() || n * f64::EPSILON > 1e-4 {
        return None;
    }
    let error = (8. * f64::EPSILON * (n + 1.) * magnitude + 4. * n * f64::from_bits(1)).next_up();
    let lo = (value - error).next_down();
    let hi = (value + error).next_up();
    (lo.is_finite() && hi.is_finite()).then_some([lo, hi])
}

#[cfg(test)]
mod value_tests {
    use super::*;
    #[derive(serde::Deserialize)]
    struct Case {
        terms: Vec<[u64; 2]>,
        nearest: Option<u64>,
        bounds: Option<[u64; 2]>,
        error: Option<u64>,
    }
    #[test]
    fn nearest_tight_bounds_and_fast_enclosures_match_fraction_oracle() {
        let cases: Vec<Case> =
            serde_json::from_str(include_str!("../tests/fixtures/exact-dot-values.json")).unwrap();
        let mut fast = 0;
        for (i, c) in cases.iter().enumerate() {
            let terms = || {
                c.terms
                    .iter()
                    .map(|&[a, b]| (f64::from_bits(a), f64::from_bits(b)))
            };
            assert_eq!(dot(terms()).map(f64::to_bits), c.nearest, "nearest case{i}");
            assert_eq!(
                dot_bounds(terms()).map(|b| b.map(f64::to_bits)),
                c.bounds,
                "bounds case{i}"
            );
            assert_eq!(
                dot_error(terms()).map(|(v, e)| (v.to_bits(), e.to_bits())),
                c.nearest.zip(c.error),
                "error case{i}"
            );
            if let Some(b) = fast_dot_bounds(terms()) {
                fast += 1;
                let expected = c.bounds.unwrap();
                assert!(
                    b[0] <= f64::from_bits(expected[0]) && b[1] >= f64::from_bits(expected[1]),
                    "fast case{i}"
                );
            }
        }
        assert!(fast > 200, "{fast}");
    }
}
