use mip_codex::{
    mps,
    solve::{Limits, solve},
};
fn limits() -> Limits {
    Limits {
        box_primal: false,
        binary_graph: false,
        objective_rounding: false,
        rins: false,
        cover_cuts: false,
        clique_cuts: false,
        rounding_cuts: false,
        primal_repair: false,
        feasibility_pump: false,
        lp_diving: false,
        primal_probe: false,
        strong_branching: true,
        ..Limits::default()
    }
}
#[test]
fn infeasible_probes_do_not_prove_the_main_problem() {
    let model=mps::parse(std::io::Cursor::new("NAME strong\nROWS\n N obj\n E r\nCOLUMNS\n x obj 1 r 1\nRHS\n rhs r 0.4\nBOUNDS\n BV b x\nENDATA\n")).unwrap();
    let result = solve(
        &model,
        &Limits {
            box_primal: false,
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            nodes: 1,
            ..limits()
        },
    );
    assert_eq!(result.status, "node_limit");
    assert_eq!(result.objective, None);
    assert!((result.dual_bound.unwrap() - 0.4).abs() < 1e-14);
    assert_eq!(result.nodes, 1);
    assert_eq!(result.strong.infeasible, 2);
    assert_eq!(result.strong.calls, 1);
    assert_eq!(solve(&model, &limits()).status, "infeasible");
}
#[test]
fn optimal_probes_only_train_costs_and_preserve_the_parent_bound() {
    let model=mps::parse(std::io::Cursor::new("NAME strong\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1 r 2\n y obj -1 r 2\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n")).unwrap();
    let on = solve(
        &model,
        &Limits {
            box_primal: false,
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            nodes: 1,
            ..limits()
        },
    );
    let off = solve(
        &model,
        &Limits {
            box_primal: false,
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            nodes: 1,
            strong_branching: false,
            ..limits()
        },
    );
    assert_eq!(on.status, "node_limit");
    assert_eq!(on.objective, None);
    assert_eq!(on.dual_bound, off.dual_bound);
    assert!(on.strong.optimal >= 2);
    assert!(on.pseudocost_updates >= 2);
    assert_eq!(on.lp_solves, off.lp_solves + on.strong.lp_solves);
}
#[test]
fn expired_deadline_does_not_start_probing() {
    let model = mps::parse(std::io::Cursor::new(
        "NAME strong\nROWS\n N obj\nCOLUMNS\n x obj 1\nRHS\nBOUNDS\n BV b x\nENDATA\n",
    ))
    .unwrap();
    let result = solve(
        &model,
        &Limits {
            box_primal: false,
            binary_graph: false,
            objective_rounding: false,
            rins: false,
            seconds: 0.,
            ..limits()
        },
    );
    assert_eq!(result.status, "time_limit");
    assert_eq!(result.strong.calls, 0);
}
#[test]
fn strong_branching_matches_exhaustive_min_and_max_integer_optima() {
    let mut seed = 9833u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed
    };
    for case in 0..80 {
        let a: Vec<Vec<i64>> = (0..5)
            .map(|_| (0..6).map(|_| (next() % 11) as i64 - 5).collect())
            .collect();
        let b: Vec<i64> = (0..5).map(|_| (next() % 11) as i64 - 3).collect();
        let c: Vec<i64> = (0..6).map(|_| (next() % 11) as i64 - 5).collect();
        let mut text = format!(
            "NAME strong\nOBJSENSE\n {}\nROWS\n N obj\n",
            if case % 2 == 0 { "MIN" } else { "MAX" }
        );
        for i in 0..5 {
            text += &format!(" L r{i}\n");
        }
        text += "COLUMNS\n";
        for j in 0..6 {
            text += &format!(" x{j} obj {}\n", c[j]);
            for i in 0..5 {
                text += &format!(" x{j} r{i} {}\n", a[i][j]);
            }
        }
        text += "RHS\n";
        for i in 0..5 {
            text += &format!(" rhs r{i} {}\n", b[i]);
        }
        text += "BOUNDS\n";
        for j in 0..6 {
            text += &format!(" BV b x{j}\n");
        }
        text += "ENDATA\n";
        let model = mps::parse(std::io::Cursor::new(text)).unwrap();
        let mut expected: Option<i64> = None;
        for mask in 0..64 {
            let x: Vec<i64> = (0..6).map(|j| (mask >> j) & 1).collect();
            if (0..5).all(|i| a[i].iter().zip(&x).map(|(a, x)| a * x).sum::<i64>() <= b[i]) {
                let value: i64 = c.iter().zip(&x).map(|(c, x)| c * x).sum();
                expected = Some(expected.map_or(value, |old| {
                    if case % 2 == 0 {
                        old.min(value)
                    } else {
                        old.max(value)
                    }
                }));
            }
        }
        for (objective, hints) in [(false, false), (true, false), (false, true), (true, true)] {
            let result = solve(
                &model,
                &Limits {
                    box_primal: false,
                    binary_graph: false,
                    strong_objective: objective,
                    strong_infeasible_hint: hints,
                    ..limits()
                },
            );
            match expected {
                None => assert_eq!(
                    result.status, "infeasible",
                    "case{case} modes{objective}/{hints}"
                ),
                Some(value) => {
                    assert_eq!(
                        result.status, "optimal",
                        "case{case} modes{objective}/{hints}"
                    );
                    assert!((result.objective.unwrap() - value as f64).abs() < 1e-6);
                }
            }
        }
    }
}

#[test]
fn infeasible_hints_select_a_variable_but_do_not_prune_or_change_parent_bound() {
    let model=mps::parse(std::io::Cursor::new("NAME strong\nROWS\n N c\n E r\nCOLUMNS\n x c 1 r 1\nRHS\n rhs r .4\nBOUNDS\n BV b x\nENDATA\n")).unwrap();
    for mixed in [false, true] {
        let result = solve(
            &model,
            &Limits {
                box_primal: false,
                binary_graph: false,
                strong_objective: mixed,
                strong_infeasible_hint: true,
                nodes: 1,
                ..limits()
            },
        );
        assert_eq!(result.status, "node_limit");
        assert_eq!(result.nodes, 1);
        assert!((result.dual_bound.unwrap() - 0.4).abs() < 1e-14);
        assert_eq!(result.objective, None);
        assert_eq!(result.strong.selected_variable, Some(0));
        assert_eq!(result.strong.infeasible_hint_selections, 1);
        assert_eq!(result.strong.infeasible, 2);
    }
}
