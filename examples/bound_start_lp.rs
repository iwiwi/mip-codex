//! 境界点初期化の比較専用。参照最適化器への委譲は含まない。
use mip_codex::{
    lp::Outcome,
    revised::{self, Problem},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::io::{self, Read};
#[derive(Deserialize)]
struct Input {
    columns: Vec<Vec<(usize, f64)>>,
    rhs: Vec<f64>,
    upper: Vec<f64>,
    equality: Vec<bool>,
    slack_upper: Option<Vec<f64>>,
    costs: Vec<Vec<f64>>,
    at_upper: Vec<bool>,
    #[serde(default)]
    rows: Vec<Row>,
}
#[derive(Deserialize)]
struct Row {
    coefficients: Vec<(usize, f64)>,
    rhs: f64,
    equality: bool,
    slack_upper: Option<f64>,
}
fn answer(a: &revised::Answer) -> Value {
    let outcome = match &a.outcome {
        Outcome::Optimal { objective, x } => {
            json!({"status":"optimal","objective":objective,"x":x})
        }
        other => json!({"status":format!("{other:?}")}),
    };
    json!({"outcome":outcome,"pivots":a.stats.pivots,"flips":a.stats.flips,"attempts":a.stats.initial_bound_start_attempts,"successes":a.stats.initial_bound_start_successes,"zero_phase_one_skips":a.stats.zero_phase_one_skips})
}
fn main() {
    if std::env::args().nth(1).is_some_and(|v| v == "build-info") {
        println!(
            "{}",
            json!({"source_directory":env!("CARGO_MANIFEST_DIR"),"source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT,"library_source_directory":mip_codex::BUILD_SOURCE_DIRECTORY,"library_source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT})
        );
        return;
    }
    let mut data = String::new();
    io::stdin().read_to_string(&mut data).unwrap();
    let inputs: Vec<Input> = serde_json::from_str(&data).unwrap();
    for (case, i) in inputs.into_iter().enumerate() {
        let mut p = Problem {
            columns: i.columns,
            slack_upper: i
                .slack_upper
                .unwrap_or_else(|| vec![f64::INFINITY; i.rhs.len()]),
            rhs: i.rhs,
            upper: i.upper,
            equality: i.equality,
            costs: vec![],
        };
        let mut state = None;
        for (step, cost) in i.costs.into_iter().enumerate() {
            p.costs = cost;
            let seeded =
                revised::minimize_warm_with_bound_start(&p, 100000, None, None, &i.at_upper);
            let warm = revised::minimize_warm_with_bound_start(
                &p,
                100000,
                None,
                state.as_deref(),
                &i.at_upper,
            );
            let cold = revised::minimize(&p, 100000, None);
            println!(
                "{}",
                json!({"case":case,"kind":"cost","step":step,"seeded":answer(&seeded),"warm":answer(&warm),"cold":answer(&cold)})
            );
            state = warm.warm;
        }
        for (step, row) in i.rows.into_iter().enumerate() {
            let r = p.rhs.len();
            p.rhs.push(row.rhs);
            p.equality.push(row.equality);
            p.slack_upper.push(row.slack_upper.unwrap_or(f64::INFINITY));
            for (j, a) in row.coefficients {
                p.columns[j].push((r, a));
            }
            let seeded =
                revised::minimize_warm_with_bound_start(&p, 100000, None, None, &i.at_upper);
            let warm = revised::minimize_warm_with_bound_start(
                &p,
                100000,
                None,
                state.as_deref(),
                &i.at_upper,
            );
            let cold = revised::minimize(&p, 100000, None);
            println!(
                "{}",
                json!({"case":case,"kind":"row","step":step,"seeded":answer(&seeded),"warm":answer(&warm),"cold":answer(&cold)})
            );
            state = warm.warm;
        }
    }
}
