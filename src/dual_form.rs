//! Tall LPだけを双対形式で同じRust LPへ送る実験。元問題の検証なしに状態を変換しない。
use crate::{lp::Outcome,revised::{Problem,Answer,WarmStart}};
use std::time::Instant;
pub(crate) fn solve(p:&Problem,limit:usize,deadline:Option<Instant>,warm:Option<&WarmStart>,retain:bool)->Answer{
 let enabled=std::env::var("MIP_LP_DUAL_FORM").is_ok_and(|v|v=="1");
 solve_enabled(p,limit,deadline,warm,retain,enabled)
}
pub(crate) fn solve_for_root(p:&Problem,limit:usize,deadline:Option<Instant>,warm:Option<&WarmStart>,retain:bool)->Answer{
 solve_enabled(p,limit,deadline,warm,retain,true)
}
fn solve_enabled(p:&Problem,limit:usize,deadline:Option<Instant>,warm:Option<&WarmStart>,retain:bool,enabled:bool)->Answer{
 if !enabled||warm.is_some()||!crate::revised::valid(p)||p.rhs.len()<32||p.rhs.len()<=4*p.costs.len()||p.costs.is_empty()||p.costs.len()>10000{return crate::scaling::solve(p,limit,deadline,warm,retain);}
 let Some((q,positive,negative))=transform(p,deadline)else{return crate::scaling::solve(p,limit,deadline,warm,retain)};
 let mut a=crate::scaling::solve(&q,limit,deadline,None,false);a.stats.dual_form_attempts+=1;
 if let Outcome::Optimal{x:ref dx,..}=a.outcome{
  if let Some(lambda)=a.witness.as_deref(){
   let x:Vec<f64>=lambda.iter().map(|v|-v).collect();let y:Vec<f64>=positive.iter().zip(&negative).map(|(&u,&v)|-dx[u]+v.map_or(0.,|j|dx[j])).collect();
   if let Some((objective,bound))=crate::scaling::primal_dual(p,&x,&y){
    if (objective-bound).abs()<=1e-9{
     a.outcome=Outcome::Optimal{objective,x};a.dual=Some(bound);a.warm=None;a.witness=Some(y);a.witness_point=None;a.stats.dual_form_successes+=1;return a;
    }
   }
  }
 }
 a.stats.dual_form_fallbacks+=1;let mut fallback=crate::scaling::solve(p,limit.saturating_sub(a.stats.pivots),deadline,None,retain);fallback.stats.add(&a.stats);fallback
}
fn transform(p:&Problem,deadline:Option<Instant>)->Option<(Problem,Vec<usize>,Vec<Option<usize>>)> {
 let n=p.costs.len();let m=p.rhs.len();let mut rows=vec![vec![];m];
 for(j,col)in p.columns.iter().enumerate(){if j%1024==0&&deadline.is_some_and(|d|Instant::now()>=d){return None;}for &(i,a)in col{if i>=m||!a.is_finite(){return None;}rows[i].push((j,a));}}
 let mut q=Problem{columns:vec![],rhs:p.costs.clone(),costs:vec![],upper:vec![],equality:vec![false;n],slack_upper:vec![f64::INFINITY;n]};let mut positive=Vec::with_capacity(m);let mut negative=Vec::with_capacity(m);
 for(i,row)in rows.iter().enumerate(){if i%1024==0&&deadline.is_some_and(|d|Instant::now()>=d){return None;}positive.push(q.costs.len());q.columns.push(row.iter().map(|&(j,a)|(j,-a)).collect());q.costs.push(p.rhs[i]);q.upper.push(f64::INFINITY);
  let lower=if p.equality[i]{Some(p.rhs[i])}else if p.slack_upper[i].is_finite(){let b=p.rhs[i]-p.slack_upper[i];if !b.is_finite()||crate::exact::dot_sign([(p.rhs[i],1.),(p.slack_upper[i],-1.),(b,-1.)].into_iter())!=Some(std::cmp::Ordering::Equal){return None;}Some(b)}else{None};
  if let Some(b)=lower{negative.push(Some(q.costs.len()));q.columns.push(row.clone());q.costs.push(-b);q.upper.push(f64::INFINITY);}else{negative.push(None);}
 }
 for(j,&u)in p.upper.iter().enumerate(){if u.is_finite(){q.columns.push(vec![(j,-1.)]);q.costs.push(u);q.upper.push(f64::INFINITY);}}
 Some((q,positive,negative))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn finite_upper_ranged_equality_and_signs_map_back(){
  for cost in [-3.,2.]{let p=Problem{columns:vec![(0..40).map(|i|(i,if i%2==0{1.}else{-1.})).collect()],rhs:(0..40).map(|i|if i%2==0{2.}else{-1.}).collect(),costs:vec![cost],upper:vec![1.5],equality:vec![false;40],slack_upper:vec![f64::INFINITY;40]};let(q,pos,neg)=transform(&p,None).unwrap();let a=crate::scaling::solve(&q,1000,None,None,false);let Outcome::Optimal{x:dx,..}=a.outcome else{panic!("{:?}",a.outcome)};let x:Vec<_>=a.witness.unwrap().iter().map(|v|-v).collect();let y:Vec<_>=pos.iter().zip(neg).map(|(&j,k)|-dx[j]+k.map_or(0.,|k|dx[k])).collect();let(v,d)=crate::scaling::primal_dual(&p,&x,&y).unwrap();assert!((v-d).abs()<1e-10);assert_eq!(x,vec![if cost<0.{1.5}else{1.}]);}
 }
}
