pub mod model{pub use mip_codex::model::*;}
#[path="support/cover_neutral_profile/mod.rs"]mod profile;
fn main(){let a:Vec<_>=std::env::args().collect();let m=mip_codex::mps::read(&a[1]).unwrap();let p:serde_json::Value=serde_json::from_slice(&std::fs::read(&a[3]).unwrap()).unwrap();let parent:Vec<_>=m.variables.iter().map(|v|p["values"][&v.name].as_f64().unwrap()).collect();let mut r=profile::diagnostic_two_remove(&m,a[2].parse().unwrap(),&parent);r["base_production_fingerprint"]=mip_codex::BUILD_SOURCE_FINGERPRINT.into();println!("{}",r);}
