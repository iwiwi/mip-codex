use flate2::read::GzDecoder;
use serde::Serialize;
use std::{collections::HashMap, fs::File, io::Read, path::Path};

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub lower: f64,
    pub upper: f64,
    pub integer: bool,
    pub cost: f64,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub name: String,
    pub lower: f64,
    pub upper: f64,
    pub terms: Vec<(usize, f64)>,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub name: String,
    pub variables: Vec<Variable>,
    pub rows: Vec<Row>,
    /// +1 for minimization, -1 for maximization. Costs retain the input sign.
    pub sense: f64,
    pub offset: f64,
}

#[derive(Debug, Serialize)]
pub struct Check {
    pub feasible: bool,
    pub objective: f64,
    pub max_bound_violation: f64,
    pub max_row_violation: f64,
    pub max_integrality_violation: f64,
}

impl Model {
    pub fn objective(&self, x: &[f64]) -> f64 {
        self.offset
            + self
                .variables
                .iter()
                .zip(x)
                .map(|(v, x)| v.cost * x)
                .sum::<f64>()
    }

    pub fn check(&self, x: &[f64], tol: f64) -> Check {
        let mut c = Check {
            feasible: x.len() == self.variables.len(),
            objective: f64::NAN,
            max_bound_violation: 0.0,
            max_row_violation: 0.0,
            max_integrality_violation: 0.0,
        };
        if !c.feasible || x.iter().any(|v| !v.is_finite()) {
            c.feasible = false;
            return c;
        }
        for (v, &x) in self.variables.iter().zip(x) {
            let d = (v.lower - x).max(x - v.upper).max(0.0);
            c.max_bound_violation = c.max_bound_violation.max(d);
            c.feasible &= d <= tol;
            if v.integer {
                let d = (x - x.round()).abs();
                c.max_integrality_violation = c.max_integrality_violation.max(d);
                c.feasible &= d <= tol;
            }
        }
        for row in &self.rows {
            let activity: f64 = row.terms.iter().map(|&(j, a)| a * x[j]).sum();
            let d = (row.lower - activity).max(activity - row.upper).max(0.0);
            c.max_row_violation = c.max_row_violation.max(d);
            // Relative row tolerance uses the bound, not cancellation in the activity.
            let scale = 1.0_f64
                .max(if row.lower.is_finite() {
                    row.lower.abs()
                } else {
                    0.0
                })
                .max(if row.upper.is_finite() {
                    row.upper.abs()
                } else {
                    0.0
                });
            c.feasible &= d <= tol * scale;
        }
        c.objective = self.objective(x);
        c.feasible &= c.objective.is_finite();
        c
    }
}

fn number(s: &str) -> Result<f64, String> {
    let x: f64 = s
        .replace(['D', 'd'], "E")
        .parse()
        .map_err(|_| format!("invalid number {s}"))?;
    if !x.is_finite() {
        return Err(format!("nonfinite input coefficient {s}"));
    }
    Ok(x)
}

pub fn read_mps(path: &Path) -> Result<Model, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    let mut s = String::new();
    if bytes.starts_with(&[0x1f, 0x8b]) {
        GzDecoder::new(&bytes[..])
            .read_to_string(&mut s)
            .map_err(|e| e.to_string())?;
    } else {
        s = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    }
    parse_mps(&s)
}

pub fn parse_mps(s: &str) -> Result<Model, String> {
    let mut model = Model {
        name: String::new(),
        variables: vec![],
        rows: vec![],
        sense: 1.0,
        offset: 0.0,
    };
    let mut section = "";
    let mut row_ids = HashMap::<String, usize>::new();
    let mut row_types = Vec::<char>::new();
    let mut row_rhs = Vec::<f64>::new();
    let mut row_ranges = HashMap::<usize, f64>::new();
    let mut col_ids = HashMap::<String, usize>::new();
    let mut objective = None::<String>;
    let mut requested_objective = None::<String>;
    let mut free_rows = Vec::<String>::new();
    let mut integer = false;
    let mut bound_seen = Vec::<bool>::new();
    let mut upper_seen = Vec::<bool>::new();
    let mut rhs_name = None::<String>;
    let mut range_name = None::<String>;
    let mut bound_name = None::<String>;
    let mut ended = false;
    for (line_no, line) in s.lines().enumerate() {
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.is_empty() || line.starts_with('*') {
            continue;
        }
        let err = |msg: String| format!("MPS line {}: {msg}", line_no + 1);
        // A section header has no leading whitespace in standard MPS.
        if !line.starts_with(char::is_whitespace) {
            match t[0] {
                "NAME" => {
                    model.name = t.get(1).unwrap_or(&"").to_string();
                    section = "NAME";
                    continue;
                }
                "OBJSENSE" => {
                    section = "OBJSENSE";
                    if t.len() == 1 {
                        continue;
                    }
                }
                "OBJNAME" => {
                    section = "OBJNAME";
                    if t.len() == 1 {
                        continue;
                    }
                }
                "ROWS" | "COLUMNS" | "RHS" | "RANGES" | "BOUNDS" => {
                    section = t[0];
                    continue;
                }
                "ENDATA" => {
                    ended = true;
                    break;
                }
                _ => {}
            }
        }
        match section {
            "OBJSENSE" => {
                let v = *t.last().unwrap();
                model.sense = match v.to_ascii_uppercase().as_str() {
                    "MIN" | "MINIMIZE" => 1.0,
                    "MAX" | "MAXIMIZE" => -1.0,
                    _ => return Err(err(format!("unknown objective sense {v}"))),
                };
            }
            "OBJNAME" => {
                requested_objective = Some(t.last().unwrap().to_string());
            }
            "ROWS" => {
                if t.len() != 2 {
                    return Err(err("invalid ROWS record".into()));
                }
                if row_ids.contains_key(t[1]) || free_rows.iter().any(|r| r == t[1]) {
                    return Err(err(format!("duplicate row {}", t[1])));
                }
                let kind = t[0].chars().next().unwrap();
                if kind == 'N' {
                    if objective.is_none() || requested_objective.as_deref() == Some(t[1]) {
                        objective = Some(t[1].to_string());
                    }
                    free_rows.push(t[1].to_string());
                    continue;
                }
                if !['L', 'G', 'E'].contains(&kind) {
                    return Err(err(format!("unsupported row type {kind}")));
                }
                row_ids.insert(t[1].to_string(), model.rows.len());
                model.rows.push(Row {
                    name: t[1].into(),
                    lower: 0.0,
                    upper: 0.0,
                    terms: vec![],
                });
                row_types.push(kind);
                row_rhs.push(0.0);
            }
            "COLUMNS" => {
                if t.iter().any(|v| v.trim_matches('\'') == "MARKER") {
                    integer = match t.last().unwrap().trim_matches('\'') {
                        "INTORG" => true,
                        "INTEND" => false,
                        other => return Err(err(format!("unknown integer marker {other}"))),
                    };
                    continue;
                }
                if t.len() != 3 && t.len() != 5 {
                    return Err(err("invalid COLUMNS record".into()));
                }
                let j = if let Some(&j) = col_ids.get(t[0]) {
                    j
                } else {
                    let j = model.variables.len();
                    col_ids.insert(t[0].into(), j);
                    model.variables.push(Variable {
                        name: t[0].into(),
                        lower: 0.0,
                        upper: f64::INFINITY,
                        integer,
                        cost: 0.0,
                    });
                    bound_seen.push(false);
                    upper_seen.push(false);
                    j
                };
                model.variables[j].integer |= integer;
                for pair in t[1..].chunks_exact(2) {
                    let value = number(pair[1]).map_err(&err)?;
                    if objective.as_deref() == Some(pair[0]) {
                        model.variables[j].cost += value;
                    } else if let Some(&i) = row_ids.get(pair[0]) {
                        model.rows[i].terms.push((j, value));
                    } else if !free_rows.iter().any(|r| r == pair[0]) {
                        return Err(err(format!("unknown row {}", pair[0])));
                    }
                }
            }
            "RHS" | "RANGES" => {
                if t.len() != 3 && t.len() != 5 {
                    return Err(err(format!("invalid {section} record")));
                }
                let selected = if section == "RHS" {
                    &mut rhs_name
                } else {
                    &mut range_name
                };
                if selected.is_none() {
                    *selected = Some(t[0].into());
                }
                if selected.as_deref() != Some(t[0]) {
                    continue;
                }
                for pair in t[1..].chunks_exact(2) {
                    let value = number(pair[1]).map_err(&err)?;
                    if let Some(&i) = row_ids.get(pair[0]) {
                        if section == "RHS" {
                            row_rhs[i] = value;
                        } else {
                            row_ranges.insert(i, value);
                        }
                    } else if objective.as_deref() == Some(pair[0]) && section == "RHS" {
                        model.offset = -value;
                    } else if !free_rows.iter().any(|r| r == pair[0]) {
                        return Err(err(format!("unknown row {}", pair[0])));
                    }
                }
            }
            "BOUNDS" => {
                if t.len() < 3 || t.len() > 4 {
                    return Err(err("invalid BOUNDS record".into()));
                }
                if bound_name.is_none() {
                    bound_name = Some(t[1].into());
                }
                if bound_name.as_deref() != Some(t[1]) {
                    continue;
                }
                let &j = col_ids
                    .get(t[2])
                    .ok_or_else(|| err(format!("bound on unknown column {}", t[2])))?;
                let v = &mut model.variables[j];
                bound_seen[j] = true;
                let val = if t.len() == 4 {
                    number(t[3]).map_err(&err)?
                } else {
                    0.0
                };
                match t[0] {
                    "LO" | "LI" => {
                        v.lower = val;
                        v.integer |= t[0] == "LI";
                    }
                    "UP" | "UI" => {
                        v.upper = val;
                        v.integer |= t[0] == "UI";
                        upper_seen[j] = true;
                    }
                    "FX" => {
                        v.lower = val;
                        v.upper = val;
                        upper_seen[j] = true;
                    }
                    "FR" => {
                        v.lower = f64::NEG_INFINITY;
                        v.upper = f64::INFINITY;
                        upper_seen[j] = true;
                    }
                    "MI" => {
                        v.lower = f64::NEG_INFINITY;
                    }
                    "PL" => {
                        v.upper = f64::INFINITY;
                        upper_seen[j] = true;
                    }
                    "BV" => {
                        v.lower = 0.0;
                        v.upper = 1.0;
                        v.integer = true;
                        upper_seen[j] = true;
                    }
                    other => return Err(err(format!("unsupported bound type {other}"))),
                }
            }
            _ => return Err(err(format!("unexpected record in section {section}"))),
        }
    }
    if !ended {
        return Err("missing ENDATA".into());
    }
    if let Some(requested) = requested_objective {
        if objective.as_deref() != Some(&requested) {
            return Err("OBJNAME row missing".into());
        }
    }
    for (j, v) in model.variables.iter_mut().enumerate() {
        if v.integer && !bound_seen[j] && !upper_seen[j] {
            v.upper = 1.0;
        }
    }
    for (i, row) in model.rows.iter_mut().enumerate() {
        let rhs = row_rhs[i];
        (row.lower, row.upper) = match row_types[i] {
            'L' => (f64::NEG_INFINITY, rhs),
            'G' => (rhs, f64::INFINITY),
            _ => (rhs, rhs),
        };
        if let Some(&r) = row_ranges.get(&i) {
            match row_types[i] {
                'L' => row.lower = rhs - r.abs(),
                'G' => row.upper = rhs + r.abs(),
                _ if r >= 0.0 => row.upper = rhs + r,
                _ => row.lower = rhs + r,
            }
        }
        row.terms.sort_unstable_by_key(|&(j, _)| j);
        let mut merged: Vec<(usize, f64)> = Vec::with_capacity(row.terms.len());
        for &(j, a) in &row.terms {
            if let Some(last) = merged.last_mut().filter(|p| p.0 == j) {
                last.1 += a;
            } else {
                merged.push((j, a));
            }
        }
        merged.retain(|&(_, a)| a != 0.0);
        row.terms = merged;
    }
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_zero_lower_is_not_erased_by_negative_upper() {
        let m = parse_mps(
            "NAME t\nROWS\n N o\nCOLUMNS\n x o 1\nBOUNDS\n LO b x 0\n UP b x -1\nENDATA\n",
        )
        .unwrap();
        assert_eq!(m.variables[0].lower, 0.0);
        assert_eq!(m.variables[0].upper, -1.0);
    }
    #[test]
    fn ranges_offsets_markers() {
        let m = parse_mps("NAME test\nOBJSENSE\n MAX\nROWS\n N obj\n E r\nCOLUMNS\n mark 'MARKER' 'INTORG'\n x obj 3 r 2\n mark 'MARKER' 'INTEND'\nRHS\n rhs obj -4 r 5\nRANGES\n rng r -2\nBOUNDS\n UP b x 8\nENDATA\n").unwrap();
        assert_eq!(m.sense, -1.0);
        assert_eq!(m.offset, 4.0);
        assert_eq!((m.rows[0].lower, m.rows[0].upper), (3.0, 5.0));
        assert!(m.variables[0].integer);
        assert_eq!(m.objective(&[2.0]), 10.0);
        assert!(m.check(&[2.0], 1e-7).feasible);
        assert!(!m.check(&[2.1], 1e-7).feasible);
    }
}
