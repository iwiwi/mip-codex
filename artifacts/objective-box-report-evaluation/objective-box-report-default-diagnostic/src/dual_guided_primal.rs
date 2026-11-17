//! 自作被覆乗数からのbox最小点とゼロ初期点を共通予算で修復する。
use crate::{model::Model,heuristic::{Repair,BoxStats},row_subgradient::Proof};
use std::time::Instant;
pub fn improve(m:&Model,repair:&mut Repair<'_>,point:&[f64],lower:&[Option<f64>],upper:&[Option<f64>],proof:&Proof,deadline:Instant)->Option<(Vec<f64>,BoxStats)>{
 let began=Instant::now();
 if began>=deadline||m.maximize||point.len()!=m.variables.len()||lower.len()!=point.len()||upper.len()!=point.len()||point.is_empty()||point.len()>100_000||m.rows.len()>100_000||m.rows.iter().map(|r|r.coefficients.len()).sum::<usize>()>10_000_000||!m.variables.iter().all(|v|v.integer&&v.lower==Some(0.)&&v.upper==Some(1.)&&v.lower_enclosure==Some([0.,0.])&&v.upper_enclosure==Some([1.,1.])&&v.objective.is_finite()&&v.objective>=0.)||!repair.candidate_check(point)||!repair.objective_reliable(point,m.objective(point)){return None;}
 let mut reduced:Vec<_>=m.variables.iter().map(|v|v.objective).collect();
 for(i,bits)in &proof.multipliers{if *i>=m.rows.len(){return None;}let y=f64::from_bits(u64::from_str_radix(bits,16).ok()?);if !y.is_finite()||y<0.{return None;}for &(j,a)in &m.rows[*i].coefficients{reduced[j]-=y*a;}}
 if reduced.iter().any(|v|!v.is_finite()){return None;}
 let mut costs:Vec<_>=m.variables.iter().map(|v|v.objective).filter(|&v|v>0.).collect();if costs.is_empty(){return None;}costs.sort_by(f64::total_cmp);let scale=costs[costs.len()/2];
 let before=repair.stats.clone();let mut x=point.to_vec();let mut best=m.objective(&x);let mut stats=BoxStats::default();
 for zero in [true,false]{
  if Instant::now()>=deadline||repair.stats.work-before.work>=100_000_000{break;}
  let initial:Vec<_>=reduced.iter().enumerate().map(|(j,&c)|if zero{0.001}else if c<0.{0.95}else if c>0.{0.001}else if x[j]>0.5{0.95}else{0.001}).collect();
  for seed in 0..16{
   let work=repair.stats.work-before.work;if Instant::now()>=deadline||work>=100_000_000{break;}
   if let Some(v)=repair.search_priced(&initial,lower,upper,deadline,100_000_000-work,seed,1.,scale){
    let objective=m.objective(&v);if !repair.candidate_check(&v)||!repair.objective_reliable(&v,objective){continue;}stats.candidates+=1;
    if objective.is_finite()&&objective<best-4e-8*(1.+objective.abs().max(best.abs())){x=v;best=objective;stats.improvements+=1;stats.first_seconds.get_or_insert_with(||began.elapsed().as_secs_f64());}
   }
  }
 }
 stats.calls=repair.stats.calls-before.calls;stats.moves=repair.stats.moves-before.moves;stats.work=repair.stats.work-before.work;stats.seconds=began.elapsed().as_secs_f64();Some((x,stats))
}
