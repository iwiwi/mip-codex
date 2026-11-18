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
        for (limit, local, root, prefix, diversify, adaptive) in [0, 1, 2, 3, 7, usize::MAX]
            .into_iter()
            .flat_map(|limit| [false, true].map(|local| (limit, local)))
            .flat_map(|(limit, local)| [false, true].map(|root| (limit, local, root)))
            .flat_map(|(limit, local, root)| {
                [false, true].map(|prefix| (limit, local, root, prefix))
            })
            .flat_map(|(limit, local, root, prefix)| {
                [false, true].map(|diversify| (limit, local, root, prefix, diversify))
            })
            .flat_map(|(limit, local, root, prefix, diversify)| {
                [false, true].map(|adaptive| (limit, local, root, prefix, diversify, adaptive))
            })
        {
            let r = binary_graph::solve_with_adaptive(
                &model,
                512 * 1024 * 1024,
                Instant::now() + Duration::from_secs(1),
                limit,
                local,
                root,
                prefix,
                diversify,
                adaptive,
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

#[test]
fn input_row_cover_certifies_the_odd_cycle_root_without_search() {
    let n=7;
    let mut lines=vec!["NAME input_cover".to_string(),"ROWS".into()," N obj".into()];
    for i in 0..n {lines.push(format!(" L r{i}"));}
    lines.push("COLUMNS".into());
    for v in 0..n {
        lines.push(format!(" x{v} obj -1"));
        for i in 0..n {if (0..3).any(|k|(i+k)%n==v) {lines.push(format!(" x{v} r{i} 1"));}}
    }
    lines.push("RHS".into());
    for i in 0..n {lines.push(format!(" rhs r{i} 1"));}
    lines.push("BOUNDS".into());
    for v in 0..n {lines.push(format!(" BV b x{v}"));}
    lines.push("ENDATA".into());
    let model=mps::parse(Cursor::new(lines.join("\n")+"\n")).unwrap();
    let result=solve(&model,&Limits{seconds:5.,nodes:0,graph_input_cover:true,..Limits::default()});
    let graph=result.binary_graph.unwrap();
    let cover=graph.fractional_cover.unwrap();
    assert_eq!(cover.stats.input_groups,7);
    assert_eq!(cover.stats.input_upper,Some(2));
    assert_eq!(cover.certificate.unwrap().upper,2);
    assert_eq!(result.objective,Some(-2.));
    assert_eq!(result.dual_bound,Some(-2.));
    assert_eq!(result.status,"optimal");
    assert_eq!(graph.nodes,0);
}

#[test]
fn integer_spectrum_certifies_a_packing_optimum_without_search_in_both_senses() {
    let n=16usize;
    let edges:Vec<_>=(0..n).flat_map(|a|(a+1..n).filter_map(move|b|((a^b).count_ones()<=2).then_some((a,b)))).collect();
    let optimum=(0usize..1<<n).filter(|&mask|edges.iter().all(|&(a,b)|mask&(1<<a)==0||mask&(1<<b)==0)).map(|m|m.count_ones()).max().unwrap();
    assert_eq!(optimum,2);
    for maximize in [false,true] {
        let model=model(n,&edges,maximize);
        let result=solve(&model,&Limits{seconds:2.,nodes:0,graph_spectral_bound:true,..Limits::default()});
        assert_eq!(result.status,"optimal");
        assert_eq!(result.objective,Some(if maximize{13.}else{1.}));
        assert_eq!(result.dual_bound,result.objective);
        assert_eq!(result.nodes,0);
        assert_eq!(result.lp_solves,1);
        let graph=result.binary_graph.as_ref().unwrap();
        let spectral=graph.spectral_bound.as_ref().unwrap();
        assert_eq!(spectral.certificate.as_ref().unwrap().upper,2);
        assert_eq!(spectral.stats.validation_failures,0);
        assert_eq!(result.lp_pivots,Some(spectral.stats.lp_pivots));
    }
}

#[test]
fn large_capacity_preserves_exact_pair_optimum_and_original_api_guard() {
    let n=5000;
    let edges:Vec<_>=(0..n/2).map(|k|(2*k,2*k+1)).collect();
    for maximize in [false,true] {
        let m=model(n,&edges,maximize);
        let deadline=Instant::now()+Duration::from_secs(30);
        assert!(binary_graph::solve(&m,64*1024*1024,deadline,0).is_none());
        let answer=binary_graph::solve_with_capacity(&m,64*1024*1024,deadline,0,false,false,false,false,false,false,false,false,false,false,true).unwrap();
        assert!(m.check(&answer.point,true,0.).is_ok());
        assert_eq!(answer.point.iter().sum::<f64>(),2500.);
        assert_eq!(answer.stats.upper,2500);
        assert_eq!(answer.stats.vertices,5000);
        assert!(answer.stats.complete);
        assert!(binary_graph::solve_with_capacity(&m,1024,deadline,0,false,false,false,false,false,false,false,false,false,false,true).is_none());
        assert!(binary_graph::solve_with_capacity(&m,64*1024*1024,Instant::now(),0,false,false,false,false,false,false,false,false,false,false,true).is_none());
    }
}

#[test]
fn large_capacity_rejects_out_of_range_vertices_before_allocating_graph() {
    let m=model(65536,&[],false);
    assert!(binary_graph::solve_with_capacity(&m,usize::MAX,Instant::now()+Duration::from_secs(1),0,false,false,false,false,false,false,false,false,false,false,true).is_none());
}
