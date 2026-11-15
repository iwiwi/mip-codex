use mip_codex::{integer_propagate::IntegerPropagator, mps, propagate::Status};
fn bounds(model: &mip_codex::model::Model) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    (
        model.variables.iter().map(|v| v.lower).collect(),
        model.variables.iter().map(|v| v.upper).collect(),
    )
}
#[test]
fn free_integer_column_gets_exact_floor_and_ceil_from_both_sides() {
    for sign in [1, -1] {
        let text = format!(
            "NAME exact\nROWS\n N c\n L hi\n G lo\nCOLUMNS\n x c 0 hi {}\n x lo {}\n y hi 1 lo 1\nRHS\n rhs hi 3 lo -1\nBOUNDS\n LI b x 0\n FR b x\n LI b y 2\n UP b y 2\nENDATA\n",
            2 * sign,
            2 * sign
        );
        let model = mps::parse(std::io::Cursor::new(text)).unwrap();
        let (mut lo, mut up) = bounds(&model);
        let mut p = IntegerPropagator::new(&model, 1 << 20).unwrap();
        assert_eq!(p.propagate(&mut lo, &mut up, None, None), Status::Stable);
        assert_eq!(
            (lo[0], up[0]),
            if sign == 1 {
                (Some(-1.), Some(0.))
            } else {
                (Some(0.), Some(1.))
            }
        );
        assert!(p.stats.tightenings >= 2);
    }
}
#[test]
fn decimal_provenance_memory_and_unbounded_continuous_guards_fail_open() {
    for (coefficient, rhs, bound) in [
        ("1.0000000000000000000001", "1", "0"),
        ("1", "1.000000000000000000001", "0"),
        ("1", "1", "0.000000000000000000001"),
    ] {
        let text = format!(
            "NAME unknown\nROWS\n N c\n L r\nCOLUMNS\n x c 0 r {coefficient}\nRHS\n rhs r {rhs}\nBOUNDS\n LI b x {bound}\n UP b x 3\nENDATA\n"
        );
        let model = mps::parse(std::io::Cursor::new(text)).unwrap();
        let mut p = IntegerPropagator::new(&model, 1 << 20).unwrap();
        assert_eq!(p.stats.inequalities, 0);
        let (mut lo, mut up) = bounds(&model);
        let before = (lo.clone(), up.clone());
        assert_eq!(p.propagate(&mut lo, &mut up, None, None), Status::Stable);
        assert_eq!((lo, up), before);
        assert!(IntegerPropagator::new(&model, 0).is_none());
    }
    let model=mps::parse(std::io::Cursor::new("NAME continuous\nROWS\n N c\n E r\nCOLUMNS\n x c 0 r 1\n y r 1\nRHS\n rhs r 0\nBOUNDS\n BV b x\n FR b y\nENDATA\n")).unwrap();
    let p = IntegerPropagator::new(&model, 1 << 20).unwrap();
    assert_eq!(p.stats.inequalities, 0);
}
#[test]
fn incremental_equality_propagation_and_deadline_preserve_parent_bounds() {
    let model=mps::parse(std::io::Cursor::new("NAME eq\nROWS\n N c\n E r\nCOLUMNS\n x c 0 r 1\n y r 1\nRHS\n rhs r 1\nBOUNDS\n BV b x\n BV b y\nENDATA\n")).unwrap();
    let mut p = IntegerPropagator::new(&model, 1 << 20).unwrap();
    let (mut lo, mut up) = bounds(&model);
    let before = (lo.clone(), up.clone());
    assert_eq!(
        p.propagate(&mut lo, &mut up, None, Some(std::time::Instant::now())),
        Status::TimeLimit
    );
    assert_eq!((lo.clone(), up.clone()), before);
    assert_eq!(p.propagate(&mut lo, &mut up, None, None), Status::Stable);
    let parent = (lo.clone(), up.clone());
    lo[0] = Some(1.);
    assert_eq!(p.propagate(&mut lo, &mut up, Some(0), None), Status::Stable);
    assert_eq!(up[1], Some(0.));
    assert_eq!(parent, before);
    lo[1] = Some(1.);
    assert_eq!(
        p.propagate(&mut lo, &mut up, Some(1), None),
        Status::Infeasible
    );
}
#[test]
fn thousand_small_domains_keep_every_exact_feasible_integer_point() {
    let mut seed = 2026111518u64;
    let mut next = |n: u64| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (seed >> 32) % n
    };
    let mut tightened = 0;
    let mut pruned = 0;
    for case in 0..1000 {
        let n = 1 + next(5) as usize;
        let m = 1 + next(6) as usize;
        let a: Vec<Vec<i64>> = (0..m)
            .map(|_| (0..n).map(|_| next(9) as i64 - 4).collect())
            .collect();
        let b: Vec<_> = (0..m).map(|_| next(19) as i64 - 9).collect();
        let kind: Vec<_> = (0..m)
            .map(|_| match next(3) {
                0 => 'L',
                1 => 'G',
                _ => 'E',
            })
            .collect();
        let mut text = String::from("NAME enumerate\nROWS\n N c\n");
        for i in 0..m {
            text += &format!(" {} r{i}\n", kind[i]);
        }
        text += "COLUMNS\n";
        for j in 0..n {
            text += &format!(" x{j} c 0\n");
            for i in 0..m {
                text += &format!(" x{j} r{i} {}\n", a[i][j]);
            }
        }
        text += "RHS\n";
        for i in 0..m {
            text += &format!(" rhs r{i} {}\n", b[i]);
        }
        text += "BOUNDS\n";
        for j in 0..n {
            text += &format!(" LI b x{j} -2\n UP b x{j} 2\n");
        }
        text += "ENDATA\n";
        let model = mps::parse(std::io::Cursor::new(text)).unwrap();
        let (mut lo, mut up) = bounds(&model);
        let mut p = IntegerPropagator::new(&model, 1 << 20).unwrap();
        let status = p.propagate(&mut lo, &mut up, None, None);
        tightened += p.stats.tightenings;
        pruned += p.stats.pruned;
        let mut feasible = 0;
        for code in 0..5usize.pow(n as u32) {
            let mut value = code;
            let x: Vec<i64> = (0..n)
                .map(|_| {
                    let v = (value % 5) as i64 - 2;
                    value /= 5;
                    v
                })
                .collect();
            if (0..m).all(|i| {
                let activity = a[i].iter().zip(&x).map(|(a, x)| a * x).sum::<i64>();
                match kind[i] {
                    'L' => activity <= b[i],
                    'G' => activity >= b[i],
                    _ => activity == b[i],
                }
            }) {
                feasible += 1;
                assert_ne!(status, Status::Infeasible, "case{case}");
                assert!(
                    (0..n).all(|j| lo[j].is_none_or(|l| x[j] as f64 >= l)
                        && up[j].is_none_or(|u| x[j] as f64 <= u)),
                    "case{case}, point{x:?}, {lo:?}/{up:?}"
                );
            }
        }
        if status == Status::Infeasible {
            assert_eq!(feasible, 0, "case{case}");
        }
    }
    assert!(tightened > 500);
    assert!(pruned > 100);
}

fn solver_limits(engine: mip_codex::solve::Engine) -> mip_codex::solve::Limits {
    mip_codex::solve::Limits {
        engine,
        integer_propagation: true,
        cover_cuts: false,
        rounding_cuts: false,
        primal_repair: false,
        primal_probe: false,
        lp_diving: false,
        rins: false,
        strong_branching: false,
        ..mip_codex::solve::Limits::default()
    }
}
#[test]
fn exact_domain_contradiction_prunes_before_lp_but_fixed_feasible_leaf_still_uses_lp() {
    use mip_codex::solve::{Engine, solve};
    for rhs in [1, 2] {
        let model=mps::parse(std::io::Cursor::new(format!("NAME integrate\nROWS\n N c\n E r\nCOLUMNS\n x c -1 r 2\nRHS\n rhs r {rhs}\nBOUNDS\n LI b x 0\n UP b x 5\nENDATA\n"))).unwrap();
        for engine in [Engine::Dense, Engine::Revised] {
            let r = solve(&model, &solver_limits(engine));
            assert_eq!(r.fixed_leaves, 0);
            if rhs == 1 {
                assert_eq!(r.status, "infeasible");
                assert_eq!(r.lp_solves, 0);
                assert_eq!(r.integer_propagation_stats.pruned, 1);
            } else {
                assert_eq!(r.status, "optimal");
                assert_eq!(r.objective, Some(-1.));
                assert_eq!(r.lp_solves, 1);
                assert_eq!(r.integer_propagation_stats.tightenings, 2);
            }
        }
    }
}
#[test]
fn no_eligible_rows_skip_per_node_propagation_work() {
    let model=mps::parse(std::io::Cursor::new("NAME skip\nROWS\n N c\n E r\nCOLUMNS\n x c 0 r 1\n y r 1\nRHS\n rhs r 0\nBOUNDS\n BV b x\n FR b y\nENDATA\n")).unwrap();
    let r = mip_codex::solve::solve(&model, &solver_limits(mip_codex::solve::Engine::Revised));
    assert_eq!(r.status, "optimal");
    assert_eq!(r.integer_propagation_stats.calls, 0);
}

#[test]
fn root_only_scope_propagates_once_and_preserves_integer_optimum() {
    use mip_codex::solve::{Engine, solve};
    let model=mps::parse(std::io::Cursor::new("NAME rootscope\nROWS\n N c\n L r\nCOLUMNS\n x c -1 r 2\n y c -1 r 2\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n")).unwrap();
    for engine in [Engine::Dense, Engine::Revised] {
        let r = solve(
            &model,
            &mip_codex::solve::Limits {
                integer_propagation_root_only: true,
                ..solver_limits(engine)
            },
        );
        assert_eq!(r.status, "optimal");
        assert_eq!(r.objective, Some(-1.));
        assert!(r.nodes >= 2);
        assert_eq!(r.integer_propagation_stats.calls, 1);
        assert!(r.integer_propagation_root_only);
    }
}
