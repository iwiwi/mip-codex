//! 独立成分ごとの暫定目的上限で主解を修復する。下界・不能・最適性は合成しない。
use crate::{heuristic::{BoxStats,Repair},model::{Model,Row}};
use std::{collections::BTreeMap,time::{Instant,Duration}};
fn find(parent:&mut[usize],mut j:usize)->usize{
 let mut root=j;while parent[root]!=root{root=parent[root];}
 while parent[j]!=j{let next=parent[j];parent[j]=root;j=next;}root
}
pub fn improve(model:&Model,original:&Repair<'_>,point:&[f64],lower:&[Option<f64>],upper:&[Option<f64>],deadline:Instant)->Option<(Vec<f64>,BoxStats)>{
 improve_shared(model,original,point,lower,upper,deadline,false)
}
pub fn improve_shared(model:&Model,original:&Repair<'_>,point:&[f64],lower:&[Option<f64>],upper:&[Option<f64>],deadline:Instant,share_work:bool)->Option<(Vec<f64>,BoxStats)>{
 let began=Instant::now();let n=model.variables.len();
 if began>=deadline||n==0||n>100_000||model.rows.len()>100_000||model.rows.iter().map(|r|r.coefficients.len()).sum::<usize>()>10_000_000||point.len()!=n||lower.len()!=n||upper.len()!=n||model.maximize||model.offset!=0.||model.offset_enclosure!=Some([0.,0.])||!model.variables.iter().all(|v|v.integer&&v.lower==Some(0.)&&v.upper==Some(1.)&&v.objective==1.&&v.objective_enclosure==Some([1.,1.]))||!original.candidate_check(point)||!original.objective_reliable(point,model.objective(point)){return None;}
 let mut parent:Vec<_>=(0..n).collect();let mut sizes=vec![1usize;n];
 for row in &model.rows{
  if Instant::now()>=deadline{return None;}if row.lower.is_none()&&row.upper.is_none(){continue;}
  if let Some(&(first,_))=row.coefficients.first(){
   if first>=n{return None;}
   for &(j,a)in &row.coefficients{if j>=n||!a.is_finite(){return None;}let(mut u,mut v)=(find(&mut parent,first),find(&mut parent,j));if u!=v{if sizes[u]<sizes[v]{std::mem::swap(&mut u,&mut v);}parent[v]=u;sizes[u]+=sizes[v];}}
  }
 }
 let roots:Vec<_>=(0..n).map(|j|find(&mut parent,j)).collect();let mut groups=BTreeMap::<usize,Vec<usize>>::new();let mut rows=BTreeMap::<usize,Vec<usize>>::new();
 for(j,&root)in roots.iter().enumerate(){groups.entry(root).or_default().push(j);}
 for(i,row)in model.rows.iter().enumerate(){if row.lower.is_none()&&row.upper.is_none(){continue;}if let Some(&(j,_))=row.coefficients.first(){rows.entry(roots[j]).or_default().push(i);}}
 let mut groups:Vec<_>=groups.into_iter().filter(|(root,_)|rows.contains_key(root)).collect();if groups.len()<2||groups.len()>64{return None;}groups.sort_by_key(|(root,cols)|(std::cmp::Reverse(cols.len()),*root));
 let mut remaining=groups.len();let mut inverse=vec![usize::MAX;n];let mut x=point.to_vec();let mut stats=BoxStats::default();
 for(root,columns)in groups{
  let now=Instant::now();if now>=deadline||stats.work>=1_000_000_000{break;}let local_end=deadline.min(now+(deadline-now)/remaining as u32);let local_work_end=if share_work{stats.work+(1_000_000_000usize.saturating_sub(stats.work))/remaining}else{1_000_000_000};remaining-=1;
  for(k,&j)in columns.iter().enumerate(){inverse[j]=k;}
  let mut sub=Model{name:format!("internal-component-{root}"),maximize:false,offset:0.,offset_enclosure:Some([0.,0.]),original_decimal:model.original_decimal,variables:columns.iter().map(|&j|model.variables[j].clone()).collect(),rows:rows[&root].iter().map(|&i|{let mut row=model.rows[i].clone();row.coefficients=row.coefficients.iter().map(|&(j,a)|(inverse[j],a)).collect();row}).collect()};
  let lo:Vec<_>=columns.iter().map(|&j|lower[j]).collect();let hi:Vec<_>=columns.iter().map(|&j|upper[j]).collect();let mut local:Vec<_>=columns.iter().map(|&j|x[j]).collect();let original_local=Repair::new(&sub);if !original_local.candidate_check(&local){return None;}
  sub.rows.push(Row {integer_bounds:None,name:"internal_component_objective_cut".into(),lower:None,upper:None,coefficients:(0..columns.len()).map(|j|(j,1.)).collect(),exact_integral:true,exact_coefficients:true,exact_coefficient_flags:Vec::new(),lower_enclosure:None,upper_enclosure:None});
  for seed in 0..10000u64{
   if Instant::now()>=local_end||stats.work>=local_work_end{break;}
   let target=local.iter().sum::<f64>()-1.;let row=sub.rows.last_mut().unwrap();row.upper=Some(target);row.upper_enclosure=Some([target,target]);
   let level=((seed/2)%4)as usize;let on=[0.99,0.95,0.90,0.80][level];let off=[0.0001,0.0005,0.001,0.002][level];let start:Vec<_>=local.iter().map(|&v|if seed<2{v}else if v>0.5{on}else{off}).collect();let mut repair=Repair::new(&sub);let end=local_end.min(Instant::now()+Duration::from_millis(100));
   let candidate=repair.search(&start,&lo,&hi,end,100_000_000usize.min(local_work_end.saturating_sub(stats.work)),seed);stats.calls+=repair.stats.calls;stats.moves+=repair.stats.moves;stats.work+=repair.stats.work;
   if let Some(v)=candidate{
    let mut combined=x.clone();for(k,&j)in columns.iter().enumerate(){combined[j]=v[k];}
    if original.candidate_check(&combined)&&original.objective_reliable(&combined,model.objective(&combined)){
     stats.candidates+=1;let value=model.objective(&combined);if value.is_finite()&&value<model.objective(&x)-4e-8*(1.+value.abs().max(model.objective(&x).abs())){x=combined;local=v;stats.improvements+=1;stats.first_seconds.get_or_insert_with(||began.elapsed().as_secs_f64());}
    }
   }
  }
 }
 stats.seconds=began.elapsed().as_secs_f64();Some((x,stats))
}
