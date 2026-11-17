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
