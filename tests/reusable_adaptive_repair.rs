use mip_codex::{adaptive_resource_repair::{self,Stats},model::Model};
use std::time::{Duration,Instant};
#[test]
fn reusable_index_continues_beyond_sixteen_targets_in_both_senses() {
 for maximize in [false,true] {
  let cost=if maximize{1.}else{-1.};
  let model:Model=serde_json::from_value(serde_json::json!({"name":"targets","maximize":maximize,"offset":7.,"offset_enclosure":[7.,7.],"variables":[{"name":"x","lower":0.,"upper":40.,"integer":true,"objective":cost,"objective_enclosure":[cost,cost]}],"rows":[]})).unwrap();
  let before=serde_json::to_string(&model).unwrap();let seed=[0.];let mut old=Stats::default();let mut new=Stats::default();
  let a=adaptive_resource_repair::search_incremental(&model,&seed,&[Some(0.)],&[Some(40.)],Instant::now()+Duration::from_secs(1),1<<20,&mut old).unwrap();
  let b=adaptive_resource_repair::search_incremental_reusing(&model,&seed,&[Some(0.)],&[Some(40.)],Instant::now()+Duration::from_secs(1),1<<20,&mut new).unwrap();
  assert_eq!(a,[16.]);assert_eq!(b,[40.]);assert_eq!(new.index_builds,1);assert!(new.index_reuses>=39);assert_eq!(old.index_reuses,0);assert!(model.check(&b,true,0.).is_ok());assert_eq!(seed,[0.]);assert_eq!(serde_json::to_string(&model).unwrap(),before);
 }
}
#[test]
fn cached_memory_overhead_falls_back_to_original_eligible_path() {
 let model:Model=serde_json::from_value(serde_json::json!({"name":"memory","maximize":false,"offset":0.,"offset_enclosure":[0.,0.],"variables":[{"name":"x","lower":0.,"upper":40.,"integer":true,"objective":-1.,"objective_enclosure":[-1.,-1.]}],"rows":[]})).unwrap();let mut stats=Stats::default();
 let x=adaptive_resource_repair::search_incremental_reusing(&model,&[0.],&[Some(0.)],&[Some(40.)],Instant::now()+Duration::from_secs(1),256,&mut stats).unwrap();
 assert_eq!(x,[16.]);assert_eq!(stats.index_reuses,0);assert_eq!(stats.index_builds,16);
}
