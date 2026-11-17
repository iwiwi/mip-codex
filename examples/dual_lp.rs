//! 双対形式の比較検証用。外部最適化処理なし。
use mip_codex::{revised::{self,Problem},lp::Outcome};
use serde::Deserialize;
use std::io::{self,Read};
#[derive(Deserialize)]struct Input{columns:Vec<Vec<(usize,f64)>>,rhs:Vec<f64>,costs:Vec<f64>,upper:Vec<Option<f64>>,equality:Vec<bool>,slack_upper:Vec<Option<f64>>}
fn main(){
 if std::env::args().nth(1).is_some_and(|v|v=="build-info"){println!("{}",serde_json::json!({"source_directory":env!("CARGO_MANIFEST_DIR"),"source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT,"library_source_directory":mip_codex::BUILD_SOURCE_DIRECTORY,"library_source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT}));return;}
 let mut s=String::new();io::stdin().read_to_string(&mut s).unwrap();let inputs:Vec<Input>=serde_json::from_str(&s).unwrap();for(case,p)in inputs.into_iter().enumerate(){let p=Problem{columns:p.columns,rhs:p.rhs,costs:p.costs,upper:p.upper.into_iter().map(|x|x.unwrap_or(f64::INFINITY)).collect(),equality:p.equality,slack_upper:p.slack_upper.into_iter().map(|x|x.unwrap_or(f64::INFINITY)).collect()};let a=revised::minimize_warm(&p,100000,None,None);let(status,objective,x)=match a.outcome{Outcome::Optimal{objective,x}=>("optimal".to_string(),Some(objective),Some(x)),other=>(format!("{other:?}"),None,None)};println!("{}",serde_json::json!({"case":case,"status":status,"objective":objective,"x":x,"dual":a.dual,"pivots":a.stats.pivots,"attempts":a.stats.dual_form_attempts,"successes":a.stats.dual_form_successes,"fallbacks":a.stats.dual_form_fallbacks}));}
}
