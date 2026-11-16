use mip_codex::{
    cuts::{Pool, verify},
    mps,
};
use std::time::{Duration, Instant};
fn triangle(negative: bool, mixed: bool) -> mip_codex::model::Model {
    let s = if negative { -1 } else { 1 };
    let mut text = format!(
        "NAME triangle\nROWS\n N obj\n L a\n L b\n L c\nCOLUMNS\n x obj -1 a {s}\n x b {s}\n y obj -1 a 1\n y c 1\n z obj -1 b 1\n z c 1\n"
    );
    if mixed {
        text += " t a -1\n t b -1\n t c -1\n";
    }
    text += &format!(
        "RHS\n rhs a {} b {}\n rhs c 1\nBOUNDS\n BV bd x\n BV bd y\n BV bd z\n",
        if negative { 0 } else { 1 },
        if negative { 0 } else { 1 }
    );
    if mixed {
        text += " UP bd t 0\n";
    }
    text += "ENDATA\n";
    mps::parse(std::io::Cursor::new(text)).unwrap()
}
#[test]
fn signed_triangle_and_original_bound_projection_have_all_pair_proofs() {
    for negative in [false, true] {
        for mixed in [false, true] {
            let m = triangle(negative, mixed);
            let mut p = Pool::default();
            let mut point = vec![0.5; 3];
            if mixed {
                point.push(0.);
            }
            let cuts = p.separate_clique(&m, &point, 32, Instant::now() + Duration::from_secs(1));
            assert_eq!(cuts.len(), 1);
            let c = &cuts[0];
            assert!(verify(&m, c));
            assert_eq!(c.proof.clique.len(), 3);
            assert_eq!(
                c.row.coefficients,
                vec![(0, if negative { -1. } else { 1. }), (1, 1.), (2, 1.)]
            );
            for bits in 0..8 {
                let mut x: Vec<_> = (0..3).map(|j| ((bits >> j) & 1) as f64).collect();
                if mixed {
                    x.push(0.);
                }
                if m.check(&x, true, 1e-9).is_ok() {
                    assert!(
                        c.row
                            .coefficients
                            .iter()
                            .map(|&(j, a)| a * x[j])
                            .sum::<f64>()
                            <= c.row.upper.unwrap()
                    );
                }
            }
            let json = serde_json::to_value(c).unwrap();
            let decoded = serde_json::from_value(json).unwrap();
            assert!(verify(&m, &decoded));
            assert!(
                p.separate_clique(&m, &point, 32, Instant::now() + Duration::from_secs(1))
                    .is_empty()
            );
        }
    }
}
#[test]
fn corrupt_pair_or_clique_is_rejected() {
    let m = triangle(false, false);
    let c = Pool::default()
        .separate_clique(&m, &[0.5; 3], 32, Instant::now() + Duration::from_secs(1))
        .remove(0);
    for change in 0..10 {
        let mut b = c.clone();
        match change {
            0 => {
                b.proof.clique.pop();
            }
            1 => b.proof.clique[0].source_row = 999,
            2 => b.proof.clique[0].source_name = "bad".into(),
            3 => b.proof.clique[0].upper_side = false,
            4 => b.proof.clique[0].variables = [0, 0],
            5 => b.proof.clique[1] = b.proof.clique[0].clone(),
            6 => b.row.coefficients[0].1 = -1.,
            7 => b.row.upper = Some(0.),
            8 => b.proof.cover.reverse(),
            _ => b.proof.divisor = Some(2),
        }
        assert!(!verify(&m, &b), "change {change}");
    }
}
#[test]
fn expiry_and_noninteger_projection_produce_no_unproved_cut() {
    let mut m = triangle(false, true);
    assert!(
        Pool::default()
            .separate_clique(&m, &[0.5, 0.5, 0.5, 0.], 32, Instant::now())
            .is_empty()
    );
    m.variables[3].upper = Some(0.5);
    m.variables[3].upper_enclosure = Some([0.5; 2]);
    assert!(
        Pool::default()
            .separate_clique(
                &m,
                &[0.5, 0.5, 0.5, 0.],
                32,
                Instant::now() + Duration::from_secs(1)
            )
            .is_empty()
    );
}
#[test]
fn random_signed_graph_cuts_preserve_every_binary_assignment() {
    let mut state = 87654321u64;
    let mut next = || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (state >> 32) as u32
    };
    let mut checked = 0;
    for _ in 0..200 {
        let signs: Vec<_> = (0..8).map(|_| next() % 2 == 0).collect();
        let mut edges = vec![];
        for a in 0..8 {
            for b in a + 1..8 {
                if next() % 3 != 0 {
                    edges.push((a, b));
                }
            }
        }
        let mut text = String::from("NAME random\nROWS\n N obj\n");
        for r in 0..edges.len() {
            text += &format!(" L r{r}\n");
        }
        text += "COLUMNS\n";
        for j in 0..8 {
            text += &format!(" x{j} obj -1\n");
            for (r, &(a, b)) in edges.iter().enumerate() {
                if j == a || j == b {
                    text += &format!(" x{j} r{r} {}\n", if signs[j] { -1 } else { 1 });
                }
            }
        }
        text += "RHS\n";
        for (r, &(a, b)) in edges.iter().enumerate() {
            text += &format!(
                " rhs r{r} {}\n",
                1 - i32::from(signs[a]) - i32::from(signs[b])
            );
        }
        text += "BOUNDS\n";
        for j in 0..8 {
            text += &format!(" BV bd x{j}\n");
        }
        text += "ENDATA\n";
        let m = mps::parse(std::io::Cursor::new(text)).unwrap();
        let cuts = Pool::default().separate_clique(
            &m,
            &[0.5; 8],
            32,
            Instant::now() + Duration::from_secs(1),
        );
        for c in &cuts {
            assert!(verify(&m, c));
            checked += 1;
        }
        for bits in 0..256 {
            let literal = |j: usize| (((bits >> j) & 1) != 0) ^ signs[j];
            if edges.iter().any(|&(a, b)| literal(a) && literal(b)) {
                continue;
            }
            for c in &cuts {
                let activity: i64 = c
                    .row
                    .coefficients
                    .iter()
                    .map(|&(j, a)| (a as i64) * ((bits >> j) & 1) as i64)
                    .sum();
                assert!(activity <= c.row.upper.unwrap() as i64);
            }
        }
    }
    assert!(checked > 200);
}
#[test]
fn clique_cut_strengthens_the_root_without_other_cuts_or_heuristics() {
    use mip_codex::solve::{Limits, solve};
    let m = triangle(false, false);
    let l = Limits {
        clique_cuts: false,
        cover_cuts: false,
        aggregate_cuts: false,
        rounding_cuts: false,
        binary_graph: false,
        box_primal: false,
        objective_rounding: false,
        primal_repair: false,
        feasibility_pump: false,
        lp_diving: false,
        primal_probe: false,
        strong_branching: false,
        rins: false,
        nodes: 1,
        ..Limits::default()
    };
    let a = solve(&m, &l);
    assert_eq!(a.status, "node_limit");
    assert_eq!(a.dual_bound, Some(-1.5));
    assert_eq!(a.objective, None);
    let b = solve(
        &m,
        &Limits {
            clique_cuts: true,
            ..l
        },
    );
    assert_eq!(b.status, "optimal");
    assert_eq!(b.dual_bound, Some(-1.));
    assert_eq!(b.objective, Some(-1.));
    assert_eq!(b.cut_stats.clique_added, 1);
    assert!(b.cut_proofs.iter().all(|c| verify(&m, c)));
}
