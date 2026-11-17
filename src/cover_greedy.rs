//! 非負費用の0/1純被覆に対する貪欲主解。証明や枝刈りには使用しない。
use crate::{heuristic::{BoxStats,Repair},model::Model};
use std::{cmp::Ordering,collections::BinaryHeap,time::Instant};
#[derive(Clone,Copy)]struct Entry{score:f64,j:usize,gain:usize}
impl PartialEq for Entry{fn eq(&self,o:&Self)->bool{self.score.to_bits()==o.score.to_bits()&&self.j==o.j&&self.gain==o.gain}}impl Eq for Entry{}
impl Ord for Entry{fn cmp(&self,o:&Self)->Ordering{o.score.total_cmp(&self.score).then_with(||o.j.cmp(&self.j)).then_with(||self.gain.cmp(&o.gain))}}impl PartialOrd for Entry{fn partial_cmp(&self,o:&Self)->Option<Ordering>{Some(self.cmp(o))}}
pub fn candidate(m:&Model,repair:&Repair<'_>,deadline:Instant)->Option<(Option<Vec<f64>>,BoxStats)>{candidate_order(m,repair,deadline,0)}
pub fn candidate_portfolio(m:&Model,repair:&Repair<'_>,deadline:Instant,enabled:bool)->Option<(Option<Vec<f64>>,BoxStats)>{
 if !enabled{return candidate(m,repair,deadline);}
 let began=Instant::now();let(mut best,mut stats)=candidate_order(m,repair,deadline,0)?;
 if best.is_none(){return Some((best,stats));}
 for variant in 1..128{
  if Instant::now()>=deadline||stats.work>=100_000_000{break;}
  let Some((point,next))=candidate_order(m,repair,deadline,variant)else{break;};
  stats.calls+=next.calls;stats.candidates+=next.candidates;stats.moves+=next.moves;stats.work+=next.work;
  if let Some(point)=point{let value=m.objective(&point);if best.as_ref().is_none_or(|old|value<m.objective(old)-4e-8*(1.+value.abs().max(m.objective(old).abs()))){best=Some(point);}}
 }
 stats.seconds=began.elapsed().as_secs_f64();Some((best,stats))
}
pub fn candidate_order(m:&Model,repair:&Repair<'_>,deadline:Instant,variant:u64)->Option<(Option<Vec<f64>>,BoxStats)>{
 let started=Instant::now();let n=m.variables.len();
 if started>=deadline||m.maximize||n==0||n>100_000||m.rows.len()>100_000||m.rows.iter().fold(0usize,|v,r|v.saturating_add(r.coefficients.len()))>10_000_000||!m.variables.iter().all(|v|v.integer&&v.lower==Some(0.)&&v.upper==Some(1.)&&v.objective.is_finite()&&v.objective>=0.)||!m.rows.iter().all(|r|r.lower.is_some_and(|l|l>0.&&l<=1.)&&r.upper.is_none()&&r.coefficients.iter().all(|&(j,c)|j<n&&c==1.)){return None;}
 let mut stats=BoxStats{calls:1,..BoxStats::default()};let mut incidence=vec![vec![];n];
 for(i,r)in m.rows.iter().enumerate(){if Instant::now()>=deadline{stats.seconds=started.elapsed().as_secs_f64();return Some((None,stats));}for&(j,_)in&r.coefficients{incidence[j].push(i);stats.work+=1;}}
 let exponent=[1.,0.8,1.2][variant as usize%3];
 let rank_cost=(variant!=0).then(||m.variables.iter().enumerate().map(|(j,v)|{let mut q=(j as u64).wrapping_mul(6364136223846793005).wrapping_add(variant.wrapping_mul(0x9e3779b97f4a7c15));q^=q<<13;q^=q>>7;q^=q<<17;let value=v.objective*(0.75+0.5*(q>>11)as f64/(1u64<<53)as f64);if value.is_finite(){value}else{v.objective}}).collect::<Vec<_>>());
 let score=|j:usize,g:usize|match rank_cost.as_ref(){None=>m.variables[j].objective/g as f64,Some(c)=>c[j]/(g as f64).powf(exponent)};
 let mut gain:Vec<_>=incidence.iter().map(Vec::len).collect();let mut heap=BinaryHeap::new();
 for j in 0..n{if gain[j]>0{heap.push(Entry{score:score(j,gain[j]),j,gain:gain[j]});}}
 let mut selected=vec![false;n];let mut counts=vec![0usize;m.rows.len()];let mut remaining=m.rows.len();
 while remaining>0&&Instant::now()<deadline&&stats.work<100_000_000{
  let Some(e)=heap.pop()else{break;};if gain[e.j]==0||selected[e.j]{continue;}
  if e.gain!=gain[e.j]{heap.push(Entry{score:score(e.j,gain[e.j]),j:e.j,gain:gain[e.j]});continue;}
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

/// Add one column and greedily remove columns that become redundant.
/// Every accepted exchange is feasible and strictly reduces the true cost.
pub fn exchange(m: &Model, repair: &Repair<'_>, x: &[f64], deadline: Instant, budget: usize)
    -> Option<(Vec<f64>, BoxStats)>
{
    let began = Instant::now();
    let n = m.variables.len();
    if began >= deadline || budget == 0 || n > 100_000 || m.rows.len() > 100_000
        || m.rows.iter().fold(0usize,|sum,row|sum.saturating_add(row.coefficients.len())) > 10_000_000
        || x.len() != n || m.maximize
        || !m.variables.iter().all(|v| v.integer && v.lower == Some(0.) && v.upper == Some(1.) && v.objective.is_finite() && v.objective >= 0.)
        || !m.rows.iter().all(|r| r.lower.is_some_and(|b| b > 0. && b <= 1.) && r.upper.is_none() && r.coefficients.iter().all(|&(j,a)| j < n && a == 1.))
        || x.iter().any(|&v| v != 0. && v != 1.) || !repair.candidate_check(x)
    { return None; }
    let mut stats = BoxStats { calls: 1, ..BoxStats::default() };
    let mut incidence = vec![Vec::new(); n];
    let mut counts = vec![0usize; m.rows.len()];
    let mut selected: Vec<_> = x.iter().map(|&v| v == 1.).collect();
    for (i,row) in m.rows.iter().enumerate() {
        if Instant::now()>=deadline {return None;}
        for &(j,_) in &row.coefficients { incidence[j].push(i); counts[i] += usize::from(selected[j]); stats.work += 1; }
    }
    let mut chosen: Vec<_> = (0..n).filter(|&j| selected[j]).collect();
    chosen.sort_by(|&j,&k| m.variables[k].objective.total_cmp(&m.variables[j].objective).then(j.cmp(&k)));
    for j in chosen {
        if Instant::now() >= deadline || stats.work >= budget { break; }
        stats.work += incidence[j].len();
        if incidence[j].iter().all(|&i| counts[i] > 1) {
            selected[j] = false; stats.moves += 1;
            for &i in &incidence[j] { counts[i] -= 1; }
        }
    }
    let mut covered = vec![0usize; n];
    for _ in 0..128 {
        if Instant::now() >= deadline || stats.work >= budget { break; }
        let mut unique = vec![0usize; n];
        let mut owner = vec![usize::MAX; m.rows.len()];
        for (i,row) in m.rows.iter().enumerate() {
            stats.work += row.coefficients.len();
            if counts[i] == 1 {
                let j = row.coefficients.iter().find(|&&(j,_)| selected[j]).unwrap().0;
                unique[j] += 1; owner[i] = j;
            }
        }
        let mut improved = false;
        for j in 0..n {
            if selected[j] { continue; }
            if Instant::now() >= deadline || stats.work >= budget { break; }
            let mut touched = Vec::new();
            for &i in &incidence[j] {
                let k = owner[i];
                if k != usize::MAX { if covered[k] == 0 { touched.push(k); } covered[k] += 1; }
                stats.work += 1;
            }
            let mut removable: Vec<_> = touched.iter().copied().filter(|&k| covered[k] == unique[k]).collect();
            for k in touched { covered[k] = 0; }
            let saving: f64 = removable.iter().map(|&k| m.variables[k].objective).sum();
            if saving <= m.variables[j].objective { continue; }
            removable.sort_by(|&k,&l| m.variables[l].objective.total_cmp(&m.variables[k].objective).then(k.cmp(&l)));
            for &i in &incidence[j] { counts[i] += 1; }
            let mut removed = Vec::new();
            for k in removable {
                if Instant::now() >= deadline || stats.work >= budget { break; }
                stats.work += incidence[k].len();
                if incidence[k].iter().all(|&i| counts[i] > 1) {
                    for &i in &incidence[k] { counts[i] -= 1; }
                    removed.push(k);
                }
            }
            let saving: f64 = removed.iter().map(|&k| m.variables[k].objective).sum();
            let cost = m.variables[j].objective;
            if saving - cost > 4e-8 * (1. + saving.abs().max(cost.abs())) {
                selected[j] = true;
                for &k in &removed { selected[k] = false; }
                stats.moves += 1 + removed.len(); stats.improvements += 1; improved = true;
                break;
            }
            for k in removed { for &i in &incidence[k] { counts[i] += 1; } }
            for &i in &incidence[j] { counts[i] -= 1; }
        }
        if !improved { break; }
    }
    let result: Vec<_> = selected.iter().map(|&v| if v {1.} else {0.}).collect();
    stats.seconds = began.elapsed().as_secs_f64();
    if !repair.candidate_check(&result) || !repair.objective_reliable(&result,m.objective(&result))
        || m.objective(&result) > m.objective(x) { return None; }
    stats.candidates = 1;
    Some((result,stats))
}
