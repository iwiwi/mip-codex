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
