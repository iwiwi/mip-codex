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
    pub exact_coefficient_flags: Vec<bool>,
    #[serde(default)]
    pub lower_enclosure: Option<[f64; 2]>,
    #[serde(default)]
    pub upper_enclosure: Option<[f64; 2]>,
    /// MPSの係数/RHS/RANGESが小さい整数そのもので、加算も正確だった行。
    /// 外部JSON/手作りModelは既定false。整数性の証明と係数区間の確定だけに使用する。
    #[serde(default)]
    pub exact_integral: bool,
    /// Parser-certified integerization of the original decimal row, at most 10^9.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimal_scale: Option<u64>,
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
// solution_number prints every integral f64 as that exact integer.
pub(crate) fn point_error(value: f64) -> f64 {
    if value.fract() == 0. {
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
    pub(crate) fn coefficient_error(&self, index: usize, value: f64) -> f64 {
        if self.exact_coefficients
            || self.exact_integral
            || self.exact_coefficient_flags.get(index) == Some(&true)
        {
            0.
        } else {
            point_error(value)
                .max((value.next_up() - value).abs())
                .max((value - value.next_down()).abs())
        }
    }
    /// Same outward sum when every point coordinate has zero serialization error.
    /// Only call after proving point_error(x[j]) == 0 for all coordinates.
    pub(crate) fn activity_error_exact_point(&self, x: &[f64]) -> f64 {
        debug_assert!(x.iter().all(|&value| point_error(value) == 0.));
        if self.exact_coefficients || self.exact_integral {
            return 0.;
        }
        let mut error = 0.;
        for (index, &(j, a)) in self.coefficients.iter().enumerate() {
            if x[j] != 0. {
                let da = self.coefficient_error(index, a);
                if da != 0. {
                    add_error(&mut error, (da * x[j].abs()).next_up());
                }
            }
        }
        error
    }
    pub(crate) fn activity_error(&self, x: &[f64]) -> f64 {
        let mut error = 0.;
        for (index, &(j, a)) in self.coefficients.iter().enumerate() {
            let da = self.coefficient_error(index, a);
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
/// Preserve integral binary values exactly when JSON's shortest decimal would change them.
pub(crate) fn solution_number(value: f64) -> Option<serde_json::Number> {
    if !value.is_finite() {
        return None;
    }
    if value.abs() < (1u64 << 53) as f64 {
        return serde_json::Number::from_f64(value);
    }
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 2047) as usize - 1075;
    let mantissa = (bits & ((1u64 << 52) - 1)) | (1u64 << 52);
    let mut integer = num_bigint::BigInt::from(mantissa) << exponent;
    if bits >> 63 != 0 {
        integer = -integer;
    }
    integer.to_string().parse().ok()
}
pub(crate) fn serialize_solution_values<S: serde::Serializer>(
    values: &Option<std::collections::BTreeMap<String, f64>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::{Error, SerializeMap};
    let Some(values) = values else {
        return serializer.serialize_none();
    };
    let mut map = serializer.serialize_map(Some(values.len()))?;
    for (name, &value) in values {
        let number =
            solution_number(value).ok_or_else(|| S::Error::custom("nonfinite solution number"))?;
        map.serialize_entry(name, &number)?;
    }
    map.end()
}
pub(crate) fn serialize_optional_solution_number<S: serde::Serializer>(
    value: &Option<f64>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::Error;
    match value {
        None => serializer.serialize_none(),
        Some(value) => solution_number(*value)
            .ok_or_else(|| S::Error::custom("nonfinite solution number"))?
            .serialize(serializer),
    }
}
// Compare differences, rather than rounding bound +/- tolerance at a large magnitude.
fn difference_lower(a: f64, b: f64) -> f64 {
    if b == 0. {
        return a;
    }
    if a == b {
        return 0.;
    }
    if a == 0. {
        return -b;
    }
    crate::exact::two_sum(a, -b).map_or_else(
        || (a - b).next_down(),
        |(v, e)| if e < 0. { v.next_down() } else { v },
    )
}
fn in_domain([lo, hi]: [f64; 2], lower: Option<f64>, upper: Option<f64>, tolerance: f64) -> bool {
    lower.is_none_or(|b| difference_lower(lo, b) >= -tolerance)
        && upper.is_none_or(|b| difference_lower(b, hi) >= -tolerance)
}
fn definitely_outside(
    [lo, hi]: [f64; 2],
    lower: Option<f64>,
    upper: Option<f64>,
    tolerance: f64,
) -> bool {
    lower.is_some_and(|b| -difference_lower(b, hi) < -tolerance)
        || upper.is_some_and(|b| -difference_lower(lo, b) < -tolerance)
}
fn serialized_in_domain(
    value: f64,
    lower: Option<f64>,
    upper: Option<f64>,
    tolerance: f64,
) -> bool {
    use std::cmp::Ordering::*;
    let Some(number) = solution_number(value) else {
        return false;
    };
    let token = number.to_string();
    lower.is_none_or(|b| {
        crate::mps::compare_float_sum_token(&token, &[b, -tolerance]).is_some_and(|c| c != Less)
    }) && upper.is_none_or(|b| {
        crate::mps::compare_float_sum_token(&token, &[b, tolerance]).is_some_and(|c| c != Greater)
    })
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
        for (v, &value) in self.variables.iter().zip(x) {
            let (lower, upper) = if self.original_decimal {
                let lower = match v.lower {
                    Some(_) => Some(
                        v.lower_enclosure.ok_or_else(|| {
                            format!("variable {} lower precision unavailable", v.name)
                        })?[1],
                    ),
                    None => None,
                };
                let upper = match v.upper {
                    Some(_) => Some(
                        v.upper_enclosure.ok_or_else(|| {
                            format!("variable {} upper precision unavailable", v.name)
                        })?[0],
                    ),
                    None => None,
                };
                (lower, upper)
            } else {
                (v.lower, v.upper)
            };
            if integer && v.integer && (value - value.round()).abs() > tolerance {
                return Err(format!("variable {} violates integrality", v.name));
            }
            let quick = if self.original_decimal && value.fract() != 0. {
                [value.next_down(), value.next_up()]
            } else {
                [value, value]
            };
            let feasible = in_domain(quick, lower, upper, tolerance)
                || (self.original_decimal
                    && !definitely_outside(quick, lower, upper, tolerance)
                    && serialized_in_domain(value, lower, upper, tolerance));
            if !feasible {
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
            let lower = if self.original_decimal {
                match row.lower {
                    Some(_) => Some(
                        row.lower_enclosure.ok_or_else(|| {
                            format!("row {} lower precision unavailable", row.name)
                        })?[1],
                    ),
                    None => None,
                }
            } else {
                row.lower
            };
            let upper = if self.original_decimal {
                match row.upper {
                    Some(_) => Some(
                        row.upper_enclosure.ok_or_else(|| {
                            format!("row {} upper precision unavailable", row.name)
                        })?[0],
                    ),
                    None => None,
                }
            } else {
                row.upper
            };
            let feasible = |[lo, hi]: [f64; 2]| {
                let lo = if error == 0. {
                    lo
                } else {
                    (lo - error).next_down()
                };
                let hi = if error == 0. {
                    hi
                } else {
                    (hi + error).next_up()
                };
                in_domain([lo, hi], lower, upper, tolerance)
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

#[cfg(test)]
mod activity_error_tests {
    use super::*;
    #[test]
    fn exact_point_specialization_matches_the_full_outward_sum_bit_for_bit() {
        let mut rng = 0x71c53b2a9084e6fdu64;
        let values = [
            0.,
            -0.,
            1.,
            -2.,
            (1u64 << 52) as f64,
            (1u64 << 53) as f64,
            ((1u64 << 60) + 256) as f64,
            2f64.powi(100),
            f64::MAX,
        ];
        for case in 0..5000 {
            let n = 1 + case % 32;
            let mut x = Vec::new();
            let mut coefficients = Vec::new();
            let mut flags = Vec::new();
            for j in 0..n {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                x.push(values[rng as usize % values.len()]);
                let bits = rng & !(0x7ffu64 << 52) | (((rng >> 20) % 2047) << 52);
                let a = match (case + j) % 5 {
                    0 => 0.,
                    1 => f64::from_bits(1),
                    2 => 0.1,
                    3 => f64::MAX,
                    _ => f64::from_bits(bits),
                };
                coefficients.push((j, a));
                flags.push(rng & 1 != 0);
            }
            flags.truncate(case % (n + 1));
            let row = Row {
                exact_coefficients: case % 7 == 0,
                exact_integral: case % 11 == 0,
                decimal_scale: None,
                exact_coefficient_flags: flags,
                lower_enclosure: None,
                upper_enclosure: None,
                name: String::new(),
                lower: None,
                upper: None,
                coefficients,
            };
            let old = row.activity_error(&x);
            let new = row.activity_error_exact_point(&x);
            assert_eq!(
                old.to_bits(),
                new.to_bits(),
                "case {case}: {old:?} != {new:?}"
            );
        }
    }
}
