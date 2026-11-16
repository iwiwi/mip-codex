use mip_codex::{heuristic::Repair,model::Model,objective_repair};
use std::time::{Instant,Duration};
#[test]
fn objective_cut_preserves_original_feasibility_and_the_seed(){
 let mut state=20261116u64;let mut gains=0;
 for case in 0..200{let n=2+case%7;let mut rows=Vec::new();for i in 0..8{let mut a=Vec::new();for j in 0..n{state=state.wrapping_mul(6364136223846793005).wrapping_add(1);if state>>62!=0{a.push((j,1.));}}if a.is_empty(){a.push((0,1.));}rows.push(serde_json::json!({"name":format!("r{i}"),"lower":1.,"upper":null,"coefficients":a,"exact_integral":true,"lower_enclosure":[1.,1.]}));}
 let m:Model=serde_json::from_value(serde_json::json!({"name":"repair","maximize":false,"offset":0.,"offset_enclosure":[0.,0.],"variables":(0..n).map(|j|serde_json::json!({"name":format!("x{j}"),"lower":0.,"upper":1.,"integer":true,"objective":1.,"objective_enclosure":[1.,1.],"lower_enclosure":[0.,0.],"upper_enclosure":[1.,1.]})).collect::<Vec<_>>(),"rows":rows})).unwrap();let repair=Repair::new(&m);let seed=vec![1.;n];let lower=vec![Some(0.);n];let upper=vec![Some(1.);n];let(x,s)=objective_repair::improve(&m,&repair,&seed,&lower,&upper,Instant::now()+Duration::from_secs(1)).unwrap();assert!(m.check(&x,true,0.).is_ok());assert!(m.objective(&x)<=m.objective(&seed));gains+=s.improvements;let opt=(0..1usize<<n).filter_map(|bits|{let x:Vec<_>=(0..n).map(|j|((bits>>j)&1)as f64).collect();m.check(&x,true,0.).is_ok().then(||m.objective(&x))}).fold(f64::INFINITY,f64::min);assert!(m.objective(&x)>=opt);assert!(objective_repair::improve(&m,&repair,&seed,&lower,&upper,Instant::now()).is_none());}
 assert!(gains>0);
}
#[test]
fn impossible_objective_cut_keeps_the_original_seed_and_node_limit(){
 use mip_codex::{mps,solve::{solve,Limits}};
 let mut text=String::from("NAME seed\nROWS\n N obj\n E row\nCOLUMNS\n");for j in 0..4096{text.push_str(&format!(" x{j} obj 1\n"));if j<2{text.push_str(&format!(" x{j} row 1\n"));}}text.push_str("RHS\n rhs row 1\nBOUNDS\n");for j in 0..4096{text.push_str(&format!(" BV b x{j}\n"));}text.push_str("ENDATA\n");let m=mps::parse(std::io::Cursor::new(text)).unwrap();let r=solve(&m,&Limits{nodes:0,seconds:1.,binary_graph:false,cover_cuts:false,clique_cuts:false,aggregate_cuts:false,mir_cuts:false,three_row_cuts:false,root_objective_repair:true,..Limits::default()});assert_eq!(r.status,"node_limit");assert_eq!(r.lp_solves,0);assert_eq!(r.objective,Some(1.));let s=r.root_objective_repair.unwrap();assert_eq!(s.improvements,0);assert!(s.calls>0);let values=r.values.unwrap();let x:Vec<_>=m.variables.iter().map(|v|values[&v.name]).collect();assert!(m.check(&x,true,0.).is_ok());
}
