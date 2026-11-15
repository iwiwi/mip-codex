use serde::{Deserialize, Serialize};

/// None denotes an absent bound, never a NaN or a sentinel magnitude.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Variable {
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
    pub lower_enclosure: Option<[f64; 2]>,
    #[serde(default)]
    pub upper_enclosure: Option<[f64; 2]>,
    /// MPSの係数/RHS/RANGESが小さい整数そのもので、加算も正確だった行。
    /// 外部JSON/手作りModelは既定false。カットの適用可否だけに使用する。
    #[serde(default)]
    pub exact_integral: bool,
    pub name: String,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    pub coefficients: Vec<(usize, f64)>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub maximize: bool,
    pub offset: f64,
    pub variables: Vec<Variable>,
    pub rows: Vec<Row>,
}
impl Model {
    pub fn objective(&self, x: &[f64]) -> f64 {
        self.offset
            + self
                .variables
                .iter()
                .zip(x)
                .map(|(v, x)| v.objective * x)
                .sum::<f64>()
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
            let activity: f64 = row.coefficients.iter().map(|&(j, a)| a * x[j]).sum();
            if !activity.is_finite() || !bounds(activity, row.lower, row.upper) {
                return Err(format!("row {} violates bounds", row.name));
            }
        }
        Ok(())
    }
}
