//! Uniform-objective binary packing recognition and independently written coloring search.
//! Theory reference: Prosser, Algorithms 2012, doi:10.3390/a5040545; no source-code port.
use crate::model::Model;
use std::time::Instant;
#[derive(Debug, Default, serde::Serialize)]
pub struct Stats {
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
    if n == 0 || n > 4096 {
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
    let root = search.color(&all);
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
