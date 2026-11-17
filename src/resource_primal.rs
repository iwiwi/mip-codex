//! 主解専用の原行実行可能性MIP。局所状態/下界/基底は元探索へ返さない。
use super::*;
#[derive(Default, Debug, Serialize)]
pub struct Stats {
    pub calls: usize,
    pub solves: usize,
    pub nodes: usize,
    pub lp_solves: usize,
    pub rows_added: usize,
    pub max_rows: usize,
    pub candidates: usize,
    pub valid: usize,
    pub improvements: usize,
    pub seconds: f64,
}
pub(super) fn search(
    model: &Model,
    root: &Node,
    limits: &Limits,
    requested_fixed: bool,
    deadline: Instant,
    stats: &mut Stats,
    lp_stats: &mut revised::Stats,
    solves: &mut usize,
    fixed: &mut usize,
) -> Option<Vec<f64>> {
    let began = Instant::now();
    stats.calls += 1;
    let result = inner(
        model,
        root,
        limits,
        requested_fixed,
        deadline,
        stats,
        lp_stats,
        solves,
        fixed,
    );
    stats.seconds += began.elapsed().as_secs_f64();
    result
}
fn inner(
    model: &Model,
    root: &Node,
    limits: &Limits,
    requested_fixed: bool,
    deadline: Instant,
    stats: &mut Stats,
    lp_stats: &mut revised::Stats,
    solves: &mut usize,
    fixed: &mut usize,
) -> Option<Vec<f64>> {
    let mut point: Vec<_> = model
        .variables
        .iter()
        .enumerate()
        .map(|(j, v)| {
            let x = 0f64
                .max(root.lower[j].unwrap_or(f64::NEG_INFINITY))
                .min(root.upper[j].unwrap_or(f64::INFINITY));
            if v.integer { x.round() } else { x }
        })
        .collect();
    let mut active = vec![false; model.rows.len()];
    let mut indices = vec![];
    for _ in 0..32 {
        if Instant::now() >= deadline {
            return None;
        }
        let mut violated = vec![];
        for (i, row) in model.rows.iter().enumerate() {
            if i % 4096 == 0 && Instant::now() >= deadline {
                return None;
            }
            let value: f64 = row.coefficients.iter().map(|&(j, a)| a * point[j]).sum();
            if !value.is_finite() {
                return None;
            }
            let gap = row
                .lower
                .map_or(0., |b| (b - value).max(0.))
                .max(row.upper.map_or(0., |b| (value - b).max(0.)));
            if gap > 1e-7 && !active[i] {
                let norm = row
                    .coefficients
                    .iter()
                    .map(|&(_, a)| a.abs())
                    .sum::<f64>()
                    .max(1.);
                violated.push((i, gap / norm));
            }
        }
        if violated.is_empty() {
            if model.check(&point, true, 1e-7).is_ok() && model.objective(&point).is_finite() {
                stats.valid += 1;
                return Some(point);
            }
            return None;
        }
        violated.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        for (i, _) in violated.into_iter().take(512) {
            active[i] = true;
            indices.push(i);
            stats.rows_added += 1;
        }
        stats.max_rows = stats.max_rows.max(indices.len());
        let mut variables = model.variables.clone();
        for (j, v) in variables.iter_mut().enumerate() {
            if j % 1024 == 0 && Instant::now() >= deadline {
                return None;
            }
            v.objective = 0.;
            v.objective_enclosure = Some([0.; 2]);
            if v.lower != root.lower[j] {
                v.lower = root.lower[j];
                v.lower_enclosure = v.lower.map(|x| [x, x]);
            }
            if v.upper != root.upper[j] {
                v.upper = root.upper[j];
                v.upper_enclosure = v.upper.map(|x| [x, x]);
            }
        }
        let subset = Model {
            name: String::new(),
            maximize: false,
            offset: 0.,
            offset_enclosure: Some([0.; 2]),
            original_decimal: model.original_decimal,
            variables,
            rows: indices.iter().map(|&i| model.rows[i].clone()).collect(),
        };
        let mut local = limits.clone();
        local.resource_primal = false;
        local.row_generation = false;
        local.row_generation_report = false;
        local.lp_dual_root = false;
        local.zero_dominance = false;
        local.box_primal = false;
        local.primal_repair = false;
        local.primal_probe = false;
        local.lp_diving = false;
        local.objective_lp_dive = false;
        local.feasibility_pump = false;
        local.rins = false;
        local.terminal_neighborhood = false;
        local.binary_graph = false;
        local.cover_cuts = false;
        local.mir_cuts = false;
        local.gomory_cuts = false;
        local.aggregate_cuts = false;
        local.clique_cuts = false;
        local.three_row_cuts = false;
        local.rounding_cuts = false;
        local.strong_branching = false;
        local.propagation = false;
        local.objective_rounding = false;
        local.objective_rounding_all = false;
        local.objective_box_report = false;
        local.objective_domain_report = false;
        local.row_ray_report = false;
        local.row_packing_report = false;
        local.row_subgradient_report = false;
        local.nodes = 4096;
        if limits.submip_fixed_columns {
            local.fixed_columns = requested_fixed;
        }
        local.seconds = deadline
            .saturating_duration_since(Instant::now())
            .as_secs_f64()
            .min(0.25);
        if local.seconds <= 0. {
            return None;
        }
        let r = solve(&subset, &local);
        stats.solves += 1;
        stats.nodes += r.nodes;
        stats.lp_solves += r.lp_solves;
        *solves += r.lp_solves;
        *fixed += r.root_fixed_columns;
        add_lp_result(lp_stats, &r);
        let values = r.values?;
        point = model
            .variables
            .iter()
            .map(|v| values.get(&v.name).copied())
            .collect::<Option<Vec<_>>>()?;
        stats.candidates += 1;
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    fn example(maximize: bool) -> Model {
        let text = format!(
            "NAME primal\nOBJSENSE\n {}\nROWS\n N c\n G need\n L cap\nCOLUMNS\n x c {} need 1\n x cap 1\n y c {} need 1\n z c 0\nRHS\n rhs need 1 cap 0\n rhs c -7\nBOUNDS\n BV b x\n BV b y\n FX b z 0\nENDATA\n",
            if maximize { "MAX" } else { "MIN" },
            if maximize { 2 } else { -2 },
            if maximize { 1 } else { -1 }
        );
        crate::mps::parse(std::io::Cursor::new(text)).unwrap()
    }
    fn node(m: &Model) -> Node {
        Node {
            certified_bound: None,
            lower: m.variables.iter().map(|v| v.lower).collect(),
            upper: m.variables.iter().map(|v| v.upper).collect(),
            bound: f64::NEG_INFINITY,
            warm: None,
            changed: None,
            edge: None,
        }
    }
    #[test]
    fn missing_original_rows_are_added_and_only_original_feasible_points_escape() {
        for maximize in [false, true] {
            for fixed in [false, true] {
                let m = example(maximize);
                let n = node(&m);
                let mut stats = Stats::default();
                let mut lp = revised::Stats::default();
                let mut solves = 0;
                let mut columns = 0;
                let p = search(
                    &m,
                    &n,
                    &Limits {
                        fixed_columns: fixed,
                        ..Limits::default()
                    },
                    fixed,
                    Instant::now() + std::time::Duration::from_secs(2),
                    &mut stats,
                    &mut lp,
                    &mut solves,
                    &mut columns,
                )
                .unwrap();
                assert_eq!(p, [0., 1., 0.]);
                assert!(m.check(&p, true, 0.).is_ok());
                assert_eq!(m.objective(&p), if maximize { 8. } else { 6. });
                assert_eq!(stats.solves, 2);
                assert_eq!(stats.rows_added, 2);
                assert_eq!(stats.valid, 1);
                assert_eq!(columns, if fixed { 2 } else { 0 });
                assert_eq!(n.upper[0], Some(1.));
                assert!(n.warm.is_none() && n.certified_bound.is_none());
                assert_eq!(m.variables[0].upper, Some(1.));
            }
        }
    }
    #[test]
    fn expired_deadline_never_solves_or_returns_a_primal() {
        let m = example(false);
        let mut stats = Stats::default();
        let p = search(
            &m,
            &node(&m),
            &Limits::default(),
            true,
            Instant::now(),
            &mut stats,
            &mut revised::Stats::default(),
            &mut 0,
            &mut 0,
        );
        assert!(p.is_none());
        assert_eq!(stats.solves, 0);
        assert_eq!(stats.valid, 0);
    }
    #[test]
    fn a_box_candidate_needs_no_local_solve_but_still_checks_the_original_model() {
        let m=crate::mps::parse(std::io::Cursor::new("NAME box\nROWS\n N c\n L cap\nCOLUMNS\n x c -1 cap 1\nRHS\n rhs cap 1\nBOUNDS\n BV b x\nENDATA\n")).unwrap();
        let mut stats = Stats::default();
        let p = search(
            &m,
            &node(&m),
            &Limits::default(),
            true,
            Instant::now() + std::time::Duration::from_secs(1),
            &mut stats,
            &mut revised::Stats::default(),
            &mut 0,
            &mut 0,
        )
        .unwrap();
        assert_eq!(p, [0.]);
        assert_eq!(stats.solves, 0);
        assert_eq!(stats.valid, 1);
        assert!(m.check(&p, true, 0.).is_ok());
    }
}
