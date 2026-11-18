//! 自作MILPソルバーの数値コア。外部の最適化ライブラリは使用しない。
#[path="support/profile_core/lp.rs"]
pub mod lp;
#[path="support/profile_core/model.rs"]
pub mod model;
#[path="support/profile_core/mps.rs"]
pub mod mps;
#[path="support/profile_core/propagate.rs"]
pub mod propagate;
#[path="support/profile_core/revised.rs"]
pub mod revised;
#[path="support/profile_core/solve.rs"]
pub mod solve;

#[path="support/profile_core/heuristic.rs"]
pub mod heuristic;

#[path="support/profile_core/branching.rs"]
pub mod branching;

#[path="support/profile_core/scaling.rs"]
pub(crate) mod scaling;

#[path="support/profile_core/exact.rs"]
pub(crate) mod exact;

#[path="support/profile_core/search.rs"]
pub mod search;

#[path="support/profile_core/cuts/mod.rs"]
pub mod cuts;
#[path="support/profile_core/rins.rs"]
pub mod rins;

#[path="support/profile_core/objective_bound.rs"]
pub mod objective_bound;

/// 計測時に共有targetから別worktreeのcrateを拾う事故を検出する。
pub const BUILD_SOURCE_DIRECTORY: &str = env!("CARGO_MANIFEST_DIR");

/// Rustソース/Cargo設定のFNV64。計測記録のSHA256とは別のcache取り違え検出用。
pub const BUILD_SOURCE_FINGERPRINT: &str = env!("MIP_SOURCE_FINGERPRINT");

#[path="support/profile_core/binary_graph/mod.rs"]
pub mod binary_graph;

#[path="support/profile_core/certificate.rs"]
pub(crate) mod certificate;

#[path="support/profile_core/parity.rs"]
pub mod parity;

#[path="support/profile_core/row_ray.rs"]
pub mod row_ray;

#[path="support/profile_core/row_packing.rs"]
pub mod row_packing;

#[path="support/profile_core/objective_repair.rs"]
pub mod objective_repair;

#[path="support/profile_core/priced_objective_repair.rs"]
pub mod priced_objective_repair;

#[path="support/profile_core/row_subgradient.rs"]
pub mod row_subgradient;

#[path="support/profile_core/dual_guided_primal.rs"]
pub mod dual_guided_primal;

#[path="support/profile_core/component_objective_repair.rs"]
pub mod component_objective_repair;

#[path="support/profile_core/cover_greedy.rs"]
pub mod cover_greedy;

#[path="support/profile_core/gomory.rs"]
pub mod gomory;

#[path="support/profile_core/dual_form.rs"]
pub(crate) mod dual_form;
#[path="support/profile_core/zero_dominance.rs"]
pub mod zero_dominance;

#[path="support/profile_core/master_bound.rs"]
pub mod master_bound;

#[path="support/profile_core/integer_rows.rs"]
pub mod integer_rows;

#[path="support/profile_core/primal_witness.rs"]
mod primal_witness;

#[path="support/profile_core/adaptive_resource_repair.rs"]
pub mod adaptive_resource_repair;

fn main(){
 let a:Vec<_>=std::env::args().collect();assert_eq!(a.len(),5);let model=mps::read(&a[1]).unwrap();let seed:serde_json::Value=serde_json::from_slice(&std::fs::read(&a[2]).unwrap()).unwrap();let point:Vec<_>=model.variables.iter().map(|v|seed["values"][&v.name].as_f64().unwrap()).collect();assert!(heuristic::Repair::new(&model).candidate_check(&point));let seconds:f64=a[3].parse().unwrap();let lower:Vec<_>=model.variables.iter().map(|v|v.lower.map(|x|if v.integer{x.ceil()}else{x})).collect();let upper:Vec<_>=model.variables.iter().map(|v|v.upper.map(|x|if v.integer{x.floor()}else{x})).collect();let mut stats=adaptive_resource_repair::Stats::default();let start=std::time::Instant::now();let result=adaptive_resource_repair::search_incremental(&model,&point,&lower,&upper,start+std::time::Duration::from_secs_f64(seconds),512*1024*1024,&mut stats);let component_wall_seconds=start.elapsed().as_secs_f64();let point=result.unwrap_or(point);let objective=model.objective(&point);let values:std::collections::BTreeMap<_,_>=model.variables.iter().zip(point).map(|(v,x)|(v.name.clone(),x)).collect();let value=serde_json::json!({"status":"primal_candidate","objective":objective,"dual_bound":null,"values":values,"diagnostic_stats":stats,"search_wall_seconds":start.elapsed().as_secs_f64(),"component_wall_seconds":component_wall_seconds,"base_production_source_fingerprint":BUILD_SOURCE_FINGERPRINT,"purpose":"standalone continuation; parsing and seed validation outside search time; no bound/optimality claim"});std::fs::write(&a[4],serde_json::to_vec_pretty(&value).unwrap()).unwrap();println!("{}",serde_json::json!({"objective":objective,"stats":value["diagnostic_stats"]}));
}
