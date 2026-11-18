from pathlib import Path
import subprocess
assert not any(x.strip()in['cargo','rustc']for x in subprocess.check_output(['ps','-eo','comm='],text=True).splitlines())
changes=[]
def replace(path,a,b):
 p=Path(path);s=next((s for q,s in changes if q==p),p.read_text());assert s.count(a)==1,(path,a[:90]);s=s.replace(a,b);changes[:]=[(q,t)for q,t in changes if q!=p];changes.append((p,s))
original=Path('/home/ubuntu/mip-cover-choice-heap/examples/support/cover_vertex_penalty/mod.rs').read_text();a=original.index('const NONE:');b=original.index('pub fn diagnostic(',a);tests=original[original.index('#[cfg(test)]mod tests{',b):];module='''//! Independent vertex-penalty primal search inspired by Pullan & Hoos, JAIR25 (2006), doi:10.1613/jair.1815. No source-code port and no target-size stopping.
use super::{Graph,strict_clique};
use std::time::Instant;
'''+original[a:b]+tests
module=module.replace('PenaltyStats','Stats').replace('pub struct Stats{','pub struct Stats{pub used:bool,pub eligible:bool,pub validation_failures:usize,')
module+='''
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
'''
changes.append((Path('src/binary_graph/vertex_penalty.rs'),module))
replace('src/binary_graph.rs','mod weighted_cover;','mod weighted_cover;\nmod vertex_penalty;\npub use vertex_penalty::Stats as VertexStats;')
replace('src/binary_graph.rs','    pub cover_local: Option<CoverStats>,','    pub cover_local: Option<CoverStats>,\n    pub vertex_local: Option<VertexStats>,')
a='''pub fn solve_with_rounds(
    model: &Model, memory: usize, deadline: Instant, limit: usize,
    local: bool, stronger_root: bool, preserve_order: bool, diversify: bool,
    adaptive: bool, cover: bool, fractional: bool, portfolio: bool,
    input_cover: bool, spectral: bool, large: bool, heap: bool, primal_budget: bool, long_rounds: bool,
) -> Option<Answer> {'''
b=a+'''
    solve_with_vertex(model,memory,deadline,limit,local,stronger_root,preserve_order,diversify,adaptive,cover,fractional,portfolio,input_cover,spectral,large,heap,primal_budget,long_rounds,false)
}
pub fn solve_with_vertex(
    model: &Model, memory: usize, deadline: Instant, limit: usize,
    local: bool, stronger_root: bool, preserve_order: bool, diversify: bool,
    adaptive: bool, cover: bool, fractional: bool, portfolio: bool,
    input_cover: bool, spectral: bool, large: bool, heap: bool, primal_budget: bool, long_rounds: bool, vertex:bool,
) -> Option<Answer> {'''
replace('src/binary_graph.rs',a,b)
a='''        if cover && Instant::now() < local_deadline {
            let (seed, stats)=weighted_cover::selected_with_rounds(&graph,&search.best,memory/2,local_deadline,portfolio,heap,long_rounds);
            search.best = seed;
            search.stats.cover_local = Some(stats);
        }'''
b='''        let vertex_allowed=vertex&&cover&&portfolio&&heap&&weighted_cover::heap_eligible(graph.n,search.stats.conflict_edges,memory/2)&&vertex_penalty::eligible(graph.n,search.stats.conflict_edges,memory/2);
        let cover_deadline=if vertex_allowed{let now=Instant::now();now+local_deadline.saturating_duration_since(now)/2}else{local_deadline};
        if cover && Instant::now() < cover_deadline {
            let (seed, stats)=weighted_cover::selected_with_rounds(&graph,&search.best,memory/2,cover_deadline,portfolio,heap,long_rounds);
            search.best = seed;
            search.stats.cover_local = Some(stats);
        }
        if vertex_allowed&&Instant::now()<local_deadline{
            let(seed,stats)=vertex_penalty::improve(&graph,&search.best,search.stats.conflict_edges,memory/2,local_deadline);search.best=seed;search.stats.vertex_local=Some(stats);
        }'''
replace('src/binary_graph.rs',a,b)
replace('src/solve.rs','    pub graph_cover_long_rounds:bool,','    pub graph_cover_long_rounds:bool,\n    pub graph_vertex_portfolio:bool,')
replace('src/solve.rs','            graph_cover_long_rounds:true,','            graph_cover_long_rounds:true,\n            graph_vertex_portfolio:false,')
replace('src/solve.rs','crate::binary_graph::solve_with_rounds(','crate::binary_graph::solve_with_vertex(')
replace('src/solve.rs','                limits.graph_cover_long_rounds,','                limits.graph_cover_long_rounds,\n                limits.graph_vertex_portfolio,')
a='''        if let Ok(v)=std::env::var("MIP_GRAPH_COVER_LONG_ROUNDS"){limits.graph_cover_long_rounds=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_GRAPH_COVER_LONG_ROUNDS must be 0 or 1".into())};}'''
replace('src/main.rs',a,a+'\n        if let Ok(v)=std::env::var("MIP_GRAPH_VERTEX_PORTFOLIO"){limits.graph_vertex_portfolio=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_GRAPH_VERTEX_PORTFOLIO must be 0 or 1".into())};}')
for p,s in changes:p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s)
print('ported',[(str(p),len(s))for p,s in changes])
