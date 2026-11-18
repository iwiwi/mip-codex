from pathlib import Path
import subprocess
root=Path.cwd();assert root==Path('/home/ubuntu/mip-graph-large-primal-budget')
for n in ['cargo','rustc']:assert subprocess.run(['pgrep','-x',n],capture_output=True).returncode==1
p=root/'src/binary_graph.rs';s=p.read_text();anchor='''    let portfolio=portfolio&&adaptive&&cover;
    let started = Instant::now();''';assert s.count(anchor)==1
wrapper='''    solve_with_primal_budget(model,memory,deadline,limit,local,stronger_root,preserve_order,diversify,adaptive,cover,fractional,portfolio,input_cover,spectral,large,heap,false)
}
pub fn solve_with_primal_budget(
    model: &Model, memory: usize, deadline: Instant, limit: usize,
    local: bool, stronger_root: bool, preserve_order: bool, diversify: bool,
    adaptive: bool, cover: bool, fractional: bool, portfolio: bool,
    input_cover: bool, spectral: bool, large: bool, heap: bool, primal_budget: bool,
) -> Option<Answer> {
'''+anchor
s=s.replace(anchor,wrapper);needle='''        let allocation = (deadline.saturating_duration_since(started).as_secs_f64() * 0.05).min(''';assert s.count(needle)==1
s=s.replace(needle,'''        // Spend more time on primal search only where the sparse large-graph heap is eligible.
        let fraction = if primal_budget && portfolio && heap
            && weighted_cover::heap_eligible(graph.n,search.stats.conflict_edges,memory/2) {0.25} else {0.05};
        let allocation = (deadline.saturating_duration_since(started).as_secs_f64() * fraction).min(''');p.write_text(s)
p=root/'src/binary_graph/weighted_cover.rs';s=p.read_text();assert s.count('fn heap_eligible(')==1;p.write_text(s.replace('fn heap_eligible(','pub(super) fn heap_eligible('))
p=root/'src/solve.rs';s=p.read_text().replace('pub graph_cover_heap:bool,','pub graph_cover_heap:bool,\n    pub graph_large_primal_budget:bool,').replace('graph_cover_heap:true,','graph_cover_heap:true,\n            graph_large_primal_budget:false,').replace('crate::binary_graph::solve_with_heap(','crate::binary_graph::solve_with_primal_budget(').replace('                limits.graph_cover_heap,','                limits.graph_cover_heap,\n                limits.graph_large_primal_budget,');p.write_text(s)
p=root/'src/main.rs';s=p.read_text();needle='        if let Ok(v)=std::env::var("MIP_GRAPH_COVER_HEAP")';i=s.index(needle);s=s[:i]+'''        if let Ok(v)=std::env::var("MIP_GRAPH_LARGE_PRIMAL_BUDGET"){limits.graph_large_primal_budget=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_GRAPH_LARGE_PRIMAL_BUDGET must be 0 or 1".into())};}
'''+s[i:];p.write_text(s)
