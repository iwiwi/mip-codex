//! Uniform-objective binary packing recognition and independently written coloring search.
//! Theory reference: Prosser, Algorithms 2012, doi:10.3390/a5040545; no source-code port.
use crate::model::Model;
use std::time::Instant;
mod fractional_cover;
mod weighted_cover;
pub use fractional_cover::Outcome as FractionalCover;
pub use weighted_cover::Stats as CoverStats;
#[derive(Debug, Default, serde::Serialize)]
pub struct Stats {
    pub local: Option<LocalStats>,
    pub cover_local: Option<CoverStats>,
    pub fractional_cover: Option<FractionalCover>,
    pub root_coloring_upper: Option<usize>,
    pub root_initial_upper: usize,
    pub root_variants: usize,
    pub root_variant_uppers: Vec<usize>,
    pub root_color_seconds: f64,
    pub vertices: usize,
    pub conflict_edges: usize,
    pub nodes: usize,
    pub colorings: usize,
    pub color_vertex_steps: usize,
    pub color_clock_checks: usize,
    pub root_upper: usize,
    pub remaining_upper: usize,
    pub root_branches_completed: usize,
    pub upper: usize,
    pub best: usize,
    pub complete: bool,
    pub root_colors: Vec<usize>,
    pub root_prefix_colorings: Vec<Vec<usize>>,
    pub root_search_order: Vec<usize>,
    pub order: Vec<usize>,
    pub first_seconds: Option<f64>,
}
pub struct Answer {
    pub point: Vec<f64>,
    pub status: &'static str,
    pub stats: Stats,
}
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
fn first(bits: &[u64]) -> Option<usize> {
    bits.iter()
        .enumerate()
        .find(|(_, w)| **w != 0)
        .map(|(k, w)| 64 * k + w.trailing_zeros() as usize)
}
fn clear(bits: &mut [u64], v: usize) {
    bits[v / 64] &= !(1 << (v % 64));
}
struct Search<'a> {
    graph: &'a Graph,
    deadline: Instant,
    limit: usize,
    started: Instant,
    best: Vec<usize>,
    stats: Stats,
    interrupted: bool,
}
impl Search<'_> {
    fn color(&mut self, p: &[u64]) -> Option<(Vec<usize>, Vec<usize>)> {
        let mut remaining = p.to_vec();
        let mut order = Vec::new();
        let mut bounds = Vec::new();
        let mut color = 0;
        while first(&remaining).is_some() {
            color += 1;
            let mut available = remaining.clone();
            while let Some(v) = first(&available) {
                // Node entry/branch checks remain immediate; a long coloring also polls every 256 selections.
                if self.stats.color_vertex_steps & 255 == 0 {
                    self.stats.color_clock_checks += 1;
                    if Instant::now() >= self.deadline {
                        return None;
                    }
                }
                self.stats.color_vertex_steps = self.stats.color_vertex_steps.wrapping_add(1);
                order.push(v);
                bounds.push(color);
                clear(&mut remaining, v);
                clear(&mut available, v);
                for (word, &neighbors) in available.iter_mut().zip(&self.graph.compatible[v]) {
                    *word &= !neighbors;
                }
            }
        }
        self.stats.colorings += 1;
        if self.stats.nodes == 0 {
            self.stats.root_upper = color;
            self.stats.remaining_upper = color;
            self.stats.root_colors = vec![0; self.graph.n];
            for (&v, &c) in order.iter().zip(&bounds) {
                self.stats.root_colors[self.graph.order[v]] = c;
            }
        }
        Some((order, bounds))
    }
    fn expand(
        &mut self,
        mut p: Vec<u64>,
        clique: &mut Vec<usize>,
        root: Option<(Vec<usize>, Vec<usize>)>,
    ) {
        if clique.len() > self.best.len() {
            self.best = clique.clone();
            self.stats
                .first_seconds
                .get_or_insert_with(|| self.started.elapsed().as_secs_f64());
        }
        if Instant::now() >= self.deadline || self.stats.nodes >= self.limit {
            self.interrupted = true;
            return;
        }
        self.stats.nodes += 1;
        let colored = match root {
            Some(c) => Some(c),
            None => self.color(&p),
        };
        let Some((order, bounds)) = colored else {
            self.interrupted = true;
            return;
        };
        for k in (0..order.len()).rev() {
            if clique.is_empty() {
                self.stats.remaining_upper = bounds[k];
            }
            if clique.len() + bounds[k] <= self.best.len() {
                return;
            }
            if Instant::now() >= self.deadline {
                self.interrupted = true;
                return;
            }
            let v = order[k];
            clique.push(v);
            let child: Vec<_> = p
                .iter()
                .zip(&self.graph.compatible[v])
                .map(|(&a, &b)| a & b)
                .collect();
            if first(&child).is_none() {
                if clique.len() > self.best.len() {
                    self.best = clique.clone();
                    self.stats
                        .first_seconds
                        .get_or_insert_with(|| self.started.elapsed().as_secs_f64());
                }
            } else {
                self.expand(child, clique, None);
            }
            clique.pop();
            if self.interrupted {
                return;
            }
            if clique.is_empty() {
                self.stats.root_branches_completed += 1;
            }
            clear(&mut p, v);
        }
    }
}
pub fn solve(model: &Model, memory: usize, deadline: Instant, limit: usize) -> Option<Answer> {
    solve_with_local(model, memory, deadline, limit, false)
}
pub fn solve_with_local(
    model: &Model,
    memory: usize,
    deadline: Instant,
    limit: usize,
    local: bool,
) -> Option<Answer> {
    solve_with_options(model, memory, deadline, limit, local, false)
}
pub fn solve_with_options(
    model: &Model,
    memory: usize,
    deadline: Instant,
    limit: usize,
    local: bool,
    stronger_root: bool,
) -> Option<Answer> {
    solve_with_prefix(model, memory, deadline, limit, local, stronger_root, false)
}
pub fn solve_with_prefix(
    model: &Model,
    memory: usize,
    deadline: Instant,
    limit: usize,
    local: bool,
    stronger_root: bool,
    preserve_order: bool,
) -> Option<Answer> {
    solve_with_diversify(
        model,
        memory,
        deadline,
        limit,
        local,
        stronger_root,
        preserve_order,
        false,
    )
}
pub fn solve_with_diversify(
    model: &Model,
    memory: usize,
    deadline: Instant,
    limit: usize,
    local: bool,
    stronger_root: bool,
    preserve_order: bool,
    diversify: bool,
) -> Option<Answer> {
    solve_with_adaptive(
        model,
        memory,
        deadline,
        limit,
        local,
        stronger_root,
        preserve_order,
        diversify,
        false,
    )
}
pub fn solve_with_adaptive(
    model: &Model,
    memory: usize,
    deadline: Instant,
    limit: usize,
    local: bool,
    stronger_root: bool,
    preserve_order: bool,
    diversify: bool,
    adaptive: bool,
) -> Option<Answer> {
    solve_with_cover(
        model,
        memory,
        deadline,
        limit,
        local,
        stronger_root,
        preserve_order,
        diversify,
        adaptive,
        false,
    )
}
pub fn solve_with_cover(
    model: &Model,
    memory: usize,
    deadline: Instant,
    limit: usize,
    local: bool,
    stronger_root: bool,
    preserve_order: bool,
    diversify: bool,
    adaptive: bool,
    cover: bool,
) -> Option<Answer> {
    solve_with_fractional(
        model,
        memory,
        deadline,
        limit,
        local,
        stronger_root,
        preserve_order,
        diversify,
        adaptive,
        cover,
        false,
    )
}
pub fn solve_with_fractional(
    model: &Model,
    memory: usize,
    deadline: Instant,
    limit: usize,
    local: bool,
    stronger_root: bool,
    preserve_order: bool,
    diversify: bool,
    adaptive: bool,
    cover: bool,
    fractional: bool,
) -> Option<Answer> {
    let started = Instant::now();
    let graph = recognize(model, memory, deadline)?;
    let mut all = vec![u64::MAX; graph.words];
    if graph.n % 64 != 0 {
        all[graph.words - 1] = (1 << (graph.n % 64)) - 1;
    }
    let mut seed = Vec::new();
    let mut candidates = all.clone();
    while let Some(v) = first(&candidates) {
        if Instant::now() >= deadline {
            break;
        }
        seed.push(v);
        for (a, &b) in candidates.iter_mut().zip(&graph.compatible[v]) {
            *a &= b;
        }
    }
    let edges = graph
        .compatible
        .iter()
        .map(|r| r.iter().map(|w| w.count_ones() as usize).sum::<usize>())
        .sum::<usize>()
        / 2;
    let mut search = Search {
        graph: &graph,
        deadline,
        limit,
        started,
        best: seed,
        stats: Stats {
            vertices: graph.n,
            conflict_edges: graph.n * (graph.n - 1) / 2 - edges,
            root_upper: graph.n,
            remaining_upper: graph.n,
            order: graph.order.clone(),
            ..Stats::default()
        },
        interrupted: false,
    };
    search.stats.first_seconds = Some(started.elapsed().as_secs_f64());
    let mut root = search.color(&all);
    search.stats.root_initial_upper = search.stats.root_upper;
    let original_root = root.clone();
    let mut prefix_colorings = vec![search.stats.root_colors.clone()];
    if stronger_root && root.is_some() && search.best.len() < search.stats.root_upper {
        let phase = Instant::now();
        let budget = (deadline.saturating_duration_since(started).as_secs_f64() * 0.05).min(0.15);
        let end = deadline.min(phase + std::time::Duration::from_secs_f64(budget));
        for mode in 0..2 {
            if let Some(colors) = alternate_root_colors(&graph, mode, end) {
                search.stats.root_variants += 1;
                let upper = colors.iter().copied().max().unwrap_or(0);
                search.stats.root_variant_uppers.push(upper);
                let mut original_colors = vec![0; graph.n];
                for v in 0..graph.n {
                    original_colors[graph.order[v]] = colors[v];
                }
                prefix_colorings.push(original_colors);
                if upper < search.stats.root_upper {
                    let mut order: Vec<_> = (0..graph.n).collect();
                    order.sort_unstable_by_key(|&v| (colors[v], v));
                    let bounds = order.iter().map(|&v| colors[v]).collect();
                    for v in 0..graph.n {
                        search.stats.root_colors[graph.order[v]] = colors[v];
                    }
                    search.stats.root_upper = upper;
                    search.stats.remaining_upper = upper;
                    root = Some((order, bounds));
                }
            }
        }
        search.stats.root_color_seconds = phase.elapsed().as_secs_f64();
    }
    if preserve_order {
        if let Some((order, _)) = original_root {
            let mut seen = vec![vec![false; graph.n + 1]; prefix_colorings.len()];
            let mut counts = vec![0usize; prefix_colorings.len()];
            let mut bounds = Vec::with_capacity(graph.n);
            for &v in &order {
                for (k, colors) in prefix_colorings.iter().enumerate() {
                    let color = colors[graph.order[v]];
                    if !seen[k][color] {
                        seen[k][color] = true;
                        counts[k] += 1;
                    }
                }
                bounds.push(*counts.iter().min().unwrap());
            }
            search.stats.root_search_order = order.iter().map(|&v| graph.order[v]).collect();
            search.stats.root_prefix_colorings = prefix_colorings.clone();
            root = Some((order, bounds));
        }
    }
    if local
        && root.is_some()
        && search.best.len() < search.stats.root_upper
        && Instant::now() < deadline
    {
        let allocation = (deadline.saturating_duration_since(started).as_secs_f64() * 0.05).min(
            if adaptive && cover {
                3.0
            } else if adaptive {
                2.0
            } else {
                0.25
            },
        );
        let local_deadline =
            deadline.min(Instant::now() + std::time::Duration::from_secs_f64(allocation));
        let first_deadline = if cover {
            Instant::now() + local_deadline.saturating_duration_since(Instant::now()) / 5
        } else {
            local_deadline
        };
        let (seed, stats) = improve_seed(&graph, &search.best, first_deadline, diversify, adaptive);
        search.best = seed;
        search.stats.local = Some(stats);
        if cover && Instant::now() < local_deadline {
            let (seed, stats) =
                weighted_cover::improve(&graph, &search.best, memory / 2, local_deadline);
            search.best = seed;
            search.stats.cover_local = Some(stats);
        }
    }
    if fractional && root.is_some() && search.best.len() < search.stats.root_upper {
        search.stats.root_coloring_upper = Some(search.stats.root_upper);
        let allocation =
            (deadline.saturating_duration_since(started).as_secs_f64() * 0.02).min(0.5);
        let end = deadline.min(Instant::now() + std::time::Duration::from_secs_f64(allocation));
        let result = fractional_cover::improve(
            &graph,
            &search.stats.root_colors,
            &prefix_colorings,
            memory / 4,
            end,
            search.best.len(),
        );
        if let Some((order, bounds)) = root.as_mut() {
            let original_order: Vec<_> = order.iter().map(|&v| graph.order[v]).collect();
            if search.stats.root_prefix_colorings.is_empty() {
                search.stats.root_prefix_colorings = vec![search.stats.root_colors.clone()];
                search.stats.root_search_order = original_order.clone();
            }
            if let Some(certificate) = &result.certificate {
                if let Some(extra) = certificate.prefix_bounds(graph.n, &original_order) {
                    for (bound, tighter) in bounds.iter_mut().zip(extra) {
                        *bound = (*bound).min(tighter);
                    }
                    search.stats.root_upper = search.stats.root_upper.min(certificate.upper);
                    search.stats.remaining_upper = search.stats.root_upper;
                }
            }
        }
        search.stats.fractional_cover = Some(result);
    }
    if search.best.len() < search.stats.root_upper {
        if let Some(root) = root {
            search.expand(all, &mut vec![], Some(root));
        } else {
            search.interrupted = true;
        }
    }
    search.stats.best = search.best.len();
    search.stats.complete =
        !search.interrupted || search.best.len() >= search.stats.remaining_upper;
    search.stats.upper = if search.stats.complete {
        search.best.len()
    } else {
        search.stats.remaining_upper.max(search.best.len())
    };
    let mut point = vec![0.; graph.n];
    for &v in &search.best {
        point[graph.order[v]] = 1.;
    }
    let status = if search.stats.complete {
        "optimal"
    } else if search.stats.nodes >= limit {
        "node_limit"
    } else {
        "time_limit"
    };
    Some(Answer {
        point,
        status,
        stats: search.stats,
    })
}

#[derive(Default, Debug, serde::Serialize)]
pub struct LocalStats {
    pub initial: usize,
    pub best: usize,
    pub iterations: usize,
    pub pair_swaps: usize,
    pub neutral_swaps: usize,
    pub perturbations: usize,
    pub additions: usize,
    pub restarts: usize,
    pub restart_removals: usize,
    pub iteration_limit: usize,
    pub stagnation_limit: usize,
    pub last_improvement_iteration: usize,
    pub stop_reason: &'static str,
    pub budget_seconds: f64,
    pub validation_failures: usize,
    pub seconds: f64,
}
fn adjacent(graph: &Graph, a: usize, b: usize) -> bool {
    graph.compatible[a][b / 64] & (1 << (b % 64)) != 0
}
fn strict_clique(graph: &Graph, vertices: &[usize]) -> bool {
    let mut seen = vec![false; graph.n];
    for (k, &v) in vertices.iter().enumerate() {
        if v >= graph.n || seen[v] || vertices[..k].iter().any(|&u| !adjacent(graph, u, v)) {
            return false;
        }
        seen[v] = true;
    }
    true
}
struct LocalState {
    selected: Vec<bool>,
    mask: Vec<u64>,
    counts: Vec<usize>,
}
impl LocalState {
    fn new(graph: &Graph, vertices: &[usize]) -> Self {
        let mut state = Self {
            selected: vec![false; graph.n],
            mask: vec![0; graph.words],
            counts: vec![0; graph.n],
        };
        for &v in vertices {
            state.change(graph, v, true);
        }
        state
    }
    fn change(&mut self, graph: &Graph, v: usize, add: bool) {
        debug_assert_ne!(self.selected[v], add);
        debug_assert!(!add || self.counts[v] == 0);
        self.selected[v] = add;
        if add {
            self.mask[v / 64] |= 1 << (v % 64)
        } else {
            clear(&mut self.mask, v)
        }
        for (word, &compatible) in graph.compatible[v].iter().enumerate() {
            let mut bits = !compatible;
            while bits != 0 {
                let j = word * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                if j >= graph.n || j == v {
                    continue;
                }
                if add {
                    self.counts[j] += 1
                } else {
                    self.counts[j] -= 1
                }
            }
        }
    }
    fn vertices(&self) -> Vec<usize> {
        self.selected
            .iter()
            .enumerate()
            .filter_map(|(j, &b)| b.then_some(j))
            .collect()
    }
    fn augment(&mut self, graph: &Graph, stats: &mut LocalStats, order: &[usize]) {
        for &v in order {
            if !self.selected[v] && self.counts[v] == 0 {
                self.change(graph, v, true);
                stats.additions += 1;
            }
        }
    }
    fn one_conflict(&self, graph: &Graph, u: usize) -> Option<usize> {
        self.mask
            .iter()
            .zip(&graph.compatible[u])
            .enumerate()
            .find_map(|(k, (&selected, &compatible))| {
                let bits = selected & !compatible;
                (bits != 0).then(|| 64 * k + bits.trailing_zeros() as usize)
            })
    }
}
fn improve_seed(
    graph: &Graph,
    initial: &[usize],
    deadline: Instant,
    diversify: bool,
    adaptive: bool,
) -> (Vec<usize>, LocalStats) {
    let started = Instant::now();
    let mut stats = LocalStats {
        initial: initial.len(),
        iteration_limit: graph
            .n
            .saturating_mul(if adaptive { 128 } else { 8 })
            .min(if adaptive { 500000 } else { 50000 }),
        stagnation_limit: if adaptive {
            graph.n.saturating_mul(16).max(256)
        } else {
            usize::MAX
        },
        budget_seconds: deadline.saturating_duration_since(started).as_secs_f64(),
        best: initial.len(),
        ..LocalStats::default()
    };
    if !strict_clique(graph, initial) {
        stats.validation_failures += 1;
        stats.best = 0;
        return (vec![], stats);
    }
    let mut best = initial.to_vec();
    let mut state = LocalState::new(graph, initial);
    let mut groups = vec![Vec::new(); graph.n];
    let mut tabu = vec![0usize; graph.n];
    let mut rng = 0x9e3779b97f4a7c15u64;
    let mut stale = 0;
    let mut stagnant = 0;
    let mut order: Vec<usize> = (0..graph.n).collect();
    for iteration in 0..stats.iteration_limit {
        if Instant::now() >= deadline {
            stats.stop_reason = "deadline";
            break;
        }
        stats.iterations += 1;
        state.augment(graph, &mut stats, &order);
        for group in &mut groups {
            group.clear();
        }
        for u in 0..graph.n {
            if !state.selected[u] && state.counts[u] == 1 {
                if let Some(v) = state.one_conflict(graph, u) {
                    groups[v].push(u);
                }
            }
        }
        let mut pair = None;
        let mut work = 0usize;
        'pairs: for (v, group) in groups.iter().enumerate() {
            for a in 0..group.len() {
                for b in a + 1..group.len() {
                    work += 1;
                    if work & 255 == 0 && Instant::now() >= deadline {
                        break 'pairs;
                    }
                    if adjacent(graph, group[a], group[b]) {
                        pair = Some((v, group[a], group[b]));
                        break 'pairs;
                    }
                }
            }
        }
        if let Some((v, a, b)) = pair {
            state.change(graph, v, false);
            state.change(graph, a, true);
            state.change(graph, b, true);
            stats.pair_swaps += 1;
        } else {
            if Instant::now() >= deadline {
                break;
            }
            let preferred = if iteration % 16 == 15 { 2 } else { 1 };
            let mut candidates: Vec<_> = (0..graph.n)
                .filter(|&u| {
                    !state.selected[u] && state.counts[u] == preferred && tabu[u] <= iteration
                })
                .collect();
            if candidates.is_empty() {
                candidates = (0..graph.n)
                    .filter(|&u| {
                        !state.selected[u]
                            && state.counts[u] > 0
                            && state.counts[u] <= 3
                            && tabu[u] <= iteration
                    })
                    .collect();
            }
            if candidates.is_empty() {
                break;
            }
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let u = candidates[rng as usize % candidates.len()];
            let removed: Vec<_> = state
                .selected
                .iter()
                .enumerate()
                .filter_map(|(v, &chosen)| (chosen && !adjacent(graph, u, v)).then_some(v))
                .collect();
            for &v in &removed {
                state.change(graph, v, false);
                tabu[v] = iteration + 7;
            }
            state.change(graph, u, true);
            if removed.len() == 1 {
                stats.neutral_swaps += 1
            } else {
                stats.perturbations += 1
            }
        }
        state.augment(graph, &mut stats, &order);
        let candidate = state.vertices();
        if candidate.len() > best.len() {
            if !strict_clique(graph, &candidate) {
                stats.validation_failures += 1;
                state = LocalState::new(graph, &best);
                continue;
            }
            best = candidate;
            stats.best = best.len();
            stats.last_improvement_iteration = stats.iterations;
            stagnant = 0;
            stale = 0;
        } else {
            // Preserve a feasible neutral point without counting it as an improvement.
            if candidate.len()==best.len() {
                if strict_clique(graph,&candidate){best=candidate;}
                else {stats.validation_failures+=1;state=LocalState::new(graph,&best);}
            }
            stale += 1;
            stagnant += 1;
        }
        if adaptive && stagnant >= stats.stagnation_limit {
            stats.stop_reason = "stagnation";
            break;
        }
        if stale >= 64 {
            state = LocalState::new(graph, &best);
            if diversify {
                stats.restarts += 1;
                for k in (1..order.len()).rev() {
                    rng ^= rng << 13;
                    rng ^= rng >> 7;
                    rng ^= rng << 17;
                    order.swap(k, rng as usize % (k + 1));
                }
                let count = (best.len() / 16).max(2).min(best.len());
                let removed: Vec<_> = order
                    .iter()
                    .copied()
                    .filter(|&v| state.selected[v])
                    .take(count)
                    .collect();
                for v in removed {
                    state.change(graph, v, false);
                    stats.restart_removals += 1;
                }
                state.augment(graph, &mut stats, &order);
            }
            stale = 0;
        }
    }
    if stats.stop_reason.is_empty() {
        stats.stop_reason = if stats.iterations == stats.iteration_limit {
            "iteration_limit"
        } else if Instant::now() >= deadline {
            "deadline"
        } else {
            "no_moves"
        };
    }
    stats.seconds = started.elapsed().as_secs_f64();
    (best, stats)
}

#[cfg(test)]
mod local_tests {
    use super::*;
    #[test]
    fn pair_exchange_improves_a_feasible_seed_and_invalid_seeds_fail_closed() {
        let graph = Graph {
            n: 3,
            words: 1,
            compatible: vec![vec![0], vec![4], vec![2]],
            order: vec![0, 1, 2],
        };
        assert!(!strict_clique(&graph, &[0, 0]));
        assert!(!strict_clique(&graph, &[0, 1]));
        assert!(strict_clique(&graph, &[1, 2]));
        let (best, stats) = improve_seed(
            &graph,
            &[0],
            Instant::now() + std::time::Duration::from_secs(1),
            false,
            false,
        );
        assert_eq!(best, vec![1, 2]);
        assert!(stats.pair_swaps > 0);
        assert_eq!(stats.validation_failures, 0);
        let (best, stats) = improve_seed(
            &graph,
            &[0, 1],
            Instant::now() + std::time::Duration::from_secs(1),
            false,
            false,
        );
        assert!(best.is_empty());
        assert_eq!(stats.validation_failures, 1);
    }
}

// Each class is pairwise nonadjacent in the compatibility graph.
// A timed-out candidate is discarded in full; the previous proper coloring remains.
fn alternate_root_colors(graph: &Graph, mode: usize, deadline: Instant) -> Option<Vec<usize>> {
    let n = graph.n;
    let mut colors = vec![0usize; n];
    let mut remaining = vec![u64::MAX; graph.words];
    if n % 64 != 0 {
        remaining[graph.words - 1] = (1 << (n % 64)) - 1;
    }
    let degree: Vec<usize> = graph
        .compatible
        .iter()
        .map(|r| r.iter().map(|b| b.count_ones() as usize).sum())
        .collect();
    if mode == 0 {
        let mut color = 0;
        while first(&remaining).is_some() {
            color += 1;
            let mut available = remaining.clone();
            while first(&available).is_some() {
                if Instant::now() >= deadline {
                    return None;
                }
                let mut choice = None;
                for (k, &bits) in available.iter().enumerate() {
                    let mut bits = bits;
                    while bits != 0 {
                        let v = k * 64 + bits.trailing_zeros() as usize;
                        bits &= bits - 1;
                        let common: usize = available
                            .iter()
                            .zip(&graph.compatible[v])
                            .map(|(&a, &b)| (a & !b).count_ones() as usize)
                            .sum();
                        if choice.is_none_or(|(score, old)| {
                            common > score
                                || (common == score
                                    && (degree[v], graph.order[v])
                                        < (degree[old], graph.order[old]))
                        }) {
                            choice = Some((common, v));
                        }
                    }
                }
                let v = choice?.1;
                colors[v] = color;
                clear(&mut remaining, v);
                clear(&mut available, v);
                for (a, &b) in available.iter_mut().zip(&graph.compatible[v]) {
                    *a &= !b;
                }
            }
        }
    } else {
        let mut used = vec![vec![0u64; graph.words]; n];
        let mut saturation = vec![0usize; n];
        let degree: Vec<usize> = graph
            .compatible
            .iter()
            .map(|r| r.iter().map(|b| b.count_ones() as usize).sum())
            .collect();
        for _ in 0..n {
            if Instant::now() >= deadline {
                return None;
            }
            let v = (0..n)
                .filter(|&v| colors[v] == 0)
                .max_by_key(|&v| (saturation[v], degree[v], std::cmp::Reverse(graph.order[v])))?;
            let color = used[v].iter().enumerate().find_map(|(k, &bits)| {
                (bits != u64::MAX).then(|| k * 64 + (!bits).trailing_zeros() as usize)
            })?;
            colors[v] = color + 1;
            for u in 0..n {
                if colors[u] == 0 && graph.compatible[v][u / 64] & (1 << (u % 64)) != 0 {
                    let mask = 1 << (color % 64);
                    if used[u][color / 64] & mask == 0 {
                        used[u][color / 64] |= mask;
                        saturation[u] += 1;
                    }
                }
            }
        }
    }
    Some(colors)
}

/// 主解専用の独立診断。グラフ上界・証明・最適性は返さない。
pub fn diagnostic(model:&Model,seconds:f64,mode:&str)->serde_json::Value{diagnostic_seeded(model,seconds,mode,None)}
pub fn diagnostic_seeded(model:&Model,seconds:f64,mode:&str,point:Option<&[f64]>)->serde_json::Value{
 let start=Instant::now();let end=start+std::time::Duration::from_secs_f64(seconds);let graph=recognize(model,512*1024*1024,end).expect("recognized graph");let mut candidates=vec![u64::MAX;graph.words];if graph.n%64!=0{candidates[graph.words-1]=(1u64<<(graph.n%64))-1;}let mut seed=vec![];while let Some(v)=first(&candidates){seed.push(v);for(a,&b)in candidates.iter_mut().zip(&graph.compatible[v]){*a&=b;}}
 if let Some(point)=point{assert!(model.check(point,true,0.).is_ok());assert!(point.iter().all(|&v|v==0.||v==1.));seed=(0..graph.n).filter(|&v|point[graph.order[v]]==1.).collect();assert!(strict_clique(&graph,&seed));}
 let seed_end=end.min(Instant::now()+std::time::Duration::from_secs_f64(0.5));let(mut best,seed_stats)=improve_seed(&graph,&seed,seed_end,true,true);let mut rounds=vec![];
 for round in 0..32u64{if Instant::now()>=end{break;}let before=best.len();let rng=if mode=="reseed"{0xd1b54a32d192ed03u64^(round.wrapping_mul(0x9e3779b97f4a7c15))}else{0xd1b54a32d192ed03};let(next,stats)=weighted_cover::improve_seeded(&graph,&best,256*1024*1024,end,rng);assert!(strict_clique(&graph,&next));if next.len()>best.len(){best=next;}rounds.push(serde_json::json!({"round":round,"rng":rng,"before":before,"after":best.len(),"stats":stats}));if mode=="single"{break;}}
 assert!(strict_clique(&graph,&best));let component_seconds=start.elapsed().as_secs_f64();let mut point=vec![0.;model.variables.len()];for &v in &best{point[graph.order[v]]=1.;}assert!(model.check(&point,true,0.).is_ok());let objective=model.objective(&point);let values:std::collections::BTreeMap<_,_>=model.variables.iter().zip(point).map(|(v,x)|(v.name.clone(),x)).collect();serde_json::json!({"status":"primal_candidate","objective":objective,"dual_bound":null,"values":values,"component_seconds":component_seconds,"vertices":graph.n,"seed_stats":seed_stats,"rounds":rounds,"purpose":"standalone profile; no parent bound or optimality claim"})
}

fn union_bipartite_independent(adj: &[Vec<usize>], right: usize) -> (Vec<usize>,Vec<usize>,usize) {
    fn augment(u:usize,adj:&[Vec<usize>],seen:&mut[bool],mate:&mut[Option<usize>])->bool {
        for &v in &adj[u] {
            if seen[v] {continue;}
            seen[v]=true;
            let old=mate[v];
            if old.is_none() || augment(old.unwrap(),adj,seen,mate) {
                mate[v]=Some(u);return true;
            }
        }
        false
    }
    let mut mate_right=vec![None;right];
    let mut matching=0;
    for u in 0..adj.len() {
        if augment(u,adj,&mut vec![false;right],&mut mate_right) {matching+=1;}
    }
    let mut mate_left=vec![None;adj.len()];
    for (v,&u) in mate_right.iter().enumerate() {if let Some(u)=u {mate_left[u]=Some(v);}}
    let mut seen_left=vec![false;adj.len()];
    let mut seen_right=vec![false;right];
    let mut queue=std::collections::VecDeque::new();
    for u in 0..adj.len() {if mate_left[u].is_none(){seen_left[u]=true;queue.push_back(u);}}
    while let Some(u)=queue.pop_front() {
        for &v in &adj[u] {
            if mate_left[u]==Some(v) || seen_right[v] {continue;}
            seen_right[v]=true;
            if let Some(w)=mate_right[v] {if !seen_left[w]{seen_left[w]=true;queue.push_back(w);}}
        }
    }
    let left:Vec<_>=(0..adj.len()).filter(|&u|seen_left[u]).collect();
    let right:Vec<_>=(0..right).filter(|&v|!seen_right[v]).collect();
    assert_eq!(left.len()+right.len(),adj.len()+seen_right.len()-matching);
    assert!(left.iter().all(|&u|adj[u].iter().all(|v|!right.contains(v))));
    (left,right,matching)
}

/// Own feasible parents only. Exact matching is restricted to their bipartite union;
/// no global bound, infeasibility, or optimality is inferred from it.
pub fn diagnostic_recombine(model:&Model,seconds:f64,parents:&[Vec<f64>])->serde_json::Value {
    let started=Instant::now();
    let deadline=started+std::time::Duration::from_secs_f64(seconds);
    let graph=recognize(model,512*1024*1024,deadline).expect("strict uniform packing graph");
    let mut pool=Vec::new();
    let mut seen=std::collections::HashSet::new();
    for point in parents {
        assert_eq!(point.len(),model.variables.len());
        assert!(model.check(point,true,0.).is_ok());
        assert!(point.iter().all(|&x|x==0.||x==1.));
        let vertices:Vec<_>=(0..graph.n).filter(|&v|point[graph.order[v]]==1.).collect();
        assert!(strict_clique(&graph,&vertices));
        if seen.insert(vertices.clone()){pool.push(vertices);}
    }
    assert!(pool.len()>=2 && pool.len()<=32);
    let mut best=pool.iter().max_by_key(|v|v.len()).unwrap().clone();
    let initial=best.len();
    let mut records=Vec::new();
    for a in 0..pool.len() {
        for b in a+1..pool.len() {
            if Instant::now()>=deadline {break;}
            let mut in_a=vec![false;graph.n];let mut in_b=vec![false;graph.n];
            for &v in &pool[a]{in_a[v]=true;}
            for &v in &pool[b]{in_b[v]=true;}
            let common:Vec<_>=(0..graph.n).filter(|&v|in_a[v]&&in_b[v]).collect();
            let left:Vec<_>=(0..graph.n).filter(|&v|in_a[v]&&!in_b[v]).collect();
            let right:Vec<_>=(0..graph.n).filter(|&v|!in_a[v]&&in_b[v]).collect();
            let adj:Vec<Vec<usize>>=left.iter().map(|&u|right.iter().enumerate().filter_map(|(k,&v)|(!adjacent(&graph,u,v)).then_some(k)).collect()).collect();
            let edges:usize=adj.iter().map(|r|r.len()).sum();
            let (take_left,take_right,matching)=union_bipartite_independent(&adj,right.len());
            let mut candidate=common.clone();
            candidate.extend(take_left.into_iter().map(|k|left[k]));
            candidate.extend(take_right.into_iter().map(|k|right[k]));
            assert_eq!(candidate.len(),common.len()+left.len()+right.len()-matching);
            assert!(strict_clique(&graph,&candidate));
            let union_best=candidate.len();
            let mut available=vec![u64::MAX;graph.words];
            if graph.n%64!=0 {available[graph.words-1]=(1u64<<(graph.n%64))-1;}
            for &v in &candidate {for(a,&b)in available.iter_mut().zip(&graph.compatible[v]){*a&=b;}}
            while let Some(v)=first(&available) {
                candidate.push(v);
                for(a,&b)in available.iter_mut().zip(&graph.compatible[v]){*a&=b;}
            }
            assert!(strict_clique(&graph,&candidate));
            let mut point=vec![0.;graph.n];for &v in &candidate{point[graph.order[v]]=1.;}
            assert!(model.check(&point,true,0.).is_ok());
            records.push(serde_json::json!({"a":a,"b":b,"parent_sizes":[pool[a].len(),pool[b].len()],"common":common.len(),"left":left.len(),"right":right.len(),"edges":edges,"matching":matching,"union_best":union_best,"augmented":candidate.len()}));
            if candidate.len()>best.len(){best=candidate;}
        }
    }
    assert!(strict_clique(&graph,&best));
    let mut point=vec![0.;graph.n];for &v in &best{point[graph.order[v]]=1.;}
    assert!(model.check(&point,true,0.).is_ok());
    let values:std::collections::BTreeMap<_,_>=model.variables.iter().zip(point.iter()).map(|(v,&x)|(v.name.clone(),x)).collect();
    serde_json::json!({"result":{"status":"primal_candidate","objective":model.objective(&point),"dual_bound":null,"values":values,"purpose":"own-parent union primal recombination; no original global bound or optimality"},"initial":initial,"best":best.len(),"pool_size":pool.len(),"seconds":started.elapsed().as_secs_f64(),"records":records})
}

#[cfg(test)] mod union_recombine_tests {
    use super::*;
    #[test] fn all_small_bipartite_graphs_match_exhaustive_independence() {
        for l in 0..=3 {for r in 0..=3 {for mask in 0usize..(1usize<<(l*r)) {
            let adj:Vec<Vec<usize>>=(0..l).map(|u|(0..r).filter(|&v|mask&(1usize<<(u*r+v))!=0).collect()).collect();
            let (a,b,m)=union_bipartite_independent(&adj,r);
            let alpha=(0usize..(1usize<<(l+r))).filter(|&x|(0..l).all(|u|x&(1usize<<u)==0||adj[u].iter().all(|&v|x&(1usize<<(l+v))==0))).map(|x|x.count_ones()as usize).max().unwrap();
            assert_eq!(a.len()+b.len(),alpha);assert_eq!(alpha,l+r-m);
        }}}
    }
}

pub fn diagnostic_diverse(model:&Model,seconds:f64,parent:&[f64],output:&std::path::Path)->serde_json::Value {
    let start=Instant::now();let deadline=start+std::time::Duration::from_secs_f64(seconds);
    let graph=recognize(model,512*1024*1024,deadline).expect("strict graph");
    assert_eq!(parent.len(),graph.n);assert!(model.check(parent,true,0.).is_ok());assert!(parent.iter().all(|&x|x==0.||x==1.));
    let initial=parent.iter().filter(|&&x|x==1.).count();let mut parents=vec![parent.to_vec()];let mut records=Vec::new();
    let point_json=|point:&[f64]|{
        assert!(model.check(point,true,0.).is_ok());
        let values:std::collections::BTreeMap<_,_>=model.variables.iter().zip(point.iter()).map(|(v,&x)|(v.name.clone(),x)).collect();
        serde_json::json!({"status":"primal_candidate","objective":model.objective(point),"dual_bound":null,"values":values,"purpose":"own feasible neutral parent; no global proof"})
    };
    std::fs::create_dir_all(output).unwrap();
    std::fs::write(output.join("parent-000.json"),serde_json::to_vec_pretty(&point_json(parent)).unwrap()).unwrap();
    for round in 0..16 {
        if Instant::now()>=deadline {break;}
        let previous=parents.last().unwrap();let seed:Vec<_>=(0..graph.n).filter(|&v|previous[graph.order[v]]==1.).collect();
        assert!(strict_clique(&graph,&seed));
        let end=deadline.min(Instant::now()+std::time::Duration::from_secs(1));
        let (next,stats)=improve_seed(&graph,&seed,end,true,true);assert!(strict_clique(&graph,&next));assert!(next.len()>=seed.len());
        let mut point=vec![0.;graph.n];for &v in &next{point[graph.order[v]]=1.;}
        let value=point_json(&point);std::fs::write(output.join(format!("parent-{:03}.json",round+1)),serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        records.push(serde_json::json!({"round":round,"before":seed.len(),"after":next.len(),"changed_point":point!=*previous,"stats":stats}));parents.push(point);
    }
    let unique:std::collections::HashSet<_>=parents.iter().map(|p|(0..p.len()).filter(|&v|p[v]==1.).collect::<Vec<_>>()).collect();
    let neutral_best=parents.iter().map(|p|p.iter().filter(|&&x|x==1.).count()).max().unwrap();
    let remaining=deadline.saturating_duration_since(Instant::now()).as_secs_f64();
    let mut combined=if unique.len()>=2 && remaining>2. {
        diagnostic_recombine(model,remaining,&parents)
    } else {
        let best=parents.iter().max_by_key(|p|p.iter().filter(|&&x|x==1.).count()).unwrap();
        serde_json::json!({"result":point_json(best),"initial":neutral_best,"best":neutral_best,"pool_size":unique.len(),"records":[],"recombine_skipped":true})
    };
    std::fs::write(output.join("result.json"),serde_json::to_vec_pretty(&combined["result"]).unwrap()).unwrap();combined.as_object_mut().unwrap().remove("result");
    serde_json::json!({"original_seed_count":initial,"neutral_best":neutral_best,"unique_parents":unique.len(),"parents":parents.len(),"neutral_records":records,"combined":combined,"seconds":start.elapsed().as_secs_f64(),"original_global_bound_or_optimality":false})
}
#[cfg(test)]mod neutral_tests {
    use super::*;
    #[test]fn maximum_neutral_points_remain_feasible_without_false_improvement_counts(){
        let graph=Graph{n:4,words:1,compatible:vec![vec![12],vec![12],vec![3],vec![3]],order:(0..4).collect()};
        let(best,stats)=improve_seed(&graph,&[0,2],Instant::now()+std::time::Duration::from_secs(1),true,true);
        assert!(strict_clique(&graph,&best));assert_eq!(best.len(),2);assert_eq!(stats.best,2);assert_eq!(stats.last_improvement_iteration,0);assert!(stats.neutral_swaps>0);assert_eq!(stats.validation_failures,0);
    }
}

// Exact feasibility of a two-out/three-in move; this is only a primal diagnostic.
fn two_remove_move(graph:&Graph,seed:&[usize],deadline:Instant,max_work:usize)->(Option<Vec<usize>>,usize,bool){
 assert!(strict_clique(graph,seed));let k=seed.len();if k<2{return(None,0,true);}
 let mut selected=vec![false;graph.n];for &v in seed{selected[v]=true;}
 let mut zero=Vec::new();let mut single=vec![Vec::new();k];let mut double=std::collections::HashMap::<(usize,usize),Vec<usize>>::new();let mut work=0;
 for v in 0..graph.n{
  if Instant::now()>=deadline{return(None,work,false);}if selected[v]{continue;}let mut blockers=Vec::new();
  for(i,&u)in seed.iter().enumerate(){work+=1;if work>max_work{return(None,work,false);}if !adjacent(graph,v,u){blockers.push(i);if blockers.len()>2{break;}}}
  match blockers.as_slice(){[]=>zero.push(v),&[i]=>single[i].push(v),&[i,j]=>double.entry((i,j)).or_default().push(v),_=>{}}
 }
 for i in 0..k{for j in i+1..k{
  if Instant::now()>=deadline{return(None,work,false);}let mut free=zero.clone();free.extend_from_slice(&single[i]);free.extend_from_slice(&single[j]);if let Some(v)=double.get(&(i,j)){free.extend_from_slice(v);}
  for a in 0..free.len(){if Instant::now()>=deadline{return(None,work,false);}for b in a+1..free.len(){work+=1;if work>max_work||work%1024==0&&Instant::now()>=deadline{return(None,work,false);}if !adjacent(graph,free[a],free[b]){continue;}for c in b+1..free.len(){work+=1;if work>max_work||work%1024==0&&Instant::now()>=deadline{return(None,work,false);}if adjacent(graph,free[a],free[c])&&adjacent(graph,free[b],free[c]){let mut next:Vec<_>=seed.iter().enumerate().filter_map(|(p,&v)|(p!=i&&p!=j).then_some(v)).collect();next.extend([free[a],free[b],free[c]]);assert!(strict_clique(graph,&next));assert_eq!(next.len(),seed.len()+1);return(Some(next),work,false);}}}}
 }}(None,work,true)
}
pub fn diagnostic_two_remove(model:&Model,seconds:f64,parent:&[f64])->serde_json::Value{
 let start=Instant::now();let deadline=start+std::time::Duration::from_secs_f64(seconds);let graph=recognize(model,512*1024*1024,deadline).expect("strict packing graph");assert!(model.check(parent,true,0.).is_ok());assert!(parent.iter().all(|&x|x==0.||x==1.));let mut best:Vec<_>=(0..graph.n).filter(|&v|parent[graph.order[v]]==1.).collect();assert!(strict_clique(&graph,&best));let initial=best.len();let mut records=vec![];
 while Instant::now()<deadline{let(next,work,complete)=two_remove_move(&graph,&best,deadline,100_000_000);records.push(serde_json::json!({"before":best.len(),"work":work,"neighborhood_complete":complete,"improved":next.is_some()}));match next{Some(v)=>best=v,None=>break,}}
 let mut point=vec![0.;model.variables.len()];for &v in &best{point[graph.order[v]]=1.;}assert!(model.check(&point,true,0.).is_ok());let values:std::collections::BTreeMap<_,_>=model.variables.iter().zip(point.iter()).map(|(v,&x)|(v.name.clone(),x)).collect();serde_json::json!({"status":"primal_candidate","objective":model.objective(&point),"dual_bound":null,"values":values,"initial":initial,"best":best.len(),"records":records,"seconds":start.elapsed().as_secs_f64(),"purpose":"two-out three-in primal diagnostic; no global bound or optimality"})
}
#[cfg(test)]mod two_remove_tests{
 use super::*;
 #[test]fn exhaustive_small_neighborhood_existence_and_feasibility(){let mut rng=19u64;for n in 2usize..=9{for _ in 0..80{let mut g=Graph{n,words:1,compatible:vec![vec![0];n],order:(0..n).collect()};for a in 0..n{for b in a+1..n{rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;if rng&1!=0{g.compatible[a][0]|=1<<b;g.compatible[b][0]|=1<<a;}}}let seed:Vec<_>=(0..n).fold(Vec::new(),|mut s,v|{if s.iter().all(|&u|adjacent(&g,u,v)){s.push(v);}s});let mut exists=false;for i in 0..seed.len(){for j in i+1..seed.len(){let keep:Vec<_>=seed.iter().enumerate().filter_map(|(p,&v)|(p!=i&&p!=j).then_some(v)).collect();for a in 0..n{for b in a+1..n{for c in b+1..n{if [a,b,c].iter().any(|v|seed.contains(v)){continue;}let mut x=keep.clone();x.extend([a,b,c]);exists|=strict_clique(&g,&x);}}}}}let(next,_,complete)=two_remove_move(&g,&seed,Instant::now()+std::time::Duration::from_secs(2),1_000_000);assert_eq!(next.is_some(),exists);if let Some(x)=next{assert!(strict_clique(&g,&x));assert_eq!(x.len(),seed.len()+1);}else{assert!(complete);}}}}
 #[test]fn expired_or_work_limited_keeps_parent(){let g=Graph{n:5,words:1,compatible:vec![vec![2],vec![1],vec![24],vec![20],vec![12]],order:(0..5).collect()};let seed=[0,1];assert_eq!(two_remove_move(&g,&seed,Instant::now(),100).0,None);assert!(!two_remove_move(&g,&seed,Instant::now()+std::time::Duration::from_secs(1),0).2);let(x,_,_)=two_remove_move(&g,&seed,Instant::now()+std::time::Duration::from_secs(1),100);assert_eq!(x.unwrap(),vec![2,3,4]);}
}

fn three_remove_move(graph:&Graph,seed:&[usize],deadline:Instant,max_work:usize)->(Option<Vec<usize>>,usize,bool,usize){
 assert!(strict_clique(graph,seed));if seed.len()<3{return(None,0,true,0);}let mut selected=vec![false;graph.n];for &v in seed{selected[v]=true;}let mut groups=std::collections::BTreeMap::<Vec<usize>,Vec<usize>>::new();let mut work=0;
 for v in 0..graph.n{if Instant::now()>=deadline{return(None,work,false,0);}if selected[v]{continue;}let mut blockers=Vec::new();for(i,&u)in seed.iter().enumerate(){work+=1;if work>max_work{return(None,work,false,0);}if !adjacent(graph,v,u){blockers.push(i);if blockers.len()>3{break;}}}if blockers.len()<=3{groups.entry(blockers).or_default().push(v);}}
 let anchors:Vec<_>=groups.keys().filter(|v|v.len()==3).cloned().collect();let total=anchors.len();
 for removed in anchors{if Instant::now()>=deadline{return(None,work,false,total);}let mut free=Vec::new();for bits in 0usize..8{let key:Vec<_>=removed.iter().enumerate().filter_map(|(i,&v)|(bits&(1<<i)!=0).then_some(v)).collect();if let Some(v)=groups.get(&key){free.extend_from_slice(v);}}
  for a in 0..free.len(){if Instant::now()>=deadline{return(None,work,false,total);}for b in a+1..free.len(){work+=1;if work>max_work{return(None,work,false,total);}if !adjacent(graph,free[a],free[b]){continue;}for c in b+1..free.len(){work+=1;if work>max_work||work%1024==0&&Instant::now()>=deadline{return(None,work,false,total);}if !adjacent(graph,free[a],free[c])||!adjacent(graph,free[b],free[c]){continue;}for d in c+1..free.len(){work+=1;if work>max_work||work%1024==0&&Instant::now()>=deadline{return(None,work,false,total);}if [a,b,c].iter().all(|&k|adjacent(graph,free[k],free[d])){let mut next:Vec<_>=seed.iter().enumerate().filter_map(|(i,&v)|(!removed.contains(&i)).then_some(v)).collect();next.extend([free[a],free[b],free[c],free[d]]);assert!(strict_clique(graph,&next));assert_eq!(next.len(),seed.len()+1);return(Some(next),work,false,total);}}}}}
 }(None,work,true,total)
}
pub fn diagnostic_three_remove(model:&Model,seconds:f64,parent:&[f64])->serde_json::Value{
 let start=Instant::now();let deadline=start+std::time::Duration::from_secs_f64(seconds);let graph=recognize(model,512*1024*1024,deadline).expect("strict packing graph");assert!(model.check(parent,true,0.).is_ok());assert!(parent.iter().all(|&x|x==0.||x==1.));let mut best:Vec<_>=(0..graph.n).filter(|&v|parent[graph.order[v]]==1.).collect();let initial=best.len();let mut records=vec![];
 while Instant::now()<deadline{let(next,work,complete,anchors)=three_remove_move(&graph,&best,deadline,100_000_000);records.push(serde_json::json!({"before":best.len(),"work":work,"anchor_triples":anchors,"anchored_neighborhood_complete":complete,"improved":next.is_some()}));match next{Some(v)=>best=v,None=>break,}}
 let mut point=vec![0.;model.variables.len()];for &v in &best{point[graph.order[v]]=1.;}assert!(model.check(&point,true,0.).is_ok());let values:std::collections::BTreeMap<_,_>=model.variables.iter().zip(point.iter()).map(|(v,&x)|(v.name.clone(),x)).collect();serde_json::json!({"status":"primal_candidate","objective":model.objective(&point),"dual_bound":null,"values":values,"initial":initial,"best":best.len(),"records":records,"seconds":start.elapsed().as_secs_f64(),"purpose":"anchored three-out four-in primal diagnostic; no global bound or optimality"})
}
#[cfg(test)]mod three_remove_tests{
 use super::*;
 #[test]fn random_anchored_neighborhood_matches_independent_exhaustion(){let mut rng=71u64;for n in 3usize..=9{for _ in 0..100{let mut g=Graph{n,words:1,compatible:vec![vec![0];n],order:(0..n).collect()};for a in 0..n{for b in a+1..n{rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;if rng&1!=0{g.compatible[a][0]|=1<<b;g.compatible[b][0]|=1<<a;}}}let seed:Vec<_>=(0..n).fold(Vec::new(),|mut s,v|{if s.iter().all(|&u|adjacent(&g,u,v)){s.push(v);}s});let mut exists=false;for i in 0..seed.len(){for j in i+1..seed.len(){for k in j+1..seed.len(){let removed=[seed[i],seed[j],seed[k]];let anchor=(0..n).any(|v|!seed.contains(&v)&&seed.iter().copied().filter(|&u|!adjacent(&g,v,u)).collect::<Vec<_>>()==removed);if !anchor{continue;}let keep:Vec<_>=seed.iter().copied().filter(|v|!removed.contains(v)).collect();for mask in 0usize..1<<n{if mask.count_ones()!=4{continue;}let add:Vec<_>=(0..n).filter(|&v|mask&(1<<v)!=0).collect();if add.iter().any(|v|seed.contains(v)){continue;}let mut x=keep.clone();x.extend(add);exists|=strict_clique(&g,&x);}}}}let(next,_,complete,_)=three_remove_move(&g,&seed,Instant::now()+std::time::Duration::from_secs(2),1_000_000);assert_eq!(next.is_some(),exists);if let Some(x)=next{assert!(strict_clique(&g,&x));assert_eq!(x.len(),seed.len()+1);}else{assert!(complete);}}}}
 #[test]fn complete_bipartite_conflicts_reveal_three_out_four_in(){let g=Graph{n:7,words:1,compatible:vec![vec![6],vec![5],vec![3],vec![112],vec![104],vec![88],vec![56]],order:(0..7).collect()};let seed=[0,1,2];let(x,_,_,anchors)=three_remove_move(&g,&seed,Instant::now()+std::time::Duration::from_secs(1),1000);assert_eq!(anchors,1);assert_eq!(x.unwrap(),vec![3,4,5,6]);assert!(!three_remove_move(&g,&seed,Instant::now(),1000).2);assert!(!three_remove_move(&g,&seed,Instant::now()+std::time::Duration::from_secs(1),0).2);}
}
