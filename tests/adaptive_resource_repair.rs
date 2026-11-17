use mip_codex::{adaptive_resource_repair::{self,Stats},heuristic::Repair,model::Model};
use std::time::{Instant,Duration};
fn linked(maximize:bool)->Model{
 serde_json::from_value(serde_json::json!({"name":"linked","maximize":maximize,"offset":7.,"offset_enclosure":[7.,7.],"variables":[{"name":"x","lower":0.,"upper":1.,"integer":true,"objective":if maximize{1.}else{-1.},"objective_enclosure":if maximize{[1.,1.]}else{[-1.,-1.]}},{"name":"y","lower":0.,"upper":1.,"integer":true,"objective":0.,"objective_enclosure":[0.,0.]}],"rows":[{"name":"link","lower":0.,"upper":0.,"coefficients":[[0,2.],[1,-2.]],"exact_integral":true,"exact_coefficients":true,"lower_enclosure":[0.,0.],"upper_enclosure":[0.,0.]}]})).unwrap()
}
#[test]fn adaptive_penalties_cross_a_coordinate_local_minimum(){
 let mut m=linked(false);m.offset=0.;m.offset_enclosure=Some([0.,0.]);
 m.rows.push(serde_json::from_value(serde_json::json!({"name":"improve","upper":-1.,"coefficients":[[0,-1.]],"upper_enclosure":[-1.,-1.]})).unwrap());
 let lo=[Some(0.);2];let up=[Some(1.);2];let seed=[0.,0.];let end=Instant::now()+Duration::from_secs(1);
 assert!(Repair::new(&m).search(&seed,&lo,&up,end,100000,0).is_none());
 let mut repair=Repair::new(&m);let x=repair.search_adaptive(&seed,&lo,&up,end,100000,0).unwrap();assert_eq!(x,vec![1.,1.]);assert!(repair.stats.penalty_updates>0);assert!(m.check(&x,true,0.).is_ok());assert_eq!(seed,[0.,0.]);
}
#[test]fn objective_repair_handles_both_senses_offsets_and_preserves_the_input(){
 for maximize in [false,true]{let m=linked(maximize);let before=serde_json::to_string(&m).unwrap();let seed=[0.,0.];let mut stats=Stats::default();let x=adaptive_resource_repair::search(&m,&seed,&[Some(0.);2],&[Some(1.);2],Instant::now()+Duration::from_millis(20),1<<20,&mut stats).unwrap();assert_eq!(x,vec![1.,1.]);assert!(m.check(&x,true,0.).is_ok());assert_eq!(m.objective(&x),if maximize{8.}else{6.});assert_eq!(serde_json::to_string(&m).unwrap(),before);assert_eq!(seed,[0.,0.]);assert!(stats.improvements>0&&stats.penalty_updates>0);}
}
#[test]fn unsupported_or_expired_search_returns_no_claim(){
 let m=linked(false);let seed=[0.,0.];let lo=[Some(0.);2];let up=[Some(1.);2];
 for (deadline,workspace)in [(Instant::now(),1<<20),(Instant::now()+Duration::from_secs(1),1)]{let mut stats=Stats::default();assert!(adaptive_resource_repair::search(&m,&seed,&lo,&up,deadline,workspace,&mut stats).is_none());assert_eq!(stats.trials,0);assert_eq!(stats.improvements,0);}
 let mut stats=Stats::default();assert!(adaptive_resource_repair::search(&m,&[f64::NAN,0.],&lo,&up,Instant::now()+Duration::from_secs(1),1<<20,&mut stats).is_none());
}
#[test]fn library_flag_controls_only_primal_search_after_the_resource_guard(){
 use mip_codex::solve::{solve,Limits};
 let mut m=linked(false);for j in 0..1500{let mut v=m.variables[1].clone();v.name=format!("pad{j}");v.integer=false;m.variables.push(v);}
 for j in 0..800{m.rows.push(serde_json::from_value(serde_json::json!({"name":format!("padrow{j}"),"upper":1000.,"coefficients":[[0,1.]],"upper_enclosure":[1000.,1000.],"exact_coefficients":true})).unwrap());}
 let limits=Limits{seconds:0.5,dense_bytes:1<<20,box_primal:false,primal_repair:false,primal_probe:false,binary_graph:false,cover_cuts:false,mir_cuts:false,clique_cuts:false,aggregate_cuts:false,three_row_cuts:false,gomory_cuts:false,rounding_cuts:false,zero_dominance:false,objective_rounding:false,row_generation:false,..Limits::default()};
 let before=serde_json::to_string(&m).unwrap();
 for enabled in [false,true]{let result=solve(&m,&Limits{adaptive_resource_repair:enabled,..limits.clone()});let stats=result.adaptive_resource_repair.unwrap();assert_eq!(stats.calls,usize::from(enabled));assert_eq!(result.objective,Some(if enabled{6.}else{7.}));let values=result.values.unwrap();let x:Vec<_>=m.variables.iter().map(|v|values[&v.name]).collect();assert!(m.check(&x,true,0.).is_ok());assert_eq!(serde_json::to_string(&m).unwrap(),before);}
}
