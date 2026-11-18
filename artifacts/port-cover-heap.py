from pathlib import Path
import subprocess,json
for name in ['cargo','rustc']:
    assert subprocess.run(['pgrep','-x',name],capture_output=True).returncode==1

def once(s,old,new):
    assert s.count(old)==1,old
    return s.replace(old,new,1)

p=Path('src/binary_graph/weighted_cover.rs')
s=p.read_text()
private=Path('/home/ubuntu/mip-cover-choice-heap/examples/support/cover_heap_profile/weighted_cover.rs').read_text()
scan=s[s.index('    fn remove_choice(&self)'):s.index('    fn raise_weights(')]
block=private[private.index('struct State {'):private.index('pub(super) fn improve(')]
block=once(block,'    heap: Vec<usize>,','    heap_enabled: bool,\n    heap: Vec<usize>,')
block=once(block,'            heap: Vec::with_capacity(graph.n),\n            heap_position: vec![usize::MAX; graph.n],','            heap_enabled: false,\n            heap: Vec::new(),\n            heap_position: Vec::new(),')
a=block.index('        self.heap.clear();')
b=block.index('    fn heap_swap(',a)
block=block[:a]+'''        if self.heap_enabled {self.heap_rebuild();}
    }
    fn heap_rebuild(&mut self) {
        self.heap.clear();self.heap_position.fill(usize::MAX);
        for v in 0..self.covered.len(){if self.covered[v]{self.heap_position[v]=self.heap.len();self.heap.push(v);}}
        for k in (0..self.heap.len()/2).rev(){self.heap_down(k);}
    }
    fn enable_heap(&mut self) {
        self.heap_enabled=true;
        self.heap=Vec::with_capacity(self.covered.len());
        self.heap_position=vec![usize::MAX;self.covered.len()];
        self.heap_rebuild();
    }
'''+block[b:]
block=once(block,'if !add {self.heap_remove(v);}','if self.heap_enabled && !add {self.heap_remove(v);}')
block=once(block,'if self.covered[u]{self.heap_refresh(u);}','if self.heap_enabled && self.covered[u]{self.heap_refresh(u);}')
block=once(block,'if add {self.heap_insert(v);}','if self.heap_enabled && add {self.heap_insert(v);}')
a=block.index('    fn remove_choice(&self)')
b=block.index('    fn raise_weights(',a)
scan=once(scan,'        let mut best = None;','        if self.heap_enabled {return self.heap.first().copied();}\n        let mut best = None;')
block=block[:a]+scan+block[b:]
a=s.index('struct State {')
b=s.index('pub(super) fn improve(',a)
s=s[:a]+block+s[b:]
s=once(s,'    pub restarts:usize,','    pub restarts:usize,\n    pub heap_used:bool,')
s=once(s,'deadline,0xd1b54a32d192ed03,0)','deadline,0xd1b54a32d192ed03,0,false)')
s=once(s,'minimum_iterations:usize)->','minimum_iterations:usize,heap:bool)->')
s=once(s,'    stats.edges = state.edges.len();','    if heap && heap_eligible(graph.n,state.edges.len(),memory) {state.enable_heap();}\n    stats.heap_used=state.heap_enabled;\n    stats.edges = state.edges.len();')
s=once(s,'pub(super) fn portfolio(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant)->(Vec<usize>,Stats){','''pub(super) fn portfolio(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant)->(Vec<usize>,Stats){
 portfolio_selected(graph,seed,memory,deadline,false)
}
pub(super) fn selected(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant,portfolio:bool,heap:bool)->(Vec<usize>,Stats){
 if portfolio {portfolio_selected(graph,seed,memory,deadline,heap)}else{improve_seeded(graph,seed,memory,deadline,0xd1b54a32d192ed03,0,heap)}
}
fn heap_eligible(n:usize,edges:usize,memory:usize)->bool {
 let bits=usize::BITS-n.saturating_sub(1).leading_zeros();
 n>4096 && n<=u16::MAX as usize
  && 8*(edges as u128)*(bits as u128)<=(n as u128)*(n as u128)
  && 32*(edges as u128)+144*(n as u128)<=memory as u128
}
fn portfolio_selected(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant,heap:bool)->(Vec<usize>,Stats){''')
s=once(s,'let(mut best,mut stats)=improve(graph,seed,memory,deadline);','let(mut best,mut stats)=improve_seeded(graph,seed,memory,deadline,0xd1b54a32d192ed03,0,heap);')
s=once(s,'deadline,rng,graph.n.saturating_mul(1024).min(2_000_000));','deadline,rng,graph.n.saturating_mul(1024).min(2_000_000),heap);')
s=once(s,'  stats.restarts+=1;','  stats.heap_used|=call.heap_used;\n  stats.restarts+=1;')
needle='        assert_eq!(s.count, s.covered.iter().filter(|v| **v).count());'
s=once(s,needle,needle+'''
        if s.heap_enabled {
            let expected=(0..s.covered.len()).filter(|&v|s.covered[v]).reduce(|a,b|if s.better(a,b){a}else{b});
            assert_eq!(s.remove_choice(),expected);assert_eq!(s.heap.len(),s.count);
            for v in 0..s.covered.len(){assert_eq!(s.heap_position[v]!=usize::MAX,s.covered[v]);}
            for(k,&v)in s.heap.iter().enumerate(){assert_eq!(s.heap_position[v],k);if k>0{assert!(!s.better(v,s.heap[(k-1)/2]));}}
        } else {assert!(s.heap.is_empty()&&s.heap_position.is_empty());}
''')
a=s.index('    #[test]\n    fn incremental_scores')
b=s.index('    #[test]',a+12)
extra=s[a:b].replace('incremental_scores_edge_sets_and_forgetting_match_full_recomputation','heap_selection_matches_full_scan_after_each_score_and_age_update',1)
extra=once(extra,'                let mut stats = Stats::default();','                state.enable_heap();\n                let mut stats = Stats::default();')
high=private[private.index('    #[test]\n    fn heap_and_u16_edges'):private.index('    #[test]\n    fn incremental_scores')]
high=once(high,'assert!(state.edges.iter().any','state.enable_heap();\n        assert!(state.edges.iter().any')
extra+='''    #[test]
    fn heap_gate_respects_size_density_and_additional_memory() {
        assert!(!heap_eligible(4096,100,usize::MAX));
        assert!(heap_eligible(5000,2500,16*1024*1024));
        assert!(heap_eligible(17937,520860,256*1024*1024));
        assert!(!heap_eligible(5000,10000000,usize::MAX));
        assert!(!heap_eligible(5000,2500,1));
        assert!(!heap_eligible(65536,1,usize::MAX));
    }
'''+high
s=s[:b]+extra+s[b:]
changes={p:s}
p=Path('src/binary_graph.rs');s=p.read_text()
a=s.index('    let portfolio=portfolio&&adaptive&&cover;',s.index('pub fn solve_with_capacity('))
s=s[:a]+'''    solve_with_heap(model,memory,deadline,limit,local,stronger_root,preserve_order,diversify,adaptive,cover,fractional,portfolio,input_cover,spectral,large,false)
}
pub fn solve_with_heap(
    model: &Model, memory: usize, deadline: Instant, limit: usize,
    local: bool, stronger_root: bool, preserve_order: bool, diversify: bool,
    adaptive: bool, cover: bool, fractional: bool, portfolio: bool,
    input_cover: bool, spectral: bool, large: bool, heap: bool,
) -> Option<Answer> {
'''+s[a:]
s=once(s,'            let improve=if portfolio{weighted_cover::portfolio}else{weighted_cover::improve};\n            let (seed, stats)=improve(&graph,&search.best,memory/2,local_deadline);','            let (seed, stats)=weighted_cover::selected(&graph,&search.best,memory/2,local_deadline,portfolio,heap);')
changes[p]=s
p=Path('src/solve.rs');s=p.read_text()
s=once(s,'    pub graph_large_capacity:bool,','    pub graph_large_capacity:bool,\n    pub graph_cover_heap:bool,')
s=once(s,'            graph_large_capacity:true,','            graph_large_capacity:true,\n            graph_cover_heap:false,')
s=once(s,'crate::binary_graph::solve_with_capacity(','crate::binary_graph::solve_with_heap(')
s=once(s,'                limits.graph_large_capacity,','                limits.graph_large_capacity,\n                limits.graph_cover_heap,')
changes[p]=s
p=Path('src/main.rs');s=p.read_text()
a=s.index('        if let Ok(v)=std::env::var("MIP_GRAPH_LARGE_CAPACITY")')
s=s[:a]+'''        if let Ok(v)=std::env::var("MIP_GRAPH_COVER_HEAP"){limits.graph_cover_heap=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_GRAPH_COVER_HEAP must be 0 or 1".into())};}
'''+s[a:]
changes[p]=s
for p,s in changes.items():p.write_text(s)
Path('artifacts/cover-heap-port-initial-failure.json').write_text(json.dumps(dict(initial_error='assert Stats.heap_used inserted failed because restarts field has no space after colon',rust_files_written_on_initial_attempt=False,recovery='validated all literal edits in memory and wrote only after all checks; reusable port script saved'),indent=2))
