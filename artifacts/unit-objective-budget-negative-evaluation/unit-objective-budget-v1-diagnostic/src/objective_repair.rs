//! 元の整数解を目的上限制約付きRepairで改善する。大域下界や不能は主張しない。
use crate::{
    heuristic::{BoxStats, Repair},
    model::{Model, Row},
};
use std::time::Instant;
pub fn improve(
    model: &Model,
    original: &Repair<'_>,
    point: &[f64],
    lower: &[Option<f64>],
    upper: &[Option<f64>],
    deadline: Instant,
) -> Option<(Vec<f64>, BoxStats)> {
    improve_budgeted(model,original,point,lower,upper,deadline,None)
}
pub fn improve_budgeted(model:&Model,original:&Repair<'_>,point:&[f64],lower:&[Option<f64>],upper:&[Option<f64>],deadline:Instant,extension:Option<Instant>)->Option<(Vec<f64>,BoxStats)>{
    let began = Instant::now();
    if began >= deadline
        || point.len() != model.variables.len()
        || lower.len() != point.len()
        || upper.len() != point.len()
        || model.maximize
        || model.offset != 0.
        || model.offset_enclosure != Some([0., 0.])
        || model.variables.len() > 100_000
        || model
            .rows
            .iter()
            .map(|r| r.coefficients.len())
            .sum::<usize>()
            > 10_000_000
        || !model.variables.iter().all(|v| {
            v.integer
                && v.lower == Some(0.)
                && v.upper == Some(1.)
                && v.objective == 1.
                && v.objective_enclosure == Some([1., 1.])
        })
        || !original.candidate_check(point)
    {
        return None;
    }
    let mut x = point.to_vec();
    let mut stats = BoxStats::default();
    let mut work = model.clone();
    work.rows.push(Row {
        name: "internal_objective_repair_cut".into(),
        lower: None,
        upper: None,
        coefficients: (0..point.len()).map(|j| (j, 1.)).collect(),
        exact_integral: true,
        exact_coefficients: true,
        exact_coefficient_flags: Vec::new(),
        lower_enclosure: None,
        upper_enclosure: None,
    });
    let extension=extension.filter(|&end|end>deadline);
    let mut active_deadline=deadline;let mut extended=false;
    for seed in 0..if extension.is_some(){1024}else{16} {
        if !extended&&(Instant::now()>=deadline||seed>=16){
            if let Some(end)=extension.filter(|&end|stats.improvements>0&&Instant::now()<end){active_deadline=end;extended=true;}else{break;}
        }
        if Instant::now()>=active_deadline||(extended&&stats.work>=1_000_000_000){break;}
        let target = model.objective(&x) - 1.;
        let row = work.rows.last_mut().unwrap();
        row.upper = Some(target);
        row.upper_enclosure = Some([target, target]);
        let mut repair = Repair::new(&work);
        let start: Vec<_> = x
            .iter()
            .map(|&v| if v > 0.5 { 0.95 } else { 0.001 })
            .collect();
        let candidate = repair.search(&start, lower, upper, active_deadline, if extended {100_000_000usize.min(1_000_000_000usize.saturating_sub(stats.work))}else{100_000_000}, seed);
        stats.calls += repair.stats.calls;
        stats.moves += repair.stats.moves;
        stats.work += repair.stats.work;
        if let Some(v) = candidate {
            if original.candidate_check(&v) {
                stats.candidates += 1;
                let value = model.objective(&v);
                if value.is_finite() && value < model.objective(&x) {
                    x = v;
                    stats.improvements += 1;
                    stats
                        .first_seconds
                        .get_or_insert_with(|| began.elapsed().as_secs_f64());
                }
            }
        }
    }
    stats.seconds = began.elapsed().as_secs_f64();
    Some((x, stats))
}
