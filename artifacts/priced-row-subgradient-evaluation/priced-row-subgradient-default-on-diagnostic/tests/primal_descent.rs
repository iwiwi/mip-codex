use mip_codex::{model::Model,heuristic::Repair};
use std::time::{Instant,Duration};
#[test]
fn descending_a_feasible_seed_is_monotone_for_both_senses(){
 let mut state=20261116u64;let mut improvements=0;
 for case in 0..200{let n=2+case%7;let maximize=case%2==1;let sign=if maximize{-1.}else{1.};let mut next=||{state=state.wrapping_mul(6364136223846793005).wrapping_add(1);state};let seed:Vec<_>=(0..n).map(|_|(next()>>63)as f64).collect();let costs:Vec<_>=(0..n).map(|_|(next()%11)as f64-5.).collect();let mut rows=Vec::new();for i in 0..5{let a:Vec<_>=(0..n).map(|j|(j,(next()%7)as f64-3.)).filter(|&(_,v)|v!=0.).collect();let activity:f64=a.iter().map(|&(j,v)|v*seed[j]).sum();let lo=activity-(next()%3)as f64;let up=activity+(next()%3)as f64;rows.push(serde_json::json!({"name":format!("r{i}"),"lower":lo,"upper":up,"coefficients":a,"exact_integral":true,"lower_enclosure":[lo,lo],"upper_enclosure":[up,up]}));}
 let m:Model=serde_json::from_value(serde_json::json!({"name":"descent","maximize":maximize,"offset":0.,"offset_enclosure":[0.,0.],"variables":(0..n).map(|j|serde_json::json!({"name":format!("x{j}"),"lower":0.,"upper":1.,"integer":true,"objective":costs[j],"objective_enclosure":[costs[j],costs[j]],"lower_enclosure":[0.,0.],"upper_enclosure":[1.,1.]})).collect::<Vec<_>>(),"rows":rows})).unwrap();let repair=Repair::new(&m);let(x,s)=repair.descend(&seed,Instant::now()+Duration::from_secs(1)).unwrap();assert!(m.check(&x,true,0.).is_ok());assert!(sign*m.objective(&x)<=sign*m.objective(&seed));improvements+=s.improvements;assert!(repair.descend(&seed,Instant::now()).is_none());let opt=(0..1usize<<n).filter_map(|bits|{let x:Vec<_>=(0..n).map(|j|((bits>>j)&1)as f64).collect();m.check(&x,true,0.).is_ok().then(||sign*m.objective(&x))}).fold(f64::INFINITY,f64::min);assert!(sign*m.objective(&x)>=opt);
 }
 assert!(improvements>0);
}
#[test]
fn nonfinite_objective_candidate_keeps_the_finite_seed(){
 let m:Model=serde_json::from_value(serde_json::json!({"name":"overflow","maximize":false,"offset":0.,"offset_enclosure":[0.,0.],"variables":(0..2).map(|j|serde_json::json!({"name":format!("x{j}"),"lower":0.,"upper":1.,"integer":true,"objective":-1e308,"objective_enclosure":[-1e308,-1e308],"lower_enclosure":[0.,0.],"upper_enclosure":[1.,1.]})).collect::<Vec<_>>(),"rows":[]})).unwrap();let(x,_)=Repair::new(&m).descend(&[0.,0.],Instant::now()+Duration::from_secs(1)).unwrap();assert_eq!(x,vec![0.,0.]);assert!(m.objective(&x).is_finite());
}
#[test]
fn decimal_objective_cancellation_keeps_the_reliable_seed(){
 let m:Model=serde_json::from_value(serde_json::json!({"name":"decimal-cancellation","maximize":true,"offset":-1e16,"offset_enclosure":[-1e16,-1e16],"variables":[{"name":"x","lower":0.,"upper":1.,"integer":true,"objective":1e16,"objective_enclosure":[1e16,10000000000000002.],"lower_enclosure":[0.,0.],"upper_enclosure":[1.,1.]}],"rows":[]})).unwrap();let(x,_)=Repair::new(&m).descend(&[0.],Instant::now()+Duration::from_secs(1)).unwrap();assert_eq!(x,vec![0.]);
}
