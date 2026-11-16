use mip_codex::{heuristic::Repair,model::Model};
use std::time::{Instant,Duration};
#[test]
fn price_zero_matches_existing_order_and_other_prices_preserve_feasibility(){
 let mut state=20261116u64;
 for case in 0..200{let n=2+case%7;let mut next=||{state=state.wrapping_mul(6364136223846793005).wrapping_add(1);state};let mut rows=Vec::new();for i in 0..8{let mut a=Vec::new();for j in 0..n{if next()>>62!=0{a.push((j,1.));}}if a.is_empty(){a.push((0,1.));}rows.push(serde_json::json!({"name":format!("r{i}"),"lower":1.,"upper":null,"lower_enclosure":[1.,1.],"coefficients":a,"exact_integral":true}));}let costs:Vec<_>=(0..n).map(|_|1.+(next()%100)as f64).collect();let m:Model=serde_json::from_value(serde_json::json!({"name":"price","maximize":false,"offset":0.,"offset_enclosure":[0.,0.],"variables":(0..n).map(|j|serde_json::json!({"name":format!("x{j}"),"lower":0.,"upper":1.,"integer":true,"objective":costs[j],"objective_enclosure":[costs[j],costs[j]],"lower_enclosure":[0.,0.],"upper_enclosure":[1.,1.]})).collect::<Vec<_>>(),"rows":rows})).unwrap();let point=vec![0.;n];let lower=vec![Some(0.);n];let upper=vec![Some(1.);n];let end=Instant::now()+Duration::from_secs(1);let old=Repair::new(&m).search(&point,&lower,&upper,end,100000,0);let zero=Repair::new(&m).search_priced(&point,&lower,&upper,end,100000,0,0.,10.);assert_eq!(old,zero);for weight in [0.1,1.,10.,100.]{if let Some(x)=Repair::new(&m).search_priced(&point,&lower,&upper,end,100000,0,weight,10.){assert!(m.check(&x,true,0.).is_ok());}}
 }
}
