//! Independently implemented edge-weighted vertex-cover search; see GRAPH_WEIGHTED_COVER_PLAN.
use super::{Graph, adjacent, strict_clique};
use std::time::Instant;
#[derive(Default, Debug, serde::Serialize)]
pub struct Stats {
    pub initial: usize,
    pub best: usize,
    pub iterations: usize,
    pub first_improvement_iteration: Option<usize>,
    pub last_improvement_iteration: usize,
    pub additions: usize,
    pub removals: usize,
    pub weight_increases: usize,
    pub forgettings: usize,
    pub validation_failures: usize,
    pub configuration_fallbacks: usize,
    pub edges: usize,
    pub restarts:usize,
    pub iteration_limit: usize,
    pub seconds: f64,
    pub stop_reason: &'static str,
}
#[derive(Clone, Copy)]
struct Edge {
    a: u16,
    b: u16,
    weight: u32,
}
struct State {
    edges: Vec<Edge>,
    neighbors: Vec<Vec<u32>>,
    covered: Vec<bool>,
    count: usize,
    score: Vec<i64>,
    changed: Vec<bool>,
    age: Vec<usize>,
    uncovered: Vec<u32>,
    positions: Vec<u32>,
    total: u64,
}
impl State {
    fn new(graph: &Graph, seed: &[usize], memory: usize, deadline: Instant) -> Option<Self> {
        let mut covered = vec![true; graph.n];
        for &v in seed {
            covered[v] = false;
        }
        let mut edges = Vec::new();
        let mut neighbors = vec![Vec::new(); graph.n];
        for a in 0..graph.n {
            if Instant::now() >= deadline {
                return None;
            }
            for b in a + 1..graph.n {
                if !adjacent(graph, a, b) {
                    if (edges.len() + 1)
                        .checked_mul(32)?
                        .checked_add(graph.n * 128)?
                        > memory
                    {
                        return None;
                    }
                    let e = edges.len() as u32;
                    edges.push(Edge {
                        a: a as u16,
                        b: b as u16,
                        weight: 1,
                    });
                    neighbors[a].push(e);
                    neighbors[b].push(e);
                }
            }
        }
        let mut s = Self {
            positions: vec![u32::MAX; edges.len()],
            score: vec![0; graph.n],
            changed: vec![true; graph.n],
            age: vec![0; graph.n],
            uncovered: vec![],
            total: edges.len() as u64,
            count: covered.iter().filter(|v| **v).count(),
            edges,
            neighbors,
            covered,
        };
        s.rebuild();
        Some(s)
    }
    fn rebuild(&mut self) {
        self.score.fill(0);
        self.uncovered.clear();
        self.positions.fill(u32::MAX);
        self.total = 0;
        for i in 0..self.edges.len() {
            let e = self.edges[i];
            let (a, b) = (e.a as usize, e.b as usize);
            let w = e.weight as i64;
            self.total += e.weight as u64;
            match (self.covered[a], self.covered[b]) {
                (false, false) => {
                    self.score[a] += w;
                    self.score[b] += w;
                    self.positions[i] = self.uncovered.len() as u32;
                    self.uncovered.push(i as u32);
                }
                (true, false) => self.score[a] -= w,
                (false, true) => self.score[b] -= w,
                _ => {}
            }
        }
    }
    fn uncover(&mut self, e: usize) {
        if self.positions[e] == u32::MAX {
            self.positions[e] = self.uncovered.len() as u32;
            self.uncovered.push(e as u32);
        }
    }
    fn cover(&mut self, e: usize) {
        let k = self.positions[e] as usize;
        if self.positions[e] != u32::MAX {
            self.uncovered.swap_remove(k);
            self.positions[e] = u32::MAX;
            if k < self.uncovered.len() {
                self.positions[self.uncovered[k] as usize] = k as u32;
            }
        }
    }
    fn change(&mut self, v: usize, add: bool, step: usize) {
        if self.covered[v] == add {
            return;
        }
        self.score[v] = -self.score[v];
        self.covered[v] = add;
        self.age[v] = step;
        if add {
            self.count += 1;
        } else {
            self.count -= 1;
        }
        for k in 0..self.neighbors[v].len() {
            let id = self.neighbors[v][k] as usize;
            let e = self.edges[id];
            let u = if e.a as usize == v {
                e.b as usize
            } else {
                e.a as usize
            };
            let w = e.weight as i64;
            self.changed[u] = true;
            if add {
                if self.covered[u] {
                    self.score[u] += w;
                } else {
                    self.score[u] -= w;
                    self.cover(id);
                }
            } else if self.covered[u] {
                self.score[u] -= w;
            } else {
                self.score[u] += w;
                self.uncover(id);
            }
        }
        if !add {
            self.changed[v] = false;
        }
    }
    fn better(&self, a: usize, b: usize) -> bool {
        self.score[a] > self.score[b]
            || self.score[a] == self.score[b]
                && (self.age[a] < self.age[b] || self.age[a] == self.age[b] && a < b)
    }
    fn remove_choice(&self) -> Option<usize> {
        let mut best = None;
        for v in 0..self.covered.len() {
            if self.covered[v] && best.is_none_or(|b| self.better(v, b)) {
                best = Some(v);
            }
        }
        best
    }
    fn raise_weights(&mut self, stats: &mut Stats) {
        for &id in &self.uncovered {
            let e = &mut self.edges[id as usize];
            e.weight += 1;
            self.total += 1;
            self.score[e.a as usize] += 1;
            self.score[e.b as usize] += 1;
            stats.weight_increases += 1;
        }
        if !self.edges.is_empty()
            && 2 * self.total >= self.edges.len() as u64 * self.covered.len() as u64
        {
            for e in &mut self.edges {
                e.weight = (e.weight as u64 * 3 / 10) as u32;
            }
            self.rebuild();
            stats.forgettings += 1;
        }
    }
}
pub(super) fn improve(
    graph: &Graph,
    seed: &[usize],
    memory: usize,
    deadline: Instant,
) -> (Vec<usize>, Stats) {
    improve_seeded(graph,seed,memory,deadline,0xd1b54a32d192ed03,0)
}
fn improve_seeded(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant,initial_rng:u64,minimum_iterations:usize)->(Vec<usize>,Stats){
    let started = Instant::now();
    let mut best = if strict_clique(graph, seed) {
        seed.to_vec()
    } else {
        vec![]
    };
    let mut stats = Stats {
        initial: best.len(),
        best: best.len(),
        iteration_limit: graph.n.saturating_mul(128).min(500000).max(minimum_iterations).min(graph.n.saturating_mul(1024).min(2_000_000)),
        ..Stats::default()
    };
    let Some(mut state) = State::new(graph, &best, memory, deadline) else {
        stats.stop_reason = "construction_limit";
        stats.seconds = started.elapsed().as_secs_f64();
        return (best, stats);
    };
    stats.edges = state.edges.len();
    let mut rng = initial_rng;
    let extended_limit = graph.n.saturating_mul(1024).min(2_000_000);
    for step in 1..=extended_limit {
        if step > stats.iteration_limit {
            break;
        }
        if Instant::now() >= deadline {
            stats.stop_reason = "deadline";
            break;
        }
        stats.iterations = step;
        if state.uncovered.is_empty() {
            let candidate: Vec<_> = (0..graph.n).filter(|&v| !state.covered[v]).collect();
            if candidate.len() > best.len() {
                if !strict_clique(graph, &candidate) {
                    stats.validation_failures += 1;
                    stats.stop_reason = "invalid_candidate";
                    break;
                }
                best = candidate;
                stats.best = best.len();
                stats.first_improvement_iteration.get_or_insert(step);
                stats.last_improvement_iteration = step;
                stats.iteration_limit = extended_limit;
            }
            let Some(v) = state.remove_choice() else {
                stats.stop_reason = "complete";
                break;
            };
            state.change(v, false, step);
            stats.removals += 1;
            continue;
        }
        let Some(v) = state.remove_choice() else {
            stats.stop_reason = "no_moves";
            break;
        };
        state.change(v, false, step);
        stats.removals += 1;
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        let e = state.edges[state.uncovered[rng as usize % state.uncovered.len()] as usize];
        let (a, b) = (e.a as usize, e.b as usize);
        let add = match (state.changed[a], state.changed[b]) {
            (true, false) => a,
            (false, true) => b,
            (false, false) => {
                stats.configuration_fallbacks += 1;
                if state.better(a, b) { a } else { b }
            }
            _ => {
                if state.better(a, b) {
                    a
                } else {
                    b
                }
            }
        };
        state.change(add, true, step);
        stats.additions += 1;
        state.raise_weights(&mut stats);
    }
    if stats.stop_reason.is_empty() {
        stats.stop_reason = "iteration_limit";
    }
    // A final feasible cover can be reached on the last exchange.
    if state.uncovered.is_empty() {
        let candidate: Vec<_> = (0..graph.n).filter(|&v| !state.covered[v]).collect();
        if candidate.len() > best.len() && strict_clique(graph, &candidate) {
            best = candidate;
            stats.best = best.len();
            stats
                .first_improvement_iteration
                .get_or_insert(stats.iterations);
            stats.last_improvement_iteration = stats.iterations;
        }
    }
    stats.seconds = started.elapsed().as_secs_f64();
    (best, stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn graph(n: usize, seed: &mut u64) -> Graph {
        let mut compatible = vec![vec![0; (n + 63) / 64]; n];
        for a in 0..n {
            for b in a + 1..n {
                *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                if *seed >> 63 != 0 {
                    compatible[a][b / 64] |= 1 << (b % 64);
                    compatible[b][a / 64] |= 1 << (a % 64);
                }
            }
        }
        Graph {
            n,
            words: (n + 63) / 64,
            compatible,
            order: (0..n).collect(),
        }
    }
    fn verify(s: &State) {
        assert_eq!(s.count, s.covered.iter().filter(|v| **v).count());
        let mut scores = vec![0i64; s.covered.len()];
        let mut uncovered = vec![];
        let mut total = 0u64;
        for (i, e) in s.edges.iter().enumerate() {
            let (a, b) = (e.a as usize, e.b as usize);
            total += e.weight as u64;
            let w = e.weight as i64;
            match (s.covered[a], s.covered[b]) {
                (false, false) => {
                    scores[a] += w;
                    scores[b] += w;
                    uncovered.push(i as u32);
                    assert!(s.changed[a] || s.changed[b]);
                }
                (true, false) => scores[a] -= w,
                (false, true) => scores[b] -= w,
                _ => {}
            }
        }
        assert_eq!(scores, s.score);
        assert_eq!(total, s.total);
        let mut actual = s.uncovered.clone();
        actual.sort_unstable();
        assert_eq!(actual, uncovered);
        for (i, &position) in s.positions.iter().enumerate() {
            if position != u32::MAX {
                assert_eq!(s.uncovered[position as usize], i as u32);
            } else {
                assert!(!uncovered.contains(&(i as u32)));
            }
        }
    }
    #[test]
    fn incremental_scores_edge_sets_and_forgetting_match_full_recomputation() {
        let mut seed = 791239u64;
        for n in 1..=20 {
            for _ in 0..5 {
                let g = graph(n, &mut seed);
                let mut state = State::new(
                    &g,
                    &[],
                    1 << 24,
                    Instant::now() + std::time::Duration::from_secs(2),
                )
                .unwrap();
                let mut stats = Stats::default();
                verify(&state);
                for step in 1..=300 {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let v = seed as usize % n;
                    state.change(v, !state.covered[v], step);
                    verify(&state);
                    state.raise_weights(&mut stats);
                    verify(&state);
                }
            }
        }
    }
    #[test]
    fn local_candidates_are_valid_and_never_exceed_enumerated_optima() {
        let mut seed = 913421u64;
        for n in 1..=12 {
            for _ in 0..8 {
                let g = graph(n, &mut seed);
                let optimum = (0u64..(1 << n))
                    .filter(|mask| {
                        let point: Vec<_> = (0..n).filter(|&j| mask & (1 << j) != 0).collect();
                        strict_clique(&g, &point)
                    })
                    .map(|mask| mask.count_ones() as usize)
                    .max()
                    .unwrap();
                let (best, stats) = improve(
                    &g,
                    &[0],
                    1 << 24,
                    Instant::now() + std::time::Duration::from_secs(2),
                );
                assert!(strict_clique(&g, &best));
                assert!(best.len() >= 1 && best.len() <= optimum);
                assert_eq!(stats.validation_failures, 0);
                assert_eq!(stats.configuration_fallbacks, 0);
            }
        }
    }
    #[test]
    fn invalid_seeds_and_construction_limits_fail_closed() {
        let mut seed = 83;
        let g = graph(8, &mut seed);
        let (best, _) = improve(
            &g,
            &[0, 0],
            0,
            Instant::now() + std::time::Duration::from_secs(1),
        );
        assert!(best.is_empty());
        let (best, _) = improve(&g, &[0], 1 << 24, Instant::now());
        assert_eq!(best, vec![0]);
    }
}

/// 改善したときだけ、残り時間で重みと乱数系列を変えて再始動する。
pub(super) fn portfolio(graph:&Graph,seed:&[usize],memory:usize,deadline:Instant)->(Vec<usize>,Stats){
 let began=Instant::now();let(mut best,mut stats)=improve(graph,seed,memory,deadline);
 let mut improved=best.len()>seed.len();
 for round in 1..=8u64{
  if !improved||Instant::now()>=deadline||best.len()==graph.n||stats.validation_failures>0{break;}
  let before=best.len();let offset=stats.iterations;
  let rng=0xd1b54a32d192ed03u64^round.wrapping_mul(0x9e3779b97f4a7c15);
  let(next,call)=improve_seeded(graph,&best,memory,deadline,rng,graph.n.saturating_mul(1024).min(2_000_000));
  stats.restarts+=1;
  stats.iterations=stats.iterations.saturating_add(call.iterations);
  stats.iteration_limit=stats.iteration_limit.saturating_add(call.iteration_limit);
  if let Some(first)=call.first_improvement_iteration{stats.first_improvement_iteration.get_or_insert(offset+first);}
  if call.last_improvement_iteration>0{stats.last_improvement_iteration=offset+call.last_improvement_iteration;}
  stats.additions+=call.additions;stats.removals+=call.removals;stats.weight_increases+=call.weight_increases;stats.forgettings+=call.forgettings;stats.validation_failures+=call.validation_failures;stats.configuration_fallbacks+=call.configuration_fallbacks;stats.stop_reason=call.stop_reason;
  improved=next.len()>before;if improved{best=next;}stats.best=best.len();
 }
 stats.seconds=began.elapsed().as_secs_f64();(best,stats)
}

#[cfg(test)]mod portfolio_tests{
 use super::*;
 #[test]fn restarted_points_respect_exhaustive_clique_optima(){
  let mut rng=2026111809u64;let mut restarts=0;
  for case in 0..100{let n=4+case%5;let mut compatible=vec![vec![0u64;1];n];for a in 0..n{for b in a+1..n{rng=rng.wrapping_mul(6364136223846793005).wrapping_add(1);if rng>>63!=0{compatible[a][0]|=1<<b;compatible[b][0]|=1<<a;}}}let graph=Graph{n,words:1,compatible,order:(0..n).collect()};let before=graph.compatible.clone();let optimum=(0..1usize<<n).filter(|bits|{let x:Vec<_>=(0..n).filter(|&v|bits&(1<<v)!=0).collect();strict_clique(&graph,&x)}).map(|bits|bits.count_ones()as usize).max().unwrap();let seed=vec![case%n];let(x,stats)=portfolio(&graph,&seed,1<<20,Instant::now()+std::time::Duration::from_millis(100));assert!(strict_clique(&graph,&x));assert!(x.len()>=seed.len()&&x.len()<=optimum);assert_eq!(stats.best,x.len());assert_eq!(stats.validation_failures,0);assert_eq!(graph.compatible,before);assert_eq!(seed,[case%n]);restarts+=stats.restarts;}
  assert!(restarts>0);
 }
 #[test]fn expired_or_insufficient_workspace_keeps_the_checked_seed(){
  let graph=Graph{n:3,words:1,compatible:vec![vec![2],vec![1],vec![0]],order:vec![0,1,2]};let seed=vec![0];for(deadline,memory)in[(Instant::now(),1<<20),(Instant::now()+std::time::Duration::from_secs(1),1)]{let(x,stats)=portfolio(&graph,&seed,memory,deadline);assert_eq!(x,seed);assert_eq!(stats.restarts,0);assert!(strict_clique(&graph,&x));}
 }
}
