use mip_codex::{heuristic::Repair,model::Model,adaptive_resource_repair::{self,Stats}};
use std::time::{Instant,Duration};
#[test]fn incremental_updates_keep_duplicate_row_entries_and_parent_points_valid(){
 let m:Model=serde_json::from_value(serde_json::json!({"name":"incremental","maximize":false,"offset":0.,"variables":[{"name":"x","lower":0.,"upper":1.,"integer":true,"objective":-1.},{"name":"y","lower":0.,"upper":1.,"integer":true,"objective":0.}],"rows":[{"name":"link","lower":0.,"upper":0.,"coefficients":[[0,1.],[0,1.],[1,-2.]]},{"name":"target","upper":-1.,"coefficients":[[0,-1.]]}]})).unwrap();
 let original=serde_json::to_string(&m).unwrap();let point=[0.,0.];let lo=[Some(0.);2];let up=[Some(1.);2];
 let mut old=Repair::new(&m);let a=old.search_adaptive(&point,&lo,&up,Instant::now()+Duration::from_secs(1),100000,0).unwrap();
 let mut new=Repair::new(&m);let b=new.search_adaptive_incremental(&point,&lo,&up,Instant::now()+Duration::from_secs(1),100000,0).unwrap();
 assert_eq!(a,vec![1.,1.]);assert_eq!(b,a);assert!(m.check(&b,true,0.).is_ok());assert_eq!(point,[0.,0.]);assert_eq!(serde_json::to_string(&m).unwrap(),original);assert!(new.stats.penalty_updates>0);
}
#[test]fn generated_incremental_candidates_respect_enumerated_optima_in_both_senses(){
 let mut rng=2026111749u64;let mut candidates=0;
 for case in 0..100{let n=2+case%5;let maximize=case%2==0;let mut variables=vec![];for j in 0..n{rng=rng.wrapping_mul(6364136223846793005).wrapping_add(1);let cost=(rng>>32)%7;let cost=cost as f64-3.;variables.push(serde_json::json!({"name":format!("x{j}"),"lower":0.,"upper":1.,"integer":true,"objective":cost,"objective_enclosure":[cost,cost]}));}let mut rows=vec![];for _ in 0..6{let mut coeff=vec![];for j in 0..n{rng=rng.wrapping_mul(6364136223846793005).wrapping_add(1);let c=((rng>>32)%5)as f64-2.;if c!=0.{coeff.push((j,c));}}rows.push(serde_json::json!({"name":"row","upper":1.,"upper_enclosure":[1.,1.],"coefficients":coeff}));}
 let m:Model=serde_json::from_value(serde_json::json!({"name":"generated","maximize":maximize,"offset":3.,"offset_enclosure":[3.,3.],"variables":variables,"rows":rows})).unwrap();let sign=if maximize{-1.}else{1.};let optimum=(0..1usize<<n).filter_map(|bits|{let x:Vec<_>=(0..n).map(|j|((bits>>j)&1)as f64).collect();m.check(&x,true,0.).is_ok().then(||sign*m.objective(&x))}).fold(f64::INFINITY,f64::min);let seed=vec![0.;n];let before=serde_json::to_string(&m).unwrap();let mut stats=Stats::default();if let Some(x)=adaptive_resource_repair::search_incremental_reusing(&m,&seed,&vec![Some(0.);n],&vec![Some(1.);n],Instant::now()+Duration::from_millis(5),1<<20,&mut stats){assert!(m.check(&x,true,0.).is_ok());let v=sign*m.objective(&x);assert!(v>=optimum);assert!(v<sign*m.objective(&seed));candidates+=1;}assert_eq!(serde_json::to_string(&m).unwrap(),before);assert_eq!(seed,vec![0.;n]);}
 assert!(candidates>0);
}
