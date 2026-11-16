//! 二段階の密表単体法。min c*x, A*x <= b, x >= 0 の参照実装。
//! 大規模問題用ではない。返却statusは浮動小数点の判定であり厳密証明ではない。
const EPS: f64 = 1e-9;
const FEAS: f64 = 1e-8;
#[derive(Debug, PartialEq)]
pub enum Outcome {
    Optimal { objective: f64, x: Vec<f64> },
    Infeasible,
    Unbounded,
    IterationLimit,
    TimeLimit,
    NumericalError,
}
struct Tableau<'a> {
    rebuild: &'a dyn Fn(&[usize]) -> Result<Vec<Vec<f64>>, Outcome>,
    costs: Vec<f64>,
    dirty: bool,
    unbounded_enter: Option<usize>,
    a: Vec<Vec<f64>>,
    basis: Vec<usize>,
    n: usize,
    iterations: usize,
    limit: usize,
    deadline: Option<std::time::Instant>,
}
impl Tableau<'_> {
    fn pivot(&mut self, r: usize, j: usize) {
        let p = self.a[r][j];
        for v in &mut self.a[r] {
            *v /= p;
        }
        for i in 0..self.a.len() {
            if i == r {
                continue;
            }
            let q = self.a[i][j];
            for k in 0..=self.n {
                self.a[i][k] -= q * self.a[r][k];
            }
            self.a[i][j] = 0.0;
        }
        self.basis[r] = j;
        self.iterations += 1;
        self.dirty = true;
    }
    fn objective(&mut self, c: &[f64]) {
        self.costs = c.to_vec();
        let m = self.basis.len();
        self.a[m].fill(0.0);
        self.a[m][..c.len()].copy_from_slice(c);
        for i in 0..m {
            let q = self.a[m][self.basis[i]];
            for j in 0..=self.n {
                self.a[m][j] -= q * self.a[i][j];
            }
        }
    }
    fn refactor(&mut self) -> Result<(), Outcome> {
        let rows = (self.rebuild)(&self.basis)?;
        self.a = rows;
        let c = self.costs.clone();
        self.objective(&c);
        self.dirty = false;
        Ok(())
    }
    fn run(&mut self, allowed: usize, pricing_eps: f64) -> Result<(), Outcome> {
        'iterations: loop {
            if self
                .deadline
                .is_some_and(|d| std::time::Instant::now() >= d)
            {
                return Err(Outcome::TimeLimit);
            }
            if self.a.iter().flatten().any(|x| !x.is_finite()) {
                return Err(numerical());
            }
            let m = self.basis.len();
            // Bland entering rule and basis-index ratio ties avoid exact cycling.
            let Some(j) = (0..allowed).find(|&j| self.a[m][j] < -pricing_eps) else {
                if self.dirty {
                    self.refactor()?;
                    continue;
                }
                return Ok(());
            };
            if self.iterations >= self.limit {
                return Err(Outcome::IterationLimit);
            }
            let mut relaxed = f64::INFINITY;
            for i in 0..m {
                if self.a[i][j] > EPS {
                    if self.a[i][self.n] < -FEAS {
                        if self.dirty {
                            self.refactor()?;
                            continue 'iterations;
                        }
                        return Err(numerical());
                    }
                    relaxed = relaxed.min((self.a[i][self.n].max(0.) + FEAS) / self.a[i][j]);
                }
            }
            let mut row = None;
            let mut largest = 0.;
            for i in 0..m {
                let pivot = self.a[i][j];
                if pivot > EPS {
                    let ratio = self.a[i][self.n].max(0.) / pivot;
                    if ratio <= relaxed
                        && (pivot > largest
                            || (pivot == largest
                                && row.is_none_or(|r| self.basis[i] < self.basis[r])))
                    {
                        row = Some(i);
                        largest = pivot;
                    }
                }
            }
            if row.is_none() {
                // 無限stepを宣言する前に、小さいが非ゼロの制限行も検査する。
                let mut relaxed = f64::INFINITY;
                for i in 0..m {
                    if self.a[i][j] > 0. {
                        relaxed = relaxed.min((self.a[i][self.n].max(0.) + FEAS) / self.a[i][j]);
                    }
                }
                let mut largest = 0.;
                for i in 0..m {
                    let pivot = self.a[i][j];
                    if pivot > 0. && self.a[i][self.n].max(0.) / pivot <= relaxed && pivot > largest
                    {
                        row = Some(i);
                        largest = pivot;
                    }
                }
            }
            let Some(r) = row else {
                if self.dirty {
                    self.refactor()?;
                    continue;
                }
                self.unbounded_enter = Some(j);
                return Err(Outcome::Unbounded);
            };
            self.a[r][self.n] = self.a[r][self.n].max(0.);
            self.pivot(r, j);
        }
    }
}
fn rebuild_tableau(
    a: &[Vec<f64>],
    b: &[f64],
    columns: &[f64],
    rows: &[f64],
    basis: &[usize],
    total: usize,
    deadline: Option<std::time::Instant>,
) -> Result<Vec<Vec<f64>>, Outcome> {
    let m = b.len();
    let n = columns.len();
    let mut matrix = vec![vec![0.; total + 1]; m];
    for i in 0..m {
        let sign = if b[i] < 0. { -1. } else { 1. };
        for j in 0..n {
            matrix[i][j] = sign * (a[i][j] / columns[j]) / rows[i];
        }
        matrix[i][n + i] = sign;
        matrix[i][n + m + i] = 1.;
        matrix[i][total] = sign * (b[i] / rows[i]);
    }
    for (k, &j) in basis.iter().enumerate() {
        if deadline.is_some_and(|d| std::time::Instant::now() >= d) {
            return Err(Outcome::TimeLimit);
        }
        let Some(r) = (k..m).max_by(|&r, &s| matrix[r][j].abs().total_cmp(&matrix[s][j].abs()))
        else {
            return Err(numerical());
        };
        if matrix[r][j] == 0. {
            return Err(numerical());
        }
        matrix.swap(k, r);
        let pivot = matrix[k][j];
        for v in &mut matrix[k] {
            *v /= pivot;
        }
        let row = matrix[k].clone();
        for i in 0..m {
            if i == k {
                continue;
            }
            let multiplier = matrix[i][j];
            if multiplier != 0. {
                for q in 0..=total {
                    matrix[i][q] -= multiplier * row[q];
                }
            }
            matrix[i][j] = 0.;
        }
        matrix[k][j] = 1.;
    }
    if matrix.iter().flatten().any(|v| !v.is_finite()) {
        return Err(numerical());
    }
    for row in &matrix[basis.len()..] {
        if row[..n + m].iter().any(|v| v.abs() > FEAS) || row[total].abs() > FEAS {
            return Err(numerical());
        }
    }
    matrix.truncate(basis.len());
    matrix.push(vec![0.; total + 1]);
    Ok(matrix)
}
/// Reject malformed or nonfinite input; a finite pivot budget bounds degenerate runs.
pub fn minimize(a: &[Vec<f64>], b: &[f64], c: &[f64], limit: usize) -> Result<Outcome, String> {
    minimize_until(a, b, c, limit, None)
}

pub fn minimize_until(
    a: &[Vec<f64>],
    b: &[f64],
    c: &[f64],
    limit: usize,
    deadline: Option<std::time::Instant>,
) -> Result<Outcome, String> {
    let m = b.len();
    let n = c.len();
    if a.len() != m || a.iter().any(|r| r.len() != n) {
        return Err("dimension mismatch".into());
    }
    if a.iter().flatten().chain(b).chain(c).any(|v| !v.is_finite()) {
        return Err("nonfinite input".into());
    }
    let total = n + 2 * m;
    let column_scales: Vec<_> = (0..n)
        .map(|j| {
            let largest = a.iter().map(|r| r[j].abs()).fold(0., f64::max);
            if largest > 0. && !(1e-4..=1e4).contains(&largest) {
                crate::scaling::power(largest)
            } else {
                1.
            }
        })
        .collect();
    let row_scales: Vec<_> = a
        .iter()
        .map(|r| {
            let largest = r
                .iter()
                .zip(&column_scales)
                .map(|(v, s)| (v / s).abs())
                .fold(0., f64::max);
            if largest > 0. {
                crate::scaling::power(largest)
            } else {
                1.
            }
        })
        .collect();
    let rebuild = |basis: &[usize]| {
        rebuild_tableau(a, b, &column_scales, &row_scales, basis, total, deadline)
    };
    let mut t = Tableau {
        rebuild: &rebuild,
        costs: vec![],
        dirty: false,
        unbounded_enter: None,
        a: vec![vec![0.0; total + 1]; m + 1],
        basis: Vec::new(),
        n: total,
        iterations: 0,
        limit,
        deadline,
    };
    for i in 0..m {
        let sign = if b[i] < 0.0 { -1.0 } else { 1.0 };
        for j in 0..n {
            let column_value = a[i][j] / column_scales[j];
            let value = column_value / row_scales[i];
            if !value.is_finite()
                || column_value * column_scales[j] != a[i][j]
                || value * row_scales[i] != column_value
            {
                return Ok(numerical());
            }
            t.a[i][j] = sign * value;
        }
        t.a[i][n + i] = sign;
        t.a[i][n + m + i] = 1.0;
        t.a[i][total] = sign * (b[i] / row_scales[i]);
        if !t.a[i][total].is_finite() || sign * t.a[i][total] * row_scales[i] != b[i] {
            return Ok(numerical());
        }
        t.basis.push(n + m + i);
    }
    let mut phase1 = vec![0.0; total];
    phase1[n + m..].fill(1.0);
    t.objective(&phase1);
    if let Err(status) = t.run(total, EPS) {
        return Ok(match status {
            Outcome::Unbounded => numerical(),
            x => x,
        });
    }
    if t.a[m][total] < -EPS {
        // Farkas multipliers from the slack reduced costs. Reject unstable
        // infeasibility claims rather than pruning on a tableau value alone.
        let lambda: Vec<_> = (0..m).map(|i| -t.a[m][n + i] / row_scales[i]).collect();
        let certified = lambda.iter().all(|&v| v <= 1e-8)
            && (0..n).all(|j| {
                let value: f64 = (0..m).map(|i| lambda[i] * a[i][j]).sum();
                let scale: f64 = (0..m).map(|i| (lambda[i] * a[i][j]).abs()).sum();
                value <= 1e-8 * (1.0 + scale)
            })
            && lambda.iter().zip(b).map(|(v, b)| v * b).sum::<f64>() > EPS;
        return Ok(if certified {
            Outcome::Infeasible
        } else {
            numerical()
        });
    }
    // Remove artificial basic variables; an all-zero row is redundant.
    let mut i = 0;
    while i < t.basis.len() {
        if t.basis[i] >= n + m {
            if t.a[i][total].abs() > FEAS {
                if std::env::var_os("MIP_LP_TRACE").is_some() {
                    eprintln!("artificial basic residual {}", t.a[i][total]);
                }
                return Ok(numerical());
            }
            // Harrisの許容範囲内の人工変数残差をゼロへ戻す。
            // 最後に元行列で主点とdualを再検査する。
            if t.a[i][total] != 0. {
                t.dirty = true;
            }
            t.a[i][total] = 0.;
            if let Some(j) = (0..n + m).find(|&j| t.a[i][j].abs() > EPS && !t.basis.contains(&j)) {
                t.pivot(i, j);
            } else {
                t.a.remove(i);
                t.basis.remove(i);
                t.dirty = true;
                continue;
            }
        }
        i += 1;
    }
    let scaled_costs: Vec<_> = c.iter().zip(&column_scales).map(|(v, s)| v / s).collect();
    if scaled_costs
        .iter()
        .zip(&column_scales)
        .zip(c)
        .any(|((v, s), original)| !v.is_finite() || v * s != *original)
    {
        return Ok(numerical());
    }
    let largest = scaled_costs.iter().map(|v| v.abs()).fold(0., f64::max);
    let objective_scale = if largest > 0. {
        crate::scaling::power(largest)
    } else {
        1.
    };
    let normalized: Vec<_> = scaled_costs.iter().map(|v| v / objective_scale).collect();
    if normalized
        .iter()
        .zip(&scaled_costs)
        .any(|(v, original)| !v.is_finite() || v * objective_scale != *original)
    {
        return Ok(numerical());
    }
    t.objective(&normalized);
    let minimum_cost = normalized
        .iter()
        .map(|v| v.abs())
        .filter(|v| *v > 0.)
        .fold(1.0, f64::min);
    if let Err(status) = t.run(n + m, EPS * minimum_cost) {
        if matches!(status, Outcome::Unbounded) {
            let Some(j) = t.unbounded_enter else {
                return Ok(numerical());
            };
            let mut ray = vec![0.; n];
            if j < n {
                ray[j] = 1. / column_scales[j];
            }
            for (i, &k) in t.basis.iter().enumerate() {
                if k < n {
                    ray[k] = -t.a[i][j] / column_scales[k];
                }
            }
            let mut point = vec![0.; n];
            for (i, &k) in t.basis.iter().enumerate() {
                if k < n {
                    point[k] = t.a[i][total] / column_scales[k];
                }
            }
            let base_valid = point.iter().all(|v| v.is_finite() && *v >= -1e-7)
                && a.iter().zip(b).all(|(r, &rhs)| {
                    r.iter().zip(&point).map(|(a, x)| a * x).sum::<f64>() <= rhs + 1e-6
                });
            let valid = base_valid
                && ray.iter().all(|v| v.is_finite() && *v >= 0.)
                && a.iter().all(|row| {
                    crate::exact::dot_sign(row.iter().copied().zip(ray.iter().copied()))
                        .is_some_and(|s| s != std::cmp::Ordering::Greater)
                })
                && crate::exact::dot_sign(c.iter().copied().zip(ray.iter().copied()))
                    == Some(std::cmp::Ordering::Less);
            if !valid {
                return Ok(numerical());
            }
        }
        return Ok(status);
    }
    let mut x = vec![0.0; n];
    for (i, &j) in t.basis.iter().enumerate() {
        if j < n {
            x[j] = t.a[i][total] / column_scales[j];
        }
    }
    // 元の非負境界・単一変数上限へ点を戻す。制約は緩めず、後で全行/dualを再検査する。
    for value in &mut x {
        *value = value.max(0.);
    }
    for (r, &rhs) in a.iter().zip(b) {
        let mut nonzero = r.iter().enumerate().filter(|(_, v)| **v != 0.);
        if let Some((j, &coefficient)) = nonzero.next() {
            if nonzero.next().is_none() && coefficient > 0. && rhs >= 0. {
                let upper = rhs / coefficient;
                if upper.is_finite() {
                    x[j] = x[j].min(upper);
                }
            }
        }
    }
    // Check primal residual independently of tableau before returning a solution.
    if x.iter().any(|&v| v < -1e-7)
        || a.iter().zip(b).any(|(r, &rhs)| {
            let value: f64 = r.iter().zip(&x).map(|(v, x)| v * x).sum();
            value > rhs + 1e-7 * (1.0 + rhs.abs())
        })
    {
        return Ok(numerical());
    }
    let objective: f64 = c.iter().zip(&x).map(|(v, x)| v * x).sum();
    let row = t.basis.len();
    let lambda: Vec<_> = (0..m)
        .map(|i| -t.a[row][n + i] * objective_scale / row_scales[i])
        .collect();
    let dual: f64 = lambda.iter().zip(b).map(|(v, b)| v * b).sum();
    let dual_ok = lambda.iter().all(|&v| v <= 1e-8)
        && (0..n).all(|j| {
            let value: f64 = (0..m).map(|i| lambda[i] * a[i][j]).sum();
            let scale: f64 = (0..m).map(|i| (lambda[i] * a[i][j]).abs()).sum();
            value <= c[j] + 1e-8 * (1.0 + scale + c[j].abs())
        });
    if !objective.is_finite()
        || !dual.is_finite()
        || !dual_ok
        || (objective - dual).abs() > 1e-7 * (1.0 + objective.abs())
    {
        return Ok(numerical());
    }
    Ok(Outcome::Optimal { objective, x })
}

#[track_caller]
fn numerical() -> Outcome {
    if std::env::var_os("MIP_LP_TRACE").is_some() {
        eprintln!(
            "Dense LP numerical failure at {}",
            std::panic::Location::caller()
        );
    }
    Outcome::NumericalError
}
