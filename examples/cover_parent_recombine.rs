pub mod model {pub use mip_codex::model::*;}
#[path="support/cover_recombine_profile/mod.rs"] mod profile;
fn main() {
    let args:Vec<_>=std::env::args().collect();assert!(args.len()>=6);
    let model=mip_codex::mps::read(&args[1]).unwrap();
    let seconds:f64=args[2].parse().unwrap();
    let mut parents=vec![];
    for path in &args[4..] {
        let value:serde_json::Value=serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(value["values"].as_object().unwrap().len(),model.variables.len());
        parents.push(model.variables.iter().map(|v|value["values"][&v.name].as_f64().unwrap()).collect());
    }
    let mut result=profile::diagnostic_recombine(&model,seconds,&parents);
    result["base_production_fingerprint"]=mip_codex::BUILD_SOURCE_FINGERPRINT.into();
    result["base_production_source_directory"]=mip_codex::BUILD_SOURCE_DIRECTORY.into();
    let out=std::path::Path::new(&args[3]);std::fs::create_dir_all(out).unwrap();
    std::fs::write(out.join("result.json"),serde_json::to_vec_pretty(&result["result"]).unwrap()).unwrap();
    result.as_object_mut().unwrap().remove("result");
    std::fs::write(out.join("profile.json"),serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    println!("{}",result);
}
