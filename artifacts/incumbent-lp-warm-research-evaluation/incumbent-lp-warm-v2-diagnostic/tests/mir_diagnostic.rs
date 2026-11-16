//! Explicit development-only diagnostic; normal test runs never read benchmark data.
use mip_codex::{cuts::Pool, mps};
use std::{
    path::Path,
    time::{Duration, Instant},
};
#[test]
#[ignore = "development diagnostic via MIP_MIR_DIAGNOSTIC_DIR"]
fn saved_development_points() {
    let output = std::env::var("MIP_MIR_DIAGNOSTIC_DIR").unwrap();
    std::fs::create_dir_all(&output).unwrap();
    let input = std::env::var("MIP_MIR_DIAGNOSTIC_INPUT_DIR")
        .unwrap_or_else(|_| "artifacts/mir-diagnostic".into());
    for entry in std::fs::read_dir(input).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let metadata: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let Some(model) = metadata["model"].as_str() else {
            continue;
        };
        assert_eq!(metadata["split"], "development");
        let m = mps::read(model).unwrap();
        let point: serde_json::Value =
            serde_json::from_slice(&std::fs::read(metadata["point"].as_str().unwrap()).unwrap())
                .unwrap();
        let x: Vec<f64> = serde_json::from_value(point["point"].clone()).unwrap();
        m.check(&x, false, 1e-6).unwrap();
        let mut pool = Pool::default();
        let cuts = pool.separate_mir(&m, &x, 8, Instant::now() + Duration::from_secs(1));
        assert!(cuts.iter().all(|c| mip_codex::cuts::verify(&m, c)));
        let record = serde_json::json!({"model":model,"name":metadata["name"],"split":"development","cut_proofs":cuts,"cut_stats":pool.stats});
        std::fs::write(
            Path::new(&output).join(path.file_name().unwrap()),
            serde_json::to_vec_pretty(&record).unwrap(),
        )
        .unwrap();
    }
}
