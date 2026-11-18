//! Read-only checker for saved LP diagnostic points; never invokes optimization.
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let model = mip_codex::mps::read(&args[1]).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[2]).unwrap()).unwrap();
    let point: Vec<_> = saved["point_bits"].as_array().unwrap().iter()
        .map(|v| f64::from_bits(u64::from_str_radix(v.as_str().unwrap(),16).unwrap())).collect();
    let relaxation = model.check(&point,false,1e-6);
    let integer = model.check(&point,true,1e-6);
    println!("{}",serde_json::json!({"purpose":"read-only LP point check, no optimization or optimality claim","relaxation_valid":relaxation.is_ok(),"relaxation_error":relaxation.err(),"integer_valid":integer.is_ok(),"integer_error":integer.err()}));
}
