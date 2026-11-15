//! Candidate fractional clique covers, certified only by exact integer coverage.
use super::{Graph, adjacent, clear, first};
use std::{
    collections::{BinaryHeap, HashSet},
    time::Instant,
};
#[derive(Debug, serde::Serialize)]
pub struct Certificate {
    pub cliques: Vec<Vec<usize>>,
    pub counts: Vec<u32>,
    pub total: u64,
    pub minimum: u32,
    pub upper: usize,
}
#[derive(Debug, Default, serde::Serialize)]
pub struct Stats {
    pub generated: usize,
    pub enumeration_calls: usize,
    pub enumeration_partial: bool,
    pub iterations: usize,
    pub heap_refreshes: usize,
    pub improvements: usize,
    pub validation_failures: usize,
    pub seconds: f64,
    pub stop_reason: &'static str,
}
#[derive(Debug, Default, serde::Serialize)]
pub struct Outcome {
    pub certificate: Option<Certificate>,
    pub stats: Stats,
}
struct Builder<'a> {
    graph: &'a Graph,
    conflict: Vec<Vec<u64>>,
    groups: Vec<Vec<usize>>,
    seen: HashSet<Vec<usize>>,
    members: usize,
    memory: usize,
    deadline: Instant,
    partial: bool,
    calls: usize,
}
impl Builder<'_> {
    fn add(&mut self, mut group: Vec<usize>) -> Option<usize> {
        group.sort_unstable();
        group.dedup();
        if group.is_empty() {
            return None;
        }
        if self.seen.contains(&group) {
            return None;
        }
        if self.groups.len() >= self.graph.n.saturating_mul(4).clamp(64, 20000)
            || (self.members + group.len())
                .checked_mul(16)?
                .checked_add((self.groups.len() + 1) * 128)?
                > self.memory
        {
            self.partial = true;
            return None;
        }
        let id = self.groups.len();
        self.members += group.len();
        self.seen.insert(group.clone());
        self.groups.push(group);
        Some(id)
    }
    fn grow(&mut self, mut group: Vec<usize>) {
        let mut candidate = vec![u64::MAX; self.graph.words];
        if self.graph.n % 64 != 0 {
            *candidate.last_mut().unwrap() = (1 << (self.graph.n % 64)) - 1;
        }
        for &v in &group {
            for (a, &b) in candidate.iter_mut().zip(&self.conflict[v]) {
                *a &= b;
            }
        }
        while first(&candidate).is_some() {
            if Instant::now() >= self.deadline {
                self.partial = true;
                break;
            }
            // Recognition orders vertices by increasing conflict degree. Prefer
            // its last available vertex to grow a large clique cheaply.
            let word = candidate.iter().rposition(|&bits| bits != 0).unwrap();
            let v = word * 64 + 63 - candidate[word].leading_zeros() as usize;
            group.push(v);
            for (a, &b) in candidate.iter_mut().zip(&self.conflict[v]) {
                *a &= b;
            }
        }
        self.add(group);
    }
    fn enumerate(&mut self, group: &mut Vec<usize>, mut p: Vec<u64>, mut x: Vec<u64>) {
        self.calls += 1;
        if Instant::now() >= self.deadline
            || self.groups.len() >= self.graph.n.saturating_mul(4).clamp(64, 20000)
        {
            self.partial = true;
            return;
        }
        if first(&p).is_none() {
            if first(&x).is_none() {
                self.add(group.clone());
            }
            return;
        }
        if group.len() >= 256 {
            self.add(group.clone());
            self.partial = true;
            return;
        }
        let mut union: Vec<_> = p.iter().zip(&x).map(|(a, b)| a | b).collect();
        let mut pivot = None;
        let mut largest = 0;
        while let Some(v) = first(&union) {
            clear(&mut union, v);
            let score = p
                .iter()
                .zip(&self.conflict[v])
                .map(|(a, b)| (a & b).count_ones())
                .sum::<u32>();
            if pivot.is_none() || score >= largest {
                pivot = Some(v);
                largest = score;
            }
        }
        let mut choices = p.clone();
        if let Some(v) = pivot {
            for (a, &b) in choices.iter_mut().zip(&self.conflict[v]) {
                *a &= !b;
            }
        }
        while let Some(v) = first(&choices) {
            clear(&mut choices, v);
            group.push(v);
            let next_p = p
                .iter()
                .zip(&self.conflict[v])
                .map(|(a, b)| a & b)
                .collect();
            let next_x = x
                .iter()
                .zip(&self.conflict[v])
                .map(|(a, b)| a & b)
                .collect();
            self.enumerate(group, next_p, next_x);
            group.pop();
            clear(&mut p, v);
            x[v / 64] |= 1 << (v % 64);
            if Instant::now() >= self.deadline
                || self.groups.len() >= self.graph.n.saturating_mul(4).clamp(64, 20000)
            {
                self.partial = true;
                return;
            }
        }
    }
}
#[derive(Clone, Copy)]
struct Entry {
    score: f64,
    id: usize,
}
impl PartialEq for Entry {
    fn eq(&self, b: &Self) -> bool {
        self.score == b.score && self.id == b.id
    }
}
impl Eq for Entry {}
impl PartialOrd for Entry {
    fn partial_cmp(&self, b: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(b))
    }
}
impl Ord for Entry {
    fn cmp(&self, b: &Self) -> std::cmp::Ordering {
        self.score
            .total_cmp(&b.score)
            .then_with(|| b.id.cmp(&self.id))
    }
}
fn certificate(graph: &Graph, groups: &[Vec<usize>], counts: &[u32]) -> Option<Certificate> {
    if groups.len() != counts.len() {
        return None;
    }
    let mut coverage = vec![0u64; graph.n];
    let mut total = 0u64;
    let mut cliques = Vec::new();
    let mut weights = Vec::new();
    for (group, &count) in groups.iter().zip(counts) {
        if count == 0 {
            continue;
        }
        let mut seen = vec![false; graph.n];
        for (k, &v) in group.iter().enumerate() {
            if v >= graph.n || seen[v] || group[..k].iter().any(|&u| adjacent(graph, u, v)) {
                return None;
            }
            seen[v] = true;
            coverage[v] = coverage[v].checked_add(count as u64)?;
        }
        total = total.checked_add(count as u64)?;
        cliques.push(group.iter().map(|&v| graph.order[v]).collect());
        weights.push(count);
    }
    let minimum = *coverage.iter().min()?;
    if minimum == 0 || minimum > u32::MAX as u64 {
        return None;
    }
    Some(Certificate {
        cliques,
        counts: weights,
        total,
        minimum: minimum as u32,
        upper: (total / minimum) as usize,
    })
}
impl Certificate {
    pub(super) fn prefix_bounds(&self, n: usize, order: &[usize]) -> Option<Vec<usize>> {
        if order.len() != n || self.cliques.len() != self.counts.len() {
            return None;
        }
        let mut members = vec![Vec::new(); n];
        let mut coverage = vec![0u64; n];
        for (i, (group, &w)) in self.cliques.iter().zip(&self.counts).enumerate() {
            for &v in group {
                if v >= n {
                    return None;
                }
                members[v].push(i);
                coverage[v] = coverage[v].checked_add(w as u64)?;
            }
        }
        let mut seen = vec![false; n];
        let mut active = vec![false; self.cliques.len()];
        let (mut total, mut minimum) = (0u64, u64::MAX);
        let mut bounds = Vec::with_capacity(n);
        for &v in order {
            if v >= n || seen[v] || coverage[v] == 0 {
                return None;
            }
            seen[v] = true;
            minimum = minimum.min(coverage[v]);
            for &i in &members[v] {
                if !active[i] {
                    active[i] = true;
                    total = total.checked_add(self.counts[i] as u64)?;
                }
            }
            bounds.push((total / minimum) as usize);
        }
        Some(bounds)
    }
}
pub(super) fn improve(
    graph: &Graph,
    colors: &[usize],
    other_colors: &[Vec<usize>],
    memory: usize,
    deadline: Instant,
    incumbent: usize,
) -> Outcome {
    let started = Instant::now();
    let mut out = Outcome::default();
    if colors.len() != graph.n || graph.n == 0 {
        return out;
    }
    let construction = started + deadline.saturating_duration_since(started) / 4;
    let mut conflict = graph.compatible.clone();
    for (v, row) in conflict.iter_mut().enumerate() {
        for word in row.iter_mut() {
            *word = !*word;
        }
        clear(row, v);
        if graph.n % 64 != 0 {
            *row.last_mut().unwrap() &= (1 << (graph.n % 64)) - 1;
        }
    }
    let mut builder = Builder {
        graph,
        conflict,
        groups: vec![],
        seen: HashSet::new(),
        members: 0,
        memory,
        deadline: construction,
        partial: false,
        calls: 0,
    };
    let max_color = colors.iter().copied().max().unwrap_or(0);
    let mut base_count = 0;
    for color in 1..=max_color {
        let group: Vec<_> = (0..graph.n)
            .filter(|&v| colors[graph.order[v]] == color)
            .collect();
        if !group.is_empty() {
            if builder.add(group).is_none() {
                out.stats.stop_reason = "construction_limit";
                return out;
            }
            base_count += 1;
        }
    }
    for v in 0..graph.n {
        if Instant::now() >= construction {
            break;
        }
        builder.grow(vec![v]);
    }
    for v in 0..graph.n {
        if Instant::now() >= construction {
            builder.partial = true;
            break;
        }
        let mut p = builder.conflict[v].clone();
        let mut x = p.clone();
        for u in 0..graph.n {
            if u <= v {
                clear(&mut p, u);
            } else {
                clear(&mut x, u);
            }
        }
        builder.enumerate(&mut vec![v], p, x);
    }
    let base = builder.groups[..base_count].to_vec();
    for group in base {
        if Instant::now() >= construction {
            break;
        }
        builder.grow(group);
    }
    for coloring in other_colors {
        if coloring.len() != graph.n {
            continue;
        }
        for color in 1..=coloring.iter().copied().max().unwrap_or(0) {
            if Instant::now() >= construction {
                break;
            }
            builder.grow(
                (0..graph.n)
                    .filter(|&v| coloring[graph.order[v]] == color)
                    .collect(),
            );
        }
    }
    out.stats.generated = builder.groups.len();
    out.stats.enumeration_calls = builder.calls;
    out.stats.enumeration_partial = builder.partial;
    let groups = builder.groups;
    if let Ok(path) = std::env::var("MIP_GRAPH_CLIQUE_DUMP") {
        if let Ok(file) = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            let original_groups: Vec<Vec<_>> = groups
                .iter()
                .map(|g| g.iter().map(|&v| graph.order[v]).collect())
                .collect();
            let _ = serde_json::to_writer(
                file,
                &serde_json::json!({
                    "vertices": graph.n, "groups": original_groups, "base_count": base_count,
                    "purpose": "diagnostic candidates only; no native weights used in Rust"
                }),
            );
        }
    }

    let mut counts = vec![0u32; groups.len()];
    for c in &mut counts[..base_count] {
        *c = 1;
    }
    let mut coverage = vec![0u32; graph.n];
    for (group, &w) in groups.iter().zip(&counts) {
        for &v in group {
            coverage[v] += w;
        }
    }
    if coverage.iter().any(|v| *v == 0) {
        out.stats.validation_failures += 1;
        return out;
    }
    let mut histogram = vec![0usize; coverage.iter().copied().max().unwrap() as usize + 2];
    for &c in &coverage {
        histogram[c as usize] += 1;
    }
    let mut minimum = *coverage.iter().min().unwrap() as usize;
    let mut total = base_count as u64;
    let mut best_upper = (total / minimum as u64) as usize;
    let mut best = counts.clone();
    let mut prices = vec![1f64; graph.n];
    let discount = (-1f64).exp();
    let score = |id: usize, prices: &[f64]| groups[id].iter().map(|&v| prices[v]).sum::<f64>();
    let mut heap: BinaryHeap<_> = (0..groups.len())
        .map(|id| Entry {
            score: score(id, &prices),
            id,
        })
        .collect();
    'search: for step in 1..=graph.n.saturating_mul(128).min(500000) {
        if Instant::now() >= deadline {
            out.stats.stop_reason = "deadline";
            break;
        }
        let chosen = loop {
            let Some(entry) = heap.pop() else {
                break 'search;
            };
            let current = score(entry.id, &prices);
            out.stats.heap_refreshes += 1;
            if current >= heap.peek().map_or(0., |e| e.score) {
                break entry.id;
            }
            heap.push(Entry {
                score: current,
                id: entry.id,
            });
            if out.stats.heap_refreshes % 128 == 0 && Instant::now() >= deadline {
                out.stats.stop_reason = "deadline";
                break 'search;
            }
        };
        counts[chosen] += 1;
        total += 1;
        out.stats.iterations = step;
        for &v in &groups[chosen] {
            let old = coverage[v] as usize;
            histogram[old] -= 1;
            coverage[v] += 1;
            if coverage[v] as usize >= histogram.len() {
                histogram.resize(coverage[v] as usize + 2, 0);
            }
            histogram[coverage[v] as usize] += 1;
            prices[v] *= discount;
        }
        while histogram[minimum] == 0 {
            minimum += 1;
        }
        heap.push(Entry {
            score: score(chosen, &prices),
            id: chosen,
        });
        let upper = (total / minimum as u64) as usize;
        if upper < best_upper {
            best_upper = upper;
            best = counts.clone();
            out.stats.improvements += 1;
            if upper <= incumbent {
                out.stats.stop_reason = "incumbent_bound";
                break;
            }
        }
        if step % 4096 == 0 {
            let maximum = prices.iter().copied().fold(0., f64::max);
            if maximum < 1e-40 {
                if maximum == 0. {
                    for (v, p) in prices.iter_mut().enumerate() {
                        *p = (-(coverage[v] as f64 - minimum as f64)).exp();
                    }
                } else {
                    for p in &mut prices {
                        *p /= maximum;
                    }
                }
                heap = (0..groups.len())
                    .map(|id| Entry {
                        score: score(id, &prices),
                        id,
                    })
                    .collect();
            }
        }
    }
    if out.stats.stop_reason.is_empty() {
        out.stats.stop_reason = "iteration_limit";
    }
    if best_upper < base_count {
        if let Some(proof) = certificate(graph, &groups, &best) {
            if proof.upper >= incumbent {
                out.certificate = Some(proof);
            } else {
                out.stats.validation_failures += 1;
            }
        } else {
            out.stats.validation_failures += 1;
        }
    }
    out.stats.seconds = started.elapsed().as_secs_f64();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    // Enumerate every graph on five vertices and every prefix/subset. This tests
    // the integer certificate and prefix theorem independently of the heuristic.
    #[test]
    fn integer_cover_bounds_all_five_vertex_graphs() {
        let n = 5;
        for mask in 0u32..(1 << 10) {
            let mut graph = Graph {
                n,
                words: 1,
                compatible: vec![vec![0]; n],
                order: vec![4, 2, 0, 3, 1],
            };
            let mut bit = 0;
            for a in 0..n {
                for b in a + 1..n {
                    if mask & (1 << bit) != 0 {
                        graph.compatible[a][0] |= 1 << b;
                        graph.compatible[b][0] |= 1 << a;
                    }
                    bit += 1;
                }
            }
            let mut groups = vec![];
            let mut counts = vec![];
            for subset in 1usize..(1 << n) {
                let group: Vec<_> = (0..n).filter(|v| subset & (1 << v) != 0).collect();
                if group
                    .iter()
                    .enumerate()
                    .all(|(i, &a)| group[..i].iter().all(|&b| !adjacent(&graph, a, b)))
                {
                    groups.push(group);
                    counts.push((subset % 3 + 1) as u32);
                }
            }
            let conflict: Vec<Vec<u64>> = graph
                .compatible
                .iter()
                .enumerate()
                .map(|(v, row)| vec![(!row[0]) & ((1 << n) - 1) & !(1 << v)])
                .collect();
            let mut builder = Builder {
                graph: &graph,
                conflict,
                groups: vec![],
                seen: HashSet::new(),
                members: 0,
                memory: 1 << 20,
                deadline: Instant::now() + std::time::Duration::from_secs(1),
                partial: false,
                calls: 0,
            };
            builder.enumerate(&mut vec![], vec![(1 << n) - 1], vec![0]);
            assert!(!builder.partial);
            for group in &groups {
                let maximal = (0..n)
                    .all(|v| group.contains(&v) || group.iter().any(|&u| adjacent(&graph, u, v)));
                assert_eq!(builder.seen.contains(group), maximal);
            }
            let cert = certificate(&graph, &groups, &counts).unwrap();
            let order = vec![1, 3, 0, 2, 4];
            let bounds = cert.prefix_bounds(n, &order).unwrap();
            for k in 1..=n {
                for subset in 0usize..(1 << k) {
                    let selected: Vec<_> = (0..k)
                        .filter(|v| subset & (1 << v) != 0)
                        .map(|i| graph.order.iter().position(|v| *v == order[i]).unwrap())
                        .collect();
                    if selected
                        .iter()
                        .enumerate()
                        .all(|(i, &a)| selected[..i].iter().all(|&b| adjacent(&graph, a, b)))
                    {
                        assert!(selected.len() <= bounds[k - 1]);
                        assert!(selected.len() <= cert.upper);
                    }
                }
            }
            assert!(bounds.windows(2).all(|w| w[0] <= w[1]));
            assert_eq!(*bounds.last().unwrap(), cert.upper);
            assert!(cert.prefix_bounds(n, &[0, 0, 1, 2, 3]).is_none());
            assert!(certificate(&graph, &[vec![0, 0]], &[1]).is_none());
            assert!(certificate(&graph, &[vec![0]], &[0]).is_none());
            if let Some((a, b)) = (0..n)
                .flat_map(|a| (a + 1..n).map(move |b| (a, b)))
                .find(|&(a, b)| adjacent(&graph, a, b))
            {
                assert!(certificate(&graph, &[vec![a, b]], &[1]).is_none());
            }
        }
    }
}
