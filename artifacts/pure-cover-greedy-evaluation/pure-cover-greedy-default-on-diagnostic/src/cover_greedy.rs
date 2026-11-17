//! 非負費用の0/1純被覆に対する貪欲主解。証明や枝刈りには使用しない。
use crate::{heuristic::{BoxStats,Repair},model::Model};
use std::{cmp::Ordering,collections::BinaryHeap,time::Instant};
#[derive(Clone,Copy)]struct Entry{score:f64,j:usize,gain:usize}
impl PartialEq for Entry{fn eq(&self,o:&Self)->bool{self.score.to_bits()==o.score.to_bits()&&self.j==o.j&&self.gain==o.gain}}impl Eq for Entry{}
impl Ord for Entry{fn cmp(&self,o:&Self)->Ordering{o.score.total_cmp(&self.score).then_with(||o.j.cmp(&self.j)).then_with(||self.gain.cmp(&o.gain))}}impl PartialOrd for Entry{fn partial_cmp(&self,o:&Self)->Option<Ordering>{Some(self.cmp(o))}}
pub fn candidate(m:&Model,repair:&Repair<'_>,deadline:Instant)->Option<(Option<Vec<f64>>,BoxStats)>{
 let started=Instant::now();let n=m.variables.len();
 if started>=deadline||m.maximize||n==0||n>100_000||m.rows.len()>100_000||m.rows.iter().fold(0usize,|v,r|v.saturating_add(r.coefficients.len()))>10_000_000||!m.variables.iter().all(|v|v.integer&&v.lower==Some(0.)&&v.upper==Some(1.)&&v.objective.is_finite()&&v.objective>=0.)||!m.rows.iter().all(|r|r.lower.is_some_and(|l|l>0.&&l<=1.)&&r.upper.is_none()&&r.coefficients.iter().all(|&(j,c)|j<n&&c==1.)){return None;}
 let mut stats=BoxStats{calls:1,..BoxStats::default()};let mut incidence=vec![vec![];n];
 for(i,r)in m.rows.iter().enumerate(){if Instant::now()>=deadline{stats.seconds=started.elapsed().as_secs_f64();return Some((None,stats));}for&(j,_)in&r.coefficients{incidence[j].push(i);stats.work+=1;}}
 let mut gain:Vec<_>=incidence.iter().map(Vec::len).collect();let mut heap=BinaryHeap::new();
 for j in 0..n{if gain[j]>0{heap.push(Entry{score:m.variables[j].objective/gain[j]as f64,j,gain:gain[j]});}}
 let mut selected=vec![false;n];let mut counts=vec![0usize;m.rows.len()];let mut remaining=m.rows.len();
 while remaining>0&&Instant::now()<deadline&&stats.work<100_000_000{
  let Some(e)=heap.pop()else{break;};if gain[e.j]==0||selected[e.j]{continue;}
  if e.gain!=gain[e.j]{heap.push(Entry{score:m.variables[e.j].objective/gain[e.j]as f64,j:e.j,gain:gain[e.j]});continue;}
  selected[e.j]=true;stats.moves+=1;
  for&i in&incidence[e.j]{if counts[i]==0{remaining-=1;for&(j,_)in&m.rows[i].coefficients{gain[j]=gain[j].saturating_sub(1);stats.work+=1;}}counts[i]+=1;}
 }
 if remaining==0{
  let mut chosen:Vec<_>=(0..n).filter(|&j|selected[j]).collect();chosen.sort_by(|&j,&k|m.variables[k].objective.total_cmp(&m.variables[j].objective).then_with(||k.cmp(&j)));
  for j in chosen{if Instant::now()>=deadline{break;}if incidence[j].iter().all(|&i|counts[i]>1){selected[j]=false;stats.moves+=1;for&i in&incidence[j]{counts[i]-=1;}}}
 }
 let x:Vec<_>=selected.iter().map(|&s|if s{1.}else{0.}).collect();let valid=remaining==0&&repair.candidate_check(&x)&&repair.objective_reliable(&x,m.objective(&x));
 if valid{stats.candidates=1;stats.first_seconds=Some(started.elapsed().as_secs_f64());}stats.seconds=started.elapsed().as_secs_f64();Some((valid.then_some(x),stats))
}
