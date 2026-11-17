//! MPS reader, implemented from format specifications, not solver source code.
//! https://www.ibm.com/docs/en/cofz/12.9.0?topic=standard-records-in-mps-format
//! Unsupported extensions and multiple vectors are errors rather than silent model changes.
use crate::model::{Model, Row, Variable};
use flate2::read::GzDecoder;
use num_bigint::BigInt;
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
// 重複項だけは元の十進値を合計する。通常の単独項で多倍長演算は行わない。
fn decimal_parts(token: &str) -> Option<(BigInt, i32)> {
    let (negative, token) = if let Some(s) = token.strip_prefix('-') {
        (true, s)
    } else {
        (false, token.strip_prefix('+').unwrap_or(token))
    };
    let split = token.find(['e', 'E', 'd', 'D']);
    let mantissa = split.map_or(token, |i| &token[..i]);
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    if digits.bytes().all(|b| b == b'0') {
        return Some((BigInt::from(0), 0));
    }
    if digits.len() > 10000 {
        return None;
    }
    let exponent = if let Some(i) = split {
        token[i + 1..].parse::<i32>().ok()?
    } else {
        0
    };
    let fractional = mantissa.find('.').map_or(0, |i| mantissa.len() - i - 1);
    let exponent = exponent.checked_sub(i32::try_from(fractional).ok()?)?;
    if !(-10000..=10000).contains(&exponent) {
        return None;
    }
    let mut value = BigInt::parse_bytes(digits.as_bytes(), 10)?;
    if negative {
        value = -value;
    }
    Some((value, exponent))
}
/// Compare the original decimal with the exact IEEE binary rational.
fn exact_float_token(token: &str, value: f64) -> bool {
    compare_float_token(token, value) == Some(std::cmp::Ordering::Equal)
}
pub(crate) fn compare_float_token(token: &str, value: f64) -> Option<std::cmp::Ordering> {
    if !value.is_finite() {
        return None;
    }
    if let Some(integer) = exact_integer(token) {
        return (integer as f64).partial_cmp(&value);
    }
    compare_float_sum_token(token, &[value])
}
/// Compare a decimal with an exact sum of finite binary constants, without rounding it first.
pub(crate) fn compare_float_sum_token(token: &str, values: &[f64]) -> Option<std::cmp::Ordering> {
    let (mut decimal, exponent) = decimal_parts(token)?;
    let mut binary_exponent = 0;
    let mut first = true;
    for &value in values {
        let (_, mantissa, e) = crate::exact::parts(value)?;
        if mantissa != 0 {
            if first || e < binary_exponent {
                binary_exponent = e;
            }
            first = false;
        }
    }
    let mut binary = BigInt::from(0);
    for &value in values {
        let (negative, mantissa, e) = crate::exact::parts(value)?;
        if mantissa == 0 {
            continue;
        }
        let term = BigInt::from(mantissa) << (e - binary_exponent) as usize;
        if negative {
            binary -= term;
        } else {
            binary += term;
        }
    }
    if exponent >= 0 {
        decimal *= BigInt::from(10u8).pow(exponent as u32);
    } else {
        binary *= BigInt::from(10u8).pow((-exponent) as u32);
    }
    if binary_exponent >= 0 {
        binary <<= binary_exponent as usize;
    } else {
        decimal <<= (-binary_exponent) as usize;
    }
    Some(decimal.cmp(&binary))
}
fn decimal_sum_metadata<'a>(
    entries: impl Iterator<Item = (f64, Option<&'a str>)>,
) -> Result<(f64, bool), String> {
    let mut parts = Vec::new();
    for (value, token) in entries {
        let pair = if let Some(token) = token {
            decimal_parts(token).ok_or("unsupported duplicate decimal size/exponent")?
        } else {
            (BigInt::from(value as i64), 0)
        };
        parts.push(pair);
    }
    let exponent = parts
        .iter()
        .map(|p| p.1)
        .min()
        .ok_or("empty coefficient sum")?;
    let mut sum = BigInt::from(0);
    for (value, e) in parts {
        sum += value * BigInt::from(10u8).pow((e - exponent) as u32);
    }
    let value: f64 = format!("{sum}e{exponent}")
        .parse()
        .map_err(|_| "coefficient conversion failed")?;
    if !value.is_finite() {
        return Err("coefficient overflow".into());
    }
    if value == 0. && sum != BigInt::from(0) {
        return Err("coefficient underflow".into());
    }
    Ok((
        value,
        exact_float_token(&format!("{sum}e{exponent}"), value),
    ))
}
// 元十進数の和のfloor/ceil。f64への変換や変数shiftより前に決める。
fn decimal_lattice(tokens: &[&str]) -> Option<(f64, f64)> {
    let mut parts = Vec::new();
    for &token in tokens {
        parts.push(decimal_parts(token)?);
    }
    let exponent = parts.iter().map(|p| p.1).min()?;
    let mut sum = BigInt::from(0);
    for (value, e) in parts {
        sum += value * BigInt::from(10u8).pow((e - exponent) as u32);
    }
    let denominator = if exponent < 0 {
        BigInt::from(10u8).pow((-exponent) as u32)
    } else {
        sum *= BigInt::from(10u8).pow(exponent as u32);
        BigInt::from(1)
    };
    let quotient = &sum / &denominator;
    let remainder = &sum % &denominator;
    let zero = BigInt::from(0);
    let floor = if remainder < zero {
        &quotient - BigInt::from(1)
    } else {
        quotient.clone()
    };
    let ceil = if remainder > zero {
        quotient + BigInt::from(1)
    } else {
        quotient
    };
    let floor = floor.to_string().parse::<i64>().ok()?;
    let ceil = ceil.to_string().parse::<i64>().ok()?;
    (floor.unsigned_abs() <= (1u64 << 52) && ceil.unsigned_abs() <= (1u64 << 52))
        .then_some((floor as f64, ceil as f64))
}
fn row_integer_bounds(
    kind: &str,
    rhs: &str,
    range: Option<(f64, &str)>,
) -> Option<[Option<f64>; 2]> {
    let (floor, ceil) = decimal_lattice(&[rhs])?;
    let mut result = match kind {
        "L" => [None, Some(floor)],
        "G" => [Some(ceil), None],
        "E" => [Some(ceil), Some(floor)],
        _ => return None,
    };
    if let Some((r, token)) = range {
        let flip = match kind {
            "L" => r >= 0.,
            "G" => r < 0.,
            _ => false,
        };
        let negated = flip.then(|| negate_token(token));
        let adjusted = negated.as_deref().unwrap_or(token);
        let (floor, ceil) = decimal_lattice(&[rhs, adjusted])?;
        match kind {
            "L" => result[0] = Some(ceil),
            "G" => result[1] = Some(floor),
            "E" if r >= 0. => result[1] = Some(floor),
            "E" => result[0] = Some(ceil),
            _ => return None,
        }
    }
    Some(result)
}
fn enclose(value: f64, exact: bool) -> Option<[f64; 2]> {
    let bounds = if exact {
        [value, value]
    } else {
        [value.next_down(), value.next_up()]
    };
    bounds.iter().all(|x| x.is_finite()).then_some(bounds)
}
fn negate_token(s: &str) -> String {
    if let Some(s) = s.strip_prefix('-') {
        s.into()
    } else {
        format!("-{}", s.strip_prefix('+').unwrap_or(s))
    }
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
        original_decimal: true,
        offset_enclosure: Some([0., 0.]),
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
    let mut rhs_tokens: Vec<Option<String>> = vec![];
    let mut range_tokens: HashMap<usize, Option<String>> = HashMap::new();
    let mut ranges = HashMap::new();
    let mut explicit_bound = vec![];
    let mut bound_tokens: Vec<[Option<String>; 2]> = vec![];
    let mut raw_rows: Vec<Row> = vec![];
    let mut raw_lexical: Vec<Vec<Option<String>>> = vec![];
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
                rhs_tokens.push(None);
                raw_lexical.push(Vec::new());
                raw_rows.push(Row {
                    integer_bounds: None,
                    lower_enclosure: None,
                    upper_enclosure: None,
                    exact_coefficients: true,
                    exact_coefficient_flags: vec![],
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
                        integer_bounds: None,
                        objective_enclosure: Some([0., 0.]),
                        lower_enclosure: Some([0., 0.]),
                        upper_enclosure: None,
                        name: t[0].into(),
                        lower: Some(0.0),
                        upper: None,
                        integer,
                        objective: 0.0,
                    });
                    explicit_bound.push(false);
                    bound_tokens.push([None, None]);
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
                    let exact = exact_integer(pair[1]).is_some_and(|n| n as f64 == value);
                    raw_rows[i].exact_integral &= exact;
                    raw_lexical[i].push(if exact {
                        None
                    } else {
                        Some(pair[1].to_string())
                    });
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
                    let exact = exact_integer(pair[1]).is_some_and(|n| n as f64 == value);
                    raw_rows[i].exact_integral &= exact;
                    let token = if exact {
                        None
                    } else {
                        Some(pair[1].to_string())
                    };
                    if section == "RHS" {
                        rhs[i] = value;
                        rhs_tokens[i] = token;
                    } else {
                        ranges.insert(i, value);
                        range_tokens.insert(i, token);
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
                let token = if t.len() == 4 && exact_integer(t[3]).is_none() {
                    Some(t[3].to_string())
                } else {
                    None
                };
                let enclosure = value.and_then(|x| enclose(x, exact_float_token(t[3], x)));
                match t[0] {
                    "LO" | "LI" => {
                        v.lower = Some(value.ok_or_else(|| fail("missing bound value".into()))?);
                        v.lower_enclosure = enclosure;
                        bound_tokens[j][0] = token.clone();
                        if t[0] == "LI" {
                            v.integer = true;
                        }
                    }
                    "UP" | "UI" => {
                        let b = value.ok_or_else(|| fail("missing bound value".into()))?;
                        // Standard MPS negative upper bounds imply -infinity lower if still default.
                        if b < 0.0 && !explicit_bound[j] {
                            v.lower = None;
                            v.lower_enclosure = None;
                            bound_tokens[j][0] = None;
                        }
                        v.upper = Some(b);
                        v.upper_enclosure = enclosure;
                        bound_tokens[j][1] = token.clone();
                        if t[0] == "UI" {
                            v.integer = true;
                        }
                    }
                    "FX" => {
                        let b = value.ok_or_else(|| fail("missing bound value".into()))?;
                        v.lower = Some(b);
                        v.lower_enclosure = enclosure;
                        bound_tokens[j][0] = token.clone();
                        v.upper = Some(b);
                        v.upper_enclosure = enclosure;
                        bound_tokens[j][1] = token.clone();
                    }
                    "FR" => {
                        v.lower = None;
                        v.lower_enclosure = None;
                        bound_tokens[j][0] = None;
                        v.upper = None;
                        v.upper_enclosure = None;
                        bound_tokens[j][1] = None;
                    }
                    "MI" => {
                        v.lower = None;
                        v.lower_enclosure = None;
                        bound_tokens[j][0] = None;
                    }
                    "PL" => {
                        v.upper = None;
                        v.upper_enclosure = None;
                        bound_tokens[j][1] = None;
                    }
                    "BV" => {
                        bound_tokens[j] = [None, None];
                        v.lower = Some(0.0);
                        v.lower_enclosure = Some([0., 0.]);
                        v.upper = Some(1.0);
                        v.upper_enclosure = Some([1., 1.]);
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
    model.offset_enclosure = enclose(
        model.offset,
        rhs_tokens[objective]
            .as_deref()
            .is_none_or(|t| exact_float_token(t, -model.offset)),
    );
    for (j, v) in model.variables.iter_mut().enumerate() {
        if v.integer && !explicit_bound[j] {
            v.upper = Some(1.0);
            v.upper_enclosure = Some([1., 1.]);
        }
        if v.integer
            && (bound_tokens[j].iter().any(Option::is_some)
                || v.lower.into_iter().chain(v.upper).any(|b| b.fract() != 0.))
        {
            let mut rounded = [None, None];
            let mut supported = true;
            for (side, (bound, enclosure)) in
                [(v.lower, v.lower_enclosure), (v.upper, v.upper_enclosure)]
                    .into_iter()
                    .enumerate()
            {
                if let Some(b) = bound {
                    rounded[side] = if let Some(token) = bound_tokens[j][side].as_deref() {
                        decimal_lattice(&[token])
                            .map(|(floor, ceil)| if side == 0 { ceil } else { floor })
                    } else if enclosure == Some([b, b]) && b.abs() <= (1u64 << 52) as f64 {
                        Some(if side == 0 { b.ceil() } else { b.floor() })
                    } else {
                        None
                    };
                    supported &= rounded[side].is_some();
                }
            }
            if supported {
                v.integer_bounds = Some(rounded);
            }
        }
    }
    for (i, (mut row, lexical)) in raw_rows.into_iter().zip(raw_lexical).enumerate() {
        let mut entries: Vec<_> = row
            .coefficients
            .into_iter()
            .zip(lexical)
            .map(|((j, a), s)| (j, a, s))
            .collect();
        entries.sort_unstable_by_key(|p| p.0);
        let mut merged: Vec<(usize, f64)> = Vec::new();
        let mut merged_exact = Vec::new();
        let mut first = 0;
        while first < entries.len() {
            let j = entries[first].0;
            let mut end = first + 1;
            while end < entries.len() && entries[end].0 == j {
                end += 1;
            }
            let (value, exact_coefficient) = if end == first + 1 {
                (
                    entries[first].1,
                    entries[first]
                        .2
                        .as_deref()
                        .is_none_or(|t| exact_float_token(t, entries[first].1)),
                )
            } else {
                decimal_sum_metadata(entries[first..end].iter().map(|p| (p.1, p.2.as_deref())))?
            };
            row.exact_coefficients &= exact_coefficient;
            row.exact_integral &= value.abs() <= (1u64 << 52) as f64 && value.fract() == 0.;
            if i == objective {
                model.variables[j].objective_enclosure = enclose(value, exact_coefficient);
            }
            if value != 0. {
                merged_exact.push(exact_coefficient);
            }
            merged.push((j, value));
            first = end;
        }
        if merged.iter().any(|p| !p.1.is_finite()) {
            return Err("coefficient overflow".into());
        }
        merged.retain(|p| p.1 != 0.0);
        row.coefficients = merged;
        if !row.exact_coefficients {
            row.exact_coefficient_flags = merged_exact;
        }
        if i == objective {
            for (j, a) in row.coefficients {
                model.variables[j].objective = a;
            }
            continue;
        }
        let b = rhs[i];
        let enclosure = enclose(
            b,
            rhs_tokens[i]
                .as_deref()
                .is_none_or(|t| exact_float_token(t, b)),
        );
        match kinds[i].as_str() {
            "L" => {
                row.upper = Some(b);
                row.upper_enclosure = enclosure;
            }
            "G" => {
                row.lower = Some(b);
                row.lower_enclosure = enclosure;
            }
            "E" => {
                row.lower = Some(b);
                row.upper = Some(b);
                row.lower_enclosure = enclosure;
                row.upper_enclosure = enclosure;
            }
            "N" => continue,
            _ => unreachable!(),
        }
        if let Some(&r) = ranges.get(&i) {
            let flip = match kinds[i].as_str() {
                "L" => r >= 0.,
                "G" => r < 0.,
                _ => false,
            };
            let token = range_tokens.get(&i).and_then(|s| s.as_deref());
            let negated = if flip { token.map(negate_token) } else { None };
            let range_token = if flip { negated.as_deref() } else { token };
            let (value, exact_endpoint) = decimal_sum_metadata(
                [
                    (b, rhs_tokens[i].as_deref()),
                    (if flip { -r } else { r }, range_token),
                ]
                .into_iter(),
            )?;
            let enclosure = enclose(value, exact_endpoint);
            match kinds[i].as_str() {
                "L" => {
                    row.lower = Some(value);
                    row.lower_enclosure = enclosure;
                }
                "G" => {
                    row.upper = Some(value);
                    row.upper_enclosure = enclosure;
                }
                "E" if r >= 0. => {
                    row.upper = Some(value);
                    row.upper_enclosure = enclosure;
                }
                "E" => {
                    row.lower = Some(value);
                    row.lower_enclosure = enclosure;
                }
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
        let exact_integer_endpoints = row
            .lower
            .is_none_or(|v| v.fract() == 0. && row.lower_enclosure == Some([v; 2]))
            && row
                .upper
                .is_none_or(|v| v.fract() == 0. && row.upper_enclosure == Some([v; 2]));
        if !exact_integer_endpoints {
            // Noneのtokenはパーサが確認済みの小さい整数（未指定RHSは0）。
            let rhs_token = rhs_tokens[i]
                .clone()
                .unwrap_or_else(|| format!("{:.0}", rhs[i]));
            let range_token = ranges.get(&i).map(|&r| {
                (
                    r,
                    range_tokens
                        .get(&i)
                        .and_then(|t| t.clone())
                        .unwrap_or_else(|| format!("{r:.0}")),
                )
            });
            row.integer_bounds = row_integer_bounds(
                &kinds[i],
                &rhs_token,
                range_token.as_ref().map(|(r, t)| (*r, t.as_str())),
            );
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
