//! 小さい整数係数の二値行に対するcover。選択はf64、成立確認はi128。
//! 数学的定義: Kati Wolter, Cutting Plane Separators in SCIP, 0-1 knapsack節。
//! https://scipopt.org/download/slides/SCIP-cuttingPlanes.pdf
//! 外部実装のコード移植は行わず、単純な貪欲選択と独立の整数証明検査を実装。
use crate::model::{Model, Row};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::Instant};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proof {
    pub source_row: usize,
    pub source_name: String,
    pub upper_side: bool,
    pub cover: Vec<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cut {
    pub row: Row,
    pub proof: Proof,
}
#[derive(Default, Debug, Serialize)]
pub struct Stats {
    pub calls: usize,
    pub eligible: usize,
    pub candidates: usize,
    pub added: usize,
    pub rejected: usize,
    pub work: usize,
}
#[derive(Clone)]
struct Item {
    j: usize,
    weight: i128,
    negative: bool,
}
fn integer(x: f64) -> Option<i128> {
    (x.is_finite() && x.fract() == 0. && x.abs() <= (1u64 << 52) as f64).then_some(x as i128)
}
fn binary(model: &Model, j: usize) -> bool {
    model.variables.get(j).is_some_and(|v| {
        v.integer && v.lower.is_some_and(|l| l >= 0.) && v.upper.is_some_and(|u| u <= 1.)
    })
}
fn packing(model: &Model, row: &Row, upper: bool) -> Option<(Vec<Item>, i128)> {
    if !row.exact_integral || row.coefficients.len() > 16384 {
        return None;
    }
    let sign = if upper { 1 } else { -1 };
    let mut capacity = integer(if upper { row.upper? } else { row.lower? })?.checked_mul(sign)?;
    let mut items = Vec::new();
    let mut previous = None;
    for &(j, a) in &row.coefficients {
        if previous.is_some_and(|k| j <= k) || !binary(model, j) {
            return None;
        }
        previous = Some(j);
        let c = integer(a)?.checked_mul(sign)?;
        if c == 0 {
            continue;
        }
        if c < 0 {
            capacity = capacity.checked_sub(c)?;
        }
        items.push(Item {
            j,
            weight: c.abs(),
            negative: c < 0,
        });
    }
    Some((items, capacity))
}
pub fn applicable(model: &Model) -> bool {
    model.rows.iter().any(|row| {
        [true, false].into_iter().any(|side| {
            packing(model, row, side).is_some_and(|(items, capacity)| {
                items
                    .iter()
                    .try_fold(0i128, |s, item| s.checked_add(item.weight))
                    .is_some_and(|sum| sum > capacity)
            })
        })
    })
}
/// 強制されたcover集合の元行の最小活動量がRHSを超えることを直接検査。
/// generatorの貪欲選択や容量計算の結果は信用せず、元行から再計算する。
pub fn verify(model: &Model, cut: &Cut) -> bool {
    let Some(row) = model.rows.get(cut.proof.source_row) else {
        return false;
    };
    if !row.exact_integral || row.name != cut.proof.source_name {
        return false;
    }
    let bound = if cut.proof.upper_side {
        row.upper
    } else {
        row.lower
    };
    let Some(bound) = bound.and_then(integer) else {
        return false;
    };
    let sign = if cut.proof.upper_side { 1 } else { -1 };
    let Some(rhs) = bound.checked_mul(sign) else {
        return false;
    };
    if cut.proof.cover.windows(2).any(|w| w[0] >= w[1]) {
        return false;
    }
    let selected: HashSet<_> = cut.proof.cover.iter().copied().collect();
    let mut minimum = 0i128;
    let mut coefficients = Vec::new();
    let mut negatives = 0i128;
    let mut previous = None;
    for &(j, a) in &row.coefficients {
        if previous.is_some_and(|k| j <= k) || !binary(model, j) {
            return false;
        }
        previous = Some(j);
        let Some(c) = integer(a).and_then(|c| c.checked_mul(sign)) else {
            return false;
        };
        let term = if selected.contains(&j) {
            if c == 0 {
                return false;
            }
            coefficients.push((j, if c > 0 { 1. } else { -1. }));
            if c < 0 {
                negatives += 1;
            }
            c.max(0)
        } else {
            c.min(0)
        };
        let Some(next) = minimum.checked_add(term) else {
            return false;
        };
        minimum = next;
    }
    let upper = cut.proof.cover.len() as i128 - 1 - negatives;
    minimum > rhs
        && coefficients.len() == selected.len()
        && cut.row.lower.is_none()
        && cut.row.upper == Some(upper as f64)
        && cut.row.coefficients == coefficients
        && upper.abs() <= (1i128 << 52)
        && cut.row.exact_integral
}
#[derive(Default)]
pub struct Pool {
    seen: HashSet<(Vec<(usize, i8)>, i64)>,
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
                                (1. - z(k)).max(0.) / items[k].weight as f64
                            } else {
                                -z(k)
                            }
                        };
                        score(a)
                            .total_cmp(&score(b))
                            .then(items[b].weight.cmp(&items[a].weight))
                            .then(a.cmp(&b))
                    });
                    self.stats.work = self
                        .stats
                        .work
                        .saturating_add(items.len().saturating_mul(8));
                    let mut chosen = Vec::new();
                    let mut weight = 0i128;
                    for k in order {
                        if weight > capacity {
                            break;
                        }
                        let Some(sum) = weight.checked_add(items[k].weight) else {
                            break;
                        };
                        weight = sum;
                        chosen.push(k);
                    }
                    if weight <= capacity {
                        continue;
                    }
                    // LP値が小さいものから除き、整数のcover条件を維持する。
                    chosen.sort_by(|&a, &b| z(a).total_cmp(&z(b)).then(a.cmp(&b)));
                    chosen.retain(|&k| {
                        if weight - items[k].weight > capacity {
                            weight -= items[k].weight;
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
                            exact_integral: true,
                            name: format!("__cover_{}", self.seen.len()),
                            lower: None,
                            upper: Some(rhs as f64),
                            coefficients,
                        },
                        proof: Proof {
                            source_row: i,
                            source_name: row.name.clone(),
                            upper_side,
                            cover,
                        },
                    };
                    if !verify(model, &cut) {
                        self.stats.rejected += 1;
                        continue;
                    }
                    let key = (
                        cut.row
                            .coefficients
                            .iter()
                            .map(|&(j, a)| (j, a as i8))
                            .collect(),
                        rhs,
                    );
                    if self.seen.insert(key) {
                        self.stats.added += 1;
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
