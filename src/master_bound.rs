//! 原行の任意の乗数と元の外側変数箱から、表示専用の下界を別証明する。
use crate::{model::Model, objective_bound::Certifier};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Instant};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proof {
    pub multipliers: Vec<(usize, String)>,
    pub normalized_bound_bits: String,
}
impl Proof {
    pub fn bound(&self) -> Option<f64> {
        let x = f64::from_bits(u64::from_str_radix(&self.normalized_bound_bits, 16).ok()?);
        x.is_finite().then_some(x)
    }
}
pub(crate) fn certify(
    model: &Model,
    sources: &[(usize, f64)],
    weights: &[f64],
    memory: usize,
    deadline: Instant,
    trials: &mut usize,
) -> Option<Proof> {
    if sources.len() != weights.len() || Instant::now() >= deadline {
        return None;
    }
    let mut combined = BTreeMap::<usize, f64>::new();
    for (&(i, s), &y) in sources.iter().zip(weights) {
        if i >= model.rows.len() || !(s == 1. || s == -1.) || !y.is_finite() {
            return None;
        }
        *combined.entry(i).or_default() += s * y;
    }
    combined.retain(|i, y| {
        *y != 0.
            && if *y > 0. {
                model.rows[*i].lower.is_some()
            } else {
                model.rows[*i].upper.is_some()
            }
    });
    if combined.is_empty() || combined.values().any(|v| !v.is_finite()) {
        return None;
    }
    let entries = model.variables.len() as u128
        + combined.len() as u128
        + combined
            .keys()
            .map(|&i| model.rows[i].coefficients.len() as u128)
            .sum::<u128>();
    if entries * 128 > memory as u128 || Instant::now() >= deadline {
        return None;
    }
    let subset = Model {
        name: String::new(),
        maximize: model.maximize,
        offset: model.offset,
        offset_enclosure: model.offset_enclosure,
        decimal_source: None,
        original_decimal: model.original_decimal,
        variables: model.variables.clone(),
        rows: combined.keys().map(|&i| model.rows[i].clone()).collect(),
    };
    let lower: Vec<_> = subset
        .variables
        .iter()
        .map(|v| v.lower_enclosure.map(|x| x[0]))
        .collect();
    let upper: Vec<_> = subset
        .variables
        .iter()
        .map(|v| v.upper_enclosure.map(|x| x[1]))
        .collect();
    let cert = Certifier::new(&subset)?;
    let raw: Vec<_> = combined.values().copied().collect();
    let mut best = None;
    let mut besty = vec![];
    for factor in [1., 1. - 2f64.powi(-40), 1. + 2f64.powi(-40)] {
        if Instant::now() >= deadline {
            break;
        }
        let candidate: Vec<_> = raw.iter().map(|v| v * factor).collect();
        *trials += 1;
        if let Some(bound) = cert.bound(&lower, &upper, &candidate) {
            if bound.is_finite() && best.is_none_or(|b| bound > b) {
                best = Some(bound);
                besty = candidate;
            }
        }
    }
    let bound = best?;
    if Instant::now() >= deadline {
        return None;
    }
    let multipliers: Vec<_> = combined
        .keys()
        .copied()
        .zip(besty)
        .filter(|(_, v)| *v != 0.)
        .map(|(i, v)| (i, format!("{:016x}", v.to_bits())))
        .collect();
    if multipliers.is_empty() {
        return None;
    }
    Some(Proof {
        multipliers,
        normalized_bound_bits: format!("{:016x}", bound.to_bits()),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_bounds_costs_and_offset_certify_min_and_max() {
        for maximize in [false, true] {
            let text = format!(
                "NAME cert\nOBJSENSE\n {}\nROWS\n N c\n G need\nCOLUMNS\n x c {} need 1\n y c {} need 1\nRHS\n rhs need 1 c -3\nBOUNDS\n BV b x\n BV b y\nENDATA\n",
                if maximize { "MAX" } else { "MIN" },
                if maximize { -1 } else { 1 },
                if maximize { -1 } else { 1 }
            );
            let m = crate::mps::parse(std::io::Cursor::new(text)).unwrap();
            let mut count = 0;
            let proof = certify(
                &m,
                &[(0, -1.)],
                &[-1.],
                1 << 20,
                Instant::now() + std::time::Duration::from_secs(1),
                &mut count,
            )
            .unwrap();
            let expected = if maximize { -2. } else { 4. };
            let b = proof.bound().unwrap();
            assert!(b <= expected && b > expected - 1e-8);
            assert!(!proof.multipliers.is_empty());
            assert_eq!(count, 3);
        }
    }
    #[test]
    fn arbitrary_approximate_multipliers_need_a_finite_original_box_bound() {
        let m = crate::mps::parse(std::io::Cursor::new(
            "NAME free\nROWS\n N c\n L cap\nCOLUMNS\n x c -1 cap 1\nRHS\n rhs cap 1\nENDATA\n",
        ))
        .unwrap();
        let end = Instant::now() + std::time::Duration::from_secs(1);
        assert!(certify(&m, &[(0, 1.)], &[-0.5], 1 << 20, end, &mut 0).is_none());
        let p = certify(&m, &[(0, 1.)], &[-1.], 1 << 20, end, &mut 0).unwrap();
        assert!(p.bound().unwrap() <= -1.);
        assert!(certify(&m, &[(0, 1.)], &[1.], 1 << 20, end, &mut 0).is_none());
        assert!(certify(&m, &[(1, 1.)], &[-1.], 1 << 20, end, &mut 0).is_none());
        assert!(certify(&m, &[(0, 1.)], &[f64::NAN], 1 << 20, end, &mut 0).is_none());
        assert!(certify(&m, &[(0, 1.)], &[-1.], 1 << 20, Instant::now(), &mut 0).is_none());
        assert!(certify(&m, &[(0, 1.)], &[-1.], 1, end, &mut 0).is_none());
    }
}
