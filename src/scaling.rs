//! LP列・行の単位を2の冪で正規化し、元のProblemへ戻して検査する。
use crate::{
    lp::Outcome,
    revised::{self, Answer, Problem, Stats, WarmStart},
};
use std::time::Instant;
fn error() -> Answer {
    Answer {
        outcome: Outcome::NumericalError,
        dual: None,
        stats: Stats::default(),
        warm: None,
        witness: None,
        witness_point: None,
    }
}
pub(crate) fn power(value: f64) -> f64 {
    let bits = value.to_bits();
    let exponent = bits & (0x7ffu64 << 52);
    if exponent != 0 {
        f64::from_bits(exponent)
    } else {
        f64::from_bits(1u64 << (63 - bits.leading_zeros()))
    }
}
fn scale(p: &Problem) -> Result<Option<(Problem, Vec<f64>, Vec<f64>)>, ()> {
    let factors: Vec<_> = p
        .columns
        .iter()
        .map(|c| {
            let largest = c.iter().map(|(_, a)| a.abs()).fold(0., f64::max);
            if largest > 0. && !(1e-4..=1e4).contains(&largest) {
                power(largest)
            } else {
                1.
            }
        })
        .collect();
    let mut row_max = vec![0.0f64; p.rhs.len()];
    for (j, col) in p.columns.iter().enumerate() {
        for &(i, a) in col {
            row_max[i] = row_max[i].max((a / factors[j]).abs());
        }
    }
    let rows: Vec<_> = row_max
        .into_iter()
        .map(|a| {
            if a > 0. && !(1e-4..=1e4).contains(&a) {
                power(a)
            } else {
                1.
            }
        })
        .collect();
    if factors.iter().chain(&rows).all(|s| *s == 1.) {
        return Ok(None);
    }
    let mut q = p.clone();
    for (j, &s) in factors.iter().enumerate() {
        for ((i, a), (_, original)) in q.columns[j].iter_mut().zip(&p.columns[j]) {
            let col_value = *a / s;
            if !col_value.is_finite() || col_value * s != *original {
                return Err(());
            }
            *a = col_value / rows[*i];
            if !a.is_finite() || *a * rows[*i] != col_value {
                return Err(());
            }
        }
        q.costs[j] /= s;
        if !q.costs[j].is_finite() || q.costs[j] * s != p.costs[j] {
            return Err(());
        }
        if p.upper[j].is_finite() {
            q.upper[j] *= s;
            if !q.upper[j].is_finite() || q.upper[j] / s != p.upper[j] {
                return Err(());
            }
        }
    }
    for (i, &r) in rows.iter().enumerate() {
        q.rhs[i] /= r;
        if !q.rhs[i].is_finite() || q.rhs[i] * r != p.rhs[i] {
            return Err(());
        }
        if p.slack_upper[i].is_finite() {
            q.slack_upper[i] /= r;
            if !q.slack_upper[i].is_finite() || q.slack_upper[i] * r != p.slack_upper[i] {
                return Err(());
            }
        }
    }
    Ok(Some((q, factors, rows)))
}
// 外向き丸め。厳密にゼロの積以外はunderflowした積も区間に含める。
fn dot_bounds(terms: impl Iterator<Item = (f64, f64)>) -> (f64, f64) {
    let (mut lo, mut hi) = (0.0f64, 0.0f64);
    for (a, b) in terms {
        if a == 0. || b == 0. {
            continue;
        }
        let t = a * b;
        lo = (lo + t.next_down()).next_down();
        hi = (hi + t.next_up()).next_up();
    }
    (lo, hi)
}
fn contradiction(p: &Problem, y: &[f64]) -> bool {
    if y.len() != p.rhs.len() || y.iter().any(|v| !v.is_finite()) {
        return false;
    }
    let lower = dot_bounds(y.iter().copied().zip(p.rhs.iter().copied())).0;
    let mut support = 0.0f64;
    for (j, col) in p.columns.iter().enumerate() {
        let alpha = dot_bounds(col.iter().map(|&(i, a)| (a, y[i]))).1;
        if !alpha.is_finite() {
            return false;
        }
        if alpha > 0. {
            if !p.upper[j].is_finite() {
                if crate::exact::dot_sign(col.iter().map(|&(i, a)| (a, y[i])))
                    .is_some_and(|sign| sign != std::cmp::Ordering::Greater)
                {
                    continue;
                }
                return false;
            }
            if p.upper[j] != 0. {
                support = (support + (alpha * p.upper[j]).next_up()).next_up();
            }
        }
    }
    for (i, &v) in y.iter().enumerate() {
        if !p.equality[i] && v > 0. {
            if !p.slack_upper[i].is_finite() {
                return false;
            }
            if p.slack_upper[i] != 0. {
                support = (support + (v * p.slack_upper[i]).next_up()).next_up();
            }
        }
    }
    lower > support
}
fn point_valid(p: &Problem, x: &[f64]) -> bool {
    if x.len() != p.costs.len() || x.iter().any(|v| !v.is_finite()) {
        return false;
    }
    let mut residual = p.rhs.clone();
    for (j, col) in p.columns.iter().enumerate() {
        if x[j] < -1e-7 || x[j] > p.upper[j] + 1e-7 {
            return false;
        }
        for &(i, a) in col {
            residual[i] -= a * x[j];
        }
    }
    for (i, &v) in residual.iter().enumerate() {
        if !v.is_finite()
            || if p.equality[i] {
                v.abs() > 1e-6
            } else {
                v < -1e-6 || v > p.slack_upper[i] + 1e-6
            }
        {
            return false;
        }
    }
    true
}
fn primal_dual(p: &Problem, x: &[f64], y: &[f64]) -> Option<(f64, f64)> {
    if !point_valid(p, x) || y.len() != p.rhs.len() || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let primal: f64 = p.costs.iter().zip(x).map(|(c, v)| c * v).sum();
    let mut dual: f64 = y.iter().zip(&p.rhs).map(|(a, b)| a * b).sum();
    for (j, col) in p.columns.iter().enumerate() {
        let r = p.costs[j] - col.iter().map(|&(i, a)| a * y[i]).sum::<f64>();
        if !r.is_finite() {
            return None;
        }
        if p.upper[j].is_finite() {
            dual += r.min(0.) * p.upper[j];
        } else if r < -1e-7 {
            return None;
        }
    }
    for (i, &v) in y.iter().enumerate() {
        if !p.equality[i] {
            if p.slack_upper[i].is_finite() {
                dual += (-v).min(0.) * p.slack_upper[i];
            } else if v > 1e-7 {
                return None;
            }
        }
    }
    if !primal.is_finite()
        || !dual.is_finite()
        || (primal - dual).abs() > 1e-6 * (1. + primal.abs())
    {
        None
    } else {
        Some((primal, dual.min(primal)))
    }
}
fn ray_valid(p: &Problem, ray: &[f64]) -> bool {
    if ray.len() != p.costs.len() || ray.iter().any(|v| !v.is_finite()) {
        return false;
    }
    let mut rows = vec![vec![]; p.rhs.len()];
    for (j, col) in p.columns.iter().enumerate() {
        if ray[j] < 0. || (p.upper[j].is_finite() && ray[j] != 0.) {
            return false;
        }
        if ray[j] == 0. {
            continue;
        }
        for &(i, a) in col {
            rows[i].push((a, ray[j]));
        }
    }
    for (i, row) in rows.into_iter().enumerate() {
        let Some(sign) = crate::exact::dot_sign(row.into_iter()) else {
            return false;
        };
        if p.equality[i] || p.slack_upper[i].is_finite() {
            if sign != std::cmp::Ordering::Equal {
                return false;
            }
        } else if sign == std::cmp::Ordering::Greater {
            return false;
        }
    }
    crate::exact::dot_sign(p.costs.iter().copied().zip(ray.iter().copied()))
        == Some(std::cmp::Ordering::Less)
}
pub(crate) fn solve(
    p: &Problem,
    limit: usize,
    deadline: Option<Instant>,
    warm: Option<&WarmStart>,
    retain: bool,
) -> Answer {
    if !revised::valid(p) {
        return error();
    }
    let normalized = match scale(p) {
        Ok(q) => q,
        Err(()) => return error(),
    };
    let Some((q, factors, rows)) = normalized else {
        let mut answer = revised::raw(p, limit, deadline, warm, retain);
        if matches!(answer.outcome, Outcome::Unbounded)
            && !(answer
                .witness
                .as_deref()
                .is_some_and(|ray| ray_valid(p, ray))
                && answer
                    .witness_point
                    .as_deref()
                    .is_some_and(|x| point_valid(p, x)))
        {
            answer.outcome = Outcome::NumericalError;
            answer.dual = None;
            answer.warm = None;
            answer.witness = None;
            answer.witness_point = None;
        }
        return answer;
    };
    let mut answer = revised::raw(&q, limit, deadline, warm, retain);
    answer.stats.scaled_columns += factors.iter().filter(|s| **s != 1.).count();
    answer.stats.scaled_rows += rows.iter().filter(|r| **r != 1.).count();
    if matches!(
        answer.outcome,
        Outcome::Optimal { .. } | Outcome::Infeasible
    ) {
        if let Some(y) = answer.witness.as_mut() {
            for (v, &r) in y.iter_mut().zip(&rows) {
                *v /= r;
            }
        }
    }
    let valid = match &mut answer.outcome {
        Outcome::Optimal { objective, x } => {
            for (v, &s) in x.iter_mut().zip(&factors) {
                *v /= s;
            }
            if let Some((obj, dual)) = answer.witness.as_deref().and_then(|y| primal_dual(p, x, y))
            {
                *objective = obj;
                answer.dual = Some(dual);
                true
            } else {
                false
            }
        }
        Outcome::Infeasible => answer.witness.as_deref().is_some_and(|y| {
            contradiction(p, y) || contradiction(p, &y.iter().map(|v| -v).collect::<Vec<_>>())
        }),
        Outcome::Unbounded => {
            let base = if let Some(point) = answer.witness_point.as_mut() {
                for (v, &s) in point.iter_mut().zip(&factors) {
                    *v /= s;
                }
                point_valid(p, point)
            } else {
                false
            };
            if let Some(ray) = answer.witness.as_mut() {
                for (v, &s) in ray.iter_mut().zip(&factors) {
                    *v /= s;
                }
                base && ray_valid(p, ray)
            } else {
                false
            }
        }
        _ => true,
    };
    if !valid {
        answer.outcome = Outcome::NumericalError;
        answer.dual = None;
        answer.warm = None;
        answer.witness = None;
        answer.witness_point = None;
    }
    answer
}
