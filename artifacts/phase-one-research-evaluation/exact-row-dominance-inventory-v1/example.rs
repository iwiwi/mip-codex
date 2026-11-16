use mip_codex::{mps, row_dominance};
fn main() {
    let m = mps::read(std::env::args().nth(1).unwrap()).unwrap();
    let (p, s) = row_dominance::detect(
        &m,
        std::time::Instant::now() + std::time::Duration::from_secs(10),
    );
    println!(
        "{}",
        serde_json::json!({"rows":m.rows.len(),"proofs":p,"stats":s,"source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT})
    );
}
