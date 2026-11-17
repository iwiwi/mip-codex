//! 元の整数係数行の十進境界を整数格子へ丸める。元MPSの独立証明を出す。
use crate::model::Model;
use serde::{Deserialize, Serialize};
use std::time::Instant;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bound {
    pub source_row: usize,
    pub lower_bits: Option<String>,
    pub upper_bits: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VariableBound {
    pub source_variable: usize,
    pub lower_bits: Option<String>,
    pub upper_bits: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proof {
    pub variables: Vec<VariableBound>,
    pub rows: Vec<Bound>,
}
pub(crate) fn prepare(model: &Model, deadline: Instant) -> Option<(Model, Proof)> {
    if !model.original_decimal || Instant::now() >= deadline {
        return None;
    }
    let mut variables = vec![];
    for (i, v) in model.variables.iter().enumerate() {
        if i % 1024 == 0 && Instant::now() >= deadline {
            return None;
        }
        if !v.integer {
            continue;
        }
        let Some([lower, upper]) = v.integer_bounds else {
            continue;
        };
        let lower = lower.filter(|&b| v.lower.is_some_and(|old| b > old));
        let upper = upper.filter(|&b| v.upper.is_some_and(|old| b < old));
        if lower.is_some() || upper.is_some() {
            variables.push(VariableBound {
                source_variable: i,
                lower_bits: lower.map(|b| format!("{:016x}", b.to_bits())),
                upper_bits: upper.map(|b| format!("{:016x}", b.to_bits())),
            });
        }
    }
    let mut rows = vec![];
    for (i, row) in model.rows.iter().enumerate() {
        if i % 1024 == 0 && Instant::now() >= deadline {
            return None;
        }
        let Some([lower, upper]) = row.integer_bounds else {
            continue;
        };
        let lower = lower.filter(|&b| row.lower.is_some_and(|old| b > old));
        let upper = upper.filter(|&b| row.upper.is_some_and(|old| b < old));
        if lower.is_none() && upper.is_none() {
            continue;
        }
        if !row.exact_coefficients
            || row.coefficients.iter().any(|&(j, a)| {
                !model.variables[j].integer
                    || !a.is_finite()
                    || a.fract() != 0.
                    || a.abs() > (1u64 << 52) as f64
            })
        {
            continue;
        }
        rows.push(Bound {
            source_row: i,
            lower_bits: lower.map(|v| format!("{:016x}", v.to_bits())),
            upper_bits: upper.map(|v| format!("{:016x}", v.to_bits())),
        });
    }
    if (rows.is_empty() && variables.is_empty()) || Instant::now() >= deadline {
        return None;
    }
    let mut work = model.clone();
    for bound in &variables {
        let v = &mut work.variables[bound.source_variable];
        if let Some(b) = bound
            .lower_bits
            .as_ref()
            .map(|s| f64::from_bits(u64::from_str_radix(s, 16).unwrap()))
        {
            v.lower = Some(b);
            v.lower_enclosure = Some([b; 2]);
        }
        if let Some(b) = bound
            .upper_bits
            .as_ref()
            .map(|s| f64::from_bits(u64::from_str_radix(s, 16).unwrap()))
        {
            v.upper = Some(b);
            v.upper_enclosure = Some([b; 2]);
        }
    }
    for bound in &rows {
        let row = &mut work.rows[bound.source_row];
        if let Some(v) = bound
            .lower_bits
            .as_ref()
            .map(|s| f64::from_bits(u64::from_str_radix(s, 16).unwrap()))
        {
            row.lower = Some(v);
            row.lower_enclosure = Some([v; 2]);
        }
        if let Some(v) = bound
            .upper_bits
            .as_ref()
            .map(|s| f64::from_bits(u64::from_str_radix(s, 16).unwrap()))
        {
            row.upper = Some(v);
            row.upper_enclosure = Some([v; 2]);
        }
        row.exact_integral = row
            .lower
            .into_iter()
            .chain(row.upper)
            .all(|v| v.fract() == 0. && v.abs() <= (1u64 << 52) as f64);
    }
    Some((work, Proof { variables, rows }))
}
