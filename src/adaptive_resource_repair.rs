//! 資源制限後の主解を目的改善行と座標修復で改善する。局所証明は返さない。
use crate::{heuristic::Repair,model::{Model,Row}};
use std::time::Instant;
#[derive(Default,Debug,serde::Serialize)]
pub struct Stats{pub calls:usize,pub trials:usize,pub candidates:usize,pub improvements:usize,pub moves:usize,pub work:usize,pub penalty_updates:usize,pub seconds:f64}
pub fn search(model:&Model,point:&[f64],lower:&[Option<f64>],upper:&[Option<f64>],deadline:Instant,workspace:usize,stats:&mut Stats)->Option<Vec<f64>>{
 search_mode(model,point,lower,upper,deadline,workspace,stats,false)
}
pub fn search_incremental(model:&Model,point:&[f64],lower:&[Option<f64>],upper:&[Option<f64>],deadline:Instant,workspace:usize,stats:&mut Stats)->Option<Vec<f64>>{
 search_mode(model,point,lower,upper,deadline,workspace,stats,true)
}
fn search_mode(model:&Model,point:&[f64],lower:&[Option<f64>],upper:&[Option<f64>],deadline:Instant,workspace:usize,stats:&mut Stats,incremental:bool)->Option<Vec<f64>>{
 let began=Instant::now();stats.calls+=1;
 let result=inner(model,point,lower,upper,deadline,workspace,stats,incremental);stats.seconds+=began.elapsed().as_secs_f64();result
}
fn inner(model:&Model,point:&[f64],lower:&[Option<f64>],upper:&[Option<f64>],deadline:Instant,workspace:usize,stats:&mut Stats,incremental:bool)->Option<Vec<f64>>{
 if Instant::now()>=deadline||point.len()!=model.variables.len()||lower.len()!=point.len()||upper.len()!=point.len(){return None;}
 let nnz=model.rows.iter().fold(0u128,|n,r|n.saturating_add(r.coefficients.len()as u128));
 let estimate=nnz.saturating_mul(64)+(model.variables.len()as u128)*256+(model.rows.len()as u128)*if incremental{192}else{128};
 if estimate>workspace as u128{return None;}
 let original=Repair::new(model);if !original.candidate_check(point)||!original.objective_reliable(point,model.objective(point)){return None;}
 let sign=if model.maximize{-1.}else{1.};let scale=model.variables.iter().map(|v|v.objective.abs()).fold(0f64,f64::max);
 let step=model.variables.iter().map(|v|v.objective.abs()).filter(|&v|v>0.).min_by(f64::total_cmp)?;
 if !scale.is_finite()||scale==0.||!step.is_finite(){return None;}
 let mut work=model.clone();work.decimal_source=None;
 work.rows.push(Row{integer_bounds:None,name:"internal_adaptive_objective_target".into(),lower:None,upper:None,coefficients:model.variables.iter().enumerate().filter(|(_,v)|v.objective!=0.).map(|(j,v)|(j,sign*v.objective/scale)).collect(),exact_integral:false,exact_coefficients:false,exact_coefficient_flags:vec![],lower_enclosure:None,upper_enclosure:None});
 let initial_improvements=stats.improvements;
 let mut best=point.to_vec();let mut value=sign*model.objective(&best);let budget=100_000_000usize;
 for trial in 0..16{
  if Instant::now()>=deadline||stats.work>=budget{break;}
  let target=(value-step-sign*model.offset)/scale;if !target.is_finite(){break;}
  let row=work.rows.last_mut().unwrap();row.upper=Some(target);row.upper_enclosure=Some([target,target]);
  let mut repair=Repair::new(&work);let candidate=if incremental{repair.search_adaptive_incremental(&best,lower,upper,deadline,budget.saturating_sub(stats.work),trial)}else{repair.search_adaptive(&best,lower,upper,deadline,budget.saturating_sub(stats.work),trial)};
  stats.trials+=1;stats.moves+=repair.stats.moves;stats.work=stats.work.saturating_add(repair.stats.work);stats.penalty_updates+=repair.stats.penalty_updates;
  let Some(x)=candidate else{break;};
  if !original.candidate_check(&x)||!original.objective_reliable(&x,model.objective(&x)){break;}
  stats.candidates+=1;let next=sign*model.objective(&x);
  if !next.is_finite()||next>=value-4e-8*(1.+value.abs().max(next.abs())){break;}
  best=x;value=next;stats.improvements+=1;
 }
 (stats.improvements>initial_improvements).then_some(best)
}
