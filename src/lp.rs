//! 二段階の密表単体法。min c*x, A*x <= b, x >= 0 の参照実装。
//! 大規模問題用ではない。返却statusは浮動小数点の判定であり厳密証明ではない。
const EPS: f64 = 1e-9;
#[derive(Debug, PartialEq)]
pub enum Outcome {
    Optimal { objective: f64, x: Vec<f64> },
    Infeasible,
    Unbounded,
    IterationLimit,
    TimeLimit,
    NumericalError,
}
struct Tableau {
    a: Vec<Vec<f64>>,
    basis: Vec<usize>,
    n: usize,
    iterations: usize,
    limit: usize,
    deadline: Option<std::time::Instant>,
}
impl Tableau {
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
    }
    fn objective(&mut self, c: &[f64]) {
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
    fn run(&mut self, allowed: usize, pricing_eps: f64) -> Result<(), Outcome> {
        loop {
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
                return Ok(());
            };
            if self.iterations >= self.limit {
                return Err(Outcome::IterationLimit);
            }
            let mut relaxed = f64::INFINITY;
            for i in 0..m {
                if self.a[i][j] > EPS {
                    if self.a[i][self.n] < -1e-8 {
                        return Err(numerical());
                    }
                    relaxed = relaxed.min((self.a[i][self.n].max(0.) + 1e-8) / self.a[i][j]);
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
            let Some(r) = row else {
                return Err(Outcome::Unbounded);
            };
            self.a[r][self.n] = self.a[r][self.n].max(0.);
            self.pivot(r, j);
        }
    }
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
    let mut t = Tableau {
        a: vec![vec![0.0; total + 1]; m + 1],
        basis: Vec::new(),
        n: total,
        iterations: 0,
        limit,
        deadline,
    };
    let row_scales: Vec<_> = a
        .iter()
        .map(|r| {
            let scale = r.iter().map(|v| v.abs()).fold(0., f64::max);
            if scale > 0. { scale } else { 1. }
        })
        .collect();
    for i in 0..m {
        let sign = if b[i] < 0.0 { -1.0 } else { 1.0 };
        for j in 0..n {
            t.a[i][j] = sign * (a[i][j] / row_scales[i]);
        }
        t.a[i][n + i] = sign;
        t.a[i][n + m + i] = 1.0;
        t.a[i][total] = sign * (b[i] / row_scales[i]);
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
            if t.a[i][total].abs() > EPS {
                return Ok(numerical());
            }
            if let Some(j) = (0..n + m).find(|&j| t.a[i][j].abs() > EPS && !t.basis.contains(&j)) {
                t.pivot(i, j);
            } else {
                t.a.remove(i);
                t.basis.remove(i);
                continue;
            }
        }
        i += 1;
    }
    let objective_scale = c.iter().map(|v| v.abs()).fold(0.0, f64::max);
    let objective_scale = if objective_scale > 0. {
        objective_scale
    } else {
        1.
    };
    let normalized: Vec<_> = c.iter().map(|v| v / objective_scale).collect();
    if c.iter().zip(&normalized).any(|(a, b)| *a != 0. && *b == 0.) {
        return Ok(numerical());
    }
    t.objective(&normalized);
    let minimum_cost = normalized
        .iter()
        .map(|v| v.abs())
        .filter(|v| *v > 0.)
        .fold(1.0, f64::min);
    if let Err(status) = t.run(n + m, EPS * minimum_cost) {
        return Ok(status);
    }
    let mut x = vec![0.0; n];
    for (i, &j) in t.basis.iter().enumerate() {
        if j < n {
            x[j] = t.a[i][total];
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
