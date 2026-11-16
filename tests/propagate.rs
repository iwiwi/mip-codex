use mip_codex::{
    model::{Model, Row, Variable},
    propagate::{Propagator, Status},
};
fn variable(name: &str, lower: Option<f64>, upper: Option<f64>, integer: bool) -> Variable {
    Variable {
        lower_enclosure: None,
        upper_enclosure: None,
        name: name.into(),
        lower,
        upper,
        integer,
        objective_enclosure: None,
        objective: 0.,
    }
}
fn model(variables: Vec<Variable>, rows: Vec<Row>) -> Model {
    Model {
        name: "propagation".into(),
        maximize: false,
        original_decimal: false,
        offset_enclosure: None,
        offset: 0.,
        variables,
        rows,
    }
}
fn row(lower: Option<f64>, upper: Option<f64>, coefficients: Vec<(usize, f64)>) -> Row {
    Row {
        lower_enclosure: None,
        upper_enclosure: None,
        exact_coefficients: false,
        exact_coefficient_flags: vec![],
        exact_integral: false,
        decimal_scale: None,
        name: "row".into(),
        lower,
        upper,
        coefficients,
    }
}
fn domains(m: &Model) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    (
        m.variables.iter().map(|v| v.lower).collect(),
        m.variables.iter().map(|v| v.upper).collect(),
    )
}
#[test]
fn branch_fixing_propagates_through_implication_chain() {
    let n = 100;
    let vars = (0..n)
        .map(|j| variable(&format!("x{j}"), Some(0.), Some(1.), true))
        .collect();
    let rows = (0..n - 1)
        .map(|j| row(None, Some(0.), vec![(j, 1.), (j + 1, -1.)]))
        .collect();
    let m = model(vars, rows);
    let (mut lo, mut up) = domains(&m);
    let mut p = Propagator::new(&m);
    assert_ne!(
        p.propagate(&mut lo, &mut up, None, None),
        Status::Infeasible
    );
    up[n - 1] = Some(0.);
    assert_eq!(
        p.propagate(&mut lo, &mut up, Some(n - 1), None),
        Status::Stable
    );
    assert!(up.iter().all(|v| *v == Some(0.)));
    assert!(p.stats.tightenings >= 99);
}
#[test]
fn unbounded_variables_can_acquire_both_bounds() {
    let m = model(
        vec![
            variable("x", None, None, true),
            variable("y", Some(2.), Some(2.), false),
        ],
        vec![row(Some(1.), Some(3.), vec![(0, 1.), (1, 1.)])],
    );
    let (mut lo, mut up) = domains(&m);
    let mut p = Propagator::new(&m);
    assert_eq!(p.propagate(&mut lo, &mut up, None, None), Status::Stable);
    assert_eq!(lo[0], Some(-1.));
    assert_eq!(up[0], Some(1.));
}
#[test]
fn mixed_signs_integer_rounding_and_tolerance() {
    let m = model(
        vec![
            variable("x", Some(0.), Some(10.), true),
            variable("y", Some(1.), Some(1.), false),
        ],
        vec![row(Some(-3.), Some(-1.), vec![(0, -2.), (1, 1.)])],
    );
    let (mut lo, mut up) = domains(&m);
    let mut p = Propagator::new(&m);
    assert_eq!(p.propagate(&mut lo, &mut up, None, None), Status::Stable);
    assert_eq!(lo[0], Some(1.));
    assert_eq!(up[0], Some(2.));
    // 行許容誤差内の整数を丸め誤差で削らない。
    let m = model(
        vec![variable("x", Some(0.), Some(2.), true)],
        vec![row(None, Some(1. - 5e-7), vec![(0, 1.)])],
    );
    let (mut lo, mut up) = domains(&m);
    let mut p = Propagator::new(&m);
    p.propagate(&mut lo, &mut up, None, None);
    assert_eq!(up[0], Some(1.));
}
#[test]
fn contradictory_activity_and_integer_domain() {
    let m = model(
        vec![variable("x", Some(0.), Some(1.), true)],
        vec![row(Some(2.), None, vec![(0, 1.)])],
    );
    let (mut lo, mut up) = domains(&m);
    assert_eq!(
        Propagator::new(&m).propagate(&mut lo, &mut up, None, None),
        Status::Infeasible
    );
    let m = model(vec![variable("x", Some(0.2), Some(0.8), true)], vec![]);
    let (mut lo, mut up) = domains(&m);
    assert_eq!(
        Propagator::new(&m).propagate(&mut lo, &mut up, None, None),
        Status::Infeasible
    );
}
#[test]
fn overflow_and_cancellation_do_not_create_false_contradiction() {
    let m = model(
        vec![
            variable("x", Some(1e200), Some(1e200), false),
            variable("y", Some(1e200), Some(1e200), false),
        ],
        vec![row(Some(0.), Some(0.), vec![(0, 1e200), (1, -1e200)])],
    );
    let (mut lo, mut up) = domains(&m);
    let status = Propagator::new(&m).propagate(&mut lo, &mut up, None, None);
    assert_ne!(status, Status::Infeasible);
    assert_eq!(lo, vec![Some(1e200); 2]);
    assert_eq!(up, vec![Some(1e200); 2]);
}
#[test]
fn deadline_stops_without_infeasibility_claim() {
    let m = model(
        vec![variable("x", Some(0.), Some(1.), true)],
        vec![row(None, Some(1.), vec![(0, 1.)])],
    );
    let (mut lo, mut up) = domains(&m);
    assert_eq!(
        Propagator::new(&m).propagate(&mut lo, &mut up, None, Some(std::time::Instant::now())),
        Status::TimeLimit
    );
}
#[test]
fn enumeration_preserves_every_feasible_integer_point() {
    let mut seed = 2026111504u64;
    fn rng(s: &mut u64, n: u64) -> u64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*s >> 32) % n
    }
    for case in 0..1000 {
        let n = 2 + rng(&mut seed, 3) as usize;
        let count = 1 + rng(&mut seed, 6) as usize;
        let vars = (0..n)
            .map(|j| variable(&format!("x{j}"), Some(-2.), Some(3.), true))
            .collect();
        let rows = (0..count)
            .map(|_| {
                let coefficients = (0..n)
                    .filter_map(|j| {
                        let a = rng(&mut seed, 9) as f64 - 4.;
                        if a == 0. { None } else { Some((j, a)) }
                    })
                    .collect();
                let b = rng(&mut seed, 21) as f64 - 10.;
                match rng(&mut seed, 3) {
                    0 => row(None, Some(b), coefficients),
                    1 => row(Some(b), None, coefficients),
                    _ => row(Some(b - 2.), Some(b + 2.), coefficients),
                }
            })
            .collect();
        let m = model(vars, rows);
        let (mut lo, mut up) = domains(&m);
        let status = Propagator::new(&m).propagate(&mut lo, &mut up, None, None);
        for index in 0..6usize.pow(n as u32) {
            let mut q = index;
            let x = (0..n)
                .map(|_| {
                    let v = (q % 6) as f64 - 2.;
                    q /= 6;
                    v
                })
                .collect::<Vec<_>>();
            if m.check(&x, true, 1e-6).is_ok() {
                assert_ne!(status, Status::Infeasible, "case {case}, feasible {x:?}");
                for j in 0..n {
                    assert!(
                        lo[j].is_none_or(|v| x[j] >= v) && up[j].is_none_or(|v| x[j] <= v),
                        "case {case} lost {x:?}"
                    );
                }
            }
        }
    }
}
#[test]
fn mixed_continuous_grid_points_are_preserved() {
    for k in -12..=12 {
        let m = model(
            vec![
                variable("x", Some(-2.), Some(3.), true),
                variable("y", Some(-2.), Some(3.), false),
            ],
            vec![row(
                Some(k as f64 / 4.),
                Some(k as f64 / 4. + 1.),
                vec![(0, -1.25), (1, 2.5)],
            )],
        );
        let (mut lo, mut up) = domains(&m);
        let status = Propagator::new(&m).propagate(&mut lo, &mut up, None, None);
        for x in -2..=3 {
            for y in -8..=12 {
                let point = [x as f64, y as f64 / 4.];
                if m.check(&point, true, 1e-6).is_ok() {
                    assert_ne!(status, Status::Infeasible);
                    for j in 0..2 {
                        assert!(lo[j].is_none_or(|v| point[j] >= v));
                        assert!(up[j].is_none_or(|v| point[j] <= v));
                    }
                }
            }
        }
    }
}
