//! 根LPなしで既存の主解修復を調べる診断。元の制約を一切変更しない。
use mip_codex::{heuristic, model::Model, solve::Result};
use std::{
    fs::File,
    time::{Duration, Instant},
};
fn main() {
    let a: Vec<_> = std::env::args().collect();
    let m: Model = if a[1].ends_with(".json") {
        serde_json::from_reader(File::open(&a[1]).unwrap()).unwrap()
    } else {
        mip_codex::mps::read(&a[1]).unwrap()
    };
    let seconds: f64 = a[2].parse().unwrap();
    let start = Instant::now();
    let end = start + Duration::from_secs_f64(seconds);
    let lower: Vec<_> = m.variables.iter().map(|v| v.lower).collect();
    let upper: Vec<_> = m.variables.iter().map(|v| v.upper).collect();
    let point: Vec<_> = m
        .variables
        .iter()
        .map(|v| {
            let lo = v.lower.map(|x| if v.integer { x.ceil() } else { x });
            let up = v.upper.map(|x| if v.integer { x.floor() } else { x });
            0f64.max(lo.unwrap_or(f64::NEG_INFINITY))
                .min(up.unwrap_or(f64::INFINITY))
        })
        .collect();
    let mut repair = heuristic::Repair::new(&m);
    let mut best = None;
    let mut trials = 0;
    for seed in 0..8 {
        if Instant::now() >= end {
            break;
        }
        trials += 1;
        let deadline = end.min(Instant::now() + Duration::from_secs_f64((seconds / 4.).max(0.01)));
        if let Some(x) = repair.search(&point, &lower, &upper, deadline, 50_000_000, seed) {
            assert!(repair.candidate_check(&x));
            best = Some(x);
            break;
        }
    }
    let r = Result {
        status: if best.is_some() {
            "feasible_repair_diagnostic"
        } else {
            "unresolved_repair_diagnostic"
        }
        .into(),
        objective: best.as_ref().map(|x| m.objective(x)),
        values: best.map(|x| {
            m.variables
                .iter()
                .zip(x)
                .map(|(v, x)| (v.name.clone(), x))
                .collect()
        }),
        seconds: start.elapsed().as_secs_f64(),
        ..Result::default()
    };
    serde_json::to_writer(File::create(&a[3]).unwrap(), &r).unwrap();
    println!(
        "{}",
        serde_json::json!({"trials":trials,"status":r.status,"objective":r.objective,"seconds":r.seconds,"repair_calls":repair.stats.calls,"repair_work":repair.stats.work,"repair_candidates":repair.stats.candidates})
    );
}
