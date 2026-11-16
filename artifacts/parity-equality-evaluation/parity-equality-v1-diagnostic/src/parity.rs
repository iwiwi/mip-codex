//! Exact binary parity equalities with one private ±2 integer per row.
//! A full-rank GF(2) system and exact reconstruction prove a unique feasible point.
use crate::model::Model;
use serde::Serialize;
use std::time::Instant;
#[derive(Debug, Serialize)]
pub struct Proof {
    pub binary_variables: Vec<usize>,
    pub auxiliary_variables: Vec<usize>,
    pub pivots: Vec<usize>,
    pub rank: usize,
    pub row_count: usize,
}
pub struct Answer {
    pub point: Vec<f64>,
    pub proof: Proof,
}
fn integer(v: f64) -> Option<i128> {
    (v.is_finite() && v.fract() == 0. && v.abs() <= (1u64 << 52) as f64).then_some(v as i128)
}
fn bit(row: &[u64], col: usize) -> bool {
    row[col / 64] >> (col % 64) & 1 != 0
}
pub fn solve(model: &Model, deadline: Instant) -> Option<Answer> {
    if model.variables.is_empty()
        || model.rows.is_empty()
        || model.rows.len() > 2048
        || model.variables.len() > 4096
        || Instant::now() >= deadline
    {
        return None;
    }
    let mut binary = Vec::new();
    let mut indices = vec![None; model.variables.len()];
    let mut degrees = vec![0usize; model.variables.len()];
    for (j, v) in model.variables.iter().enumerate() {
        if !v.integer {
            return None;
        }
        if v.lower == Some(0.)
            && v.upper == Some(1.)
            && v.lower_enclosure == Some([0., 0.])
            && v.upper_enclosure == Some([1., 1.])
        {
            indices[j] = Some(binary.len());
            binary.push(j);
        }
    }
    if binary.is_empty() || binary.len() > 2048 {
        return None;
    }
    for row in &model.rows {
        for &(j, _) in &row.coefficients {
            *degrees.get_mut(j)? += 1;
        }
    }
    let n = binary.len();
    let words = (n + 1).div_ceil(64);
    let mut matrix = Vec::with_capacity(model.rows.len());
    let mut auxiliaries = Vec::new();
    let mut rhs_values = Vec::new();
    let mut aux_coefficients = Vec::new();
    for row in &model.rows {
        let rhs = row.lower?;
        if row.upper != Some(rhs)
            || !row.exact_integral
            || row.lower_enclosure != Some([rhs, rhs])
            || row.upper_enclosure != Some([rhs, rhs])
        {
            return None;
        }
        let rhs = integer(rhs)?;
        let mut bits = vec![0u64; words];
        let mut auxiliary = None;
        for &(j, a) in &row.coefficients {
            let coefficient = integer(a)?;
            if let Some(k) = indices[j] {
                if coefficient % 2 != 0 {
                    bits[k / 64] ^= 1u64 << (k % 64);
                }
            } else {
                if auxiliary.is_some() || degrees[j] != 1 || coefficient.abs() != 2 {
                    return None;
                }
                auxiliary = Some((j, coefficient));
            }
        }
        let (j, a) = auxiliary?;
        if rhs % 2 != 0 {
            bits[n / 64] |= 1u64 << (n % 64);
        }
        matrix.push(bits);
        auxiliaries.push(j);
        rhs_values.push(rhs);
        aux_coefficients.push(a);
    }
    if indices
        .iter()
        .enumerate()
        .any(|(j, i)| i.is_none() && degrees[j] != 1)
    {
        return None;
    }
    let mut rank = 0;
    let mut pivots = Vec::new();
    let mut work = 0usize;
    for col in 0..n {
        if Instant::now() >= deadline {
            return None;
        }
        let Some(pivot) = (rank..matrix.len()).find(|&i| bit(&matrix[i], col)) else {
            continue;
        };
        matrix.swap(rank, pivot);
        for i in 0..matrix.len() {
            if i == rank || !bit(&matrix[i], col) {
                continue;
            }
            work = work.saturating_add(words);
            if work > 20_000_000 {
                return None;
            }
            for k in 0..words {
                matrix[i][k] ^= matrix[rank][k];
            }
        }
        pivots.push(col);
        rank += 1;
    }
    if rank != n {
        return None;
    }
    for row in &matrix[rank..] {
        if bit(row, n) {
            return None;
        }
    }
    let mut point = vec![0.; model.variables.len()];
    for (i, &col) in pivots.iter().enumerate() {
        point[binary[col]] = if bit(&matrix[i], n) { 1. } else { 0. };
    }
    for (i, row) in model.rows.iter().enumerate() {
        let mut residual = rhs_values[i];
        for &(j, a) in &row.coefficients {
            if indices[j].is_some() {
                residual = residual.checked_sub(integer(a)?.checked_mul(point[j] as i128)?)?;
            }
        }
        let a = aux_coefficients[i];
        if residual % a != 0 {
            return None;
        }
        let value = residual / a;
        if value.abs() > (1i128 << 52) {
            return None;
        }
        point[auxiliaries[i]] = value as f64;
    }
    for (v, &x) in model.variables.iter().zip(&point) {
        if let Some(l) = v.lower {
            let b = v.lower_enclosure?;
            if b[0] > l || b[1] < l || x < b[1] {
                return None;
            }
        }
        if let Some(u) = v.upper {
            let b = v.upper_enclosure?;
            if b[0] > u || b[1] < u || x > b[0] {
                return None;
            }
        }
    }
    if Instant::now() >= deadline || model.check(&point, true, 0.).is_err() {
        return None;
    }
    Some(Answer {
        point,
        proof: Proof {
            binary_variables: binary,
            auxiliary_variables: auxiliaries,
            pivots,
            rank,
            row_count: model.rows.len(),
        },
    })
}
