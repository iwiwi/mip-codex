//! Positive integer row aggregation followed by the existing exact CG rounding.
//! See AGGREGATE_CUT_DIAGNOSTIC; no external optimization implementation is used.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn applicable(model: &Model) -> bool {
    if !model
        .variables
        .iter()
        .any(|v| v.integer && finite(v.lower_enclosure).is_some_and(|b| b[0] >= 0.))
    {
        return false;
    }

    model
        .rows
        .iter()
        .filter(|r| {
            r.exact_integral
                && r.coefficients.len() <= 4096
                && (r.upper.is_some_and(|v| integer(v).is_some())
                    || r.lower.is_some_and(|v| integer(v).is_some()))
        })
        .take(2)
        .count()
        == 2
}
pub(super) fn source(model: &Model, terms: &[RowWeight]) -> Option<Row> {
    match terms {
        [a, b] if a.source_row != b.source_row => {}
        [a, b, c]
            if a.source_row != b.source_row
                && a.source_row != c.source_row
                && b.source_row != c.source_row => {}
        _ => return None,
    }
    let mut coefficients = BTreeMap::<usize, i128>::new();
    let mut rhs = 0i128;
    for term in terms {
        if term.weight == 0 || term.weight > 1u64 << 52 {
            return None;
        }
        let row = model.rows.get(term.source_row)?;
        if !row.exact_integral || row.name != term.source_name || row.coefficients.len() > 4096 {
            return None;
        }
        let weight = term.weight as i128 * if term.upper_side { 1 } else { -1 };
        rhs = rhs.checked_add(
            integer(if term.upper_side {
                row.upper?
            } else {
                row.lower?
            })?
            .checked_mul(weight)?,
        )?;
        let mut previous = None;
        for &(j, a) in &row.coefficients {
            if j >= model.variables.len() || previous.is_some_and(|p| p >= j) {
                return None;
            }
            previous = Some(j);
            let value = integer(a)?.checked_mul(weight)?;
            let entry = coefficients.entry(j).or_default();
            *entry = entry.checked_add(value)?;
        }
    }
    if rhs.abs() > 1i128 << 52 || coefficients.values().any(|a| a.abs() > 1i128 << 52) {
        return None;
    }
    Some(Row {
        name: String::new(),
        coefficients: coefficients
            .into_iter()
            .filter(|(_, a)| *a != 0)
            .map(|(j, a)| (j, a as f64))
            .collect(),
        lower: None,
        upper: Some(rhs as f64),
        lower_enclosure: None,
        upper_enclosure: Some([rhs as f64; 2]),
        exact_integral: true,
        exact_coefficients: true,
        exact_coefficient_flags: vec![],
    })
}
struct Side {
    term: RowWeight,
    coefficients: Vec<(usize, i64)>,
    score: f64,
}
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}
fn divisors(row: &Row) -> Vec<i64> {
    let mut values = vec![2, 3, 4, 5, 10];
    let mut common = 0;
    for &(_, a) in &row.coefficients {
        let b = (a as i64).unsigned_abs();
        common = gcd(common, b);
        if b >= 2 && values.len() < 24 && !values.contains(&(b as i64)) {
            values.push(b as i64);
        }
    }
    if common >= 2 && !values.contains(&(common as i64)) {
        values.push(common as i64);
    }
    values
}
impl Pool {
    pub fn separate_aggregate(
        &mut self,
        model: &Model,
        x: &[f64],
        limit: usize,
        deadline: Instant,
    ) -> Vec<Cut> {
        self.separate_aggregate_mode::<false>(model, x, limit, deadline)
    }
    pub fn separate_three_aggregate(
        &mut self,
        model: &Model,
        x: &[f64],
        limit: usize,
        deadline: Instant,
    ) -> Vec<Cut> {
        self.separate_aggregate_mode::<true>(model, x, limit, deadline)
    }
    fn separate_aggregate_mode<const THREE: bool>(
        &mut self,
        model: &Model,
        x: &[f64],
        limit: usize,
        deadline: Instant,
    ) -> Vec<Cut> {
        let started = Instant::now();
        self.stats.aggregate_calls += 1;
        if limit == 0 || x.len() != model.variables.len() || x.iter().any(|v| !v.is_finite()) {
            return vec![];
        }
        let mut sides = Vec::new();
        for (i, row) in model.rows.iter().enumerate() {
            if Instant::now() >= deadline || self.stats.aggregate_work >= 1_000_000 {
                break;
            }
            if !row.exact_integral || row.coefficients.len() > 4096 {
                continue;
            }
            for upper in [true, false] {
                let Some(bound) = (if upper { row.upper } else { row.lower }).and_then(integer)
                else {
                    continue;
                };
                let sign = if upper { 1i64 } else { -1 };
                let mut coefficients = Vec::new();
                let mut valid = true;
                for &(j, a) in &row.coefficients {
                    let Some(a) = integer(a) else {
                        valid = false;
                        break;
                    };
                    if j >= x.len() {
                        valid = false;
                        break;
                    }
                    coefficients.push((j, sign * a as i64));
                }
                if !valid {
                    continue;
                }
                let rhs = sign as f64 * bound as f64;
                let activity = coefficients
                    .iter()
                    .map(|&(j, a)| a as f64 * x[j])
                    .sum::<f64>();
                let norm = (1.
                    + coefficients
                        .iter()
                        .map(|&(_, a)| (a as f64).powi(2))
                        .sum::<f64>())
                .sqrt();
                let score = (rhs - activity) / norm;
                if !score.is_finite() {
                    continue;
                }
                sides.push(Side {
                    term: RowWeight {
                        source_row: i,
                        source_name: row.name.clone(),
                        upper_side: upper,
                        weight: 1,
                    },
                    coefficients,
                    score,
                });
            }
        }
        sides.sort_by(|a, b| {
            a.score
                .total_cmp(&b.score)
                .then(a.term.source_row.cmp(&b.term.source_row))
                .then(a.term.upper_side.cmp(&b.term.upper_side))
        });
        sides.truncate(512);
        let mut incidence = vec![Vec::<(usize, i64)>::new(); x.len()];
        for (i, row) in sides.iter().enumerate() {
            for &(j, a) in &row.coefficients {
                let v = &model.variables[j];
                let active = if v.integer {
                    (x[j] - x[j].round()).abs() > 1e-7
                } else {
                    v.lower.is_none_or(|b| (x[j] - b).abs() > 1e-7)
                        && v.upper.is_none_or(|b| (x[j] - b).abs() > 1e-7)
                };
                if active && incidence[j].len() < 16 {
                    incidence[j].push((i, a));
                }
            }
        }
        let mut pairs = BTreeMap::<(usize, usize), BTreeSet<(u64, u64)>>::new();
        'columns: for column in &incidence {
            for k in 0..column.len() {
                for l in k + 1..column.len() {
                    let (a, ca) = column[k];
                    let (b, cb) = column[l];
                    if sides[a].term.source_row == sides[b].term.source_row {
                        continue;
                    }
                    let weights = pairs.entry((a, b)).or_default();
                    weights.insert((1, 1));
                    if ca.signum() != cb.signum() && ca != 0 && cb != 0 {
                        let divisor = gcd(ca.unsigned_abs(), cb.unsigned_abs());
                        weights.insert((cb.unsigned_abs() / divisor, ca.unsigned_abs() / divisor));
                    }
                    if pairs.len() >= 4096 {
                        break 'columns;
                    }
                }
            }
        }
        let mut candidates = Vec::new();
        'pairs: for ((a, b), weights) in pairs {
            self.stats.aggregate_pairs += 1;
            for (wa, wb) in weights {
                if Instant::now() >= deadline || self.stats.aggregate_work >= 1_000_000 {
                    break 'pairs;
                }
                let mut terms = vec![sides[a].term.clone(), sides[b].term.clone()];
                terms[0].weight = wa;
                terms[1].weight = wb;
                let Some(combined) = source(model, &terms) else {
                    continue;
                };
                let mut variants = Vec::new();
                if THREE {
                    let mut used = BTreeSet::new();
                    'third: for &(j, coefficient) in &combined.coefficients {
                        let coefficient = coefficient as i64;
                        for &(k, other) in &incidence[j] {
                            if sides[k].term.source_row == terms[0].source_row
                                || sides[k].term.source_row == terms[1].source_row
                                || other == 0
                                || coefficient.signum() == other.signum()
                            {
                                continue;
                            }
                            let common = gcd(coefficient.unsigned_abs(), other.unsigned_abs());
                            if common == 0 {
                                continue;
                            }
                            let multiplier = other.unsigned_abs() / common;
                            let mut triple = terms.clone();
                            let Some(a) = triple[0].weight.checked_mul(multiplier) else {
                                continue;
                            };
                            let Some(b) = triple[1].weight.checked_mul(multiplier) else {
                                continue;
                            };
                            triple[0].weight = a;
                            triple[1].weight = b;
                            let mut third = sides[k].term.clone();
                            third.weight = coefficient.unsigned_abs() / common;
                            triple.push(third);
                            let mut key: Vec<_> = triple
                                .iter()
                                .map(|t| (t.source_row, t.upper_side, t.weight))
                                .collect();
                            key.sort_unstable();
                            if !used.insert(key) {
                                continue;
                            }
                            self.stats.aggregate_work = self.stats.aggregate_work.saturating_add(
                                combined.coefficients.len() + sides[k].coefficients.len(),
                            );
                            if Instant::now() >= deadline || self.stats.aggregate_work >= 1_000_000
                            {
                                break 'third;
                            }
                            if let Some(row) = source(model, &triple) {
                                variants.push((triple, row));
                            }
                            if variants.len() >= 8 {
                                break 'third;
                            }
                        }
                    }
                }
                let mut singleton = (!THREE).then_some((terms, combined));
                let mut many = variants.into_iter();
                for (terms, combined) in
                    std::iter::from_fn(|| if THREE { many.next() } else { singleton.take() })
                {
                    for d in divisors(&combined) {
                        if Instant::now() >= deadline || self.stats.aggregate_work >= 1_000_000 {
                            break 'pairs;
                        }
                        self.stats.aggregate_work = self
                            .stats
                            .aggregate_work
                            .saturating_add(combined.coefficients.len());
                        let Some(row) = rounding_row(model, &combined, true, d) else {
                            continue;
                        };
                        let rhs = row.upper.unwrap();
                        let amount =
                            row.coefficients.iter().map(|&(j, a)| a * x[j]).sum::<f64>() - rhs;
                        if !amount.is_finite() || amount <= 1e-5 {
                            continue;
                        }
                        let norm =
                            (1. + row.coefficients.iter().map(|&(_, a)| a * a).sum::<f64>()).sqrt();
                        let cut = Cut {
                            row,
                            proof: Proof {
                            gomory: None,
                                mir: None,
                                clique: vec![],
                                aggregation: terms.clone(),
                                divisor: Some(d),
                                source_row: terms[0].source_row,
                                source_name: terms[0].source_name.clone(),
                                upper_side: terms[0].upper_side,
                                cover: vec![],
                            },
                        };
                        self.stats.candidates += 1;
                        if verify(model, &cut) {
                            candidates.push((amount / norm, cut));
                        } else {
                            self.stats.rejected += 1;
                        }
                    }
                }
            }
        }
        candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mut cuts = Vec::new();
        for (_, mut cut) in candidates {
            let key = (
                cut.row
                    .coefficients
                    .iter()
                    .map(|&(j, a)| (j, a as i64))
                    .collect(),
                cut.row.upper.unwrap() as i64,
            );
            if self.seen.insert(key) {
                cut.row.name = if THREE {
                    format!("__aggregate_three_rounding_{}", self.seen.len())
                } else {
                    format!("__aggregate_rounding_{}", self.seen.len())
                };
                self.stats.added += 1;
                self.stats.aggregate_added += 1;
                cuts.push(cut);
                if cuts.len() >= limit {
                    break;
                }
            }
        }
        self.stats.aggregate_seconds += started.elapsed().as_secs_f64();
        cuts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    fn toy(lower: bool) -> Model {
        let kind = if lower { "G" } else { "L" };
        let sign = if lower { -1 } else { 1 };
        crate::mps::parse(std::io::Cursor::new(format!("NAME aggregate\nROWS\n N obj\n L a\n {kind} b\nCOLUMNS\n x obj -1 a 2\n z obj -1 b {}\n t a 1 b {}\nRHS\n rhs a 2 b {sign}\nBOUNDS\n BV bd x\n BV bd z\n FR bd t\nENDATA\n",2*sign,-sign))).unwrap()
    }
    #[test]
    fn cancels_free_continuous_variable_and_rejects_corruption() {
        for lower in [false, true] {
            let model = toy(lower);
            assert!(applicable(&model));
            let mut continuous = model.clone();
            for variable in &mut continuous.variables {
                variable.integer = false;
            }
            assert!(!applicable(&continuous));
            let x = [1., 0.5, 0.];
            assert!(model.check(&x, false, 1e-12).is_ok());
            assert!(
                Pool::default()
                    .separate_rounding(&model, &x, 32, Instant::now() + Duration::from_secs(1))
                    .is_empty()
            );
            let mut pool = Pool::default();
            let cuts =
                pool.separate_aggregate(&model, &x, 32, Instant::now() + Duration::from_secs(1));
            let cut = cuts
                .iter()
                .find(|c| c.row.coefficients == vec![(0, 1.), (1, 1.)] && c.row.upper == Some(1.))
                .expect("new aggregate cut");
            assert!(verify(&model, cut));
            for mode in 0..6 {
                let mut bad = cut.clone();
                match mode {
                    0 => bad.row.upper = Some(0.),
                    1 => bad.proof.aggregation[1].weight = 0,
                    2 => bad.proof.aggregation[1].source_name.push('!'),
                    3 => bad.proof.aggregation[1].source_row = bad.proof.aggregation[0].source_row,
                    4 => bad.proof.divisor = Some(0),
                    _ => bad.proof.aggregation[1].weight = u64::MAX,
                }
                assert!(!verify(&model, &bad));
            }
            assert!(
                pool.separate_aggregate(&model, &x, 32, Instant::now() + Duration::from_secs(1))
                    .is_empty()
            );
            let mut decimal = model.clone();
            decimal.rows[0].exact_integral = false;
            assert!(
                Pool::default()
                    .separate_aggregate(&decimal, &x, 32, Instant::now() + Duration::from_secs(1))
                    .is_empty()
            );
        }
    }
    #[test]
    fn three_rows_cancel_two_free_variables_and_validate_original_sides() {
        for lower in [false, true] {
            let sign = if lower { -1 } else { 1 };
            let kind = if lower { "G" } else { "L" };
            let text = format!(
                "NAME aggregate3\nROWS\n N obj\n L a\n {kind} b\n L c\nCOLUMNS\n x obj -1 a 3\n z obj -1 c 3\n t a 2 b {}\n u b {} c -6\nRHS\n rhs a 3 b 0\n rhs c 1\nBOUNDS\n BV bd x\n BV bd z\n FR bd t\n FR bd u\nENDATA\n",
                -sign,
                3 * sign
            );
            let model = crate::mps::parse(std::io::Cursor::new(text)).unwrap();
            let terms: Vec<_> = [(0, true, 1), (1, !lower, 2), (2, true, 1)]
                .into_iter()
                .map(|(i, upper_side, weight)| RowWeight {
                    source_row: i,
                    source_name: model.rows[i].name.clone(),
                    upper_side,
                    weight,
                })
                .collect();
            let point = [1., 1. / 3., 0., 0.];
            assert!(model.check(&point, false, 1e-12).is_ok());
            for indices in [[0, 1], [0, 2], [1, 2]] {
                let pair = [terms[indices[0]].clone(), terms[indices[1]].clone()];
                let combined = source(&model, &pair).unwrap();
                assert!(rounding_row(&model, &combined, true, 3).is_none());
            }
            let combined = source(&model, &terms).unwrap();
            let row = rounding_row(&model, &combined, true, 3).unwrap();
            assert_eq!(row.coefficients, vec![(0, 1.), (1, 1.)]);
            assert_eq!(row.upper, Some(1.));
            let cut = Cut {
                row,
                proof: Proof {
                            gomory: None,
                    mir: None,
                    clique: vec![],
                    aggregation: terms,
                    divisor: Some(3),
                    source_row: 0,
                    source_name: "a".into(),
                    upper_side: true,
                    cover: vec![],
                },
            };
            assert!(verify(&model, &cut));
            let generated = Pool::default().separate_three_aggregate(
                &model,
                &point,
                8,
                Instant::now() + Duration::from_secs(1),
            );
            assert!(
                generated
                    .iter()
                    .any(|c| c.row.coefficients == cut.row.coefficients
                        && c.row.upper == cut.row.upper
                        && c.proof.aggregation.len() == 3)
            );
            assert!(
                Pool::default()
                    .separate_aggregate(&model, &point, 8, Instant::now() + Duration::from_secs(1))
                    .is_empty()
            );
            for x in 0..=1 {
                for z in 0..=1 {
                    for t in -8..=8 {
                        for u in -8..=8 {
                            let point = [x as f64, z as f64, t as f64 / 4., u as f64 / 4.];
                            if model.check(&point, true, 1e-12).is_ok() {
                                assert!(point[0] + point[1] <= cut.row.upper.unwrap());
                            }
                        }
                    }
                }
            }
            for mode in 0..6 {
                let mut bad = cut.clone();
                match mode {
                    0 => bad.proof.aggregation[2].weight = 0,
                    1 => bad.proof.aggregation[2].source_row = 0,
                    2 => bad.proof.aggregation[2].source_name.push('!'),
                    3 => bad.proof.aggregation.push(bad.proof.aggregation[0].clone()),
                    4 => bad.row.upper = Some(0.),
                    _ => {
                        bad.proof.aggregation.pop();
                    }
                }
                assert!(!verify(&model, &bad));
            }
        }
    }
    #[test]
    fn aggregate_cuts_preserve_every_binary_feasible_point() {
        let mut seed = 2026111522u64;
        for _ in 0..128 {
            let mut draw = || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                (seed >> 32) as i64
            };
            let coefficients: Vec<Vec<i64>> = (0..6)
                .map(|_| (0..8).map(|_| draw() % 13 - 6).collect())
                .collect();
            let rhs: Vec<_> = (0..6).map(|_| draw() % 11).collect();
            let mut text = String::from("NAME enumerate\nROWS\n N obj\n");
            for i in 0..6 {
                text += &format!(" L r{i}\n");
            }
            text += "COLUMNS\n";
            for j in 0..8 {
                text += &format!(" x{j} obj 0\n");
                for i in 0..6 {
                    text += &format!(" x{j} r{i} {}\n", coefficients[i][j]);
                }
            }
            text += "RHS\n";
            for i in 0..6 {
                text += &format!(" rhs r{i} {}\n", rhs[i]);
            }
            text += "BOUNDS\n";
            for j in 0..8 {
                text += &format!(" BV bd x{j}\n");
            }
            text += "ENDATA\n";
            let model = crate::mps::parse(std::io::Cursor::new(text)).unwrap();
            let cuts = Pool::default().separate_aggregate(
                &model,
                &[0.5; 8],
                32,
                Instant::now() + Duration::from_secs(1),
            );
            for cut in &cuts {
                assert!(verify(&model, cut));
            }
            for bits in 0..256 {
                let point: Vec<_> = (0..8).map(|j| ((bits >> j) & 1) as f64).collect();
                if coefficients.iter().zip(&rhs).all(|(row, &b)| {
                    row.iter()
                        .enumerate()
                        .map(|(j, &a)| a * ((bits >> j) & 1) as i64)
                        .sum::<i64>()
                        <= b
                }) {
                    for cut in &cuts {
                        assert!(
                            cut.row
                                .coefficients
                                .iter()
                                .map(|&(j, a)| a * point[j])
                                .sum::<f64>()
                                <= cut.row.upper.unwrap()
                        );
                    }
                }
            }
        }
    }
}
