from pathlib import Path
import subprocess
root=Path.cwd();assert root==Path('/home/ubuntu/mip-graph-cover-long-rounds')
for n in ['cargo','rustc']:assert subprocess.run(['pgrep','-x',n],capture_output=True).returncode==1
edits={}
def change(text,old,new):
 assert text.count(old)==1,old[:100]
 return text.replace(old,new)
p=root/'src/binary_graph/weighted_cover.rs';s=p.read_text();s=change(s,'    pub heap_used:bool,','    pub heap_used:bool,\n    pub long_rounds_used:bool,')
a='fn improve_seeded(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant,initial_rng:u64,minimum_iterations:usize,heap:bool)->(Vec<usize>,Stats){'
s=change(s,a,a+'\n improve_seeded_with_rounds(graph,seed,memory,deadline,initial_rng,minimum_iterations,heap,false)\n}\nfn improve_seeded_with_rounds(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant,initial_rng:u64,minimum_iterations:usize,heap:bool,long_rounds:bool)->(Vec<usize>,Stats){')
a='''    stats.heap_used=state.heap_enabled;
    stats.edges = state.edges.len();
    let mut rng = initial_rng;
    let extended_limit = graph.n.saturating_mul(1024).min(2_000_000);'''
b='''    stats.heap_used=state.heap_enabled;
    // 大規模疎heapで、改善後の状態遷移を長く継続する。64bitでカウンタ範囲を保証する。
    stats.long_rounds_used=long_rounds && state.heap_enabled && usize::BITS>=64;
    let extended_limit=if stats.long_rounds_used {100_000_000}else{graph.n.saturating_mul(1024).min(2_000_000)};
    stats.iteration_limit=graph.n.saturating_mul(128).min(500000).max(minimum_iterations).min(extended_limit);
    stats.edges = state.edges.len();
    let mut rng = initial_rng;'''
s=change(s,a,b)
a='''pub(super) fn selected(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant,portfolio:bool,heap:bool)->(Vec<usize>,Stats){
 if portfolio {portfolio_selected(graph,seed,memory,deadline,heap)}else{improve_seeded(graph,seed,memory,deadline,0xd1b54a32d192ed03,0,heap)}
}'''
b='''pub(super) fn selected(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant,portfolio:bool,heap:bool)->(Vec<usize>,Stats){
 selected_with_rounds(graph,seed,memory,deadline,portfolio,heap,false)
}
pub(super) fn selected_with_rounds(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant,portfolio:bool,heap:bool,long_rounds:bool)->(Vec<usize>,Stats){
 if portfolio {portfolio_selected_with_rounds(graph,seed,memory,deadline,heap,long_rounds)}else{improve_seeded_with_rounds(graph,seed,memory,deadline,0xd1b54a32d192ed03,0,heap,long_rounds)}
}'''
s=change(s,a,b)
a='''fn portfolio_selected(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant,heap:bool)->(Vec<usize>,Stats){
 let began=Instant::now();let(mut best,mut stats)=improve_seeded(graph,seed,memory,deadline,0xd1b54a32d192ed03,0,heap);'''
b='''fn portfolio_selected(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant,heap:bool)->(Vec<usize>,Stats){
 portfolio_selected_with_rounds(graph,seed,memory,deadline,heap,false)
}
fn portfolio_selected_with_rounds(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant,heap:bool,long_rounds:bool)->(Vec<usize>,Stats){
 let began=Instant::now();let(mut best,mut stats)=improve_seeded_with_rounds(graph,seed,memory,deadline,0xd1b54a32d192ed03,0,heap,long_rounds);'''
s=change(s,a,b)
a='''  let(next,call)=improve_seeded(graph,&best,memory,deadline,rng,graph.n.saturating_mul(1024).min(2_000_000),heap);
  stats.heap_used|=call.heap_used;'''
b='''  let minimum=if stats.long_rounds_used {100_000_000}else{graph.n.saturating_mul(1024).min(2_000_000)};
  let(next,call)=improve_seeded_with_rounds(graph,&best,memory,deadline,rng,minimum,heap,long_rounds);
  stats.heap_used|=call.heap_used;
  stats.long_rounds_used|=call.long_rounds_used;'''
s=change(s,a,b);edits[p]=s
p=root/'src/binary_graph.rs';s=p.read_text();a='''    let portfolio=portfolio&&adaptive&&cover;
    let started = Instant::now();'''
b='''    solve_with_rounds(model,memory,deadline,limit,local,stronger_root,preserve_order,diversify,adaptive,cover,fractional,portfolio,input_cover,spectral,large,heap,primal_budget,false)
}
pub fn solve_with_rounds(
    model: &Model, memory: usize, deadline: Instant, limit: usize,
    local: bool, stronger_root: bool, preserve_order: bool, diversify: bool,
    adaptive: bool, cover: bool, fractional: bool, portfolio: bool,
    input_cover: bool, spectral: bool, large: bool, heap: bool, primal_budget: bool, long_rounds: bool,
) -> Option<Answer> {
'''+a
s=change(s,a,b);s=change(s,'weighted_cover::selected(&graph,&search.best,memory/2,local_deadline,portfolio,heap)','weighted_cover::selected_with_rounds(&graph,&search.best,memory/2,local_deadline,portfolio,heap,long_rounds)');edits[p]=s
p=root/'src/solve.rs';s=p.read_text();s=change(s,'    pub graph_large_primal_budget:bool,','    pub graph_large_primal_budget:bool,\n    pub graph_cover_long_rounds:bool,');s=change(s,'            graph_large_primal_budget:true,','            graph_large_primal_budget:true,\n            graph_cover_long_rounds:false,');s=change(s,'crate::binary_graph::solve_with_primal_budget(','crate::binary_graph::solve_with_rounds(');s=change(s,'                limits.graph_large_primal_budget,','                limits.graph_large_primal_budget,\n                limits.graph_cover_long_rounds,');edits[p]=s
p=root/'src/main.rs';s=p.read_text();a='        if let Ok(v)=std::env::var("MIP_GRAPH_LARGE_PRIMAL_BUDGET")';assert s.count(a)==1;i=s.index(a);s=s[:i]+'''        if let Ok(v)=std::env::var("MIP_GRAPH_COVER_LONG_ROUNDS"){limits.graph_cover_long_rounds=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_GRAPH_COVER_LONG_ROUNDS must be 0 or 1".into())};}
'''+s[i:];edits[p]=s
for p,s in edits.items():p.write_text(s)
print('edited',len(edits),'Rust files; no source writes before all anchors validated')
