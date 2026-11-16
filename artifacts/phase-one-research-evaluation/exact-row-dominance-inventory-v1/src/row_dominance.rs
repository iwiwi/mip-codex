//! Exact integer row-range dominance; original row indices remain unchanged.
use crate::model::{Model, Row};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Instant};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub removed: usize,
    pub keeper: usize,
    pub removed_name: String,
    pub keeper_name: String,
}
#[derive(Debug, Default, Serialize)]
pub struct Stats {
    pub examined: usize,
    pub work: usize,
    pub removed: usize,
    pub seconds: f64,
    pub limited: bool,
}
fn small(v: f64) -> bool {
    v.is_finite() && v.fract() == 0. && v.abs() <= (1u64 << 52) as f64
}
fn supported(r: &Row) -> bool {
    r.exact_integral
        && r.exact_coefficients
        && !r.coefficients.is_empty()
        && r.coefficients.len() <= 4096
        && r.coefficients.iter().all(|&(_, a)| small(a))
        && [(r.lower, r.lower_enclosure), (r.upper, r.upper_enclosure)]
            .iter()
            .all(|&(value, enclosure)| value.is_none_or(|v| small(v) && enclosure == Some([v, v])))
}
fn contains(outer: &Row, inner: &Row) -> bool {
    outer
        .lower
        .is_none_or(|l| inner.lower.is_some_and(|x| x >= l))
        && outer
            .upper
            .is_none_or(|u| inner.upper.is_some_and(|x| x <= u))
}
pub fn detect(model: &Model, deadline: Instant) -> (Vec<Proof>, Stats) {
    let start = Instant::now();
    let mut stats = Stats::default();
    let mut groups = HashMap::<Vec<(usize, i64)>, Vec<usize>>::new();
    let mut proofs = Vec::new();
    for (i, row) in model.rows.iter().enumerate() {
        if Instant::now() >= deadline || stats.work >= 2_000_000 {
            stats.limited = true;
            break;
        }
        if stats.work.saturating_add(row.coefficients.len() + 1) > 2_000_000 {
            stats.limited = true;
            break;
        }
        stats.work = stats.work.saturating_add(row.coefficients.len() + 1);
        stats.examined += 1;
        if !supported(row) {
            continue;
        }
        let key: Vec<_> = row
            .coefficients
            .iter()
            .map(|&(j, a)| (j, a as i64))
            .collect();
        let previous = groups.entry(key).or_default();
        if let Some(&keeper) = previous.iter().find(|&&k| contains(row, &model.rows[k])) {
            proofs.push(Proof {
                removed: i,
                keeper,
                removed_name: row.name.clone(),
                keeper_name: model.rows[keeper].name.clone(),
            });
        } else if previous.len() < 32 {
            previous.push(i);
        }
    }
    stats.removed = proofs.len();
    stats.seconds = start.elapsed().as_secs_f64();
    (proofs, stats)
}
pub fn apply(model: &Model, proofs: &[Proof]) -> Option<Model> {
    let mut removed = vec![false; model.rows.len()];
    for proof in proofs {
        let r = model.rows.get(proof.removed)?;
        let k = model.rows.get(proof.keeper)?;
        if proof.keeper >= proof.removed
            || removed[proof.removed]
            || !supported(r)
            || !supported(k)
            || r.name != proof.removed_name
            || k.name != proof.keeper_name
            || r.coefficients != k.coefficients
            || !contains(r, k)
        {
            return None;
        }
        removed[proof.removed] = true;
    }
    if proofs.iter().any(|p| removed[p.keeper]) {
        return None;
    }
    let mut work = model.clone();
    for p in proofs {
        let r = &mut work.rows[p.removed];
        r.coefficients.clear();
        r.lower = None;
        r.upper = None;
        r.lower_enclosure = None;
        r.upper_enclosure = None;
    }
    Some(work)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn model() -> Model {
        crate::mps::parse(std::io::Cursor::new("NAME duplicate\nROWS\n N obj\n L strong\n L weak\n G opposite\nCOLUMNS\n x obj 1 strong 2\n x weak 2 opposite 2\nRHS\n b strong 3 weak 5\n b opposite 1\nBOUNDS\n UP bd x 5\nENDATA\n")).unwrap()
    }
    #[test]
    fn only_retained_stronger_same_coefficients_can_justify_removal() {
        let m = model();
        let (p, s) = detect(&m, Instant::now() + std::time::Duration::from_secs(1));
        assert_eq!(s.removed, 1);
        assert_eq!((p[0].removed, p[0].keeper), (1, 0));
        let w = apply(&m, &p).unwrap();
        assert_eq!(w.rows.len(), m.rows.len());
        assert!(w.rows[1].coefficients.is_empty());
        for x in 0..=20 {
            let x = [x as f64 / 4.];
            assert_eq!(
                m.check(&x, false, 0.).is_ok(),
                w.check(&x, false, 0.).is_ok()
            );
        }
    }
    #[test]
    fn expired_or_rounded_sources_do_not_prove_dominance() {
        let m = model();
        assert!(detect(&m, Instant::now()).0.is_empty());
        let mut m = model();
        m.rows[0].exact_coefficients = false;
        assert!(
            detect(&m, Instant::now() + std::time::Duration::from_secs(1))
                .0
                .is_empty()
        );
        let mut m = model();
        m.rows[0].upper_enclosure = Some([3., 3f64.next_up()]);
        assert!(
            detect(&m, Instant::now() + std::time::Duration::from_secs(1))
                .0
                .is_empty()
        );
    }
    #[test]
    fn reversed_or_modified_proofs_fail_closed() {
        let m = model();
        let (mut p, _) = detect(&m, Instant::now() + std::time::Duration::from_secs(1));
        p[0].keeper = 2;
        assert!(apply(&m, &p).is_none());
    }
}
