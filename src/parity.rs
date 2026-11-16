//! Exact binary parity equalities with one private ±2 integer per row.
//! A full-rank GF(2) system and exact reconstruction prove a unique feasible point.
use crate::model::Model;
use serde::Serialize;
use std::time::Instant;
#[derive(Debug, Serialize)]
pub struct Proof {
    pub binary_variables: Vec<usize>,
    pub auxiliary_variables: Vec<usize>,
    pub pivots: Vec<usize>,
    pub rank: usize,
    pub row_count: usize,
}
pub struct Answer {
    pub point: Vec<f64>,
    pub proof: Proof,
}
fn integer(v: f64) -> Option<i128> {
    (v.is_finite() && v.fract() == 0. && v.abs() <= (1u64 << 52) as f64).then_some(v as i128)
}
fn bit(row: &[u64], col: usize) -> bool {
    row[col / 64] >> (col % 64) & 1 != 0
}
pub fn solve(model: &Model, deadline: Instant) -> Option<Answer> {
    if model.variables.is_empty()
        || model.rows.is_empty()
        || model.rows.len() > 2048
        || model.variables.len() > 4096
        || Instant::now() >= deadline
    {
        return None;
    }
    let mut binary = Vec::new();
    let mut indices = vec![None; model.variables.len()];
    let mut degrees = vec![0usize; model.variables.len()];
    for (j, v) in model.variables.iter().enumerate() {
        if !v.integer {
            return None;
        }
        if v.lower == Some(0.)
            && v.upper == Some(1.)
            && v.lower_enclosure == Some([0., 0.])
            && v.upper_enclosure == Some([1., 1.])
        {
            indices[j] = Some(binary.len());
            binary.push(j);
        }
    }
    if binary.is_empty() || binary.len() > 2048 {
        return None;
    }
    for row in &model.rows {
        for &(j, _) in &row.coefficients {
            *degrees.get_mut(j)? += 1;
        }
    }
    let n = binary.len();
    let words = (n + 1).div_ceil(64);
    let mut matrix = Vec::with_capacity(model.rows.len());
    let mut auxiliaries = Vec::new();
    let mut rhs_values = Vec::new();
    let mut aux_coefficients = Vec::new();
    for row in &model.rows {
        let rhs = row.lower?;
        if row.upper != Some(rhs)
            || !row.exact_integral
            || row.lower_enclosure != Some([rhs, rhs])
            || row.upper_enclosure != Some([rhs, rhs])
        {
            return None;
        }
        let rhs = integer(rhs)?;
        let mut bits = vec![0u64; words];
        let mut auxiliary = None;
        for &(j, a) in &row.coefficients {
            let coefficient = integer(a)?;
            if let Some(k) = indices[j] {
                if coefficient % 2 != 0 {
                    bits[k / 64] ^= 1u64 << (k % 64);
                }
            } else {
                if auxiliary.is_some() || degrees[j] != 1 || coefficient.abs() != 2 {
                    return None;
                }
                auxiliary = Some((j, coefficient));
            }
        }
        let (j, a) = auxiliary?;
        if rhs % 2 != 0 {
            bits[n / 64] |= 1u64 << (n % 64);
        }
        matrix.push(bits);
        auxiliaries.push(j);
        rhs_values.push(rhs);
        aux_coefficients.push(a);
    }
    if indices
        .iter()
        .enumerate()
        .any(|(j, i)| i.is_none() && degrees[j] != 1)
    {
        return None;
    }
    let mut rank = 0;
    let mut pivots = Vec::new();
    let mut work = 0usize;
    for col in 0..n {
        if Instant::now() >= deadline {
            return None;
        }
        let Some(pivot) = (rank..matrix.len()).find(|&i| bit(&matrix[i], col)) else {
            continue;
        };
        matrix.swap(rank, pivot);
        for i in 0..matrix.len() {
            if i == rank || !bit(&matrix[i], col) {
                continue;
            }
            work = work.saturating_add(words);
            if work > 20_000_000 {
                return None;
            }
            for k in 0..words {
                matrix[i][k] ^= matrix[rank][k];
            }
        }
        pivots.push(col);
        rank += 1;
    }
    if rank != n {
        return None;
    }
    for row in &matrix[rank..] {
        if bit(row, n) {
            return None;
        }
    }
    let mut point = vec![0.; model.variables.len()];
    for (i, &col) in pivots.iter().enumerate() {
        point[binary[col]] = if bit(&matrix[i], n) { 1. } else { 0. };
    }
    for (i, row) in model.rows.iter().enumerate() {
        let mut residual = rhs_values[i];
        for &(j, a) in &row.coefficients {
            if indices[j].is_some() {
                residual = residual.checked_sub(integer(a)?.checked_mul(point[j] as i128)?)?;
            }
        }
        let a = aux_coefficients[i];
        if residual % a != 0 {
            return None;
        }
        let value = residual / a;
        if value.abs() > (1i128 << 52) {
            return None;
        }
        point[auxiliaries[i]] = value as f64;
    }
    for (v, &x) in model.variables.iter().zip(&point) {
        if let Some(l) = v.lower {
            let b = v.lower_enclosure?;
            if b[0] > l || b[1] < l || x < b[1] {
                return None;
            }
        }
        if let Some(u) = v.upper {
            let b = v.upper_enclosure?;
            if b[0] > u || b[1] < u || x > b[0] {
                return None;
            }
        }
    }
    if Instant::now() >= deadline || model.check(&point, true, 0.).is_err() {
        return None;
    }
    Some(Answer {
        point,
        proof: Proof {
            binary_variables: binary,
            auxiliary_variables: auxiliaries,
            pivots,
            rank,
            row_count: model.rows.len(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const MODEL: &str = "NAME parity\nROWS\n N obj\n E r1\n E r2\nCOLUMNS\n x obj 2 r1 1\n y obj 3 r1 1\n y r2 1\n a r1 -2\n b r2 2\nRHS\n rhs r1 1 r2 0\nBOUNDS\n BV bd x\n BV bd y\n LI bd a -5\n UI bd a 5\n LI bd b -5\n UI bd b 5\nENDATA\n";
    fn run(text: &str) -> Option<Answer> {
        let m = crate::mps::parse(std::io::Cursor::new(text)).unwrap();
        solve(&m, Instant::now() + std::time::Duration::from_secs(1))
    }
    #[test]
    fn unique_point_has_private_integer_reconstruction() {
        let a = run(MODEL).unwrap();
        assert_eq!(a.point, vec![1., 0., 0., 0.]);
        assert_eq!(a.proof.rank, 2);
        let m = crate::mps::parse(std::io::Cursor::new(MODEL)).unwrap();
        assert!(m.check(&a.point, true, 0.).is_ok());
        assert_eq!(m.objective(&a.point), 2.);
    }
    #[test]
    fn unsupported_or_rounded_original_data_never_certify() {
        assert!(run(&MODEL.replace("r1 -2", "r1 -4")).is_none());
        assert!(run(&MODEL.replace("rhs r1 1", "rhs r1 1.00000000000000000001")).is_none());
        assert!(run(&MODEL.replace("BV bd x", "LO bd x 0\n UP bd x 1")).is_none());
        assert!(run(&MODEL.replace("UI bd a 5", "UI bd a -1")).is_none());
    }
    #[test]
    fn rank_deficiency_and_contradiction_fall_back_without_declaration() {
        let text = "NAME deficient\nROWS\n N c\n E r\nCOLUMNS\n x c 1 r 1\n y r 1\n z r -2\nRHS\n b r 1\nBOUNDS\n BV bd x\n BV bd y\n LI bd z -2\n UI bd z 2\nENDATA\n";
        assert!(run(text).is_none());
        assert!(
            run(&MODEL
                .replace("y r2 1", "x r2 1\n y r2 2")
                .replace("rhs r1 1 r2 0", "rhs r1 1 r2 0")
                .replace("y obj 3 r1 1", "y obj 3 r1 2"))
            .is_none()
        );
    }
    #[test]
    fn full_rank_claims_match_exhaustive_original_integer_domains() {
        let mut seed = 2026111611u64;
        for case in 0..300 {
            let mut next = || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                seed
            };
            let n = (next() % 6 + 1) as usize;
            let m = (next() % 8 + 1) as usize;
            let planted = next() as usize % (1 << n);
            let a: Vec<Vec<i64>> = (0..m)
                .map(|_| (0..n).map(|_| (next() % 7) as i64 - 3).collect())
                .collect();
            let mut rhs: Vec<i64> = a
                .iter()
                .map(|row| {
                    row.iter()
                        .enumerate()
                        .map(|(j, a)| a * ((planted >> j) & 1) as i64)
                        .sum::<i64>()
                })
                .collect();
            if case % 4 == 0 {
                rhs[0] += 1;
            }
            let mut text = String::from("NAME enumeration\nROWS\n N obj\n");
            for i in 0..m {
                text += &format!(" E r{i}\n");
            }
            text += "COLUMNS\n";
            for j in 0..n {
                text += &format!(" x{j} obj {}\n", (next() % 7) as i64 - 3);
                for (i, row) in a.iter().enumerate() {
                    text += &format!(" x{j} r{i} {}\n", row[j]);
                }
            }
            for i in 0..m {
                text += &format!(" z{i} r{i} -2\n");
            }
            text += "RHS\n";
            for (i, b) in rhs.iter().enumerate() {
                text += &format!(" b r{i} {b}\n");
            }
            text += "BOUNDS\n";
            for j in 0..n {
                text += &format!(" BV bd x{j}\n");
            }
            for i in 0..m {
                text += &format!(" LI bd z{i} -20\n UI bd z{i} 20\n");
            }
            text += "ENDATA\n";
            let model = crate::mps::parse(std::io::Cursor::new(text)).unwrap();
            let mut solutions = Vec::new();
            for mask in 0..(1usize << n) {
                let mut p: Vec<_> = (0..n).map(|j| ((mask >> j) & 1) as f64).collect();
                let mut valid = true;
                for (i, row) in a.iter().enumerate() {
                    let r = rhs[i]
                        - row
                            .iter()
                            .enumerate()
                            .map(|(j, a)| a * ((mask >> j) & 1) as i64)
                            .sum::<i64>();
                    if r % 2 != 0 || r / 2 < -20 || r / 2 > 20 {
                        valid = false;
                        break;
                    }
                    p.push((-r / 2) as f64);
                }
                if valid {
                    solutions.push(p);
                }
            }
            if let Some(answer) = solve(&model, Instant::now() + std::time::Duration::from_secs(1))
            {
                assert_eq!(solutions.len(), 1, "case={case}");
                assert_eq!(answer.point, solutions[0]);
            }
        }
    }
    #[test]
    fn deadline_and_empty_models_are_noops() {
        let m = crate::mps::parse(std::io::Cursor::new(MODEL)).unwrap();
        assert!(solve(&m, Instant::now()).is_none());
    }
}
