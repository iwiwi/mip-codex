use mip_codex::{
    cuts, mps,
    solve::{Limits, solve},
};
use std::{
    io::Cursor,
    time::{Duration, Instant},
};
fn triangle(negative: bool) -> mip_codex::model::Model {
    let cols = if negative {
        " x c 1\n x a -1 b -1\n y c 1\n y a -1 d -1\n z c 1\n z b -1 d -1"
    } else {
        " x c -1\n x a 1 b 1\n y c -1\n y a 1 d 1\n z c -1\n z b 1 d 1"
    };
    let rhs = if negative { -1 } else { 1 };
    mps::parse(Cursor::new(format!("NAME triangle\nROWS\n N c\n L a\n L b\n L d\nCOLUMNS\n{cols}\nRHS\n r a {rhs} b {rhs}\n r d {rhs}\nBOUNDS\n BV b x\n BV b y\n BV b z\nENDATA\n"))).unwrap()
}
#[test]
fn all_pair_conflicts_give_a_stronger_cut_for_both_literal_signs() {
    for negative in [false, true] {
        let model = triangle(negative);
        let mut pool = cuts::Pool::default();
        let c = pool.separate_cliques(
            &model,
            &[0.5; 3],
            32,
            Instant::now() + Duration::from_secs(1),
        );
        assert_eq!(c.len(), 1);
        assert!(cuts::verify(&model, &c[0]));
        for bits in 0..8 {
            let x: Vec<_> = (0..3).map(|j| ((bits >> j) & 1) as f64).collect();
            if model.check(&x, true, 1e-9).is_ok() {
                let a: f64 = c[0].row.coefficients.iter().map(|&(j, v)| v * x[j]).sum();
                assert!(a <= c[0].row.upper.unwrap());
            }
        }
        let mut forged = c[0].clone();
        forged.proof.clique.as_mut().unwrap().pop();
        assert!(!cuts::verify(&model, &forged));
        let mut forged = c[0].clone();
        let edges = forged.proof.clique.as_mut().unwrap();
        edges[1] = edges[0].clone();
        assert!(!cuts::verify(&model, &forged));
        let mut forged = c[0].clone();
        forged.row.upper = Some(-10.);
        assert!(!cuts::verify(&model, &forged));
        let mut forged = c[0].clone();
        forged.row.coefficients[0].1 *= -1.;
        assert!(!cuts::verify(&model, &forged));
        let result = solve(
            &model,
            &Limits {
                clique_cuts: true,
                cover_cuts: false,
                primal_repair: false,
                lp_diving: false,
                primal_probe: false,
                rins: false,
                strong_branching: false,
                ..Limits::default()
            },
        );
        assert_eq!(result.status, "optimal");
        assert_eq!(result.objective, Some(if negative { 2. } else { -1. }));
        assert_eq!(result.cut_stats.clique_added, 1);
        assert_eq!(result.nodes, 1);
    }
}
#[test]
fn a_missing_pair_does_not_imply_a_clique() {
    let mut model = triangle(false);
    model.rows.pop();
    let mut pool = cuts::Pool::default();
    assert!(
        pool.separate_cliques(
            &model,
            &[0.5; 3],
            32,
            Instant::now() + Duration::from_secs(1)
        )
        .is_empty()
    );
    assert!(
        pool.separate_cliques(
            &model,
            &[0.5; 3],
            32,
            Instant::now() - Duration::from_secs(1)
        )
        .is_empty()
    );
}
