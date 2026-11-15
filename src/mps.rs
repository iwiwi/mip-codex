//! MPS reader, implemented from format specifications, not solver source code.
//! https://www.ibm.com/docs/en/cofz/12.9.0?topic=standard-records-in-mps-format
//! Unsupported extensions and multiple vectors are errors rather than silent model changes.
use crate::model::{Model, Row, Variable};
use flate2::read::GzDecoder;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

fn number(s: &str) -> Result<f64, String> {
    let x: f64 = s
        .replace(['d', 'D'], "E")
        .parse()
        .map_err(|_| format!("invalid number {s}"))?;
    if !x.is_finite() {
        return Err(format!("nonfinite number {s}"));
    }
    // 非ゼロ十進数を0へ落とし、元の制約/目的を変えて解かない。
    if x == 0.
        && s.split(['e', 'E', 'd', 'D'])
            .next()
            .unwrap_or("")
            .bytes()
            .any(|b| matches!(b, b'1'..=b'9'))
    {
        return Err(format!("number underflow {s}"));
    }
    Ok(x)
}
/// 元の十進トークンが厳密に小さい整数か。f64への丸めだけで判定しない。
fn exact_integer(token: &str) -> Option<i64> {
    let (negative, token) = if let Some(s) = token.strip_prefix('-') {
        (true, s)
    } else {
        (false, token.strip_prefix('+').unwrap_or(token))
    };
    let split = token.find(['e', 'E', 'd', 'D']);
    let (mantissa, exponent) = if let Some(i) = split {
        (&token[..i], token[i + 1..].parse::<i32>().ok()?)
    } else {
        (token, 0)
    };
    let fractional = mantissa.find('.').map_or(0, |i| mantissa.len() - i - 1);
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    if !digits.bytes().all(|b| b.is_ascii_digit()) || digits.is_empty() {
        return None;
    }
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Some(0);
    }
    let shift = i64::from(exponent).checked_sub(i64::try_from(fractional).ok()?)?;
    let value = if shift >= 0 {
        if digits.len() as i64 + shift > 16 {
            return None;
        }
        digits
            .parse::<i64>()
            .ok()?
            .checked_mul(10i64.checked_pow(shift as u32)?)?
    } else {
        let remove = usize::try_from(-shift).ok()?;
        if remove >= digits.len() || !digits[digits.len() - remove..].bytes().all(|b| b == b'0') {
            return None;
        }
        digits[..digits.len() - remove].parse::<i64>().ok()?
    };
    if value > (1i64 << 52) {
        return None;
    }
    Some(if negative { -value } else { value })
}
pub fn read(path: impl AsRef<Path>) -> Result<Model, String> {
    let path = path.as_ref();
    let file = File::open(path).map_err(|e| e.to_string())?;
    if path.extension().is_some_and(|e| e == "gz") {
        parse(BufReader::new(GzDecoder::new(file)))
    } else {
        parse(BufReader::new(file))
    }
}
pub fn parse(reader: impl BufRead) -> Result<Model, String> {
    let mut model = Model {
        name: String::new(),
        maximize: false,
        offset: 0.0,
        variables: vec![],
        rows: vec![],
    };
    let mut row_names = HashMap::new();
    let mut columns = HashMap::new();
    let mut kinds = vec![];
    let mut rhs = vec![];
    let mut ranges = HashMap::new();
    let mut explicit_bound = vec![];
    let mut raw_rows: Vec<Row> = vec![];
    let mut section = String::new();
    let mut objective_name: Option<String> = None;
    let mut integer = false;
    let mut ended = false;
    let mut vectors: HashMap<String, String> = HashMap::new();
    for (line_no, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| e.to_string())?;
        let fail = |s: String| format!("line {}: {s}", line_no + 1);
        if line.trim().is_empty() || line.starts_with('*') {
            continue;
        }
        let tokens: Vec<&str> = line
            .split_whitespace()
            .take_while(|s| !s.starts_with('$'))
            .collect();
        if tokens.is_empty() {
            continue;
        }
        let t = &tokens;
        let header = !line.starts_with(char::is_whitespace);
        if header {
            section = t[0].to_string();
            match t[0] {
                "NAME" => {
                    model.name = t.get(1).unwrap_or(&"").to_string();
                }
                "ROWS" | "COLUMNS" | "RHS" | "RANGES" | "BOUNDS" => {}
                "OBJSENSE" => {
                    if let Some(s) = t.get(1) {
                        model.maximize = sense(s).map_err(&fail)?;
                    }
                }
                "OBJNAME" => {
                    objective_name = t.get(1).map(|s| s.to_string());
                }
                "ENDATA" => {
                    ended = true;
                    break;
                }
                _ => return Err(fail(format!("unsupported MPS section {}", t[0]))),
            }
            continue;
        }
        match section.as_str() {
            "OBJSENSE" => {
                if t.len() != 1 {
                    return Err(fail("invalid OBJSENSE".into()));
                }
                model.maximize = sense(t[0]).map_err(&fail)?;
            }
            "OBJNAME" => {
                if t.len() != 1 {
                    return Err(fail("invalid OBJNAME".into()));
                }
                objective_name = Some(t[0].into());
            }
            "ROWS" => {
                if t.len() != 2 || !["N", "L", "G", "E"].contains(&t[0]) {
                    return Err(fail("invalid row".into()));
                }
                let i = raw_rows.len();
                if row_names.insert(t[1].to_string(), i).is_some() {
                    return Err(fail("duplicate row".into()));
                }
                kinds.push(t[0].to_string());
                rhs.push(0.0);
                raw_rows.push(Row {
                    exact_integral: true,
                    name: t[1].into(),
                    lower: None,
                    upper: None,
                    coefficients: vec![],
                });
            }
            "COLUMNS" => {
                if t.len() >= 3 && t[1].trim_matches('\'') == "MARKER" {
                    integer = match t[2].trim_matches('\'') {
                        "INTORG" if !integer => true,
                        "INTEND" if integer => false,
                        _ => return Err(fail("invalid integer marker nesting".into())),
                    };
                    continue;
                }
                if t.len() != 3 && t.len() != 5 {
                    return Err(fail("expected column and row/value pairs".into()));
                }
                let j = if let Some(&j) = columns.get(t[0]) {
                    j
                } else {
                    let j = model.variables.len();
                    columns.insert(t[0].to_string(), j);
                    model.variables.push(Variable {
                        name: t[0].into(),
                        lower: Some(0.0),
                        upper: None,
                        integer,
                        objective: 0.0,
                    });
                    explicit_bound.push(false);
                    j
                };
                if integer {
                    model.variables[j].integer = true;
                }
                for pair in t[1..].chunks_exact(2) {
                    let &i = row_names
                        .get(pair[0])
                        .ok_or_else(|| fail(format!("unknown row {}", pair[0])))?;
                    let value = number(pair[1]).map_err(&fail)?;
                    raw_rows[i].exact_integral &=
                        exact_integer(pair[1]).is_some_and(|n| n as f64 == value);
                    raw_rows[i].coefficients.push((j, value));
                }
            }
            "RHS" | "RANGES" => {
                if t.len() != 3 && t.len() != 5 {
                    return Err(fail("expected vector and row/value pairs".into()));
                }
                let vector = vectors
                    .entry(section.clone())
                    .or_insert_with(|| t[0].into());
                if vector != t[0] {
                    return Err(fail("multiple RHS/RANGES vectors unsupported".into()));
                }
                for pair in t[1..].chunks_exact(2) {
                    let &i = row_names
                        .get(pair[0])
                        .ok_or_else(|| fail(format!("unknown row {}", pair[0])))?;
                    let value = number(pair[1]).map_err(&fail)?;
                    raw_rows[i].exact_integral &=
                        exact_integer(pair[1]).is_some_and(|n| n as f64 == value);
                    if section == "RHS" {
                        rhs[i] = value;
                    } else {
                        ranges.insert(i, value);
                    }
                }
            }
            "BOUNDS" => {
                if t.len() < 3 || t.len() > 4 {
                    return Err(fail("invalid bound".into()));
                }
                let vector = vectors
                    .entry(section.clone())
                    .or_insert_with(|| t[1].into());
                if vector != t[1] {
                    return Err(fail("multiple BOUNDS vectors unsupported".into()));
                }
                let &j = columns
                    .get(t[2])
                    .ok_or_else(|| fail(format!("unknown column {}", t[2])))?;
                let v = &mut model.variables[j];
                let value = if t.len() == 4 {
                    Some(number(t[3]).map_err(&fail)?)
                } else {
                    None
                };
                match t[0] {
                    "LO" | "LI" => {
                        v.lower = Some(value.ok_or_else(|| fail("missing bound value".into()))?);
                        if t[0] == "LI" {
                            v.integer = true;
                        }
                    }
                    "UP" | "UI" => {
                        let b = value.ok_or_else(|| fail("missing bound value".into()))?;
                        // Standard MPS negative upper bounds imply -infinity lower if still default.
                        if b < 0.0 && !explicit_bound[j] {
                            v.lower = None;
                        }
                        v.upper = Some(b);
                        if t[0] == "UI" {
                            v.integer = true;
                        }
                    }
                    "FX" => {
                        let b = value.ok_or_else(|| fail("missing bound value".into()))?;
                        v.lower = Some(b);
                        v.upper = Some(b);
                    }
                    "FR" => {
                        v.lower = None;
                        v.upper = None;
                    }
                    "MI" => {
                        v.lower = None;
                    }
                    "PL" => {
                        v.upper = None;
                    }
                    "BV" => {
                        v.lower = Some(0.0);
                        v.upper = Some(1.0);
                        v.integer = true;
                    }
                    _ => return Err(fail(format!("unsupported bound type {}", t[0]))),
                }
                explicit_bound[j] = true;
            }
            _ => return Err(fail(format!("unexpected data in section {section}"))),
        }
    }
    if !ended || integer {
        return Err("truncated MPS or unclosed integer marker".into());
    }
    let objective = if let Some(name) = objective_name {
        *row_names.get(&name).ok_or("unknown OBJNAME")?
    } else {
        kinds
            .iter()
            .position(|s| s == "N")
            .ok_or("missing objective row")?
    };
    if kinds[objective] != "N" {
        return Err("objective must be a free row".into());
    }
    model.offset = -rhs[objective];
    for (j, v) in model.variables.iter_mut().enumerate() {
        if v.integer && !explicit_bound[j] {
            v.upper = Some(1.0);
        }
    }
    for (i, mut row) in raw_rows.into_iter().enumerate() {
        row.coefficients.sort_unstable_by_key(|p| p.0);
        let mut merged: Vec<(usize, f64)> = Vec::new();
        for (j, a) in row.coefficients {
            if let Some(last) = merged.last_mut().filter(|p| p.0 == j) {
                last.1 += a;
                row.exact_integral &= last.1.abs() <= (1u64 << 52) as f64 && last.1.fract() == 0.;
            } else {
                merged.push((j, a));
            }
        }
        if merged.iter().any(|p| !p.1.is_finite()) {
            return Err("coefficient overflow".into());
        }
        merged.retain(|p| p.1 != 0.0);
        row.coefficients = merged;
        if i == objective {
            for (j, a) in row.coefficients {
                model.variables[j].objective = a;
            }
            continue;
        }
        let b = rhs[i];
        match kinds[i].as_str() {
            "L" => row.upper = Some(b),
            "G" => row.lower = Some(b),
            "E" => {
                row.lower = Some(b);
                row.upper = Some(b);
            }
            "N" => continue,
            _ => unreachable!(),
        }
        if let Some(&r) = ranges.get(&i) {
            match kinds[i].as_str() {
                "L" => row.lower = Some(b - r.abs()),
                "G" => row.upper = Some(b + r.abs()),
                "E" if r >= 0.0 => row.upper = Some(b + r),
                "E" => row.lower = Some(b + r),
                _ => unreachable!(),
            }
        }
        if row
            .lower
            .into_iter()
            .chain(row.upper)
            .any(|v| !v.is_finite())
        {
            return Err("range overflow".into());
        }
        model.rows.push(row);
    }
    Ok(model)
}
fn sense(s: &str) -> Result<bool, String> {
    match s {
        "MIN" | "MINIMIZE" => Ok(false),
        "MAX" | "MAXIMIZE" => Ok(true),
        _ => Err(format!("unsupported objective sense {s}")),
    }
}
