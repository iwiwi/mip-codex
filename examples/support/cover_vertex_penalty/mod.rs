//! Independent implementation inspired by Pullan & Hoos, JAIR25 (2006), doi:10.1613/jair.1815. No implementation code copied. No target-size stopping.
use crate::model::Model;
use std::time::Instant;
struct Graph {
    n: usize,
    words: usize,
    compatible: Vec<Vec<u64>>,
    order: Vec<usize>,
}
fn endpoint(
    model: &Model,
    value: Option<f64>,
    enclosure: Option<[f64; 2]>,
    upper: bool,
) -> Option<f64> {
    let value = value?;
    if !value.is_finite() {
        return None;
    }
    if !model.original_decimal {
        return Some(value);
    }
    let b = enclosure?;
    (b[0].is_finite() && b[1].is_finite() && b[0] <= value && value <= b[1])
        .then_some(b[usize::from(upper)])
}
fn row_vertices(model: &Model, row: &crate::model::Row) -> Option<Vec<usize>> {
    if row.upper.is_none() && row.lower.is_none() {
        return Some(vec![]);
    }
    let sign = if row.coefficients.iter().any(|p| p.1 < 0.)
        && row.coefficients.iter().all(|p| p.1 <= 0.)
    {
        -1.
    } else {
        1.
    };
    let (lower, upper, lower_enclosure, upper_enclosure) = if sign < 0. {
        (
            row.upper.map(|v| -v),
            row.lower.map(|v| -v),
            row.upper_enclosure.map(|[l, u]| [-u, -l]),
            row.lower_enclosure.map(|[l, u]| [-u, -l]),
        )
    } else {
        (
            row.lower,
            row.upper,
            row.lower_enclosure,
            row.upper_enclosure,
        )
    };
    if lower.is_some() && endpoint(model, lower, lower_enclosure, true)? > 0. {
        return None;
    }
    if model.original_decimal && !row.exact_coefficients {
        return None;
    }
    let mut vertices = Vec::new();
    let mut smallest = [f64::INFINITY; 2];
    let mut largest = 0f64;
    for (k, &(j, a)) in row.coefficients.iter().enumerate() {
        let a = sign * a;
        if j >= model.variables.len()
            || !a.is_finite()
            || a < 0.
            || a.fract() != 0.
            || a > 2f64.powi(52)
        {
            return None;
        }
        if a == 0. {
            continue;
        }
        if model.original_decimal
            && !(row.exact_integral
                || row.exact_coefficients
                || row.exact_coefficient_flags.get(k) == Some(&true))
        {
            return None;
        }
        if vertices.last().is_some_and(|&v| v >= j) {
            return None;
        }
        vertices.push(j);
        largest = largest.max(a);
        if a < smallest[0] {
            smallest[1] = smallest[0];
            smallest[0] = a
        } else if a < smallest[1] {
            smallest[1] = a
        }
    }
    if upper.is_none() {
        return Some(vec![]);
    }
    let lo = endpoint(model, upper, upper_enclosure, false)?;
    let hi = endpoint(model, upper, upper_enclosure, true)?;
    if lo < largest {
        return None;
    }
    if vertices.len() < 2 {
        return Some(vec![]);
    }
    (hi < smallest[0] + smallest[1]).then_some(vertices)
}
fn recognize(model: &Model, memory: usize, deadline: Instant) -> Option<Graph> {
    let n = model.variables.len();
    if n == 0 || n > 65535 {
        return None;
    }
    let sign = if model.maximize { -1. } else { 1. };
    let cost = model.variables[0].objective;
    if sign * cost >= 0. || !cost.is_finite() || cost.fract() != 0. || cost.abs() > 2f64.powi(52) {
        return None;
    }
    if model.original_decimal && model.offset_enclosure != Some([model.offset, model.offset]) {
        return None;
    }
    for v in &model.variables {
        if !v.integer || v.lower != Some(0.) || v.upper != Some(1.) || v.objective != cost {
            return None;
        }
        if model.original_decimal
            && (v.lower_enclosure != Some([0., 0.])
                || v.upper_enclosure != Some([1., 1.])
                || v.objective_enclosure != Some([cost, cost]))
        {
            return None;
        }
    }
    if !model.objective(&vec![0.; n]).is_finite() || !model.objective(&vec![1.; n]).is_finite() {
        return None;
    }
    let words = n.div_ceil(64);
    if n as u128 * words as u128 * 16 > memory as u128 / 2 {
        return None;
    }
    let mut pairs = 0usize;
    for row in &model.rows {
        if Instant::now() >= deadline {
            return None;
        }
        let v = row_vertices(model, row)?;
        pairs = pairs.saturating_add(v.len().saturating_mul(v.len().saturating_sub(1)) / 2);
        if pairs > 10_000_000 {
            return None;
        }
    }
    let mut conflict = vec![vec![0u64; words]; n];
    for row in &model.rows {
        if Instant::now() >= deadline {
            return None;
        }
        let v = row_vertices(model, row)?;
        for a in 0..v.len() {
            for b in a + 1..v.len() {
                conflict[v[a]][v[b] / 64] |= 1 << (v[b] % 64);
                conflict[v[b]][v[a] / 64] |= 1 << (v[a] % 64);
            }
        }
    }
    let mut order: Vec<_> = (0..n).collect();
    order.sort_unstable_by_key(|&j| (conflict[j].iter().map(|v| v.count_ones()).sum::<u32>(), j));
    let mut compatible = vec![vec![0u64; words]; n];
    for (a, &i) in order.iter().enumerate() {
        if Instant::now() >= deadline {
            return None;
        }
        for (b, &j) in order.iter().enumerate() {
            if a != b && conflict[i][j / 64] & (1 << (j % 64)) == 0 {
                compatible[a][b / 64] |= 1 << (b % 64);
            }
        }
    }
    Some(Graph {
        n,
        words,
        compatible,
        order,
    })
}
const NONE:usize=usize::MAX;
struct Bucket { lists:[Vec<usize>;11],pos:Vec<usize> }
impl Bucket{
 fn new(n:usize)->Self{Self{lists:std::array::from_fn(|_|Vec::new()),pos:vec![NONE;n]}}
 fn clear(&mut self){for v in &mut self.lists{v.clear();}self.pos.fill(NONE);}
 fn insert(&mut self,v:usize,p:u64){assert!(p<=10);assert_eq!(self.pos[v],NONE);self.pos[v]=self.lists[p as usize].len();self.lists[p as usize].push(v);}
 fn remove(&mut self,v:usize,p:u64){let k=self.pos[v];if k==NONE{return;}let list=&mut self.lists[p as usize];list.swap_remove(k);self.pos[v]=NONE;if k<list.len(){self.pos[list[k]]=k;}}
 fn pick(&self,rng:&mut Rng)->Option<usize>{self.lists.iter().find(|b|!b.is_empty()).map(|b|b[rng.index(b.len())])}
}
struct Rng(u64);
impl Rng{
 fn next(&mut self)->u64{self.0^=self.0<<13;self.0^=self.0>>7;self.0^=self.0<<17;self.0}
 fn index(&mut self,n:usize)->usize{assert!(n>0);let n=n as u64;let zone=u64::MAX-u64::MAX%n;loop{let x=self.next();if x<zone{return(x%n)as usize;}}}
}
struct State{
 neighbors:Vec<Vec<usize>>,selected:Vec<usize>,position:Vec<usize>,conflicts:Vec<usize>,blocked:Vec<bool>,penalty:Vec<u64>,zero:Bucket,one:Bucket,maintain:bool,
}
impl State{
 fn from_neighbors(neighbors:Vec<Vec<usize>>,seed:&[usize])->Self{
  let n=neighbors.len();let mut s=Self{neighbors,selected:vec![],position:vec![NONE;n],conflicts:vec![0;n],blocked:vec![false;n],penalty:vec![0;n],zero:Bucket::new(n),one:Bucket::new(n),maintain:false};
  for &v in seed{s.add(v);}s.rebuild();s
 }
 fn has(&self,v:usize)->bool{self.position[v]!=NONE}
 fn refresh(&mut self,v:usize){if !self.maintain{return;}let p=self.penalty[v];self.zero.remove(v,p);self.one.remove(v,p);if self.has(v)||self.blocked[v]||p>10{return;}match self.conflicts[v]{0=>self.zero.insert(v,p),1=>self.one.insert(v,p),_=>{}}}
 fn rebuild(&mut self){self.zero.clear();self.one.clear();self.maintain=true;for v in 0..self.neighbors.len(){self.refresh(v);}}
 fn add(&mut self,v:usize){assert!(!self.has(v)&&self.conflicts[v]==0);self.position[v]=self.selected.len();self.selected.push(v);self.blocked[v]=true;self.refresh(v);for k in 0..self.neighbors[v].len(){let u=self.neighbors[v][k];self.conflicts[u]+=1;self.refresh(u);}}
 fn remove(&mut self,v:usize){let k=self.position[v];assert_ne!(k,NONE);self.selected.swap_remove(k);self.position[v]=NONE;if k<self.selected.len(){self.position[self.selected[k]]=k;}self.refresh(v);for k in 0..self.neighbors[v].len(){let u=self.neighbors[v][k];self.conflicts[u]-=1;self.refresh(u);}}
 fn swap(&mut self,v:usize)->usize{assert!(!self.has(v)&&self.conflicts[v]==1);let u=*self.neighbors[v].iter().find(|&&u|self.has(u)).unwrap();self.remove(u);self.add(v);u}
 fn perturb(&mut self,last:usize,pd:usize,cycle:usize,rng:&mut Rng){
  self.maintain=false;
  for &v in &self.selected{self.penalty[v]=self.penalty[v].checked_add(1).unwrap();}
  if cycle%pd==0{for p in &mut self.penalty{*p=p.saturating_sub(1);}}
  if pd>1{let others:Vec<_>=self.selected.iter().copied().filter(|&v|v!=last).collect();for v in others{self.remove(v);}assert!(self.has(last));}
  else{let v=rng.index(self.neighbors.len());let others:Vec<_>=self.neighbors[v].iter().copied().filter(|&u|self.has(u)).collect();for u in others{self.remove(u);}if !self.has(v){self.add(v);}}
  self.blocked.fill(false);for &v in &self.selected{self.blocked[v]=true;}self.rebuild();
 }
 #[cfg(test)]fn verify(&self){
  for v in 0..self.neighbors.len(){assert_eq!(self.conflicts[v],self.neighbors[v].iter().filter(|&&u|self.has(u)).count());assert!(!self.has(v)||self.conflicts[v]==0);assert_eq!(self.has(v),self.selected.contains(&v));if self.has(v){assert_eq!(self.selected[self.position[v]],v);}let eligible=!self.has(v)&&!self.blocked[v]&&self.penalty[v]<=10;assert_eq!(self.zero.pos[v]!=NONE,eligible&&self.conflicts[v]==0);assert_eq!(self.one.pos[v]!=NONE,eligible&&self.conflicts[v]==1);}
  for pool in [&self.zero,&self.one]{for(p,b)in pool.lists.iter().enumerate(){for(k,&v)in b.iter().enumerate(){assert_eq!(pool.pos[v],k);assert_eq!(self.penalty[v]as usize,p);}}}
 }
}
#[derive(Default,Debug,serde::Serialize)]pub struct PenaltyStats{initial:usize,best:usize,steps:usize,expansions:usize,swaps:usize,cycles:usize,improvements:usize,first_improvement_step:Option<usize>,last_improvement_step:usize,penalty_delay:usize,seconds:f64,stop_reason:&'static str}
fn search(neighbors:Vec<Vec<usize>>,seed:&[usize],pd:usize,max_steps:usize,deadline:Instant,rng_seed:u64)->(Vec<usize>,PenaltyStats){
 assert!(pd>0&&rng_seed!=0);let start=Instant::now();let n=neighbors.len();assert!(n>0);let mut rng=Rng(rng_seed);let mut state=State::from_neighbors(neighbors,seed);let mut best=seed.to_vec();let mut last=seed.first().copied().unwrap_or_else(||{let v=rng.index(n);state.add(v);v});if state.selected.len()>best.len(){best=state.selected.clone();}let mut stats=PenaltyStats{initial:seed.len(),best:best.len(),penalty_delay:pd,..PenaltyStats::default()};let mut recorded=vec![false;n];
 let max_cycles=max_steps.saturating_add(1);
 while stats.steps<max_steps&&stats.cycles<max_cycles&&Instant::now()<deadline{
  while let Some(v)=state.zero.pick(&mut rng){if stats.steps>=max_steps||Instant::now()>=deadline{break;}state.add(v);last=v;stats.steps+=1;stats.expansions+=1;if state.selected.len()>best.len(){best=state.selected.clone();stats.best=best.len();stats.improvements+=1;stats.first_improvement_step.get_or_insert(stats.steps);stats.last_improvement_step=stats.steps;}}
  recorded.fill(false);for &v in &state.selected{recorded[v]=true;}let mut overlap=state.selected.len();
  loop{
   if stats.steps>=max_steps||Instant::now()>=deadline{break;}
   if let Some(v)=state.zero.pick(&mut rng){state.add(v);last=v;stats.steps+=1;stats.expansions+=1;if state.selected.len()>best.len(){best=state.selected.clone();stats.best=best.len();stats.improvements+=1;stats.first_improvement_step.get_or_insert(stats.steps);stats.last_improvement_step=stats.steps;}continue;}
   if overlap==0{break;}let Some(v)=state.one.pick(&mut rng)else{break;};let u=state.swap(v);if recorded[u]{recorded[u]=false;overlap-=1;}last=v;stats.steps+=1;stats.swaps+=1;
  }
  if stats.steps>=max_steps||Instant::now()>=deadline{break;}stats.cycles+=1;state.perturb(last,pd,stats.cycles,&mut rng);last=*state.selected.first().unwrap();
 }
 stats.stop_reason=if stats.steps>=max_steps{"step_limit"}else if stats.cycles>=max_cycles{"cycle_limit"}else{"deadline"};stats.seconds=start.elapsed().as_secs_f64();(best,stats)
}
pub fn diagnostic(model:&Model,seconds:f64,pd:usize,seed:Option<&[f64]>,rng_seed:u64,max_steps:usize)->serde_json::Value{
 assert!(seconds.is_finite()&&seconds>=0.&&seconds<=86400.&&max_steps<=100_000_000);let start=Instant::now();let deadline=start+std::time::Duration::from_secs_f64(seconds);let graph=recognize(model,512*1024*1024,deadline).expect("recognized packing");let mut initial=vec![];
 if let Some(point)=seed{assert!(model.check(point,true,0.).is_ok());assert!(point.iter().all(|&x|x==0.||x==1.));initial=(0..graph.n).filter(|&v|point[graph.order[v]]==1.).collect();}
 let mut neighbors=vec![vec![];graph.n];for a in 0..graph.n{for b in a+1..graph.n{if graph.compatible[a][b/64]&(1u64<<(b%64))==0{neighbors[a].push(b);neighbors[b].push(a);}}}
 let (best,stats)=search(neighbors,&initial,pd,max_steps,deadline,rng_seed);assert!(best.iter().enumerate().all(|(i,&a)|best[i+1..].iter().all(|&b|a!=b&&graph.compatible[a][b/64]&(1u64<<(b%64))!=0)));let mut point=vec![0.;model.variables.len()];for &v in &best{point[graph.order[v]]=1.;}assert!(model.check(&point,true,0.).is_ok());let values:std::collections::BTreeMap<_,_>=model.variables.iter().zip(&point).map(|(v,&x)|(v.name.clone(),x)).collect();serde_json::json!({"status":"primal_candidate","objective":model.objective(&point),"dual_bound":null,"values":values,"stats":stats,"component_seconds":start.elapsed().as_secs_f64(),"purpose":"independent Rust vertex-penalty diagnostic, no global bound/optimality claim"})
}
#[cfg(test)]mod tests{
 use super::*;
 fn random_graph(n:usize,rng:&mut Rng)->Vec<Vec<usize>>{let mut a=vec![vec![];n];for i in 0..n{for j in i+1..n{if rng.next()&1!=0{a[i].push(j);a[j].push(i);}}}a}
 #[test]fn buckets_remove_middle_and_choose_minimum_penalty(){let mut b=Bucket::new(5);b.insert(0,2);b.insert(1,2);b.insert(2,0);b.insert(3,2);b.remove(1,2);assert_eq!(b.pos[3],1);let mut r=Rng(1);for _ in 0..100{assert_eq!(b.pick(&mut r),Some(2));}b.remove(2,0);for _ in 0..100{assert!([0,3].contains(&b.pick(&mut r).unwrap()));}}
 #[test]fn incremental_counts_buckets_and_penalties_match_recomputation(){let mut rng=Rng(0x36dfad6a591287bb);for _ in 0..100{let n=3+rng.index(18);let mut s=State::from_neighbors(random_graph(n,&mut rng),&[]);s.add(0);for cycle in 1..100{for _ in 0..20{s.verify();if let Some(v)=s.zero.pick(&mut rng){s.add(v);}else if let Some(v)=s.one.pick(&mut rng){s.swap(v);}else{break;}}s.verify();let last=*s.selected.last().unwrap();s.perturb(last,[1,5,10,20][cycle%4],cycle,&mut rng);s.verify();}}}
 #[test]fn all_small_outputs_respect_exhaustive_optima_and_keep_seed(){let mut rng=Rng(0x4c2309dcfb659187);for _ in 0..200{let n=3+rng.index(9);let a=random_graph(n,&mut rng);let optimum=(0usize..1<<n).filter(|&m|(0..n).all(|i|m&(1<<i)==0||a[i].iter().all(|&j|m&(1<<j)==0))).map(|m|m.count_ones()as usize).max().unwrap();for pd in [1,5,10,20]{let(seed,stats)=search(a.clone(),&[0],pd,1000,Instant::now()+std::time::Duration::from_secs(1),rng.next()|1);assert!(seed.len()>=1&&seed.len()<=optimum);assert_eq!(seed.len(),stats.best);assert!(seed.iter().enumerate().all(|(k,&v)|seed[k+1..].iter().all(|u|!a[v].contains(u))));}}}
 #[test]fn retained_singleton_can_exceed_selection_penalty_and_still_terminates(){let mut s=State::from_neighbors(vec![vec![]],&[0]);let mut rng=Rng(1);for cycle in 1..=100{s.perturb(0,20,cycle,&mut rng);s.verify();}assert!(s.penalty[0]>11);let(best,stats)=search(vec![vec![]],&[0],20,100,Instant::now()+std::time::Duration::from_secs(1),1);assert_eq!(best,vec![0]);assert_eq!(stats.steps,0);assert_eq!(stats.cycles,101);assert_eq!(stats.stop_reason,"cycle_limit");}
 #[test]fn expired_deadline_keeps_valid_seed(){let(a,s)=search(vec![vec![1],vec![0],vec![]],&[0,2],5,1000,Instant::now(),1);assert_eq!(a,vec![0,2]);assert_eq!(s.steps,0);}
}
