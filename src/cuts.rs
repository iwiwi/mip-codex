//! 二値変数のcover。非二値項を原境界で緩和し、成立をi128または外向き区間で検査。
//! 数学的定義: Kati Wolter, Cutting Plane Separators in SCIP, 0-1 knapsack節。
//! https://scipopt.org/download/slides/SCIP-cuttingPlanes.pdf
//! 外部実装のコード移植は行わず、単純な貪欲選択と独立の整数証明検査を実装。
use crate::model::{Model, Row};
mod aggregate;
pub use aggregate::applicable as aggregate_applicable;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::Instant};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proof {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aggregation: Vec<RowWeight>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub divisor: Option<i64>,
    pub source_row: usize,
    pub source_name: String,
    pub upper_side: bool,
    pub cover: Vec<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RowWeight {
    pub source_row: usize,
    pub source_name: String,
    pub upper_side: bool,
    pub weight: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cut {
    pub row: Row,
    pub proof: Proof,
}
#[derive(Default, Debug, Serialize)]
pub struct Stats {
    pub aggregate_calls: usize,
    pub aggregate_pairs: usize,
    pub aggregate_work: usize,
    pub aggregate_added: usize,
    pub aggregate_seconds: f64,
    pub rounding_calls: usize,
    pub rounding: usize,
    pub rounding_projected: usize,
    pub rounding_work: usize,
    pub calls: usize,
    pub eligible: usize,
    pub candidates: usize,
    pub added: usize,
    pub integer: usize,
    pub interval: usize,
    pub projected: usize,
    pub rejected: usize,
    pub work: usize,
}
#[derive(Clone)]
struct Item {
    j: usize,
    weight: f64,
    negative: bool,
}
fn integer(x: f64) -> Option<i128> {
    (x.is_finite() && x.fract() == 0. && x.abs() <= (1u64 << 52) as f64).then_some(x as i128)
}
fn finite(bounds: Option<[f64; 2]>) -> Option<[f64; 2]> {
    bounds.filter(|b| b[0].is_finite() && b[1].is_finite() && b[0] <= b[1])
}
fn binary(model: &Model, j: usize) -> bool {
    model.variables.get(j).is_some_and(|v| {
        v.integer
            && finite(v.lower_enclosure).is_some_and(|b| b[0] > -1.)
            && finite(v.upper_enclosure).is_some_and(|b| b[1] < 2.)
    })
}
fn column_bounds(model: &Model, j: usize, c: f64) -> Option<([f64; 2], f64)> {
    let v = model.variables.get(j)?;
    let (bound, interval) = if c > 0. {
        (v.lower?, finite(v.lower_enclosure)?)
    } else {
        (v.upper?, finite(v.upper_enclosure)?)
    };
    (bound.is_finite() && interval[0] <= bound && bound <= interval[1]).then_some((interval, bound))
}
fn packing(model: &Model, row: &Row, upper: bool) -> Option<(Vec<Item>, f64)> {
    if row.coefficients.len() > 16384 {
        return None;
    }
    let bounds = finite(if upper {
        row.upper_enclosure
    } else {
        row.lower_enclosure
    })?;
    let sign = if upper { 1. } else { -1. };
    let mut capacity = if upper { bounds[1] } else { -bounds[0] };
    let mut items = Vec::new();
    let mut previous = None;
    for &(j, a) in &row.coefficients {
        if previous.is_some_and(|k| j <= k) || !a.is_finite() {
            return None;
        }
        previous = Some(j);
        let c = sign * a;
        if c == 0. {
            continue;
        }
        let enclosure = coefficient(row, c);
        if binary(model, j) {
            if c < 0. {
                capacity = (capacity - enclosure[0]).next_up();
            }
            let weight = if c > 0. { enclosure[0] } else { -enclosure[1] };
            if weight > 0. {
                items.push(Item {
                    j,
                    weight,
                    negative: c < 0.,
                });
            }
        } else {
            let (bound, _) = column_bounds(model, j, c)?;
            capacity = (capacity - lower_product(enclosure, bound)?).next_up();
        }
        if !capacity.is_finite() {
            return None;
        }
    }
    Some((items, capacity))
}
pub fn applicable(model: &Model) -> bool {
    model.rows.iter().any(|row| {
        [true, false].into_iter().any(|side| {
            packing(model, row, side).is_some_and(|(items, capacity)| {
                items.iter().map(|p| p.weight).sum::<f64>() > capacity
            })
        })
    })
}
fn integer_proof(
    model: &Model,
    row: &Row,
    proof: &Proof,
    selected: &HashSet<usize>,
) -> Option<bool> {
    if !row.exact_integral {
        return None;
    }
    let sign = if proof.upper_side { 1 } else { -1 };
    let rhs = integer(if proof.upper_side {
        row.upper?
    } else {
        row.lower?
    })?
    .checked_mul(sign)?;
    let mut minimum = 0i128;
    for &(j, a) in &row.coefficients {
        let c = integer(a)?.checked_mul(sign)?;
        let term = if selected.contains(&j) {
            c.max(0)
        } else if binary(model, j) {
            c.min(0)
        } else {
            let (interval, bound) = column_bounds(model, j, c as f64)?;
            if interval[0] != interval[1] {
                return None;
            }
            c.checked_mul(integer(bound)?)?
        };
        minimum = minimum.checked_add(term)?;
    }
    Some(minimum > rhs)
}
fn coefficient(row: &Row, a: f64) -> [f64; 2] {
    if row.exact_integral {
        [a, a]
    } else {
        [a.next_down(), a.next_up()]
    }
}
fn lower_product(a: [f64; 2], b: [f64; 2]) -> Option<f64> {
    if b == [0., 0.] {
        return Some(0.);
    }
    let terms = [a[0] * b[0], a[0] * b[1], a[1] * b[0], a[1] * b[1]];
    if terms.iter().any(|x| x.is_nan()) {
        return None;
    }
    Some(
        terms
            .into_iter()
            .map(f64::next_down)
            .fold(f64::INFINITY, f64::min),
    )
}
/// 選択された二値変数を強制した元行の最小活動量を、原境界から検査する。
/// i128で正確に扱える行は整数、それ以外は外向き区間。選択用の近似容量は使わない。
fn proof_kind(model: &Model, cut: &Cut) -> Option<ProofKind> {
    if cut.proof.divisor.is_some() {
        return rounding_verify(model, cut).then_some(ProofKind::Rounding);
    }
    if !cut.proof.aggregation.is_empty() {
        return None;
    }
    let Some(row) = model.rows.get(cut.proof.source_row) else {
        return None;
    };
    if row.name != cut.proof.source_name || cut.proof.cover.windows(2).any(|w| w[0] >= w[1]) {
        return None;
    }
    let selected: HashSet<_> = cut.proof.cover.iter().copied().collect();
    let sign = if cut.proof.upper_side { 1. } else { -1. };
    let mut coefficients = Vec::new();
    let mut negative = 0i64;
    let mut previous = None;
    for &(j, a) in &row.coefficients {
        if j >= model.variables.len() || !a.is_finite() || previous.is_some_and(|k| j <= k) {
            return None;
        }
        previous = Some(j);
        let c = sign * a;
        if selected.contains(&j) {
            if c == 0. || !binary(model, j) {
                return None;
            }
            coefficients.push((j, if c > 0. { 1. } else { -1. }));
            negative += i64::from(c < 0.);
        }
    }
    let upper = cut.proof.cover.len() as i64 - 1 - negative;
    if coefficients.len() != selected.len()
        || cut.row.lower.is_some()
        || cut.row.upper != Some(upper as f64)
        || cut.row.coefficients != coefficients
        || !cut.row.exact_integral
    {
        return None;
    }
    if let Some(valid) = integer_proof(model, row, &cut.proof, &selected) {
        return valid.then_some(ProofKind::Integer);
    }
    let bounds = if cut.proof.upper_side {
        finite(row.upper_enclosure)
    } else {
        finite(row.lower_enclosure)
    };
    let Some(bounds) = bounds else {
        return None;
    };
    let rhs = if cut.proof.upper_side {
        bounds[1]
    } else {
        -bounds[0]
    };
    let mut minimum = 0f64;
    for &(j, a) in &row.coefficients {
        let c = sign * a;
        if c == 0. {
            continue;
        }
        let enclosure = coefficient(row, c);
        let term = if selected.contains(&j) {
            if c > 0. { enclosure[0] } else { 0. }
        } else if binary(model, j) {
            if c < 0. { enclosure[0] } else { 0. }
        } else {
            let Some((bound, _)) = column_bounds(model, j, c) else {
                return None;
            };
            let Some(term) = lower_product(enclosure, bound) else {
                return None;
            };
            term
        };
        if term != 0. {
            minimum = (minimum + term).next_down();
        }
        if !minimum.is_finite() {
            return None;
        }
    }
    (minimum > rhs).then_some(ProofKind::Interval)
}
#[derive(Clone, Copy)]
enum ProofKind {
    Integer,
    Interval,
    Rounding,
}
pub fn verify(model: &Model, cut: &Cut) -> bool {
    proof_kind(model, cut).is_some()
}

#[derive(Default)]
pub struct Pool {
    seen: HashSet<(Vec<(usize, i64)>, i64)>,
    pub stats: Stats,
}
impl Pool {
    pub fn separate(
        &mut self,
        model: &Model,
        x: &[f64],
        limit: usize,
        deadline: Instant,
    ) -> Vec<Cut> {
        self.stats.calls += 1;
        if x.len() != model.variables.len() || x.iter().any(|x| !x.is_finite()) {
            return vec![];
        }
        let mut cuts = Vec::new();
        for (i, row) in model.rows.iter().enumerate() {
            if Instant::now() >= deadline || cuts.len() >= limit || self.stats.work >= 2_000_000 {
                break;
            }
            for upper_side in [true, false] {
                let Some((items, capacity)) = packing(model, row, upper_side) else {
                    continue;
                };
                self.stats.eligible += 1;
                for weighted in [true, false] {
                    let mut order: Vec<_> = (0..items.len()).collect();
                    let z = |k: usize| {
                        let item = &items[k];
                        if item.negative {
                            1. - x[item.j]
                        } else {
                            x[item.j]
                        }
                    };
                    order.sort_by(|&a, &b| {
                        let score = |k| {
                            if weighted {
                                (1. - z(k)).max(0.) / items[k].weight
                            } else {
                                -z(k)
                            }
                        };
                        score(a)
                            .total_cmp(&score(b))
                            .then(items[b].weight.total_cmp(&items[a].weight))
                            .then(a.cmp(&b))
                    });
                    self.stats.work = self
                        .stats
                        .work
                        .saturating_add(items.len().saturating_mul(8));
                    let mut chosen = Vec::new();
                    let mut weight = 0f64;
                    for k in order {
                        if weight > capacity {
                            break;
                        }
                        let sum = (weight + items[k].weight).next_down();
                        if !sum.is_finite() {
                            break;
                        }
                        weight = sum;
                        chosen.push(k);
                    }
                    if weight <= capacity {
                        continue;
                    }
                    // LP値が小さいものから除き、整数のcover条件を維持する。
                    chosen.sort_by(|&a, &b| z(a).total_cmp(&z(b)).then(a.cmp(&b)));
                    chosen.retain(|&k| {
                        let reduced = (weight - items[k].weight).next_down();
                        if reduced > capacity {
                            weight = reduced;
                            false
                        } else {
                            true
                        }
                    });
                    if chosen.len() > 256 {
                        continue;
                    }
                    let mut cover: Vec<_> = chosen.iter().map(|&k| items[k].j).collect();
                    cover.sort_unstable();
                    let negatives = chosen.iter().filter(|&&k| items[k].negative).count();
                    let mut coefficients: Vec<_> = chosen
                        .iter()
                        .map(|&k| (items[k].j, if items[k].negative { -1. } else { 1. }))
                        .collect();
                    coefficients.sort_unstable_by_key(|p| p.0);
                    let rhs = cover.len() as i64 - 1 - negatives as i64;
                    let activity: f64 = coefficients.iter().map(|&(j, a)| a * x[j]).sum();
                    if activity - rhs as f64 <= 1e-5 {
                        continue;
                    }
                    self.stats.candidates += 1;
                    let cut = Cut {
                        row: Row {
                            lower_enclosure: None,
                            upper_enclosure: Some([rhs as f64, rhs as f64]),
                            exact_coefficients: true,
                            exact_coefficient_flags: vec![],
                            exact_integral: true,
                            name: format!("__cover_{}", self.seen.len()),
                            lower: None,
                            upper: Some(rhs as f64),
                            coefficients,
                        },
                        proof: Proof {
                            aggregation: vec![],
                            divisor: None,
                            source_row: i,
                            source_name: row.name.clone(),
                            upper_side,
                            cover,
                        },
                    };
                    let Some(kind) = proof_kind(model, &cut) else {
                        self.stats.rejected += 1;
                        continue;
                    };
                    let key = (
                        cut.row
                            .coefficients
                            .iter()
                            .map(|&(j, a)| (j, a as i64))
                            .collect(),
                        rhs,
                    );
                    if self.seen.insert(key) {
                        self.stats.added += 1;
                        match kind {
                            ProofKind::Integer => self.stats.integer += 1,
                            ProofKind::Interval => self.stats.interval += 1,
                            ProofKind::Rounding => unreachable!(),
                        }
                        if row
                            .coefficients
                            .iter()
                            .any(|&(j, a)| a != 0. && !binary(model, j))
                        {
                            self.stats.projected += 1;
                        }
                        cuts.push(cut);
                    }
                    if cuts.len() >= limit || Instant::now() >= deadline {
                        break;
                    }
                }
                if cuts.len() >= limit || Instant::now() >= deadline {
                    break;
                }
            }
        }
        cuts
    }
}

// 一行を正整数dで割り、非負整数変数の係数と右辺をfloorするCG cut。
// Fabricio Oliveira, Linear Optimisation Notes, §10.2:
// https://gamma-opt.github.io/linopt-notes/pdfs/linopt-notes.pdf
// 数学的定義のみ参照。コード移植/最適化委譲なし。
fn rounding_row(model: &Model, source: &Row, upper: bool, d: i64) -> Option<Row> {
    if d < 2 || d > (1i64 << 52) || !source.exact_integral || source.coefficients.len() > 16384 {
        return None;
    }
    let sign = if upper { 1i128 } else { -1 };
    let mut bound = integer(if upper { source.upper? } else { source.lower? })? * sign;
    let mut coefficients = Vec::new();
    let mut previous = None;
    for &(j, a) in &source.coefficients {
        if previous.is_some_and(|k| j <= k) {
            return None;
        }
        previous = Some(j);
        let a = integer(a)? * sign;
        if a == 0 {
            continue;
        }
        let v = model.variables.get(j)?;
        if !(v.integer && finite(v.lower_enclosure).is_some_and(|b| b[0] >= 0.)) {
            let (interval, b) = column_bounds(model, j, a as f64)?;
            if interval[0] != interval[1] {
                return None;
            }
            bound = bound.checked_sub(a.checked_mul(integer(b)?)?)?;
            continue;
        }
        let c = a.div_euclid(d as i128);
        if c != 0 {
            coefficients.push((j, c as f64));
        }
    }
    if bound.abs() > (1i128 << 52) {
        return None;
    }
    let rhs = bound.div_euclid(d as i128);
    Some(Row {
        name: String::new(),
        exact_coefficients: true,
        exact_coefficient_flags: vec![],
        exact_integral: true,
        lower: None,
        upper: Some(rhs as f64),
        lower_enclosure: None,
        upper_enclosure: Some([rhs as f64, rhs as f64]),
        coefficients,
    })
}
fn rounding_verify(model: &Model, cut: &Cut) -> bool {
    let Some(source) = model.rows.get(cut.proof.source_row) else {
        return false;
    };
    if cut.proof.source_name != source.name || !cut.proof.cover.is_empty() {
        return false;
    }
    let combined;
    let source = if cut.proof.aggregation.is_empty() {
        source
    } else {
        let first = &cut.proof.aggregation[0];
        if first.source_row != cut.proof.source_row
            || first.source_name != cut.proof.source_name
            || first.upper_side != cut.proof.upper_side
        {
            return false;
        }
        let Some(row) = aggregate::source(model, &cut.proof.aggregation) else {
            return false;
        };
        combined = row;
        &combined
    };
    let Some(row) = rounding_row(
        model,
        source,
        cut.proof
            .aggregation
            .is_empty()
            .then_some(cut.proof.upper_side)
            .unwrap_or(true),
        cut.proof.divisor.unwrap_or(0),
    ) else {
        return false;
    };
    cut.row.exact_integral
        && cut.row.lower.is_none()
        && cut.row.upper == row.upper
        && cut.row.coefficients == row.coefficients
}
pub fn rounding_applicable(model: &Model) -> bool {
    model.rows.iter().any(|r| {
        [true, false]
            .into_iter()
            .any(|side| rounding_row(model, r, side, 2).is_some())
    })
}
impl Pool {
    pub fn separate_rounding(
        &mut self,
        model: &Model,
        x: &[f64],
        limit: usize,
        deadline: Instant,
    ) -> Vec<Cut> {
        self.stats.rounding_calls += 1;
        let mut cuts = Vec::new();
        if x.len() != model.variables.len() || x.iter().any(|v| !v.is_finite()) {
            return cuts;
        }
        for (i, source) in model.rows.iter().enumerate() {
            if cuts.len() >= limit
                || Instant::now() >= deadline
                || self.stats.rounding_work >= 500_000
            {
                break;
            }
            if !source.exact_integral || source.coefficients.len() > 16384 {
                continue;
            }
            if ![true, false]
                .into_iter()
                .any(|side| rounding_row(model, source, side, 2).is_some())
            {
                continue;
            }
            let mut divisors = vec![2i64, 3, 4, 5, 10];
            let mut gcd = 0i64;
            for &(_, a) in &source.coefficients {
                let Some(a) = integer(a) else {
                    continue;
                };
                let mut b = (a as i64).abs();
                if b >= 2 && divisors.len() < 24 && !divisors.contains(&b) {
                    divisors.push(b);
                }
                while b != 0 {
                    (gcd, b) = (b, gcd % b);
                }
            }
            if gcd >= 2 && !divisors.contains(&gcd) {
                divisors.push(gcd);
            }
            for side in [true, false] {
                if rounding_row(model, source, side, 2).is_none() {
                    continue;
                }
                for &d in &divisors {
                    if cuts.len() >= limit
                        || Instant::now() >= deadline
                        || self.stats.rounding_work >= 500_000
                    {
                        break;
                    }
                    self.stats.rounding_work = self
                        .stats
                        .rounding_work
                        .saturating_add(source.coefficients.len());
                    let Some(mut row) = rounding_row(model, source, side, d) else {
                        continue;
                    };
                    let rhs = row.upper.unwrap();
                    let activity: f64 = row.coefficients.iter().map(|&(j, a)| a * x[j]).sum();
                    if !activity.is_finite() || activity - rhs <= 1e-5 {
                        continue;
                    }
                    let key = (
                        row.coefficients
                            .iter()
                            .map(|&(j, a)| (j, a as i64))
                            .collect(),
                        rhs as i64,
                    );
                    row.name = format!("__rounding_{}", self.seen.len());
                    let cut = Cut {
                        row,
                        proof: Proof {
                            aggregation: vec![],
                            divisor: Some(d),
                            source_row: i,
                            source_name: source.name.clone(),
                            upper_side: side,
                            cover: vec![],
                        },
                    };
                    self.stats.candidates += 1;
                    if !verify(model, &cut) {
                        self.stats.rejected += 1;
                        continue;
                    }
                    if self.seen.insert(key) {
                        self.stats.added += 1;
                        self.stats.rounding += 1;
                        if source.coefficients.iter().any(|&(j, a)| {
                            a != 0.
                                && !(model.variables[j].integer
                                    && finite(model.variables[j].lower_enclosure)
                                        .is_some_and(|b| b[0] >= 0.))
                        }) {
                            self.stats.rounding_projected += 1;
                        }
                        cuts.push(cut);
                    }
                }
            }
        }
        cuts
    }
}
