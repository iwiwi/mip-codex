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

fn lns_random(rng:&mut u64)->usize {
    *rng^=*rng<<13;*rng^=*rng>>7;*rng^=*rng<<17;*rng as usize
}
fn lns_allowed(neighbors:&[Vec<usize>],selected:&[bool],dropped:&[bool],cap:usize)->Vec<usize> {
    let mut free:Vec<_>=(0..selected.len()).filter(|&v|selected[v]&&dropped[v]).collect();
    assert!(free.len()<=cap);
    for v in 0..selected.len() {
        if !selected[v]&&neighbors[v].iter().all(|&u|!selected[u]||dropped[u]) {
            if free.len()==cap {break;}
            free.push(v);
        }
    }
    free.sort_unstable_by_key(|&v|(neighbors[v].len(),v));
    free
}
fn lns_seeded_core(graph:&Graph,seed:&[usize],deadline:Instant)->(Vec<usize>,serde_json::Value) {
    assert!(strict_clique(graph,seed));
    let started=Instant::now();
    let mut all=vec![u64::MAX;graph.words];
    if graph.n%64!=0 {all[graph.words-1]=(1u64<<(graph.n%64))-1;}
    let mut greedy=Vec::new();let mut available=all.clone();
    while let Some(v)=first(&available) {
        greedy.push(v);
        for(a,&b)in available.iter_mut().zip(&graph.compatible[v]){*a&=b;}
    }
    let best=if greedy.len()>seed.len(){greedy}else{seed.to_vec()};
    let mut search=Search{graph,deadline,limit:2000000,started,best,stats:Stats{vertices:graph.n,root_upper:graph.n,remaining_upper:graph.n,order:(0..graph.n).collect(),..Stats::default()},interrupted:false};
    if let Some(root)=search.color(&all) {
        if search.best.len()<search.stats.root_upper {search.expand(all,&mut Vec::new(),Some(root));}
    } else {search.interrupted=true;}
    assert!(strict_clique(graph,&search.best));
    let complete=!search.interrupted||search.best.len()>=search.stats.remaining_upper;
    let record=serde_json::json!({"nodes":search.stats.nodes,"best":search.best.len(),"root_upper":search.stats.root_upper,"remaining_upper":search.stats.remaining_upper,"complete_restricted_only":complete,"seconds":started.elapsed().as_secs_f64()});
    (search.best,record)
}
fn lns_point(model:&Model,graph:&Graph,selected:&[bool])->serde_json::Value {
    let mut point=vec![0.;graph.n];
    for v in 0..graph.n {if selected[v]{point[graph.order[v]]=1.;}}
    assert!(model.check(&point,true,0.).is_ok());
    let values:std::collections::BTreeMap<_,_>=model.variables.iter().zip(point.iter()).map(|(v,&x)|(v.name.clone(),x)).collect();
    serde_json::json!({"status":"primal_candidate","objective":model.objective(&point),"dual_bound":null,"values":values,"purpose":"own Rust restricted-neighborhood primal only; no original global bound or optimality"})
}
fn lns_write_restricted(graph:&Graph,take:&[usize],path:&std::path::Path) {
    use std::io::Write;
    let mut edges=Vec::new();let mut columns=vec![Vec::new();graph.n];
    for a in 0..graph.n {for b in a+1..graph.n {
        if !adjacent(graph,a,b){let k=edges.len();edges.push((a,b));columns[a].push(k);columns[b].push(k);}
    }}
    let mut file=std::io::BufWriter::new(std::fs::File::create(path).unwrap());
    writeln!(file,"NAME neighborhood\nROWS\n N obj").unwrap();
    for k in 0..edges.len(){writeln!(file," L r{k}").unwrap();}
    writeln!(file,"COLUMNS").unwrap();
    for v in 0..graph.n {
        writeln!(file," x{v} obj -1").unwrap();
        for &k in &columns[v]{writeln!(file," x{v} r{k} 1").unwrap();}
    }
    writeln!(file,"RHS").unwrap();
    for k in 0..edges.len(){writeln!(file," rhs r{k} 1").unwrap();}
    writeln!(file,"BOUNDS").unwrap();
    for v in 0..graph.n{writeln!(file," BV b x{v}").unwrap();}
    writeln!(file,"ENDATA").unwrap();file.flush().unwrap();
    let values:std::collections::BTreeMap<_,_>=(0..graph.n).map(|v|(format!("x{v}"),if take.contains(&v){1}else{0})).collect();
    let point=serde_json::json!({"status":"primal_candidate","objective":-(take.len()as f64),"dual_bound":null,"values":values,"purpose":"restricted model feasible point only"});
    std::fs::write(path.with_extension("json"),serde_json::to_vec_pretty(&point).unwrap()).unwrap();
}

pub fn diagnostic_lns(model:&Model,seconds:f64,parent:&[f64],output:&std::path::Path)->serde_json::Value {
    let began=Instant::now();let deadline=began+std::time::Duration::from_secs_f64(seconds);
    let graph=recognize(model,512*1024*1024,deadline).expect("strict packing graph");
    assert_eq!(parent.len(),graph.n);assert!(model.check(parent,true,0.).is_ok());assert!(parent.iter().all(|&x|x==0.||x==1.));
    let mut selected:Vec<_>=(0..graph.n).map(|v|parent[graph.order[v]]==1.).collect();
    let initial=selected.iter().filter(|&&x|x).count();assert!(initial>0);
    let mut neighbors=vec![Vec::new();graph.n];
    for v in 0..graph.n {for(k,&word)in graph.compatible[v].iter().enumerate(){
        let mut bits=!word;
        if k+1==graph.words&&graph.n%64!=0 {bits&=(1u64<<(graph.n%64))-1;}
        if k==v/64 {bits&=!(1u64<<(v%64));}
        while bits!=0 {let u=k*64+bits.trailing_zeros()as usize;bits&=bits-1;neighbors[v].push(u);}
    }}
    let mut rng=0xa0761d6478bd642fu64;let mut records=Vec::new();let mut improvements=0;
    std::fs::create_dir_all(output).unwrap();
    for trial in 0..200 {
        if Instant::now()>=deadline {break;}
        let chosen:Vec<_>=(0..graph.n).filter(|&v|selected[v]).collect();
        let blockers:Vec<Vec<usize>>=(0..graph.n).map(|v|neighbors[v].iter().copied().filter(|&u|selected[u]).collect()).collect();
        let anchors:Vec<_>=(0..graph.n).filter(|&v|!selected[v]&&blockers[v].len()<=8).collect();
        if anchors.is_empty(){break;}
        let anchor=anchors[lns_random(&mut rng)%anchors.len()];
        let target=[32usize,64,128][trial%3].min(chosen.len());
        let mut dropped=vec![false;graph.n];let mut removed=Vec::new();
        for &u in &blockers[anchor]{if !dropped[u]{dropped[u]=true;removed.push(u);}}
        for _ in 0..target*32+128 {
            if removed.len()>=target {break;}
            let v=if removed.is_empty(){chosen[lns_random(&mut rng)%chosen.len()]}else{removed[lns_random(&mut rng)%removed.len()]};
            if !dropped[v]{dropped[v]=true;removed.push(v);}
            if !neighbors[v].is_empty(){
                let w=neighbors[v][lns_random(&mut rng)%neighbors[v].len()];
                for &u in &blockers[w]{if !dropped[u]{dropped[u]=true;removed.push(u);}}
            }
        }
        for &v in &chosen {if removed.len()<target&&!dropped[v]{dropped[v]=true;removed.push(v);}}
        let free=lns_allowed(&neighbors,&selected,&dropped,1024);let n=free.len();let words=n.div_ceil(64);
        let mut compatible=vec![vec![0u64;words];n];
        for a in 0..n {for b in 0..n {if a!=b&&adjacent(&graph,free[a],free[b]){compatible[a][b/64]|=1u64<<(b%64);}}}
        let core=Graph{n,words,compatible,order:(0..n).collect()};
        let seed:Vec<_>=(0..n).filter(|&v|selected[free[v]]).collect();assert_eq!(seed.len(),removed.len());
        let end=deadline.min(Instant::now()+std::time::Duration::from_secs(2));
        let (take,local)=lns_seeded_core(&core,&seed,end);
        let before=chosen.len();let mut candidate=selected.clone();
        for &v in &removed{candidate[v]=false;}
        for &v in &take{candidate[free[v]]=true;}
        let candidate_vertices:Vec<_>=(0..graph.n).filter(|&v|candidate[v]).collect();
        assert!(strict_clique(&graph,&candidate_vertices));
        let point=lns_point(model,&graph,&candidate);let after=candidate_vertices.len();
        assert!(after>=before);
        let improved=after>before;
        if improved {
            improvements+=1;selected=candidate;
            let name=format!("improvement-{improvements:03}");
            std::fs::write(output.join(format!("{name}-original.json")),serde_json::to_vec_pretty(&point).unwrap()).unwrap();
            lns_write_restricted(&core,&take,&output.join(format!("{name}-restricted.mps")));
            std::fs::write(output.join(format!("{name}-mapping.json")),serde_json::to_vec_pretty(&serde_json::json!({"free_original_variable_indices":free.iter().map(|&v|graph.order[v]).collect::<Vec<_>>(),"retained_original_variable_indices":chosen.iter().filter(|&&v|!dropped[v]).map(|&v|graph.order[v]).collect::<Vec<_>>()})).unwrap()).unwrap();
        }
        records.push(serde_json::json!({"trial":trial,"anchor":graph.order[anchor],"target":target,"removed":removed.len(),"free":n,"before":before,"after":after,"improved":improved,"restricted":local}));
    }
    let best=selected.iter().filter(|&&x|x).count();
    let point=lns_point(model,&graph,&selected);
    std::fs::write(output.join("result.json"),serde_json::to_vec_pretty(&point).unwrap()).unwrap();
    serde_json::json!({"initial":initial,"best":best,"improvements":improvements,"seconds":began.elapsed().as_secs_f64(),"records":records,"original_global_bound_or_optimality":false})
}

#[cfg(test)] mod lns_tests {
    use super::*;
    fn graph(n:usize,random:&mut u64)->Graph {
        let mut compatible=vec![vec![0u64;n.div_ceil(64)];n];
        for a in 0..n{for b in a+1..n{if lns_random(random)&1!=0{compatible[a][b/64]|=1u64<<(b%64);compatible[b][a/64]|=1u64<<(a%64);}}}
        Graph{n,words:n.div_ceil(64),compatible,order:(0..n).collect()}
    }
    #[test] fn seeded_core_matches_exhaustive_optima() {
        let mut random=0x123456789abcdefu64;
        for case in 0..200 {
            let g=graph(1+case%9,&mut random);
            let feasible:Vec<_>=(0usize..1usize<<g.n).filter(|&bits|{let v:Vec<_>=(0..g.n).filter(|&v|bits&(1<<v)!=0).collect();strict_clique(&g,&v)}).collect();
            let optimum=feasible.iter().map(|b|b.count_ones()as usize).max().unwrap();
            let bits=feasible[lns_random(&mut random)%feasible.len()];let seed:Vec<_>=(0..g.n).filter(|&v|bits&(1<<v)!=0).collect();
            let (best,_)=lns_seeded_core(&g,&seed,Instant::now()+std::time::Duration::from_secs(2));
            assert!(strict_clique(&g,&best));assert_eq!(best.len(),optimum);
        }
    }
    #[test] fn every_restricted_independent_set_expands_to_original_feasibility() {
        let mut random=0x9988776655443322u64;
        for case in 0..200 {
            let g=graph(1+case%9,&mut random);let mut chosen=Vec::new();
            for v in 0..g.n {if chosen.iter().all(|&u|adjacent(&g,u,v)){chosen.push(v);}}
            let selected:Vec<_>=(0..g.n).map(|v|chosen.contains(&v)).collect();
            let dropped:Vec<_>=(0..g.n).map(|v|selected[v]&&lns_random(&mut random)&1!=0).collect();
            let neighbors:Vec<Vec<usize>>=(0..g.n).map(|v|(0..g.n).filter(|&u|u!=v&&!adjacent(&g,u,v)).collect()).collect();
            let free=lns_allowed(&neighbors,&selected,&dropped,g.n);
            for bits in 0usize..1usize<<free.len() {
                let part:Vec<_>=(0..free.len()).filter(|&k|bits&(1<<k)!=0).map(|k|free[k]).collect();
                if strict_clique(&g,&part){let mut full:Vec<_>=(0..g.n).filter(|&v|selected[v]&&!dropped[v]).collect();full.extend(part);assert!(strict_clique(&g,&full));}
            }
        }
    }
}

fn parent_union_domain(parents:&[Vec<usize>],n:usize)->(Vec<usize>,Vec<usize>) {
    let mut counts=vec![0usize;n];
    for parent in parents {for &v in parent{counts[v]+=1;}}
    ((0..n).filter(|&v|counts[v]==parents.len()).collect(),(0..n).filter(|&v|counts[v]>0&&counts[v]<parents.len()).collect())
}
pub fn diagnostic_parent_lns(model:&Model,seconds:f64,parents:&[Vec<f64>],output:&std::path::Path)->serde_json::Value {
    let began=Instant::now();let deadline=began+std::time::Duration::from_secs_f64(seconds);
    let graph=recognize(model,512*1024*1024,deadline).expect("strict packing graph");
    let mut pool=Vec::new();let mut seen=std::collections::HashSet::new();
    for point in parents {
        assert_eq!(point.len(),graph.n);assert!(model.check(point,true,0.).is_ok());assert!(point.iter().all(|&x|x==0.||x==1.));
        let v:Vec<_>=(0..graph.n).filter(|&v|point[graph.order[v]]==1.).collect();assert!(strict_clique(&graph,&v));if seen.insert(v.clone()){pool.push(v);}
    }
    assert!(pool.len()>=3&&pool.len()<=32);
    let mut best=pool.iter().max_by_key(|p|p.len()).unwrap().clone();let initial=best.len();
    let mut pairs:Vec<_>=(0..pool.len()).flat_map(|a|(a+1..pool.len()).map(move|b|(a,b))).collect();
    let mut rng=0xe7037ed1a0b428dbu64;
    for k in(1..pairs.len()).rev(){let j=lns_random(&mut rng)%(k+1);pairs.swap(k,j);}
    let mut records=Vec::new();let mut improvements=0;std::fs::create_dir_all(output).unwrap();
    for (trial,&(a,b))in pairs.iter().take(60).enumerate(){
        if Instant::now()>=deadline {break;}
        if best==pool[a]||best==pool[b]{continue;}
        let (common,available)=parent_union_domain(&[best.clone(),pool[a].clone(),pool[b].clone()],graph.n);
        let mut in_best=vec![false;graph.n];let mut fixed=vec![false;graph.n];for &v in &best{in_best[v]=true;}for &v in &common{fixed[v]=true;}
        let mut free:Vec<_>=best.iter().copied().filter(|&v|!fixed[v]).collect();let seed_size=free.len();
        if free.len()>2048{records.push(serde_json::json!({"trial":trial,"a":a,"b":b,"skipped":"seed exceeds free cap"}));continue;}
        for v in available{if !in_best[v]&&free.len()<2048{free.push(v);}}
        free.sort_unstable();let n=free.len();let words=n.div_ceil(64);let mut compatible=vec![vec![0u64;words];n];
        for u in 0..n{for v in 0..n{if u!=v&&adjacent(&graph,free[u],free[v]){compatible[u][v/64]|=1u64<<(v%64);}}}
        let core=Graph{n,words,compatible,order:(0..n).collect()};let seed:Vec<_>=(0..n).filter(|&v|in_best[free[v]]).collect();assert_eq!(seed.len(),seed_size);
        let end=deadline.min(Instant::now()+std::time::Duration::from_secs(10));let(take,restricted)=lns_seeded_core(&core,&seed,end);
        let mut candidate=common.clone();candidate.extend(take.iter().map(|&v|free[v]));assert!(strict_clique(&graph,&candidate));assert!(candidate.len()>=best.len());
        let mut mask=vec![false;graph.n];for &v in &candidate{mask[v]=true;}let point=lns_point(model,&graph,&mask);
        let before=best.len();let after=candidate.len();let improved=after>before;
        if improved {
            improvements+=1;best=candidate;best.sort_unstable();let name=format!("improvement-{improvements:03}");
            std::fs::write(output.join(format!("{name}-original.json")),serde_json::to_vec_pretty(&point).unwrap()).unwrap();
            lns_write_restricted(&core,&take,&output.join(format!("{name}-restricted.mps")));
            std::fs::write(output.join(format!("{name}-mapping.json")),serde_json::to_vec_pretty(&serde_json::json!({"free_original_variable_indices":free.iter().map(|&v|graph.order[v]).collect::<Vec<_>>(),"retained_original_variable_indices":common.iter().map(|&v|graph.order[v]).collect::<Vec<_>>()})).unwrap()).unwrap();
        }
        records.push(serde_json::json!({"trial":trial,"a":a,"b":b,"common":common.len(),"free":n,"seed_size":seed_size,"before":before,"after":after,"improved":improved,"restricted":restricted}));
    }
    let mut mask=vec![false;graph.n];for &v in &best{mask[v]=true;}let point=lns_point(model,&graph,&mask);
    std::fs::write(output.join("result.json"),serde_json::to_vec_pretty(&point).unwrap()).unwrap();
    serde_json::json!({"initial":initial,"best":best.len(),"pool_size":pool.len(),"improvements":improvements,"records":records,"seconds":began.elapsed().as_secs_f64(),"original_global_bound_or_optimality":false})
}
#[cfg(test)]mod parent_lns_tests {
    use super::*;
    #[test]fn every_triplet_union_independent_set_expands_with_the_common_points(){
        let mut rng=0x76d532e5a8f3b96du64;
        for case in 0..100 {
            let n=1+case%9;let mut compatible=vec![vec![0u64;1];n];for a in 0..n{for b in a+1..n{if lns_random(&mut rng)&1!=0{compatible[a][0]|=1u64<<b;compatible[b][0]|=1u64<<a;}}}
            let graph=Graph{n,words:1,compatible,order:(0..n).collect()};
            let sets:Vec<Vec<usize>>=(0usize..1usize<<n).map(|bits|(0..n).filter(|&v|bits&(1<<v)!=0).collect::<Vec<_>>()).filter(|v|strict_clique(&graph,v)).collect();
            for _ in 0..10 {
                let parents:Vec<_>=(0..3).map(|_|sets[lns_random(&mut rng)%sets.len()].clone()).collect();let(common,free)=parent_union_domain(&parents,n);
                assert!((0..n).all(|v|common.contains(&v)==parents.iter().all(|p|p.contains(&v))));
                assert!((0..n).all(|v|common.contains(&v)||free.contains(&v)==parents.iter().any(|p|p.contains(&v))));
                for bits in 0usize..1usize<<free.len(){let part:Vec<_>=(0..free.len()).filter(|&k|bits&(1<<k)!=0).map(|k|free[k]).collect();if strict_clique(&graph,&part){let mut full=common.clone();full.extend(part);assert!(strict_clique(&graph,&full));}}
            }
        }
    }
}
