//! Single original-row mixed integer rounding, with exact checked integer scaling.
//! Mathematical definition: https://scipopt.org/doc-10.0.2/html/cuts_8c.php
//! No external implementation or optimization call is used.
use super::*;
use std::time::Duration;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MirProof {
    pub upper_bounds: Vec<usize>,
}
fn bound(model: &Model, j: usize, upper: bool) -> Option<i128> {
    let v = model.variables.get(j)?;
    let (b, enclosure) = if upper {
        (v.upper?, v.upper_enclosure?)
    } else {
        (v.lower?, v.lower_enclosure?)
    };
    (enclosure == [b; 2]).then_some(())?;
    integer(b)
}
pub fn applicable(model: &Model) -> bool {
    model.variables.iter().any(|v| v.integer)
        && model
            .rows
            .iter()
            .any(|r| r.exact_integral && r.coefficients.len() <= 4096)
}
fn derive(model: &Model, source: &Row, upper: bool, d: i64, complements: &[usize]) -> Option<Row> {
    if !source.exact_integral
        || source.coefficients.len() > 4096
        || !(2..=1i64 << 52).contains(&d)
        || complements.windows(2).any(|w| w[0] >= w[1])
    {
        return None;
    }
    let sign = if upper { 1 } else { -1 };
    let rhs = if upper { source.upper? } else { source.lower? };
    if (if upper {
        source.upper_enclosure?
    } else {
        source.lower_enclosure?
    }) != [rhs; 2]
    {
        return None;
    }
    let mut b = integer(rhs)?.checked_mul(sign)?;
    let mut terms = Vec::new();
    let mut previous = None;
    let mut used = 0;
    let mut any_integer = false;
    for &(j, a) in &source.coefficients {
        if previous.is_some_and(|k| k >= j) {
            return None;
        }
        previous = Some(j);
        let mut a = integer(a)?.checked_mul(sign)?;
        if a == 0 {
            continue;
        }
        let upper = complements.binary_search(&j).is_ok();
        used += usize::from(upper);
        let bnd = bound(model, j, upper)?;
        b = b.checked_sub(a.checked_mul(bnd)?)?;
        let direction = if upper { -1 } else { 1 };
        a = a.checked_mul(direction)?;
        let int = model.variables.get(j)?.integer;
        any_integer |= int;
        terms.push((j, a, bnd, direction, int));
    }
    if used != complements.len() || !any_integer {
        return None;
    }
    let d = d as i128;
    let r = b.rem_euclid(d);
    if r == 0 {
        return None;
    }
    let h = d - r;
    let mut rhs = h.checked_mul(b.div_euclid(d))?;
    let mut coefficients = Vec::new();
    for (j, a, bnd, direction, int) in terms {
        let q = if int {
            h.checked_mul(a.div_euclid(d))?
                .checked_add((a.rem_euclid(d) - r).max(0))?
        } else {
            a.min(0)
        };
        let c = q.checked_mul(direction)?;
        rhs = rhs.checked_add(c.checked_mul(bnd)?)?;
        if c.abs() > 1i128 << 52 {
            return None;
        }
        if c != 0 {
            coefficients.push((j, c as f64));
        }
    }
    if rhs.abs() > 1i128 << 52 || coefficients.is_empty() {
        return None;
    }
    Some(Row {integer_bounds:None,
        name: String::new(),
        coefficients,
        lower: None,
        upper: Some(rhs as f64),
        lower_enclosure: None,
        upper_enclosure: Some([rhs as f64; 2]),
        exact_integral: true,
        exact_coefficients: true,
        exact_coefficient_flags: vec![],
    })
}
pub(super) fn verify(model: &Model, cut: &Cut) -> bool {
    let p = &cut.proof;
    if !p.cover.is_empty() || !p.clique.is_empty() || !p.aggregation.is_empty() {
        return false;
    }
    let Some(source) = model.rows.get(p.source_row) else {
        return false;
    };
    if source.name != p.source_name {
        return false;
    }
    let Some(row) = p.divisor.and_then(|d| {
        derive(
            model,
            source,
            p.upper_side,
            d,
            &p.mir.as_ref().unwrap().upper_bounds,
        )
    }) else {
        return false;
    };
    cut.row.coefficients == row.coefficients
        && cut.row.upper == row.upper
        && cut.row.lower == None
        && cut.row.upper_enclosure == row.upper_enclosure
        && cut.row.lower_enclosure == None
        && cut.row.exact_integral
        && cut.row.exact_coefficients
        && cut.row.exact_coefficient_flags.is_empty()
}
impl Pool {
    pub fn separate_mir(
        &mut self,
        model: &Model,
        x: &[f64],
        limit: usize,
        deadline: Instant,
    ) -> Vec<Cut> {
        let started = Instant::now();
        let deadline = deadline.min(started + Duration::from_millis(20));
        let search_deadline = deadline
            .checked_sub(Duration::from_millis(3))
            .unwrap_or(started);
        self.stats.mir_calls += 1;
        let mut candidates = Vec::new();
        if x.len() != model.variables.len() || x.iter().any(|v| !v.is_finite()) {
            return vec![];
        }
        'rows: for (i, source) in model.rows.iter().enumerate() {
            // A unit-coefficient row intersected with integer box bounds has
            // integral vertices; MIR cannot separate a feasible point from it.
            if !source.exact_integral
                || source.coefficients.len() > 4096
                || source.coefficients.iter().all(|&(_, a)| a.abs() <= 1.)
            {
                continue;
            }
            let mut ds = vec![2, 3, 4, 5, 10];
            for &(_, a) in &source.coefficients {
                if a.abs() >= 2.
                    && a.abs() <= 64.
                    && a.fract() == 0.
                    && ds.len() < 24
                    && !ds.contains(&(a.abs() as i64))
                {
                    ds.push(a.abs() as i64);
                }
            }
            for mode in 0..3 {
                let mut complements = Vec::new();
                let mut valid = true;
                for &(j, a) in &source.coefficients {
                    if a == 0. {
                        continue;
                    }
                    if j >= x.len() {
                        valid = false;
                        break;
                    }
                    let lo = bound(model, j, false);
                    let hi = bound(model, j, true);
                    let upper = match (lo, hi) {
                        (Some(l), Some(h)) => {
                            mode == 2
                                || (mode == 0 && (h as f64 - x[j]).abs() < (x[j] - l as f64).abs())
                        }
                        (None, Some(_)) => true,
                        (Some(_), None) => false,
                        _ => {
                            valid = false;
                            break;
                        }
                    };
                    if upper {
                        complements.push(j);
                    }
                }
                if !valid {
                    continue;
                }
                for side in [true, false] {
                    for &d in &ds {
                        if Instant::now() >= search_deadline || self.stats.mir_work >= 1_000_000 {
                            break 'rows;
                        }
                        self.stats.mir_work = self
                            .stats
                            .mir_work
                            .saturating_add(source.coefficients.len());
                        let Some(row) = derive(model, source, side, d, &complements) else {
                            continue;
                        };
                        let violation =
                            row.coefficients.iter().map(|&(j, a)| a * x[j]).sum::<f64>()
                                - row.upper.unwrap();
                        if !violation.is_finite() || violation <= 1e-5 {
                            continue;
                        }
                        let norm = row
                            .coefficients
                            .iter()
                            .map(|&(_, a)| a * a)
                            .sum::<f64>()
                            .sqrt()
                            .max(1.);
                        candidates.push((
                            violation / norm,
                            Cut {
                                row,
                                proof: Proof {
                            gomory: None,
                                    mir: Some(MirProof {
                                        upper_bounds: complements.clone(),
                                    }),
                                    clique: vec![],
                                    aggregation: vec![],
                                    divisor: Some(d),
                                    source_row: i,
                                    source_name: source.name.clone(),
                                    upper_side: side,
                                    cover: vec![],
                                },
                            },
                        ));
                    }
                }
            }
        }
        candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mut cuts = Vec::new();
        for (_, mut cut) in candidates {
            if cuts.len() >= limit.min(8) || Instant::now() >= deadline {
                break;
            }
            if !super::verify(model, &cut) {
                self.stats.rejected += 1;
                continue;
            }
            let key = (
                cut.row
                    .coefficients
                    .iter()
                    .map(|&(j, a)| (j, a as i64))
                    .collect(),
                cut.row.upper.unwrap() as i64,
            );
            if self.seen.insert(key) {
                cut.row.name = format!("__mir_{}", self.seen.len());
                self.stats.mir_added += 1;
                self.stats.added += 1;
                cuts.push(cut);
            }
        }
        self.stats.mir_seconds += started.elapsed().as_secs_f64();
        cuts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Variable;
    fn model(a: [i32; 3], b: i32) -> Model {
        Model {
            name: "mir".into(),
            maximize: false,
            offset: 0.,
            offset_enclosure: Some([0.; 2]),
            decimal_source: None,
            original_decimal: false,
            variables: (0..3)
                .map(|j| Variable {integer_bounds:None,
                    name: format!("x{j}"),
                    objective: 0.,
                    objective_enclosure: Some([0.; 2]),
                    integer: j == 0,
                    lower: Some(-2.),
                    upper: Some(3.),
                    lower_enclosure: Some([-2.; 2]),
                    upper_enclosure: Some([3.; 2]),
                })
                .collect(),
            rows: vec![Row {integer_bounds:None,
                name: "r".into(),
                coefficients: a
                    .into_iter()
                    .enumerate()
                    .filter(|(_, a)| *a != 0)
                    .map(|(j, a)| (j, a as f64))
                    .collect(),
                lower: None,
                upper: Some(b as f64),
                lower_enclosure: None,
                upper_enclosure: Some([b as f64; 2]),
                exact_integral: true,
                exact_coefficients: true,
                exact_coefficient_flags: vec![],
            }],
        }
    }
    #[test]
    fn mixed_rows_preserve_all_continuous_polygon_vertices() {
        let mut count = 0;
        for seed in 0..160 {
            let a = [seed % 7 - 3, (seed / 7) % 7 - 3, (seed / 49) % 7 - 3];
            let b = seed % 11 - 5;
            let m = model(a, b);
            for mask in 0..8 {
                let upper: Vec<_> = (0..3)
                    .filter(|j| mask & (1 << j) != 0 && a[*j] != 0)
                    .collect();
                for d in 2..8 {
                    let Some(cut) = derive(&m, &m.rows[0], true, d, &upper) else {
                        continue;
                    };
                    count += 1;
                    for x in -2..=3 {
                        // Vertices: box corners and intersections of the source line with each box edge.
                        let mut points = vec![(-2., -2.), (-2., 3.), (3., -2.), (3., 3.)];
                        for edge in [-2., 3.] {
                            if a[2] != 0 {
                                points.push((
                                    edge,
                                    (b as f64 - a[0] as f64 * x as f64 - a[1] as f64 * edge)
                                        / a[2] as f64,
                                ));
                            }
                            if a[1] != 0 {
                                points.push((
                                    (b as f64 - a[0] as f64 * x as f64 - a[2] as f64 * edge)
                                        / a[1] as f64,
                                    edge,
                                ));
                            }
                        }
                        for (y, z) in points {
                            if y < -2. - 1e-10
                                || y > 3. + 1e-10
                                || z < -2. - 1e-10
                                || z > 3. + 1e-10
                            {
                                continue;
                            }
                            let p = [x as f64, y, z];
                            if (0..3).map(|j| a[j] as f64 * p[j]).sum::<f64>() > b as f64 + 1e-9 {
                                continue;
                            }
                            assert!(
                                cut.coefficients.iter().map(|&(j, c)| c * p[j]).sum::<f64>()
                                    <= cut.upper.unwrap() + 1e-8,
                                "{a:?} {b} {d} {upper:?} {p:?}"
                            );
                        }
                    }
                }
            }
        }
        assert!(count > 1000);
    }
    #[test]
    fn unbounded_continuous_term_and_corruption() {
        let mut m = model([3, -1, 0], 1);
        for v in &mut m.variables {
            v.lower = Some(0.);
            v.lower_enclosure = Some([0.; 2]);
            v.upper = None;
            v.upper_enclosure = None;
        }
        let row = derive(&m, &m.rows[0], true, 3, &[]).unwrap();
        assert_eq!(row.coefficients, vec![(0, 2.), (1, -1.)]);
        assert_eq!(row.upper, Some(0.));
        let mut cut = Cut {
            row,
            proof: Proof {
                            gomory: None,
                mir: Some(MirProof {
                    upper_bounds: vec![],
                }),
                clique: vec![],
                aggregation: vec![],
                divisor: Some(3),
                source_row: 0,
                source_name: "r".into(),
                upper_side: true,
                cover: vec![],
            },
        };
        assert!(super::verify(&m, &cut));
        cut.row.upper = Some(1.);
        assert!(!super::verify(&m, &cut));
        cut.row.upper = Some(0.);
        cut.proof.mir.as_mut().unwrap().upper_bounds.push(1);
        assert!(!super::verify(&m, &cut));
        cut.proof.mir.as_mut().unwrap().upper_bounds.clear();
        m.variables[0].lower_enclosure = Some([-1e-20, 0.]);
        assert!(derive(&m, &m.rows[0], true, 3, &[]).is_none());
        assert!(
            Pool::default()
                .separate_mir(&m, &[0.5, 0.5, 0.], 8, Instant::now())
                .is_empty()
        );
    }
}
