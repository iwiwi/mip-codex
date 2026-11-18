//! Independent vertex-penalty primal search inspired by Pullan & Hoos, JAIR25 (2006), doi:10.1613/jair.1815. No source-code port and no target-size stopping.
use super::{Graph,strict_clique};
use std::time::Instant;
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
#[derive(Default,Debug,serde::Serialize)]pub struct Stats{pub used:bool,pub eligible:bool,pub validation_failures:usize,initial:usize,best:usize,steps:usize,expansions:usize,swaps:usize,cycles:usize,improvements:usize,first_improvement_step:Option<usize>,last_improvement_step:usize,penalty_delay:usize,seconds:f64,stop_reason:&'static str}
fn search(neighbors:Vec<Vec<usize>>,seed:&[usize],pd:usize,max_steps:usize,deadline:Instant,rng_seed:u64)->(Vec<usize>,Stats){
 assert!(pd>0&&rng_seed!=0);let start=Instant::now();let n=neighbors.len();assert!(n>0);let mut rng=Rng(rng_seed);let mut state=State::from_neighbors(neighbors,seed);let mut best=seed.to_vec();let mut last=seed.first().copied().unwrap_or_else(||{let v=rng.index(n);state.add(v);v});if state.selected.len()>best.len(){best=state.selected.clone();}let mut stats=Stats{initial:seed.len(),best:best.len(),penalty_delay:pd,..Stats::default()};let mut recorded=vec![false;n];
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
#[cfg(test)]mod tests{
 use super::*;
 fn random_graph(n:usize,rng:&mut Rng)->Vec<Vec<usize>>{let mut a=vec![vec![];n];for i in 0..n{for j in i+1..n{if rng.next()&1!=0{a[i].push(j);a[j].push(i);}}}a}
 #[test]fn buckets_remove_middle_and_choose_minimum_penalty(){let mut b=Bucket::new(5);b.insert(0,2);b.insert(1,2);b.insert(2,0);b.insert(3,2);b.remove(1,2);assert_eq!(b.pos[3],1);let mut r=Rng(1);for _ in 0..100{assert_eq!(b.pick(&mut r),Some(2));}b.remove(2,0);for _ in 0..100{assert!([0,3].contains(&b.pick(&mut r).unwrap()));}}
 #[test]fn incremental_counts_buckets_and_penalties_match_recomputation(){let mut rng=Rng(0x36dfad6a591287bb);for _ in 0..100{let n=3+rng.index(18);let mut s=State::from_neighbors(random_graph(n,&mut rng),&[]);s.add(0);for cycle in 1..100{for _ in 0..20{s.verify();if let Some(v)=s.zero.pick(&mut rng){s.add(v);}else if let Some(v)=s.one.pick(&mut rng){s.swap(v);}else{break;}}s.verify();let last=*s.selected.last().unwrap();s.perturb(last,[1,5,10,20][cycle%4],cycle,&mut rng);s.verify();}}}
 #[test]fn all_small_outputs_respect_exhaustive_optima_and_keep_seed(){let mut rng=Rng(0x4c2309dcfb659187);for _ in 0..200{let n=3+rng.index(9);let a=random_graph(n,&mut rng);let optimum=(0usize..1<<n).filter(|&m|(0..n).all(|i|m&(1<<i)==0||a[i].iter().all(|&j|m&(1<<j)==0))).map(|m|m.count_ones()as usize).max().unwrap();for pd in [1,5,10,20]{let(seed,stats)=search(a.clone(),&[0],pd,1000,Instant::now()+std::time::Duration::from_secs(1),rng.next()|1);assert!(seed.len()>=1&&seed.len()<=optimum);assert_eq!(seed.len(),stats.best);assert!(seed.iter().enumerate().all(|(k,&v)|seed[k+1..].iter().all(|u|!a[v].contains(u))));}}}
 #[test]fn retained_singleton_can_exceed_selection_penalty_and_still_terminates(){let mut s=State::from_neighbors(vec![vec![]],&[0]);let mut rng=Rng(1);for cycle in 1..=100{s.perturb(0,20,cycle,&mut rng);s.verify();}assert!(s.penalty[0]>11);let(best,stats)=search(vec![vec![]],&[0],20,100,Instant::now()+std::time::Duration::from_secs(1),1);assert_eq!(best,vec![0]);assert_eq!(stats.steps,0);assert_eq!(stats.cycles,101);assert_eq!(stats.stop_reason,"cycle_limit");}
 #[test]fn expired_deadline_keeps_valid_seed(){let(a,s)=search(vec![vec![1],vec![0],vec![]],&[0,2],5,1000,Instant::now(),1);assert_eq!(a,vec![0,2]);assert_eq!(s.steps,0);}
}

pub(super) fn eligible(n:usize,edges:usize,memory:usize)->bool {
 usize::BITS==64 && n>4096 && n<=65535 && n.checked_mul(512).and_then(|v|edges.checked_mul(32).and_then(|e|v.checked_add(e))).is_some_and(|bytes|bytes<=memory)
}
fn neighbors(graph:&Graph,edges:usize,deadline:Instant)->Option<Vec<Vec<usize>>>{
 let mut result=vec![vec![];graph.n];let mut observed=0usize;
 for a in 0..graph.n{
  if Instant::now()>=deadline{return None;}
  for(k,&word)in graph.compatible[a].iter().enumerate(){
   if k<a/64{continue;}let mut bits=!word;
   if k==a/64{bits&=u64::MAX.checked_shl((a%64+1)as u32).unwrap_or(0);}
   if k+1==graph.words&&graph.n%64!=0{bits&=(1u64<<(graph.n%64))-1;}
   while bits!=0{let b=k*64+bits.trailing_zeros()as usize;bits&=bits-1;if b>=graph.n{return None;}observed+=1;if observed>edges{return None;}result[a].push(b);result[b].push(a);}
  }
 }
 (observed==edges).then_some(result)
}
pub(super) fn improve(graph:&Graph,seed:&[usize],edges:usize,memory:usize,deadline:Instant)->(Vec<usize>,Stats){
 let start=Instant::now();let mut stats=Stats{initial:seed.len(),best:seed.len(),eligible:eligible(graph.n,edges,memory),penalty_delay:1,..Stats::default()};
 if !strict_clique(graph,seed){stats.validation_failures=1;stats.best=0;stats.stop_reason="invalid_seed";return(vec![],stats);}
 if !stats.eligible||Instant::now()>=deadline{stats.stop_reason="ineligible_or_deadline";return(seed.to_vec(),stats);}
 let Some(adj)=neighbors(graph,edges,deadline)else{stats.stop_reason="construction_limit";stats.seconds=start.elapsed().as_secs_f64();return(seed.to_vec(),stats);};
 let(best,mut stats)=search(adj,seed,1,100_000_000,deadline,149372736029117923);stats.used=true;stats.eligible=true;
 let best=if strict_clique(graph,&best){best}else{stats.validation_failures+=1;stats.best=seed.len();stats.stop_reason="invalid_candidate";seed.to_vec()};stats.seconds=start.elapsed().as_secs_f64();(best,stats)
}
#[cfg(test)]mod wrapper_tests{
 use super::*;
 #[test]fn operation_memory_gate_is_checked(){if usize::BITS==64{assert!(eligible(5000,2500,5000*512+2500*32));assert!(!eligible(5000,2500,5000*512+2500*32-1));}assert!(!eligible(4096,1,usize::MAX));assert!(!eligible(65536,1,usize::MAX));assert!(!eligible(5000,usize::MAX,usize::MAX));}
 #[test]fn complement_neighbor_construction_matches_random_graphs(){let mut rng=Rng(0x56bc1239ae17d383);for n in (1usize..30).chain([63,64,65,127,128,129]){let mut rows=vec![vec![0u64;n.div_ceil(64)];n];let mut expected=vec![vec![];n];let mut edges=0;for a in 0..n{for b in a+1..n{if rng.next()&1==0{rows[a][b/64]|=1u64<<(b%64);rows[b][a/64]|=1u64<<(a%64);}else{expected[a].push(b);expected[b].push(a);edges+=1;}}}let g=Graph{n,words:n.div_ceil(64),compatible:rows,order:(0..n).collect()};assert_eq!(neighbors(&g,edges,Instant::now()+std::time::Duration::from_secs(1)).unwrap(),expected);assert!(neighbors(&g,edges+1,Instant::now()+std::time::Duration::from_secs(1)).is_none());assert!(neighbors(&g,edges,Instant::now()).is_none());}}
}
