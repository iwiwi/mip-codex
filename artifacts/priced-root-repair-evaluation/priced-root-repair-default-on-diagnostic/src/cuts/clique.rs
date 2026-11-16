//! Clique inequalities from exact original-row pair conflicts.
use super::*;
use std::collections::{BTreeMap, HashMap};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pair {
    pub source_row: usize,
    pub source_name: String,
    pub upper_side: bool,
    pub variables: [usize; 2],
}
// Exact minimum with no selected literals. Selecting a positive/negative
// literal raises this minimum by |a|. Only original bounds are used.
fn packing_exact(model: &Model, row: &Row, upper: bool) -> Option<(Vec<(usize, i128)>, i128)> {
    if !row.exact_integral || row.coefficients.len() > 16384 {
        return None;
    }
    let sign = if upper { 1i128 } else { -1 };
    let mut capacity = integer(if upper { row.upper? } else { row.lower? })?.checked_mul(sign)?;
    let mut items = vec![];
    let mut previous = None;
    for &(j, a) in &row.coefficients {
        if j >= model.variables.len() || previous.is_some_and(|p| p >= j) {
            return None;
        }
        previous = Some(j);
        let a = integer(a)?.checked_mul(sign)?;
        if a == 0 {
            continue;
        }
        if binary(model, j) {
            capacity = capacity.checked_sub(a.min(0))?;
            items.push((2 * j + usize::from(a < 0), a.abs()));
        } else {
            let (interval, bound) = column_bounds(model, j, a as f64)?;
            if interval[0] != interval[1] {
                return None;
            }
            capacity = capacity.checked_sub(a.checked_mul(integer(bound)?)?)?;
        }
    }
    Some((items, capacity))
}
pub(super) fn verify(model: &Model, cut: &Cut) -> bool {
    verify_until(model, cut, None)
}
fn verify_until(model: &Model, cut: &Cut, deadline: Option<Instant>) -> bool {
    let p = &cut.proof;
    let k = cut.row.coefficients.len();
    if !(3..=32).contains(&k)
        || p.divisor.is_some()
        || !p.aggregation.is_empty()
        || p.clique.len() != k * (k - 1) / 2
        || cut.row.lower.is_some()
        || !cut.row.exact_integral
        || cut.row.coefficients.windows(2).any(|w| w[0].0 >= w[1].0)
        || cut
            .row
            .coefficients
            .iter()
            .any(|&(j, a)| !binary(model, j) || (a != 1. && a != -1.))
        || p.cover
            != cut
                .row
                .coefficients
                .iter()
                .map(|&(j, _)| j)
                .collect::<Vec<_>>()
        || cut.row.upper
            != Some(1. - cut.row.coefficients.iter().filter(|(_, a)| *a < 0.).count() as f64)
    {
        return false;
    }
    let Some(first) = p.clique.first() else {
        return false;
    };
    if (p.source_row, &p.source_name, p.upper_side)
        != (first.source_row, &first.source_name, first.upper_side)
    {
        return false;
    }
    let coefficients: BTreeMap<_, _> = cut.row.coefficients.iter().copied().collect();
    let mut seen = HashSet::new();
    for pair in &p.clique {
        if deadline.is_some_and(|d| Instant::now() >= d) {
            return false;
        }
        let [a, b] = pair.variables;
        if a >= b
            || !coefficients.contains_key(&a)
            || !coefficients.contains_key(&b)
            || !seen.insert((a, b))
        {
            return false;
        }
        let Some(row) = model.rows.get(pair.source_row) else {
            return false;
        };
        if row.name != pair.source_name {
            return false;
        }
        // Reuse the original single-row cover proof checker, not the graph's
        // approximate candidate selection or its cached capacity.
        let pair_cut = Cut {
            row: Row {
                name: String::new(),
                coefficients: vec![(a, coefficients[&a]), (b, coefficients[&b])],
                lower: None,
                upper: Some(
                    1. - usize::from(coefficients[&a] < 0.) as f64
                        - usize::from(coefficients[&b] < 0.) as f64,
                ),
                lower_enclosure: None,
                upper_enclosure: None,
                exact_integral: true,
                exact_coefficients: true,
                exact_coefficient_flags: vec![],
            },
            proof: Proof {
                mir: None,
                clique: vec![],
                aggregation: vec![],
                divisor: None,
                source_row: pair.source_row,
                source_name: pair.source_name.clone(),
                upper_side: pair.upper_side,
                cover: vec![a, b],
            },
        };
        if proof_kind(model, &pair_cut).is_none() {
            return false;
        }
    }
    true
}
impl Pool {
    pub fn separate_clique(
        &mut self,
        model: &Model,
        x: &[f64],
        limit: usize,
        deadline: Instant,
    ) -> Vec<Cut> {
        let started = Instant::now();
        self.stats.clique_calls += 1;
        let result = self.separate_clique_impl(model, x, limit, deadline);
        self.stats.clique_seconds += started.elapsed().as_secs_f64();
        result
    }
    fn separate_clique_impl(
        &mut self,
        model: &Model,
        x: &[f64],
        limit: usize,
        deadline: Instant,
    ) -> Vec<Cut> {
        let started = Instant::now();
        let deadline = deadline.min(started + std::time::Duration::from_millis(20));
        if limit == 0 || x.len() != model.variables.len() || x.iter().any(|v| !v.is_finite()) {
            return vec![];
        }
        let mut literals = vec![];
        for (j, &v) in x.iter().enumerate() {
            if j % 1024 == 0 && Instant::now() >= deadline {
                return vec![];
            }
            if v > 1e-4 && v < 1. - 1e-4 && binary(model, j) {
                literals.extend([2 * j, 2 * j + 1]);
            }
        }
        let value = |l: usize| if l % 2 == 0 { x[l / 2] } else { 1. - x[l / 2] };
        literals.sort_by(|&a, &b| value(b).total_cmp(&value(a)).then(a.cmp(&b)));
        literals.truncate(512);
        let n = literals.len();
        if n < 3 {
            return vec![];
        }
        let lookup: HashMap<_, _> = literals.iter().enumerate().map(|(i, &l)| (l, i)).collect();
        let graph_deadline = deadline.min(started + std::time::Duration::from_millis(10));
        let mut adjacency = vec![vec![false; n]; n];
        let mut witnesses = HashMap::<(usize, usize), (usize, bool)>::new();
        'rows: for (ri, row) in model.rows.iter().enumerate() {
            if Instant::now() >= graph_deadline || self.stats.clique_work >= 1_000_000 {
                break;
            }
            for upper in [true, false] {
                self.stats.clique_work = self
                    .stats
                    .clique_work
                    .saturating_add(row.coefficients.len());
                let Some((items, capacity)) = packing_exact(model, row, upper) else {
                    continue;
                };
                let items: Vec<_> = items
                    .into_iter()
                    .filter_map(|(l, a)| lookup.get(&l).map(|&i| (i, a)))
                    .collect();
                for k in 0..items.len() {
                    if Instant::now() >= graph_deadline {
                        break 'rows;
                    }
                    for &(b, wb) in &items[k + 1..] {
                        self.stats.clique_work += 1;
                        if self.stats.clique_work >= 1_000_000 {
                            break 'rows;
                        }
                        let (a, wa) = items[k];
                        if wa.checked_add(wb).is_none_or(|sum| sum <= capacity) {
                            continue;
                        }
                        let key = (a.min(b), a.max(b));
                        if let std::collections::hash_map::Entry::Vacant(e) = witnesses.entry(key) {
                            e.insert((ri, upper));
                            adjacency[a][b] = true;
                            adjacency[b][a] = true;
                        }
                    }
                }
            }
        }
        self.stats.clique_edges += witnesses.len();
        let grow_deadline = deadline.min(started + std::time::Duration::from_millis(15));
        let mut candidates = BTreeMap::<Vec<usize>, f64>::new();
        for seed in 0..n.min(64) {
            if Instant::now() >= grow_deadline {
                break;
            }
            let mut clique = vec![seed];
            let mut possible: Vec<_> = (0..n).filter(|&i| adjacency[seed][i]).collect();
            while !possible.is_empty() && clique.len() < 32 {
                let next = *possible
                    .iter()
                    .max_by(|&&a, &&b| {
                        value(literals[a])
                            .total_cmp(&value(literals[b]))
                            .then(b.cmp(&a))
                    })
                    .unwrap();
                clique.push(next);
                possible.retain(|&i| adjacency[next][i] && literals[i] / 2 != literals[next] / 2);
            }
            if clique.len() < 3 {
                continue;
            }
            clique.sort_unstable();
            let violation = clique.iter().map(|&i| value(literals[i])).sum::<f64>() - 1.;
            if violation > 1e-7 {
                candidates.insert(clique, violation);
            }
        }
        let mut candidates: Vec<_> = candidates.into_iter().collect();
        candidates.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        let mut cuts = vec![];
        for (indices, _) in candidates {
            if Instant::now() >= deadline || cuts.len() >= limit {
                break;
            }
            let mut selected: Vec<_> = indices.iter().map(|&i| literals[i]).collect();
            selected.sort_unstable();
            let mut pairs = vec![];
            for k in 0..selected.len() {
                for &b in &selected[k + 1..] {
                    let a = selected[k];
                    let ia = lookup[&a];
                    let ib = lookup[&b];
                    let (ri, upper) = witnesses[&(ia.min(ib), ia.max(ib))];
                    pairs.push(Pair {
                        source_row: ri,
                        source_name: model.rows[ri].name.clone(),
                        upper_side: upper,
                        variables: [a / 2, b / 2],
                    });
                    self.stats.clique_work = self
                        .stats
                        .clique_work
                        .saturating_add(model.rows[ri].coefficients.len());
                }
            }
            if self.stats.clique_work >= 2_000_000 {
                break;
            }
            let coefficients: Vec<_> = selected
                .iter()
                .map(|&l| (l / 2, if l % 2 == 0 { 1. } else { -1. }))
                .collect();
            let rhs = 1. - selected.iter().filter(|&&l| l % 2 != 0).count() as f64;
            let key = (
                coefficients.iter().map(|&(j, a)| (j, a as i64)).collect(),
                rhs as i64,
            );
            if self.seen.contains(&key) {
                continue;
            }
            let first = &pairs[0];
            let cut = Cut {
                row: Row {
                    name: format!("clique_{}", self.seen.len()),
                    coefficients,
                    lower: None,
                    upper: Some(rhs),
                    lower_enclosure: None,
                    upper_enclosure: Some([rhs; 2]),
                    exact_integral: true,
                    exact_coefficients: true,
                    exact_coefficient_flags: vec![],
                },
                proof: Proof {
                    mir: None,
                    source_row: first.source_row,
                    source_name: first.source_name.clone(),
                    upper_side: first.upper_side,
                    cover: selected.iter().map(|l| l / 2).collect(),
                    clique: pairs,
                    aggregation: vec![],
                    divisor: None,
                },
            };
            if !verify_until(model, &cut, Some(deadline)) {
                self.stats.rejected += 1;
                continue;
            }
            self.seen.insert(key);
            self.stats.clique_added += 1;
            self.stats.added += 1;
            cuts.push(cut);
        }
        cuts
    }
}
