//! 費用変更warmの比較/検証用。既存ソルバー呼出しは含まない。
use mip_codex::{
    lp::Outcome,
    revised::{self, Problem},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    io::{self, Read},
    time::Instant,
};
#[derive(Deserialize)]
struct Input {
    columns: Vec<Vec<(usize, f64)>>,
    rhs: Vec<f64>,
    upper: Vec<f64>,
    equality: Vec<bool>,
    costs: Vec<Vec<f64>>,
}
fn answer(a: &revised::Answer) -> Value {
    match &a.outcome {
        Outcome::Optimal { objective, x } => {
            json!({"status":"optimal","objective":objective,"x":x})
        }
        other => json!({"status":format!("{other:?}")}),
    }
}
fn main() {
    if std::env::args().nth(1).as_deref()==Some("build-info"){
        println!("{}",serde_json::json!({"source_directory":env!("CARGO_MANIFEST_DIR"),"source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT,"library_source_directory":mip_codex::BUILD_SOURCE_DIRECTORY,"library_source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT}));return;
    }
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let cases: Vec<Input> = serde_json::from_str(&input).unwrap();
    for (case, i) in cases.into_iter().enumerate() {
        let mut p = Problem {
            columns: i.columns,
            slack_upper: vec![f64::INFINITY; i.rhs.len()],
            rhs: i.rhs,
            upper: i.upper,
            equality: i.equality,
            costs: vec![],
        };
        let mut state = None;
        for (step, cost) in i.costs.into_iter().enumerate() {
            p.costs = cost;
            let begin = Instant::now();
            let warm = revised::minimize_warm(&p, 100000, None, state.as_deref());
            let warm_seconds = begin.elapsed().as_secs_f64();
            let begin = Instant::now();
            let cold = revised::minimize(&p, 100000, None);
            let cold_seconds = begin.elapsed().as_secs_f64();
            println!(
                "{}",
                json!({"case":case,"step":step,"warm":answer(&warm),"cold":answer(&cold),"warm_seconds":warm_seconds,"cold_seconds":cold_seconds,"warm_dual_relative_calls":warm.stats.dual_relative_calls,"cold_dual_relative_calls":cold.stats.dual_relative_calls,"warm_pivots":warm.stats.pivots,"cold_pivots":cold.stats.pivots,"warm_cost_successes":warm.stats.warm_cost_successes,"warm_fallbacks":warm.stats.warm_fallbacks})
            );
            state = warm.warm;
        }
    }
}
