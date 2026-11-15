use serde::{Deserialize, Serialize};

/// None denotes an absent bound, never a NaN or a sentinel magnitude.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Variable {
    #[serde(default)]
    pub objective_enclosure: Option<[f64; 2]>,
    /// 元入力の境界定数を外側から囲む有限区間。Noneは証明情報なし。
    #[serde(default)]
    pub lower_enclosure: Option<[f64; 2]>,
    #[serde(default)]
    pub upper_enclosure: Option<[f64; 2]>,
    pub name: String,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    pub integer: bool,
    pub objective: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Row {
    #[serde(default)]
    pub exact_coefficients: bool,
    #[serde(default)]
    pub lower_enclosure: Option<[f64; 2]>,
    #[serde(default)]
    pub upper_enclosure: Option<[f64; 2]>,
    /// MPSの係数/RHS/RANGESが小さい整数そのもので、加算も正確だった行。
    /// 外部JSON/手作りModelは既定false。整数性の証明と係数区間の確定だけに使用する。
    #[serde(default)]
    pub exact_integral: bool,
    pub name: String,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    pub coefficients: Vec<(usize, f64)>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    #[serde(default)]
    pub original_decimal: bool,
    #[serde(default)]
    pub offset_enclosure: Option<[f64; 2]>,
    pub name: String,
    pub maximize: bool,
    pub offset: f64,
    pub variables: Vec<Variable>,
    pub rows: Vec<Row>,
}
/// Outward error budgets for the original decimal model and serialized point.
pub(crate) fn scalar_error(value: f64, enclosure: Option<[f64; 2]>) -> f64 {
    enclosure.map_or(f64::INFINITY, |[l, u]| {
        (value - l).abs().max((u - value).abs())
    })
}
pub(crate) fn point_error(value: f64) -> f64 {
    if value.fract() == 0. && value.abs() <= (1u64 << 53) as f64 {
        0.
    } else {
        (value.next_up() - value)
            .abs()
            .max((value - value.next_down()).abs())
    }
}
pub(crate) fn add_error(total: &mut f64, term: f64) {
    if term != 0. {
        *total = (*total + term).next_up();
    }
}
impl Row {
    pub(crate) fn coefficient_error(&self, value: f64) -> f64 {
        if self.exact_coefficients || self.exact_integral {
            0.
        } else {
            point_error(value)
                .max((value.next_up() - value).abs())
                .max((value - value.next_down()).abs())
        }
    }
    pub(crate) fn activity_error(&self, x: &[f64]) -> f64 {
        let mut error = 0.;
        for &(j, a) in &self.coefficients {
            let da = self.coefficient_error(a);
            let dx = point_error(x[j]);
            for (a, b) in [(da, x[j].abs()), (a.abs(), dx), (da, dx)] {
                if a != 0. && b != 0. {
                    add_error(&mut error, (a * b).next_up());
                }
            }
        }
        error
    }
}
impl Model {
    pub(crate) fn needs_original_feasibility_check(&self) -> bool {
        self.original_decimal
            && (self.rows.iter().any(|r| {
                !r.exact_coefficients
                    || [(r.lower, r.lower_enclosure), (r.upper, r.upper_enclosure)]
                        .into_iter()
                        .any(|(v, e)| v.is_some_and(|v| e != Some([v, v])))
            }) || self.variables.iter().any(|v| {
                [(v.lower, v.lower_enclosure), (v.upper, v.upper_enclosure)]
                    .into_iter()
                    .any(|(v, e)| v.is_some_and(|v| e != Some([v, v])))
            }))
    }
    pub fn objective(&self, x: &[f64]) -> f64 {
        let value = crate::exact::dot(
            std::iter::once((self.offset, 1.))
                .chain(self.variables.iter().zip(x).map(|(v, &x)| (v.objective, x))),
        )
        .unwrap_or(f64::NAN);
        if self.original_decimal {
            let mut error = scalar_error(self.offset, self.offset_enclosure);
            for (v, &x) in self.variables.iter().zip(x) {
                if v.objective == 0. && v.objective_enclosure == Some([0., 0.]) {
                    continue;
                }
                let dc = scalar_error(v.objective, v.objective_enclosure);
                let dx = point_error(x);
                add_error(&mut error, dc * x.abs());
                add_error(&mut error, v.objective.abs() * dx);
                add_error(&mut error, dc * dx);
            }
            if !error.is_finite() || error > 1e-10 * (1. + value.abs()) {
                return f64::NAN;
            }
        }
        value
    }
    /// A quick in-process guard. Independent original-MPS validation is separate.
    pub fn check(&self, x: &[f64], integer: bool, tolerance: f64) -> Result<(), String> {
        if x.len() != self.variables.len() || x.iter().any(|v| !v.is_finite()) {
            return Err("invalid solution dimension or nonfinite value".into());
        }
        let bounds = |value: f64, lower: Option<f64>, upper: Option<f64>| {
            lower.is_none_or(|b| value >= b - tolerance)
                && upper.is_none_or(|b| value <= b + tolerance)
        };
        for (v, &value) in self.variables.iter().zip(x) {
            if !bounds(value, v.lower, v.upper)
                || (integer && v.integer && (value - value.round()).abs() > tolerance)
            {
                return Err(format!("variable {} violates domain", v.name));
            }
        }
        for row in &self.rows {
            let error = if self.original_decimal {
                row.activity_error(x)
            } else {
                0.
            };
            let terms = || row.coefficients.iter().map(|&(j, a)| (a, x[j]));
            let feasible = |[lo, hi]: [f64; 2]| {
                row.lower.is_none_or(|l| {
                    lo - error >= row.lower_enclosure.map_or(l, |b| b[1]) - tolerance
                }) && row.upper.is_none_or(|u| {
                    hi + error <= row.upper_enclosure.map_or(u, |b| b[0]) + tolerance
                })
            };
            if !crate::exact::fast_dot_bounds(terms()).is_some_and(feasible)
                && !crate::exact::dot_bounds(terms()).is_some_and(feasible)
            {
                return Err(format!(
                    "row {} violates bounds or loses precision",
                    row.name
                ));
            }
        }
        Ok(())
    }
}
