pub mod model{pub use mip_codex::model::*;}
#[path="../src/exact.rs"]mod exact;
#[path="support/fractional_profile/mod.rs"]mod profile;
fn main(){let args:Vec<_>=std::env::args().collect();assert_eq!(args.len(),4);let model=mip_codex::mps::read(&args[1]).unwrap();let r=profile::bound_profile(&model,args[2].parse().unwrap());std::fs::write(&args[3],serde_json::to_vec_pretty(&r).unwrap()).unwrap();println!("{}",serde_json::json!({"dual_bound":r["dual_bound"],"component_seconds":r["component_seconds"],"base_production_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT,"fractional_stats":r["binary_graph"]["fractional_cover"]["stats"]}));}
