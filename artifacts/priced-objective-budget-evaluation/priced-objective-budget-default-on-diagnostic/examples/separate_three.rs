//! Diagnostic separator on a saved Rust LP point; no external optimizer calls.
use mip_codex::{cuts::Pool, mps};
use std::time::{Duration, Instant};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let model = mps::read(&args[1]).unwrap();
    let point: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&args[2]).unwrap()).unwrap();
    let names: Vec<_> = point["names"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(model.variables.iter().map(|v| v.name.as_str()).eq(names));
    let x: Vec<_> = point["point"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    let mut pool = Pool::default();
    let started = Instant::now();
    let cuts = pool.separate_three_aggregate(
        &model,
        &x,
        32,
        started + Duration::from_millis(args.get(3).map_or(20, |v| v.parse().unwrap())),
    );
    println!(
        "{}",
        serde_json::json!({"library_source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT,"source_directory":mip_codex::BUILD_SOURCE_DIRECTORY,"cut_proofs":cuts,"aggregate_stats":pool.stats,"separation_seconds":started.elapsed().as_secs_f64()})
    );
}
