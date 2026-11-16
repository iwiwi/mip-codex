//! 非負費用の単位箱への被覆乗数。元入力で検証した下界を表示する。
use crate::{model::Model,objective_bound::Certifier,row_ray};
use serde::{Deserialize,Serialize};
use std::time::Instant;
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Proof{pub multipliers:Vec<(usize,String)>,pub normalized_bound:f64,pub iterations:usize,pub seconds:f64}
pub fn compute(m:&Model,target:f64,initial:Option<&row_ray::Proof>,deadline:Instant)->Option<Proof>{
 compute_with_uniform(m,target,initial,deadline,false)
}
pub fn compute_with_uniform(m:&Model,target:f64,initial:Option<&row_ray::Proof>,deadline:Instant,allow_uniform:bool)->Option<Proof>{
 let began=Instant::now();
 if began>=deadline||m.maximize||!target.is_finite()||!m.offset.is_finite()||m.variables.is_empty()||m.variables.len()>100_000||m.rows.len()>100_000||m.rows.iter().map(|r|r.coefficients.len()).sum::<usize>()>10_000_000{return None;}
 if !m.variables.iter().all(|v|v.lower==Some(0.)&&v.upper==Some(1.)&&v.lower_enclosure==Some([0.,0.])&&v.upper_enclosure==Some([1.,1.])&&v.objective.is_finite()&&v.objective>=0.)||(!allow_uniform&&m.variables.iter().all(|v|v.objective==m.variables[0].objective)){return None;}
 let owned=if initial.is_none(){row_ray::compute(m,deadline)}else{None};
 let initial=initial.or(owned.as_ref())?;
 if allow_uniform&&target<=initial.normalized_bound+4e-8*(1.+target.abs().max(initial.normalized_bound.abs())){return None;}
 let alpha=f64::from_bits(u64::from_str_radix(&initial.multiplier_bits,16).ok()?);if !alpha.is_finite()||alpha<0.{return None;}
 let rows:Vec<_>=m.rows.iter().enumerate().filter(|(_,r)|r.exact_integral&&r.lower.is_some_and(|v|v>0.)&&r.coefficients.iter().all(|&(_,a)|a>=0.)).map(|(i,_)|i).collect();if rows.is_empty(){return None;}
 let mut y=vec![0.;m.rows.len()];for &i in &initial.rows{if i>=y.len()||rows.binary_search(&i).is_err(){return None;}y[i]=alpha;}
 let mut best=f64::NEG_INFINITY;let mut besty=y.clone();let mut reduced=vec![0.;m.variables.len()];let mut point=vec![0.;reduced.len()];let mut gradient=vec![0.;rows.len()];let mut iterations=0;
 let now=Instant::now();if now>=deadline{return None;}let search_end=now+(deadline-now).mul_f64(0.75);
 while Instant::now()<search_end&&iterations<10000{
  for(j,v)in m.variables.iter().enumerate(){reduced[j]=v.objective;}
  let mut value=m.offset;
  for &i in &rows{value+=y[i]*m.rows[i].lower?;for &(j,a)in &m.rows[i].coefficients{reduced[j]-=y[i]*a;}}
  for j in 0..reduced.len(){value+=reduced[j].min(0.);point[j]=if reduced[j]<0.{1.}else{0.};}
  if !value.is_finite(){break;}if value>best{best=value;besty.clone_from(&y);}
  let mut norm=0.;for(k,&i)in rows.iter().enumerate(){let g=m.rows[i].lower?-m.rows[i].coefficients.iter().map(|&(j,a)|a*point[j]).sum::<f64>();gradient[k]=g;norm+=g*g;}
  if !norm.is_finite()||norm<=0.{break;}let step=2.*(target-value).max(0.)/(norm*((iterations+1)as f64).sqrt());if !step.is_finite(){break;}
  for(k,&i)in rows.iter().enumerate(){y[i]=(y[i]+step*gradient[k]).max(0.);}
  iterations+=1;
 }
 if Instant::now()>=deadline{return None;}
 let lower:Vec<_>=m.variables.iter().map(|v|v.lower).collect();let upper:Vec<_>=m.variables.iter().map(|v|v.upper).collect();let bound=Certifier::new(m)?.bound(&lower,&upper,&besty)?.next_down();
 if !bound.is_finite()||bound<=initial.normalized_bound{return None;}
 Some(Proof{multipliers:besty.iter().enumerate().filter(|(_,v)|**v>0.).map(|(i,v)|(i,format!("{:016x}",v.to_bits()))).collect(),normalized_bound:bound,iterations,seconds:began.elapsed().as_secs_f64()})
}
