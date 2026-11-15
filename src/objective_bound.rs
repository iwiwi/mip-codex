//! 原データのLagrangian下界を外向き計算し、証明された目的latticeへ丸める。
use crate::{exact, model::Model};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Implication {
    pub variable: usize,
    pub source_row: usize,
    pub source_name: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grid {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_domain: Option<Domain>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_domain: Option<Domain>,
    pub offset: i64,
    pub step: i64,
    pub implied: Vec<Implication>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Domain {
    pub base: i64,
    pub first_increment: i64,
}
fn domain(model: &Model, sign: i64) -> Option<Domain> {
    let mut base =
        (small_integer(point(model.offset, model.offset_enclosure)?)? as i128) * sign as i128;
    let mut first = i64::MAX;
    for v in &model.variables {
        let c = small_integer(point(v.objective, v.objective_enclosure)?)?.checked_mul(sign)?;
        if c == 0 {
            continue;
        }
        let (bound, enclosure) = if c > 0 {
            (v.lower?, v.lower_enclosure)
        } else {
            (v.upper?, v.upper_enclosure)
        };
        let bound = small_integer(point(bound, enclosure)?)?;
        base = base.checked_add((c as i128).checked_mul(bound as i128)?)?;
        first = first.min(c.abs());
    }
    (base.abs() <= (1i128 << 52) && first != i64::MAX).then_some(Domain {
        base: base as i64,
        first_increment: first,
    })
}
fn small_integer(v: f64) -> Option<i64> {
    (v.is_finite() && v.fract() == 0. && v.abs() <= (1u64 << 52) as f64).then_some(v as i64)
}
fn point(v: f64, b: Option<[f64; 2]>) -> Option<f64> {
    b.filter(|b| b[0].is_finite() && b[0] == v && b[1] == v)
        .map(|b| b[0])
}
impl Grid {
    pub fn upper_box(
        &self,
        model: &Model,
        lower: &[Option<f64>],
        upper: &[Option<f64>],
    ) -> Option<i128> {
        if lower.len() != model.variables.len() || upper.len() != lower.len() {
            return None;
        }
        let sign = if model.maximize { -1i64 } else { 1 };
        let mut value = (self.offset as i128) * sign as i128;
        for (j, v) in model.variables.iter().enumerate() {
            let c = small_integer(point(v.objective, v.objective_enclosure)?)?.checked_mul(sign)?;
            if c == 0 {
                continue;
            }
            let bound = small_integer(if c > 0 { upper[j]? } else { lower[j]? })?;
            value = value.checked_add((c as i128).checked_mul(bound as i128)?)?;
        }
        Some(value)
    }
    pub fn from_model(model: &Model) -> Option<Self> {
        let offset = small_integer(point(model.offset, model.offset_enclosure)?)?;
        let mut step = 0i64;
        let mut needed = vec![false; model.variables.len()];
        for (j, v) in model.variables.iter().enumerate() {
            let c = small_integer(point(v.objective, v.objective_enclosure)?)?;
            let mut a = c.abs();
            while a != 0 {
                (step, a) = (a, step % a);
            }
            needed[j] = c != 0;
        }
        if step == 0 {
            return None;
        }
        let mut known: Vec<_> = model.variables.iter().map(|v| v.integer).collect();
        let mut unknown = needed
            .iter()
            .zip(&known)
            .filter(|(need, known)| **need && !**known)
            .count();
        let mut implied = Vec::new();
        if unknown != 0 {
            let mut adjacency = vec![vec![]; known.len()];
            let mut remaining = vec![0usize; model.rows.len()];
            let mut queue = VecDeque::new();
            for (i, row) in model.rows.iter().enumerate() {
                if !row.exact_integral
                    || row
                        .coefficients
                        .iter()
                        .any(|&(_, a)| small_integer(a).is_none())
                    || !row.lower.zip(row.upper).is_some_and(|(l, u)| {
                        l == u
                            && small_integer(l).is_some()
                            && point(l, row.lower_enclosure).is_some()
                            && point(u, row.upper_enclosure).is_some()
                    })
                {
                    continue;
                }
                for &(j, a) in &row.coefficients {
                    if a != 0. && !known.get(j).copied()? {
                        remaining[i] += 1;
                        adjacency[j].push(i);
                    }
                }
                if remaining[i] == 1 {
                    queue.push_back(i);
                }
            }
            while unknown != 0 {
                let Some(i) = queue.pop_front() else {
                    break;
                };
                if remaining[i] != 1 {
                    continue;
                }
                let row = &model.rows[i];
                let &(j, a) = row
                    .coefficients
                    .iter()
                    .find(|&&(j, a)| a != 0. && !known[j])?;
                if a.abs() != 1. {
                    continue;
                }
                known[j] = true;
                if needed[j] {
                    unknown -= 1;
                }
                implied.push(Implication {
                    variable: j,
                    source_row: i,
                    source_name: row.name.clone(),
                });
                for &i in &adjacency[j] {
                    remaining[i] -= 1;
                    if remaining[i] == 1 {
                        queue.push_back(i);
                    }
                }
            }
        }
        if unknown != 0 {
            return None;
        }
        Some(Self {
            min_domain: domain(model, 1),
            max_domain: domain(model, -1),
            offset,
            step,
            implied,
        })
    }
    /// Nodeの境界が原境界と整数分枝だけから成る、初版の保守的な対象。
    pub fn supported_bounds(model: &Model) -> bool {
        model.variables.iter().all(|v| {
            [(v.lower, v.lower_enclosure), (v.upper, v.upper_enclosure)]
                .into_iter()
                .all(|(value, b)| {
                    value.is_none_or(|x| point(x, b).and_then(small_integer).is_some())
                })
        })
    }
    pub fn round_lower(&self, lower: f64, maximize: bool) -> Option<f64> {
        if self.step <= 0 || self.step > (1i64 << 52) || self.offset.unsigned_abs() > (1u64 << 52) {
            return None;
        }
        let offset = if maximize { -self.offset } else { self.offset };
        let delta = I::point(lower)?.sub(I::point(offset as f64)?)?;
        let k = (delta.lo / self.step as f64).next_down().ceil();
        if !k.is_finite() || k.abs() > (1u64 << 52) as f64 {
            return None;
        }
        let result = (offset as i128).checked_add((self.step as i128).checked_mul(k as i128)?)?;
        if result.abs() > (1i128 << 52) {
            return None;
        }
        let mut result = result as f64;
        let domain = if maximize {
            &self.max_domain
        } else {
            &self.min_domain
        };
        if let Some(domain) = domain {
            if domain.first_increment <= 0 {
                return None;
            }
            let base = domain.base as i128;
            if base.abs() > (1i128 << 52) {
                return None;
            }
            result = result.max(domain.base as f64);
            if lower > domain.base as f64 {
                let next = base.checked_add(domain.first_increment as i128)?;
                if next.abs() <= (1i128 << 52) {
                    result = result.max(next as f64);
                }
            }
        }
        Some(result)
    }
}
#[derive(Clone, Copy, Debug)]
struct I {
    lo: f64,
    hi: f64,
}
impl I {
    fn new(lo: f64, hi: f64) -> Option<Self> {
        (lo.is_finite() && hi.is_finite() && lo <= hi).then_some(Self { lo, hi })
    }
    fn point(x: f64) -> Option<Self> {
        Self::new(x, x)
    }
    fn metadata(value: f64, b: Option<[f64; 2]>) -> Option<Self> {
        let [lo, hi] = b?;
        let i = Self::new(lo, hi)?;
        (lo <= value && value <= hi).then_some(i)
    }
    fn neg(self) -> Self {
        Self {
            lo: -self.hi,
            hi: -self.lo,
        }
    }
    fn zero(self) -> bool {
        self.lo == 0. && self.hi == 0.
    }
    fn add(self, b: Self) -> Option<Self> {
        if self.zero() {
            return Some(b);
        }
        if b.zero() {
            return Some(self);
        }
        if self.lo == self.hi && b.lo == b.hi && self.lo == -b.lo {
            return Self::point(0.);
        }
        Self::new((self.lo + b.lo).next_down(), (self.hi + b.hi).next_up())
    }
    fn sub(self, b: Self) -> Option<Self> {
        self.add(b.neg())
    }
    fn mul(self, b: Self) -> Option<Self> {
        if self.zero() || b.zero() {
            return Self::point(0.);
        }
        if self.lo == 1. && self.hi == 1. {
            return Some(b);
        }
        if b.lo == 1. && b.hi == 1. {
            return Some(self);
        }
        if self.lo == -1. && self.hi == -1. {
            return Some(b.neg());
        }
        if b.lo == -1. && b.hi == -1. {
            return Some(self.neg());
        }
        let v = [
            self.lo * b.lo,
            self.lo * b.hi,
            self.hi * b.lo,
            self.hi * b.hi,
        ];
        Self::new(
            v.into_iter()
                .map(f64::next_down)
                .fold(f64::INFINITY, f64::min),
            v.into_iter()
                .map(f64::next_up)
                .fold(f64::NEG_INFINITY, f64::max),
        )
    }
}
pub struct Certifier {
    implied_box: Option<(Vec<Option<f64>>, Vec<Option<f64>>)>,
    costs: Vec<I>,
    offset: I,
    rows: Vec<[Option<I>; 2]>,
    columns: Vec<Vec<(usize, I)>>,
}
impl Certifier {
    pub fn new(model: &Model) -> Option<Self> {
        Self::with_objective(model, false)
    }
    pub fn feasibility(model: &Model) -> Option<Self> {
        Self::with_objective(model, true)
    }
    fn with_objective(model: &Model, feasibility: bool) -> Option<Self> {
        let sign = if model.maximize { -1. } else { 1. };
        let s = I::point(sign)?;
        let costs: Option<Vec<_>> = model
            .variables
            .iter()
            .map(|v| {
                if feasibility {
                    I::point(0.)
                } else {
                    I::metadata(v.objective, v.objective_enclosure)?.mul(s)
                }
            })
            .collect();
        let mut rows = Vec::new();
        let mut columns = vec![vec![]; model.variables.len()];
        for (i, row) in model.rows.iter().enumerate() {
            rows.push([
                row.lower.and_then(|v| I::metadata(v, row.lower_enclosure)),
                row.upper.and_then(|v| I::metadata(v, row.upper_enclosure)),
            ]);
            for (index, &(j, a)) in row.coefficients.iter().enumerate() {
                let interval = if row.exact_integral
                    || row.exact_coefficients
                    || row.exact_coefficient_flags.get(index) == Some(&true)
                {
                    I::point(a)?
                } else {
                    I::new(a.next_down(), a.next_up())?
                };
                columns.get_mut(j)?.push((i, interval));
            }
        }
        let mut result = Self {
            implied_box: None,
            costs: costs?,
            offset: if feasibility {
                I::point(0.)?
            } else {
                I::metadata(model.offset, model.offset_enclosure)?.mul(s)?
            },
            rows,
            columns,
        };
        if feasibility {
            result.implied_box = Some(result.infer_finite_box(model));
        }
        Some(result)
    }
    // Only fill missing sides. These bounds are used in certificates, never LP search.
    fn infer_finite_box(&self, model: &Model) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
        let mut lower: Vec<_> = model
            .variables
            .iter()
            .map(|v| {
                v.lower
                    .and_then(|x| I::metadata(x, v.lower_enclosure))
                    .map(|b| b.lo)
            })
            .collect();
        let mut upper: Vec<_> = model
            .variables
            .iter()
            .map(|v| {
                v.upper
                    .and_then(|x| I::metadata(x, v.upper_enclosure))
                    .map(|b| b.hi)
            })
            .collect();
        let mut rows = vec![Vec::new(); self.rows.len()];
        for (j, col) in self.columns.iter().enumerate() {
            for &(i, a) in col {
                rows[i].push((j, a));
            }
        }
        for _ in 0..4 {
            let mut changed = false;
            for (i, row) in rows.iter().enumerate() {
                for (sign, side) in [(1., 1), (-1., 0)] {
                    let Some(rhs) = self.rows[i][side].and_then(|b| b.mul(I::point(sign)?)) else {
                        continue;
                    };
                    let mut terms = Vec::with_capacity(row.len());
                    let mut total = Some(I::point(0.).unwrap());
                    let mut unknown = 0;
                    for &(j, a) in row {
                        let a = if sign > 0. {
                            a
                        } else {
                            I {
                                lo: -a.hi,
                                hi: -a.lo,
                            }
                        };
                        let domain = match (lower[j], upper[j]) {
                            (Some(l), Some(u)) => I::new(l, u),
                            (Some(l), None) if a.lo >= 0. => I::point(l),
                            (None, Some(u)) if a.hi <= 0. => I::point(u),
                            _ => None,
                        };
                        let minimum = if a.zero() {
                            Some(0.)
                        } else {
                            domain.and_then(|d| a.mul(d)).map(|p| p.lo)
                        };
                        if let Some(v) = minimum {
                            total = total.and_then(|t| t.add(I::point(v)?));
                        } else {
                            unknown += 1;
                        }
                        terms.push((j, a, minimum));
                    }
                    let Some(total) = total else {
                        continue;
                    };
                    for (j, a, minimum) in terms {
                        if !(a.lo > 0. && upper[j].is_none() || a.hi < 0. && lower[j].is_none()) {
                            continue;
                        }
                        let others = match (unknown, minimum) {
                            (0, Some(v)) => total.sub(I::point(v).unwrap()).map(|v| v.lo),
                            (1, None) => Some(total.lo),
                            _ => None,
                        };
                        let Some(numerator) =
                            others.and_then(|v| rhs.sub(I::point(v)?)).map(|v| v.hi)
                        else {
                            continue;
                        };
                        let a1 = numerator / a.lo;
                        let a2 = numerator / a.hi;
                        if a.lo > 0. {
                            let value = a1.max(a2).next_up();
                            if value.is_finite() {
                                upper[j] = Some(value);
                                changed = true;
                            }
                        } else {
                            let value = a1.min(a2).next_down();
                            if value.is_finite() {
                                lower[j] = Some(value);
                                changed = true;
                            }
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        (lower, upper)
    }
    /// 任意の有限行乗数に対する、指定されたbox内の元問題の下界。
    /// 乗数が双対実行可能であるとの仮定は置かない。
    /// Propose joint perturbations; only `bound` can accept a certificate.
    pub(crate) fn joint_positive_bound(
        &self,
        lower: &[Option<f64>],
        upper: &[Option<f64>],
        weights: &[f64],
        deadline: Option<std::time::Instant>,
    ) -> (Option<f64>, usize) {
        let mut active: Vec<_> = weights
            .iter()
            .enumerate()
            .filter(|(_, v)| **v != 0.)
            .map(|(j, v)| (j, v.abs()))
            .collect();
        active.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        active.truncate(8);
        if active.len() < 2 || weights.iter().any(|v| !v.is_finite()) {
            return (None, 0);
        }
        let mut candidate = weights.to_vec();
        let mut trials = 0;
        for sign in [1., -1.] {
            for mask in 0..(1usize << active.len()) {
                if deadline.is_some_and(|d| std::time::Instant::now() >= d) {
                    return (None, trials);
                }
                for (v, w) in candidate.iter_mut().zip(weights) {
                    *v = sign * w;
                }
                for (k, &(j, _)) in active.iter().enumerate() {
                    let change = if mask & (1 << k) == 0 { -1. } else { 1. };
                    candidate[j] *= 1. + change * 2f64.powi(-32);
                }
                trials += 1;
                if let Some(b) = self.bound(lower, upper, &candidate).filter(|b| *b > 0.) {
                    return (Some(b), trials);
                }
            }
        }
        (None, trials)
    }
    pub fn bound(
        &self,
        lower: &[Option<f64>],
        upper: &[Option<f64>],
        multipliers: &[f64],
    ) -> Option<f64> {
        if lower.len() != self.costs.len()
            || upper.len() != lower.len()
            || multipliers.len() != self.rows.len()
            || multipliers.iter().any(|v| !v.is_finite())
        {
            return None;
        }
        let lower_extended = self.implied_box.as_ref().map(|(l, _)| {
            lower
                .iter()
                .zip(l)
                .map(|(&a, &b)| a.or(b))
                .collect::<Vec<_>>()
        });
        let upper_extended = self.implied_box.as_ref().map(|(_, u)| {
            upper
                .iter()
                .zip(u)
                .map(|(&a, &b)| a.or(b))
                .collect::<Vec<_>>()
        });
        let lower = lower_extended.as_deref().unwrap_or(lower);
        let upper = upper_extended.as_deref().unwrap_or(upper);
        let mut y = multipliers.to_vec();
        let mut total = self.offset;
        for (i, v) in y.iter_mut().enumerate() {
            if *v == 0. {
                continue;
            }
            if let Some(b) = self.rows[i][usize::from(*v < 0.)] {
                total = total.add(I::point(*v)?.mul(b)?)?;
            } else {
                *v = 0.;
            }
        }
        for j in 0..self.costs.len() {
            let mut r = self.costs[j];
            for &(i, a) in &self.columns[j] {
                if y[i] != 0. {
                    r = r.sub(I::point(y[i])?.mul(a)?)?;
                }
            }
            if (lower[j].is_none() && r.hi > 0.) || (upper[j].is_none() && r.lo < 0.) {
                let mut terms = vec![(self.costs[j].lo, 1.)];
                if self.costs[j].lo != self.costs[j].hi {
                    return None;
                }
                for &(i, a) in &self.columns[j] {
                    if y[i] != 0. {
                        if a.lo != a.hi {
                            return None;
                        }
                        terms.push((-y[i], a.lo));
                    }
                }
                let [lo, hi] = exact::dot_enclosure(terms.into_iter())?;
                r = I::new(lo, hi)?;
            }
            if (lower[j].is_none() && r.hi > 0.) || (upper[j].is_none() && r.lo < 0.) {
                return None;
            }
            if r.zero() {
                continue;
            }
            let contribution = match (lower[j], upper[j]) {
                (Some(l), Some(u)) => {
                    if l > u {
                        return None;
                    }
                    let a = r.mul(I::point(l)?)?;
                    let b = r.mul(I::point(u)?)?;
                    a.lo.min(b.lo)
                }
                (Some(l), None) => r.mul(I::point(l)?)?.lo,
                (None, Some(u)) => r.mul(I::point(u)?)?.lo,
                (None, None) => return None,
            };
            total = total.add(I::point(contribution)?)?;
        }
        Some(total.lo)
    }
}

#[cfg(test)]
mod implied_box_tests {
    use super::*;
    #[test]
    fn decimal_interval_bounds_preserve_every_enumerated_feasible_point() {
        let mut seed = 98173u64;
        for _ in 0..100 {
            let mut next = || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                seed
            };
            let a: Vec<i64> = (0..3).map(|_| (next() % 11) as i64 - 5).collect();
            let rhs = (next() % 21) as i64 - 10;
            let mut text = String::from("NAME proofbox\nROWS\n N c\n L mixed\n");
            for j in 0..3 {
                text += &format!(" L up{j}\n L lo{j}\n");
            }
            text += "COLUMNS\n";
            for (j, a) in a.iter().enumerate() {
                text += &format!(
                    " x{j} c 0 mixed {}\n x{j} up{j} 0.1 lo{j} -0.2\n",
                    *a as f64 / 10.
                );
            }
            text += &format!("RHS\n rhs mixed {}\n", rhs as f64 / 10.);
            for j in 0..3 {
                text += &format!(" rhs up{j} 0.3 lo{j} 0.4\n");
            }
            text += "BOUNDS\n";
            for j in 0..3 {
                text += &format!(" FR b x{j}\n");
            }
            text += "ENDATA\n";
            let model = crate::mps::parse(std::io::Cursor::new(text)).unwrap();
            let certifier = Certifier::feasibility(&model).unwrap();
            let (lower, upper) = certifier.implied_box.as_ref().unwrap();
            assert!(lower.iter().chain(upper).all(Option::is_some));
            for x in -2..=3 {
                for y in -2..=3 {
                    for z in -2..=3 {
                        let point = [x, y, z];
                        if a.iter().zip(point).map(|(a, x)| a * x).sum::<i64>() > rhs {
                            continue;
                        }
                        for j in 0..3 {
                            assert!(lower[j].unwrap() <= point[j] as f64);
                            assert!(upper[j].unwrap() >= point[j] as f64);
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn two_unbounded_terms_do_not_imply_an_individual_bound() {
        let model=crate::mps::parse(std::io::Cursor::new("NAME free\nROWS\n N c\n L r\nCOLUMNS\n x c 0 r 0.1\n y c 0 r -0.2\nRHS\n rhs r 1\nBOUNDS\n FR b x\n FR b y\nENDATA\n")).unwrap();
        let certifier = Certifier::feasibility(&model).unwrap();
        let (lower, upper) = certifier.implied_box.unwrap();
        assert!(lower.iter().chain(&upper).all(Option::is_none));
    }
}

#[cfg(test)]
mod joint_proof_diagnostic {
    use super::*;
    #[test]
    fn joint_trials_never_certify_a_planted_feasible_decimal_model() {
        let mut seed = 412983u64;
        for _ in 0..80 {
            let mut text = String::from("NAME planted\nROWS\n N obj\n");
            for i in 0..5 {
                text += &format!(" L r{i}\n");
            }
            text += "COLUMNS\n";
            for j in 0..4 {
                text += &format!(" x{j} obj 0\n");
                for i in 0..5 {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let a = (seed % 21) as i64 - 10;
                    if a != 0 {
                        text += &format!(" x{j} r{i} {}\n", a as f64 / 10.);
                    }
                }
            }
            text += "RHS\n";
            for i in 0..5 {
                text += &format!(" rhs r{i} 0.1\n");
            }
            text += "ENDATA\n";
            let model = crate::mps::parse(std::io::Cursor::new(text)).unwrap();
            // The origin is strictly feasible; all variables have their default [0,+inf) domain.
            let c = Certifier::feasibility(&model).unwrap();
            for mask in 0..8 {
                let weights: Vec<_> = (0..5)
                    .map(|j| if mask & (1 << (j % 3)) == 0 { -1. } else { 1. })
                    .collect();
                let (bound, trials) =
                    c.joint_positive_bound(&[Some(0.); 4], &[None; 4], &weights, None);
                assert_eq!(bound, None);
                assert_eq!(trials, 64);
            }
        }
    }
    #[test]
    fn joint_trials_reject_bad_shapes_and_nonfinite_weights() {
        let model=crate::mps::parse(std::io::Cursor::new("NAME impossible\nROWS\n N obj\n L a\n G b\nCOLUMNS\n x obj 0 a 1\n x b 1\nRHS\n rhs a 0 b 1\nENDATA\n")).unwrap();
        let c = Certifier::feasibility(&model).unwrap();
        assert_eq!(
            c.joint_positive_bound(
                &[Some(0.)],
                &[None],
                &[-1., 1.],
                Some(std::time::Instant::now())
            ),
            (None, 0)
        );
        assert!(
            c.joint_positive_bound(&[Some(0.)], &[None], &[-1., 1.], None)
                .0
                .is_some()
        );
        assert_eq!(c.joint_positive_bound(&[], &[], &[-1., 1.], None).0, None);
        assert_eq!(
            c.joint_positive_bound(&[Some(0.)], &[None], &[f64::NAN, 1.], None),
            (None, 0)
        );
    }
    #[test]
    #[ignore = "saved development failure via MIP_PROOF_DUMP"]
    fn inspect_joint_perturbations() {
        let d: serde_json::Value = serde_json::from_reader(
            std::fs::File::open(std::env::var("MIP_PROOF_DUMP").unwrap()).unwrap(),
        )
        .unwrap();
        let model: Model = serde_json::from_value(d["model"].clone()).unwrap();
        let lower: Vec<Option<f64>> = serde_json::from_value(d["lower"].clone()).unwrap();
        let upper: Vec<Option<f64>> = serde_json::from_value(d["upper"].clone()).unwrap();
        let weights: Vec<f64> = serde_json::from_value(d["multipliers"].clone()).unwrap();
        let c = Certifier::feasibility(&model).unwrap();
        let (bound, trials) = c.joint_positive_bound(&lower, &upper, &weights, None);
        println!("{}", serde_json::json!({"bound":bound,"trials":trials}));
        assert!(bound.is_some_and(|b| b > 0.));
    }
}
