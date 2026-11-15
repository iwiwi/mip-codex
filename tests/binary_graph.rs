use mip_codex::{
    binary_graph, mps,
    solve::{Limits, solve},
};
use std::{
    io::Cursor,
    time::{Duration, Instant},
};
fn model(n: usize, edges: &[(usize, usize)], maximize: bool) -> mip_codex::model::Model {
    let mut lines = vec![
        "NAME graph".into(),
        "OBJSENSE".into(),
        if maximize {
            " MAX".into()
        } else {
            " MIN".into()
        },
        "ROWS".into(),
        " N obj".into(),
    ];
    for k in 0..edges.len() {
        lines.push(format!(" L r{k}"));
    }
    lines.push("COLUMNS".into());
    for j in 0..n {
        lines.push(format!(" x{j} obj {}", if maximize { 3 } else { -3 }));
        for (k, &(a, b)) in edges.iter().enumerate() {
            if j == a || j == b {
                lines.push(format!(" x{j} r{k} 1"));
            }
        }
    }
    lines.push("RHS".into());
    lines.push(" rhs obj -7".into());
    for k in 0..edges.len() {
        lines.push(format!(" rhs r{k} 1"));
    }
    lines.push("BOUNDS".into());
    for j in 0..n {
        lines.push(format!(" BV b x{j}"));
    }
    lines.push("ENDATA".into());
    mps::parse(Cursor::new(lines.join("\n") + "\n")).unwrap()
}
#[test]
fn exact_search_and_interruption_bounds_match_enumeration_and_root_coloring() {
    let mut rng = 2026111509u64;
    for case in 0..300 {
        let n = 1 + case % 10;
        let mut edges = Vec::new();
        for a in 0..n {
            for b in a + 1..n {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                if rng % 10 < (case % 9 + 1) as u64 {
                    edges.push((a, b));
                }
            }
        }
        let expected = (0usize..1 << n)
            .filter(|&mask| {
                edges
                    .iter()
                    .all(|&(a, b)| mask & (1 << a) == 0 || mask & (1 << b) == 0)
            })
            .map(|mask| mask.count_ones() as usize)
            .max()
            .unwrap();
        let model = model(n, &edges, case % 2 == 0);
        for (limit, local, root) in [0, 1, 2, 3, 7, usize::MAX]
            .into_iter()
            .flat_map(|limit| [false, true].map(|local| (limit, local)))
            .flat_map(|(limit, local)| [false, true].map(|root| (limit, local, root)))
        {
            let r = binary_graph::solve_with_options(
                &model,
                512 * 1024 * 1024,
                Instant::now() + Duration::from_secs(1),
                limit,
                local,
                root,
            )
            .unwrap();
            if let Some(local) = &r.stats.local {
                assert!(local.best >= local.initial);
                assert_eq!(local.validation_failures, 0);
            }
            assert!(model.check(&r.point, true, 1e-9).is_ok());
            assert_eq!(r.point.iter().sum::<f64>() as usize, r.stats.best);
            assert!(r.stats.best <= expected && expected <= r.stats.upper);
            assert_eq!(r.stats.conflict_edges, edges.len());
            assert_eq!(r.stats.root_colors.len(), n);
            for a in 0..n {
                for b in a + 1..n {
                    if r.stats.root_colors[a] == r.stats.root_colors[b] {
                        assert!(edges.contains(&(a, b)));
                    }
                }
            }
            if r.stats.complete {
                assert_eq!(r.stats.best, expected);
                assert_eq!(r.status, "optimal");
            }
            if limit == usize::MAX {
                assert!(r.stats.complete);
            }
        }
    }
}
#[test]
fn dispatch_reports_the_original_objective_and_uses_no_lp() {
    let edges = [(0, 1), (0, 2), (1, 2)];
    for maximize in [false, true] {
        let model = model(3, &edges, maximize);
        let r = solve(
            &model,
            &Limits {
                binary_graph: true,
                ..Limits::default()
            },
        );
        assert_eq!(r.lp_engine, "BinaryGraph");
        assert_eq!(r.lp_solves, 0);
        assert_eq!(r.status, "optimal");
        assert_eq!(r.objective, Some(if maximize { 10. } else { 4. }));
        assert_eq!(r.objective, r.dual_bound);
        assert!(r.binary_graph.unwrap().complete);
    }
}
#[test]
fn recognition_rejects_constraints_and_objectives_outside_the_exact_class() {
    let base = model(3, &[(0, 1), (1, 2)], false);
    let deadline = || Instant::now() + Duration::from_secs(1);
    let mut m = base.clone();
    m.variables[0].integer = false;
    assert!(binary_graph::solve(&m, 1 << 24, deadline(), usize::MAX).is_none());
    let mut m = base.clone();
    m.variables[0].objective = -4.;
    assert!(binary_graph::solve(&m, 1 << 24, deadline(), usize::MAX).is_none());
    let mut m = base.clone();
    m.rows[0].lower = Some(1.);
    m.rows[0].lower_enclosure = Some([1., 1.]);
    assert!(binary_graph::solve(&m, 1 << 24, deadline(), usize::MAX).is_none());
    let mut m = base.clone();
    m.rows[0].exact_coefficients = false;
    m.rows[0].exact_integral = false;
    assert!(binary_graph::solve(&m, 1 << 24, deadline(), usize::MAX).is_none());
    let m=mps::parse(Cursor::new("NAME lower\nROWS\n N c\n G r\nCOLUMNS\n x c -1 r -1\nRHS\n rhs r 0\nBOUNDS\n BV b x\nENDATA\n")).unwrap();
    assert!(binary_graph::solve(&m, 1 << 24, deadline(), usize::MAX).is_none());
    assert!(binary_graph::solve(&base, 1, deadline(), usize::MAX).is_none());
    assert!(
        binary_graph::solve(
            &base,
            1 << 24,
            Instant::now() - Duration::from_secs(1),
            usize::MAX
        )
        .is_none()
    );
}

#[test]
fn negative_lower_rows_have_the_same_conflict_meaning() {
    let mut m = model(4, &[(0, 1), (1, 2), (2, 3)], false);
    for row in &mut m.rows {
        for (_, a) in &mut row.coefficients {
            *a = -*a;
        }
        row.lower = row.upper.map(|v| -v);
        row.lower_enclosure = row.upper_enclosure.map(|[l, u]| [-u, -l]);
        row.upper = None;
        row.upper_enclosure = None;
    }
    let r = solve(
        &m,
        &Limits {
            binary_graph: true,
            ..Limits::default()
        },
    );
    assert_eq!(r.lp_engine, "BinaryGraph");
    assert_eq!(r.status, "optimal");
    assert_eq!(r.objective, Some(1.));
}
