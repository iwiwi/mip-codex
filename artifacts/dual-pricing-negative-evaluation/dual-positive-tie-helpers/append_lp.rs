//! 行追加warmの比較/検証用。既存ソルバー呼出しは含まない。
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
    costs: Vec<f64>,
    base_rows: usize,
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
    if std::env::args().nth(1).is_some_and(|v|v=="build-info"){println!("{}",json!({"source_directory":env!("CARGO_MANIFEST_DIR"),"source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT,"library_source_directory":mip_codex::BUILD_SOURCE_DIRECTORY,"library_source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT}));return;}
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let cases: Vec<Input> = serde_json::from_str(&input).unwrap();
    for (case, i) in cases.into_iter().enumerate() {
        let mut p = Problem {
            columns: i
                .columns
                .iter()
                .map(|c| {
                    c.iter()
                        .copied()
                        .filter(|&(r, _)| r < i.base_rows)
                        .collect()
                })
                .collect(),
            slack_upper: vec![f64::INFINITY; i.base_rows],
            rhs: i.rhs[..i.base_rows].to_vec(),
            upper: i.upper,
            equality: i.equality[..i.base_rows].to_vec(),
            costs: i.costs,
        };
        let mut state = None;
        for step in 0..=i.rhs.len() - i.base_rows {
            if step > 0 {
                let r = i.base_rows + step - 1;
                for (dest, source) in p.columns.iter_mut().zip(&i.columns) {
                    dest.extend(source.iter().copied().filter(|&(row, _)| row == r));
                }
                p.rhs.push(i.rhs[r]);
                p.equality.push(i.equality[r]);
                p.slack_upper.push(f64::INFINITY);
            }
            let begin = Instant::now();
            let warm = revised::minimize_warm(&p, 100000, None, state.as_deref());
            let warm_seconds = begin.elapsed().as_secs_f64();
            let begin = Instant::now();
            let cold = revised::minimize(&p, 100000, None);
            let cold_seconds = begin.elapsed().as_secs_f64();
            println!(
                "{}",
                json!({"case":case,"step":step,"warm":answer(&warm),"cold":answer(&cold),"warm_seconds":warm_seconds,"cold_seconds":cold_seconds,"warm_pivots":warm.stats.pivots,"warm_tie_calls":warm.stats.dual_tie_calls,"warm_tie_comparisons":warm.stats.dual_tie_comparisons,"cold_pivots":cold.stats.pivots,"warm_row_extensions":warm.stats.warm_row_extensions,"warm_successes":warm.stats.warm_successes,"warm_fallbacks":warm.stats.warm_fallbacks})
            );
            state = warm.warm;
        }
    }
}
