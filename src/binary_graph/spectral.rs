//! Integer spectral certificates. Theory: Haemers (2021), arXiv:2102.05529.
//! LP points suggest weights only; no LP feasibility or optimality is trusted.
use super::{adjacent, Graph};
use crate::{lp, revised};
use std::{collections::BTreeSet, time::Instant};
const SCALE: i64 = 1 << 20;
#[derive(Debug, serde::Serialize)]
pub struct Certificate {
    pub recognized: bool,
    pub vertices: usize,
    pub vertex_labels: Vec<usize>,
    pub weights: Vec<i64>,
    pub degree: i64,
    pub minimum_eigenvalue: i64,
    pub upper: usize,
}
#[derive(Debug, Default, serde::Serialize)]
pub struct Stats {
    pub recognized: bool,
    pub mappings: usize,
    pub lp_solves: usize,
    pub lp_pivots: usize,
    pub lp_refactors: usize,
    pub validation_failures: usize,
    pub seconds: f64,
    pub stop_reason: &'static str,
}
#[derive(Debug, Default, serde::Serialize)]
pub struct Outcome {
    pub certificate: Option<Certificate>,
    pub stats: Stats,
}
fn fwht(a: &mut [i64]) -> Option<()> {
    let mut step = 1;
    while step < a.len() {
        for start in (0..a.len()).step_by(step * 2) {
            for i in start..start + step {
                let (x, y) = (a[i], a[i + step]);
                a[i] = x.checked_add(y)?;
                a[i + step] = x.checked_sub(y)?;
            }
        }
        step *= 2;
    }
    Some(())
}
fn combinations(bits: usize, left: usize, start: usize, mask: usize, out: &mut Vec<usize>) {
    if left == 0 {
        out.push(mask);
        return;
    }
    for bit in start..bits {
        if bits - bit < left {
            break;
        }
        combinations(bits, left - 1, bit + 1, mask | (1 << bit), out);
    }
}
fn certify(
    graph: &Graph,
    labels: Vec<usize>,
    weights: Vec<i64>,
    deadline: Instant,
) -> Option<Certificate> {
    let n = graph.n;
    if !n.is_power_of_two()
        || labels.len() != n
        || weights.len() != n
        || weights[0] != 0
        || weights.iter().any(|w| w.unsigned_abs() > SCALE as u64)
    {
        return None;
    }
    let mut seen = vec![false; n];
    for &v in &labels {
        if v >= n || seen[v] {
            return None;
        }
        seen[v] = true;
    }
    // For any independent set S, its indicator has f^T A f <= 0.
    for a in 0..n {
        if Instant::now() >= deadline {
            return None;
        }
        for b in a + 1..n {
            if adjacent(graph, a, b) && weights[labels[graph.order[a]] ^ labels[graph.order[b]]] > 0
            {
                return None;
            }
        }
    }
    let mut eigenvalues = weights.clone();
    fwht(&mut eigenvalues)?;
    let degree = eigenvalues[0];
    let minimum = *eigenvalues.iter().min()?;
    if minimum >= 0 || degree <= minimum {
        return None;
    }
    let upper = (n as i128 * (-minimum as i128) / (degree as i128 - minimum as i128))
        .try_into()
        .ok()?;
    Some(Certificate {
        recognized: true,
        vertices: n,
        vertex_labels: labels,
        weights,
        degree,
        minimum_eigenvalue: minimum,
        upper,
    })
}
pub(super) fn improve(
    graph: &Graph,
    memory: usize,
    deadline: Instant,
    incumbent: usize,
) -> Outcome {
    let started = Instant::now();
    let mut out = Outcome::default();
    let n = graph.n;
    if !n.is_power_of_two() || n < 2 {
        out.stats.stop_reason = "not_power_two";
        return out;
    }
    let bits = n.trailing_zeros() as usize;
    if n.checked_mul(bits + 8)
        .and_then(|v| v.checked_mul(32))
        .is_none_or(|v| v > memory)
    {
        out.stats.stop_reason = "memory";
        return out;
    }
    let mut inverse = vec![0; n];
    for (v, &original) in graph.order.iter().enumerate() {
        inverse[original] = v;
    }
    for mode in 0..3 {
        if Instant::now() >= deadline {
            out.stats.stop_reason = "deadline";
            break;
        }
        let mut labels: Vec<_> = (0..n).collect();
        if mode == 1 {
            labels.clear();
            for k in 0..=bits {
                combinations(bits, k, 0, 0, &mut labels);
            }
        } else if mode == 2 {
            labels.sort_unstable_by_key(|v| (v.count_ones(), *v));
        }
        out.stats.mappings += 1;
        let mut original_for_label = vec![0; n];
        for (v, &label) in labels.iter().enumerate() {
            original_for_label[label] = v;
        }
        let zero = inverse[original_for_label[0]];
        let mask: Vec<_> = (0..n)
            .map(|d| d != 0 && !adjacent(graph, zero, inverse[original_for_label[d]]))
            .collect();
        let mut shell_values = vec![None; bits + 1];
        let mut symmetric = true;
        for (d, &edge) in mask.iter().enumerate() {
            let k = d.count_ones() as usize;
            if shell_values[k].is_some_and(|old| old != edge) {
                symmetric = false;
                break;
            }
            shell_values[k] = Some(edge);
        }
        if !symmetric || !mask.iter().any(|&v| v) {
            continue;
        }
        let mut invariant = true;
        'check: for a in 0..n {
            if Instant::now() >= deadline {
                invariant = false;
                break;
            }
            for b in a + 1..n {
                let conflict = !adjacent(graph, a, b);
                if conflict != mask[labels[graph.order[a]] ^ labels[graph.order[b]]] {
                    invariant = false;
                    break 'check;
                }
            }
        }
        if !invariant {
            continue;
        }
        out.stats.recognized = true;
        let mut spectra = Vec::new();
        let mut multiplicities = Vec::new();
        for k in 1..=bits {
            let mut a: Vec<i64> = (0..n)
                .map(|d| i64::from(d.count_ones() as usize == k))
                .collect();
            if fwht(&mut a).is_none() {
                out.stats.validation_failures += 1;
                return out;
            }
            multiplicities.push(a[0]);
            spectra.push(a);
        }
        let rows: BTreeSet<Vec<i64>> = (1..n)
            .map(|s| spectra.iter().map(|a| a[s]).collect())
            .collect();
        let rows: Vec<_> = rows.into_iter().collect();
        let spans: Vec<f64> = (1..=bits)
            .map(|k| {
                if shell_values[k] == Some(true) {
                    2.
                } else {
                    1.
                }
            })
            .collect();
        let problem = revised::Problem {
            columns: (0..bits)
                .map(|j| {
                    rows.iter()
                        .enumerate()
                        .filter_map(|(i, row)| {
                            (row[j] != 0).then_some((i, -spans[j] * row[j] as f64))
                        })
                        .collect()
                })
                .collect(),
            rhs: rows
                .iter()
                .map(|row| 1. - row.iter().sum::<i64>() as f64)
                .collect(),
            costs: multiplicities
                .iter()
                .enumerate()
                .map(|(j, &v)| -spans[j] * v as f64)
                .collect(),
            upper: vec![1.; bits],
            equality: vec![false; rows.len()],
            slack_upper: vec![f64::INFINITY; rows.len()],
        };
        out.stats.lp_solves += 1;
        let answer = revised::minimize(&problem, 10000, Some(deadline));
        out.stats.lp_pivots += answer.stats.pivots;
        out.stats.lp_refactors += answer.stats.refactors;
        let x = match answer.outcome {
            lp::Outcome::Optimal { x, .. } => Some(x),
            _ => answer.limited_point,
        };
        let Some(x) = x else {
            out.stats.stop_reason = "no_lp_point";
            break;
        };
        if x.len() != bits || x.iter().any(|v| !v.is_finite()) {
            out.stats.stop_reason = "invalid_lp_point";
            break;
        }
        let shell_weights: Vec<i64> = x
            .iter()
            .enumerate()
            .map(|(j, v)| ((spans[j] * v.clamp(0., 1.) - 1.) * SCALE as f64).round() as i64)
            .collect();
        let weights: Vec<i64> = (0..n)
            .map(|d| {
                if d == 0 {
                    0
                } else {
                    shell_weights[d.count_ones() as usize - 1]
                }
            })
            .collect();
        if let Some(c) = certify(graph, labels, weights, deadline) {
            if c.upper >= incumbent {
                out.certificate = Some(c);
                out.stats.stop_reason = "certified";
            } else {
                out.stats.validation_failures += 1;
                out.stats.stop_reason = "incumbent_conflict";
            }
        } else {
            out.stats.stop_reason = "certificate_unavailable";
        }
        break;
    }
    if out.stats.stop_reason.is_empty() {
        out.stats.stop_reason = "not_recognized";
    }
    out.stats.seconds = started.elapsed().as_secs_f64();
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transform_matches_direct_integer_characters() {
        let mut seed = 73u64;
        for n in [2, 4, 8, 16, 32] {
            for _ in 0..40 {
                let a: Vec<i64> = (0..n)
                    .map(|_| {
                        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                        (seed % 21) as i64 - 10
                    })
                    .collect();
                let expected: Vec<i64> = (0..n)
                    .map(|s| {
                        a.iter()
                            .enumerate()
                            .map(|(d, &v)| if (s & d).count_ones() % 2 == 0 { v } else { -v })
                            .sum()
                    })
                    .collect();
                let mut actual = a;
                fwht(&mut actual).unwrap();
                assert_eq!(actual, expected);
            }
        }
    }
    #[test]
    fn signed_certificates_bound_exhaustive_independent_sets_and_reject_nonedges() {
        let mut seed = 913u64;
        for n in [4usize, 8, 16] {
            for _ in 0..10 {
                let mut weights = vec![0; n];
                for w in &mut weights[1..] {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    *w = (seed % 11) as i64 - 4;
                }
                let order: Vec<_> = (0..n).map(|v| (v + 1) % n).collect();
                let mut graph = Graph {
                    n,
                    words: 1,
                    compatible: vec![vec![0]; n],
                    order,
                };
                for a in 0..n {
                    for b in 0..n {
                        if a != b && weights[graph.order[a] ^ graph.order[b]] <= 0 {
                            graph.compatible[a][0] |= 1 << b;
                        }
                    }
                }
                let deadline = Instant::now() + std::time::Duration::from_secs(2);
                let labels: Vec<_> = (0..n).collect();
                if let Some(c) = certify(&graph, labels.clone(), weights.clone(), deadline) {
                    let optimum = (0u64..1 << n)
                        .filter(|&mask| {
                            (0..n).all(|a| {
                                mask & (1 << a) == 0
                                    || (graph.compatible[a][0] & mask) == (mask & !(1 << a))
                            })
                        })
                        .map(|m| m.count_ones() as usize)
                        .max()
                        .unwrap();
                    assert!(optimum <= c.upper);
                }
                if let Some((a, b)) = (0..n)
                    .flat_map(|a| (a + 1..n).map(move |b| (a, b)))
                    .find(|&(a, b)| adjacent(&graph, a, b))
                {
                    weights[graph.order[a] ^ graph.order[b]] = 1;
                    assert!(certify(&graph, labels, weights, deadline).is_none());
                }
            }
        }
    }
}
