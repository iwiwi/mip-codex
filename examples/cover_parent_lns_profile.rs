pub mod model{pub use mip_codex::model::*;}
#[path="support/cover_parent_lns_profile/mod.rs"]mod profile;
fn main(){
 let args:Vec<_>=std::env::args().collect();assert!(args.len()>=7);let model=mip_codex::mps::read(&args[1]).unwrap();let seconds:f64=args[2].parse().unwrap();let mut parents=Vec::new();
 for path in &args[4..]{let value:serde_json::Value=serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();assert_eq!(value["values"].as_object().unwrap().len(),model.variables.len());parents.push(model.variables.iter().map(|v|value["values"][&v.name].as_f64().unwrap()).collect());}
 let mut result=profile::diagnostic_parent_lns(&model,seconds,&parents,std::path::Path::new(&args[3]));result["base_production_fingerprint"]=mip_codex::BUILD_SOURCE_FINGERPRINT.into();result["base_production_source_directory"]=mip_codex::BUILD_SOURCE_DIRECTORY.into();std::fs::write(std::path::Path::new(&args[3]).join("profile.json"),serde_json::to_vec_pretty(&result).unwrap()).unwrap();println!("{}",result);
}
