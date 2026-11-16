//! incumbentとLPで一致する整数変数を固定する主解専用近傍。
//! 定義: Danna, Rothberg, Le Pape (2005), DOI 10.1007/s10107-004-0518-7。
//! https://www.ibm.com/docs/en/icos/22.1.2?topic=heuristics-relaxation-induced-neighborhood-search-rins-heuristic
//! 外部実装は移植せず、近傍の探索も自作Rust B&Bで行う。
use crate::{heuristic, model::Model};
pub struct Neighborhood {
    pub model: Model,
    pub seed: Vec<f64>,
    pub fixed: usize,
    pub eligible: usize,
}
pub fn neighborhood(model: &Model, point: &[f64], seed: &[f64]) -> Option<Neighborhood> {
    let n = model.variables.len();
    if point.len() != n || seed.len() != n || point.iter().chain(seed).any(|v| !v.is_finite()) {
        return None;
    }
    let seed: Vec<_> = seed
        .iter()
        .enumerate()
        .map(|(j, &v)| {
            if model.variables[j].integer {
                v.round()
            } else {
                v
            }
        })
        .collect();
    let mut fixed = Vec::new();
    let mut eligible = 0;
    for (j, v) in model.variables.iter().enumerate() {
        if !v.integer || matches!((v.lower,v.upper),(Some(a),Some(b)) if a==b) {
            continue;
        }
        eligible += 1;
        if (point[j] - seed[j]).abs() <= 1e-6 {
            fixed.push(j);
        }
    }
    if fixed.len() < 2 || fixed.len().saturating_mul(10) < eligible {
        return None;
    }
    if !heuristic::Repair::new(model).candidate_check(&seed) {
        return None;
    }
    let mut sub = model.clone();
    for &j in &fixed {
        let v = &mut sub.variables[j];
        v.lower = Some(seed[j]);
        v.upper = Some(seed[j]);
        v.lower_enclosure = Some([seed[j], seed[j]]);
        v.upper_enclosure = Some([seed[j], seed[j]]);
    }
    Some(Neighborhood {
        model: sub,
        seed,
        fixed: fixed.len(),
        eligible,
    })
}
