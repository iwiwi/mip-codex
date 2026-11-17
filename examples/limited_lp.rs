//! Iteration-limited LP points, independently checked by external test drivers.
use mip_codex::{revised::{self,Problem},lp::Outcome};
use serde::Deserialize;
use std::io::{self,Read};
#[derive(Deserialize)]struct Input{columns:Vec<Vec<(usize,f64)>>,rhs:Vec<f64>,costs:Vec<f64>,upper:Vec<Option<f64>>,equality:Vec<bool>,slack_upper:Vec<Option<f64>>,limits:Vec<usize>}
fn main(){
 if std::env::args().nth(1).is_some_and(|v|v=="build-info"){println!("{}",serde_json::json!({"source_directory":env!("CARGO_MANIFEST_DIR"),"source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT,"library_source_directory":mip_codex::BUILD_SOURCE_DIRECTORY,"library_source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT}));return;}
 let mut text=String::new();io::stdin().read_to_string(&mut text).unwrap();let inputs:Vec<Input>=serde_json::from_str(&text).unwrap();
 for(case,v)in inputs.into_iter().enumerate(){let p=Problem{columns:v.columns,rhs:v.rhs,costs:v.costs,upper:v.upper.into_iter().map(|v|v.unwrap_or(f64::INFINITY)).collect(),equality:v.equality,slack_upper:v.slack_upper.into_iter().map(|v|v.unwrap_or(f64::INFINITY)).collect()};let optimal=revised::minimize_warm(&p,100000,None,None);for limit in v.limits{for warm in [false,true]{let a=revised::minimize_warm(&p,limit,None,if warm{optimal.warm.as_deref()}else{None});let(status,point)=match a.outcome{Outcome::Optimal{x,..}=>("optimal",Some(x)),Outcome::IterationLimit=>("iteration_limit",a.limited_point),Outcome::TimeLimit=>("time_limit",a.limited_point),other=>{println!("{}",serde_json::json!({"case":case,"limit":limit,"warm":warm,"status":format!("{other:?}")}));continue;}};println!("{}",serde_json::json!({"case":case,"limit":limit,"warm":warm,"status":status,"point":point,"pivots":a.stats.pivots,"dual":a.dual}));}}}
}
