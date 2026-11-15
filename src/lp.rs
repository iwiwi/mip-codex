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
    fn run(&mut self, allowed: usize) -> Result<(), Outcome> {
        loop {
            if self
                .deadline
                .is_some_and(|d| std::time::Instant::now() >= d)
            {
                return Err(Outcome::TimeLimit);
            }
            if self.a.iter().flatten().any(|x| !x.is_finite()) {
                return Err(Outcome::NumericalError);
            }
            let m = self.basis.len();
            // Bland entering rule and basis-index ratio ties avoid exact cycling.
            let Some(j) = (0..allowed).find(|&j| self.a[m][j] < -EPS) else {
                return Ok(());
            };
            if self.iterations >= self.limit {
                return Err(Outcome::IterationLimit);
            }
            let mut row = None;
            let mut ratio = f64::INFINITY;
            for i in 0..m {
                if self.a[i][j] > EPS {
                    let q = self.a[i][self.n] / self.a[i][j];
                    if q < ratio - EPS
                        || ((q - ratio).abs() <= EPS
                            && row.is_none_or(|r| self.basis[i] < self.basis[r]))
                    {
                        row = Some(i);
                        ratio = q;
                    }
                }
            }
            let Some(r) = row else {
                return Err(Outcome::Unbounded);
            };
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
    for i in 0..m {
        let sign = if b[i] < 0.0 { -1.0 } else { 1.0 };
        for j in 0..n {
            t.a[i][j] = sign * a[i][j];
        }
        t.a[i][n + i] = sign;
        t.a[i][n + m + i] = 1.0;
        t.a[i][total] = sign * b[i];
        t.basis.push(n + m + i);
    }
    let mut phase1 = vec![0.0; total];
    phase1[n + m..].fill(1.0);
    t.objective(&phase1);
    if let Err(status) = t.run(total) {
        return Ok(match status {
            Outcome::Unbounded => Outcome::NumericalError,
            x => x,
        });
    }
    if t.a[m][total] < -EPS {
        // Farkas multipliers from the slack reduced costs. Reject unstable
        // infeasibility claims rather than pruning on a tableau value alone.
        let lambda: Vec<_> = (0..m).map(|i| -t.a[m][n + i]).collect();
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
            Outcome::NumericalError
        });
    }
    // Remove artificial basic variables; an all-zero row is redundant.
    let mut i = 0;
    while i < t.basis.len() {
        if t.basis[i] >= n + m {
            if t.a[i][total].abs() > EPS {
                return Ok(Outcome::NumericalError);
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
    t.objective(c);
    if let Err(status) = t.run(n + m) {
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
        return Ok(Outcome::NumericalError);
    }
    let objective: f64 = c.iter().zip(&x).map(|(v, x)| v * x).sum();
    let row = t.basis.len();
    let lambda: Vec<_> = (0..m).map(|i| -t.a[row][n + i]).collect();
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
        return Ok(Outcome::NumericalError);
    }
    Ok(Outcome::Optimal { objective, x })
}
