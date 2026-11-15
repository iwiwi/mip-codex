use mip_codex::{
    model::{Model, Row, Variable},
    search::Strategy,
    solve::{Limits, solve},
};
#[test]
fn all_search_orders_match_binary_enumeration() {
    let mut seed = 2026111507u64;
    for case in 0..80 {
        let mut draw = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            (seed >> 32) as i64
        };
        let costs: Vec<_> = (0..8).map(|_| draw() % 17 - 7).collect();
        let weights: Vec<Vec<_>> = (0..3)
            .map(|_| (0..8).map(|_| draw() % 13 - 3).collect())
            .collect();
        let rhs: Vec<_> = (0..3).map(|_| draw() % 25 - 3).collect();
        let maximize = case % 2 == 0;
        let expected = (0..256)
            .filter_map(|mask| {
                if weights.iter().zip(&rhs).any(|(a, b)| {
                    (0..8)
                        .filter(|j| mask & (1 << j) != 0)
                        .map(|j| a[j])
                        .sum::<i64>()
                        > *b
                }) {
                    return None;
                }
                Some(
                    3 + (0..8)
                        .filter(|j| mask & (1 << j) != 0)
                        .map(|j| costs[j])
                        .sum::<i64>(),
                )
            })
            .reduce(|a, b| if maximize { a.max(b) } else { a.min(b) });
        let model = Model {
            name: "enumerated".into(),
            maximize,
            offset_enclosure: None,
            offset: 3.,
            variables: costs
                .iter()
                .enumerate()
                .map(|(j, c)| Variable {
                    lower_enclosure: None,
                    upper_enclosure: None,
                    name: format!("x{j}"),
                    lower: Some(0.),
                    upper: Some(1.),
                    integer: true,
                    objective_enclosure: None,
                    objective: *c as f64,
                })
                .collect(),
            rows: weights
                .iter()
                .zip(&rhs)
                .enumerate()
                .map(|(i, (a, b))| Row {
                    lower_enclosure: None,
                    upper_enclosure: None,
                    exact_integral: false,
                    name: format!("r{i}"),
                    lower: None,
                    upper: Some(*b as f64),
                    coefficients: a
                        .iter()
                        .enumerate()
                        .filter(|(_, a)| **a != 0)
                        .map(|(j, a)| (j, *a as f64))
                        .collect(),
                })
                .collect(),
        };
        for strategy in [Strategy::DepthFirst, Strategy::BestBound, Strategy::Hybrid] {
            let result = solve(
                &model,
                &Limits {
                    objective_rounding: false,
                    rins: false,
                    strong_branching: false,
                    cover_cuts: false,
                    search: strategy,
                    primal_repair: false,
                    primal_probe: false,
                    lp_diving: false,
                    ..Limits::default()
                },
            );
            assert_eq!(
                result.status,
                if expected.is_some() {
                    "optimal"
                } else {
                    "infeasible"
                },
                "case{case} {strategy:?} {result:?}"
            );
            assert_eq!(
                result.objective,
                expected.map(|x| x as f64),
                "case{case} {strategy:?}"
            );
            if let Some(expected) = expected {
                assert!((result.dual_bound.unwrap() - expected as f64).abs() < 1e-7);
            }
        }
    }
}
#[test]
fn interruption_keeps_all_pending_nodes_in_global_bound() {
    let model = Model {
        name: "parity".into(),
        maximize: false,
        offset_enclosure: None,
        offset: 0.,
        variables: (0..70)
            .map(|j| Variable {
                lower_enclosure: None,
                upper_enclosure: None,
                name: format!("x{j}"),
                lower: Some(0.),
                upper: Some(1.),
                integer: true,
                objective_enclosure: None,
                objective: 0.,
            })
            .collect(),
        rows: vec![Row {
            lower_enclosure: None,
            upper_enclosure: None,
            exact_integral: false,
            name: "half".into(),
            lower: Some(35.5),
            upper: Some(35.5),
            coefficients: (0..70).map(|j| (j, 1.)).collect(),
        }],
    };
    for strategy in [Strategy::DepthFirst, Strategy::BestBound, Strategy::Hybrid] {
        let result = solve(
            &model,
            &Limits {
                objective_rounding: false,
                rins: false,
                strong_branching: false,
                cover_cuts: false,
                search: strategy,
                nodes: 130,
                primal_repair: false,
                primal_probe: false,
                lp_diving: false,
                ..Limits::default()
            },
        );
        assert_eq!(result.status, "node_limit", "{strategy:?}: {result:?}");
        assert_eq!(result.nodes, 130);
        assert_eq!(result.objective, None);
        assert_eq!(result.dual_bound, Some(0.));
        assert!(result.max_open_nodes > 1);
        if matches!(strategy, Strategy::Hybrid) {
            assert!(result.search_transfers > 0 && result.search_best_pops > 0);
        }
    }
}
