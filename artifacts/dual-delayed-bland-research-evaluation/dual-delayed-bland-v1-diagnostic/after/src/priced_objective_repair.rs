//! 元の整数解を目的上限制約付きRepairで改善する。大域下界や不能は主張しない。
use crate::{
    heuristic::{BoxStats, Repair},
    model::{Model, Row},
};
use std::time::Instant;
pub fn improve_priced(
    model: &Model,
    original: &Repair<'_>,
    point: &[f64],
    lower: &[Option<f64>],
    upper: &[Option<f64>],
    deadline: Instant,
) -> Option<(Vec<f64>, BoxStats)> {
    improve_priced_continued(model,original,point,lower,upper,deadline,false)
}
pub fn improve_priced_continued(model:&Model,original:&Repair<'_>,point:&[f64],lower:&[Option<f64>],upper:&[Option<f64>],deadline:Instant,continue_success:bool)->Option<(Vec<f64>,BoxStats)>{
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
                && v.objective.is_finite() && v.objective >= 0.
        })
        || !original.candidate_check(point) || !original.objective_reliable(point,model.objective(point))
    {
        return None;
    }
    let mut costs:Vec<_>=model.variables.iter().map(|v|v.objective).filter(|&c|c>0.).collect();
    if costs.is_empty(){return None;}costs.sort_by(f64::total_cmp);let scale=costs[costs.len()/2];let step=costs[0];
    if model.variables.iter().any(|v|!(v.objective/scale).is_finite()){return None;}
    if model.variables.iter().all(|v|v.objective==model.variables[0].objective){return None;}
    let mut x = point.to_vec();
    let mut stats = BoxStats::default();
    let mut work = model.clone();
    work.rows.push(Row {
        name: "internal_objective_repair_cut".into(),
        lower: None,
        upper: None,
        coefficients: model.variables.iter().enumerate().filter(|(_,v)|v.objective!=0.).map(|(j,v)|(j,v.objective/scale)).collect(),
        exact_integral: false,
        exact_coefficients: false,
        exact_coefficient_flags: Vec::new(),
        lower_enclosure: None,
        upper_enclosure: None,
    });
    for seed in 0..if continue_success{64}else{16} {
        if seed>=16 && stats.improvements==0 {break;}
        if Instant::now() >= deadline || stats.work>=100_000_000 {
            break;
        }
        let target = (model.objective(&x) - step)/scale;
        if !target.is_finite(){break;}
        let row = work.rows.last_mut().unwrap();
        row.upper = Some(target);
        row.upper_enclosure = Some([target, target]);
        let mut repair = Repair::new(&work);
        let start: Vec<_> = x
            .iter()
            .map(|&v| if v > 0.5 { 0.95 } else { 0.001 })
            .collect();
        let candidate = repair.search(&start, lower, upper, deadline, 100_000_000usize.saturating_sub(stats.work), seed);
        stats.calls += repair.stats.calls;
        stats.moves += repair.stats.moves;
        stats.work += repair.stats.work;
        if let Some(v) = candidate {
            if original.candidate_check(&v) && original.objective_reliable(&v,model.objective(&v)) {
                stats.candidates += 1;
                let value = model.objective(&v);
                if value.is_finite() && value < model.objective(&x)-4e-8*(1.+value.abs().max(model.objective(&x).abs())) {
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
