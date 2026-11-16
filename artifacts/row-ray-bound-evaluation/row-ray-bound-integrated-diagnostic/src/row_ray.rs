//! 元の単位箱に対する非負被覆行のLagrangian下界。探索や枝刈りには使わない。
use crate::{model::Model,objective_bound::Certifier};
use serde::{Serialize,Deserialize};
use std::time::Instant;
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Proof{
 pub rows:Vec<usize>,
 pub multiplier_bits:String,
 pub normalized_bound:f64,
 pub seconds:f64,
}
pub fn compute(m:&Model,deadline:Instant)->Option<Proof>{
 let start=Instant::now();
 if start>=deadline || m.variables.len()>100_000 || m.rows.len()>100_000 || m.rows.iter().map(|r|r.coefficients.len()).sum::<usize>()>1_000_000{return None;}
 let unit=m.variables.iter().all(|v|v.lower==Some(0.)&&v.upper==Some(1.)&&v.lower_enclosure==Some([0.,0.])&&v.upper_enclosure==Some([1.,1.]));
 if !unit{return None;}
 let sign=if m.maximize{-1.}else{1.};
 let mut d=vec![0.;m.variables.len()];let mut b=0.;let mut rows=Vec::new();
 for(i,r)in m.rows.iter().enumerate(){
  if Instant::now()>=deadline{return None;}
  if r.exact_integral&&r.lower.is_some_and(|v|v>0.)&&r.coefficients.iter().all(|&(_,v)|v>=0.){
   rows.push(i);b+=r.lower.unwrap();for &(j,a)in &r.coefficients{d[j]+=a;}
  }
 }
 let mut slope=b;let mut breaks=Vec::new();
 for(j,v)in m.variables.iter().enumerate(){let c=sign*v.objective;if c<=0.{slope-=d[j];}else if d[j]>0.{breaks.push((c/d[j],d[j]));}}
 breaks.sort_by(|a,b|a.0.total_cmp(&b.0));
 let mut alpha=0.;if slope>0.{for(t,w)in breaks{alpha=t;slope-=w;if slope<=0.{break;}}}
 let mut y=vec![0.;m.rows.len()];for &i in &rows{y[i]=alpha;}
 let lower:Vec<_>=m.variables.iter().map(|v|v.lower).collect();let upper:Vec<_>=m.variables.iter().map(|v|v.upper).collect();
 if rows.is_empty() || Instant::now()>=deadline{return None;}
 let bound=Certifier::new(m).and_then(|c|c.bound(&lower,&upper,&y));

 let bound=bound?.next_down();
 if !bound.is_finite() || Instant::now()>=deadline{return None;}
 Some(Proof{rows,multiplier_bits:format!("{:016x}",alpha.to_bits()),normalized_bound:bound,seconds:start.elapsed().as_secs_f64()})
}
#[cfg(test)]
mod tests{
 use super::*;
 fn model(maximize:bool)->Model{
  serde_json::from_value(serde_json::json!({"name":"cover","maximize":maximize,"offset":3.,"offset_enclosure":[3.,3.],"variables":(0..3).map(|j|serde_json::json!({"name":format!("x{j}"),"lower":0.,"upper":1.,"integer":true,"objective":if maximize{-1.}else{1.},"objective_enclosure":if maximize{[-1.,-1.]}else{[1.,1.]},"lower_enclosure":[0.,0.],"upper_enclosure":[1.,1.]})).collect::<Vec<_>>(),"rows":[{"name":"a","lower":1.,"upper":null,"coefficients":[[0,1.],[1,1.]],"exact_integral":true,"lower_enclosure":[1.,1.]},{"name":"b","lower":1.,"upper":null,"coefficients":[[1,1.],[2,1.]],"exact_integral":true,"lower_enclosure":[1.,1.]}]})).unwrap()
 }
 #[test]fn min_max_bounds_match_exhaustive_box(){
  for maximize in [false,true]{let m=model(maximize);let proof=compute(&m,Instant::now()+std::time::Duration::from_secs(1)).unwrap();let sign=if maximize{-1.}else{1.};let opt=(0..8).filter_map(|bits|{let x:Vec<_>=(0..3).map(|j|((bits>>j)&1)as f64).collect();m.check(&x,true,1e-8).is_ok().then(||sign*m.objective(&x))}).fold(f64::INFINITY,f64::min);assert!(proof.normalized_bound<=opt);assert!(proof.normalized_bound>sign*3.+0.9);assert_eq!(proof.rows,vec![0,1]);}
 }
 #[test]fn unsupported_bounds_and_expiration_skip(){let mut m=model(false);assert!(compute(&m,Instant::now()).is_none());m.variables[0].upper=Some(2.);assert!(compute(&m,Instant::now()+std::time::Duration::from_secs(1)).is_none());}
}
