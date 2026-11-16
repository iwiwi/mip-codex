//! 与えたLP/端点からの初期基底を独立oracleと照合するための診断。
use mip_codex::{
    lp::Outcome,
    revised::{self, Problem},
};
use serde::Deserialize;
#[derive(Deserialize)]
struct Input {
    columns: Vec<Vec<(usize, f64)>>,
    rhs: Vec<f64>,
    costs: Vec<f64>,
    upper: Vec<Option<f64>>,
    equality: Vec<bool>,
    slack_upper: Vec<Option<f64>>,
    point: Vec<f64>,
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let input: Input = serde_json::from_reader(std::fs::File::open(&args[1]).unwrap()).unwrap();
    let p = Problem {
        columns: input.columns,
        rhs: input.rhs,
        costs: input.costs,
        upper: input
            .upper
            .into_iter()
            .map(|x| x.unwrap_or(f64::INFINITY))
            .collect(),
        equality: input.equality,
        slack_upper: input
            .slack_upper
            .into_iter()
            .map(|x| x.unwrap_or(f64::INFINITY))
            .collect(),
    };
    let started = std::time::Instant::now();
    let deadline = Some(started + std::time::Duration::from_secs(5));
    let a = if args[2] == "seeded" {
        revised::minimize_from_point(&p, 10000, deadline, &input.point)
    } else {
        revised::minimize_warm(&p, 10000, deadline, None)
    };
    let (status, objective, x) = match &a.outcome {
        Outcome::Optimal { objective, x } => ("optimal", Some(*objective), Some(x)),
        Outcome::Infeasible => ("infeasible", None, None),
        Outcome::Unbounded => ("unbounded", None, None),
        Outcome::TimeLimit => ("time_limit", None, None),
        Outcome::IterationLimit => ("iteration_limit", None, None),
        Outcome::NumericalError => ("numerical_error", None, None),
    };
    let value = serde_json::json!({"status":status,"objective":objective,"x":x,"dual":a.dual,"seconds":started.elapsed().as_secs_f64(),"pivots":a.stats.pivots,"flips":a.stats.flips,"warm_attempts":a.stats.warm_attempts,"warm_successes":a.stats.warm_successes,"warm_rejections":a.stats.warm_rejections,"warm_fallbacks":a.stats.warm_fallbacks,"scaling_columns":a.stats.scaled_columns,"scaling_rows":a.stats.scaled_rows});
    serde_json::to_writer(std::fs::File::create(&args[3]).unwrap(), &value).unwrap();
    println!("{value}");
}
