use mip_codex::{
    mps,
    solve::{Limits, solve},
};
use std::io::Cursor;
fn model(
    kind: &str,
    rhs: &str,
    lower: &str,
    upper: &str,
    cost: i32,
    max: bool,
    n: usize,
) -> mip_codex::model::Model {
    let mut text = format!(
        "NAME boundary\nOBJSENSE\n {}\nROWS\n N obj\n {kind} r\nCOLUMNS\n",
        if max { "MAX" } else { "MIN" }
    );
    for j in 0..n {
        text += &format!(" x{j} obj {cost} r 1\n");
    }
    text += &format!("RHS\n rhs r {rhs} obj -7\nBOUNDS\n");
    for j in 0..n {
        text += &format!(" LI b x{j} {lower}\n UI b x{j} {upper}\n");
    }
    text += "ENDATA\n";
    mps::parse(Cursor::new(text)).unwrap()
}
#[test]
fn original_decimal_boundaries_are_solved_before_binary64_shifts() {
    for max in [false, true] {
        for n in [1, 2] {
            for (kind, rhs, lo, hi, cost, opt) in [
                ("G", "1e-16", "-2", "2", 1, 1.),
                ("L", "-1e-16", "-2", "0", -1, 1.),
                ("L", "0.9999999999999999", "0", "2", -1, 0.),
            ] {
                let m = model(kind, rhs, lo, hi, if max { -cost } else { cost }, max, n);
                let before = serde_json::to_string(&m).unwrap();
                let r = solve(
                    &m,
                    &Limits {
                        seconds: 2.,
                        ..Limits::default()
                    },
                );
                assert_eq!(r.status, "optimal", "{kind} {rhs} {max} {n}");
                assert_eq!(r.objective, Some(7. + if max { -opt } else { opt }));
                assert!(r.integer_row_proof.as_ref().unwrap().rows.len() > 0);
                let x = m
                    .variables
                    .iter()
                    .map(|v| r.values.as_ref().unwrap()[&v.name])
                    .collect::<Vec<_>>();
                assert!(m.check(&x, true, 1e-6).is_ok());
                assert_eq!(serde_json::to_string(&m).unwrap(), before);
            }
        }
    }
}
#[test]
fn lexical_variable_bounds_survive_f64_rounding() {
    for (lo, hi, cost, opt) in [
        ("1.0000000000000001", "3", 1, 2.),
        ("-3", "-1.0000000000000001", -1, 2.),
        ("0", "0.99999999999999999", -1, 0.),
    ] {
        let m = model("L", "100", lo, hi, cost, false, 1);
        let r = solve(&m, &Limits::default());
        assert_eq!(r.status, "optimal");
        assert_eq!(r.objective, Some(7. + opt));
        assert_eq!(r.integer_row_proof.unwrap().variables.len(), 1);
    }
    let m = model("L", "100", "0.1", "0.9", 1, false, 1);
    let r = solve(&m, &Limits::default());
    assert_eq!(r.status, "infeasible");
    assert!(r.integer_row_proof.is_some());
}
#[test]
fn mixed_continuous_rows_are_never_rounded() {
    let m = model("G", "0.25", "0", "2", 1, false, 1);
    let mut m = m;
    m.variables[0].integer = false;
    let r = solve(&m, &Limits::default());
    assert_eq!(r.status, "optimal");
    assert_eq!(r.objective, Some(7.25));
    assert!(r.integer_row_proof.is_none());
}

#[test]
fn ranges_use_the_exact_sum_and_both_range_signs() {
    for (kind, rhs, range, expected) in [
        ("L", "1.00000000000000001", "1", [Some(1.), Some(1.)]),
        ("L", "-0.00000000000000001", "-1", [Some(-1.), Some(-1.)]),
        ("G", "-0.00000000000000001", "-1", [Some(0.), Some(0.)]),
        ("E", "-0.00000000000000001", "1", [Some(0.), Some(0.)]),
        ("E", "1.00000000000000001", "-1", [Some(1.), Some(1.)]),
    ] {
        let text = format!(
            "NAME ranges\nROWS\n N obj\n {kind} r\nCOLUMNS\n x obj 1 r 1\nRHS\n rhs r {rhs}\nRANGES\n rng r {range}\nBOUNDS\n LI b x -3\n UI b x 3\nENDATA\n"
        );
        let m = mps::parse(Cursor::new(text)).unwrap();
        assert_eq!(
            m.rows[0].integer_bounds,
            Some(expected),
            "{kind} {rhs} {range}"
        );
        let r = solve(&m, &Limits::default());
        assert_eq!(r.status, "optimal");
        assert_eq!(r.objective, expected[0]);
    }
}
#[test]
fn overwritten_bound_tokens_are_discarded_and_large_bounds_are_unsupported() {
    let text = "NAME bounds\nROWS\n N obj\nCOLUMNS\n x obj 1\nBOUNDS\n LI b x 1.0000000000000001\n BV b x\nENDATA\n";
    let m = mps::parse(Cursor::new(text)).unwrap();
    assert!(m.variables[0].integer_bounds.is_none());
    assert_eq!(solve(&m, &Limits::default()).objective, Some(0.));
    let m = model(
        "L",
        "100000000000000000000",
        "0",
        "100000000000000000000.1",
        1,
        false,
        1,
    );
    assert!(m.variables[0].integer_bounds.is_none());
    assert!(m.rows[0].integer_bounds.is_none());
}

#[test]
fn nonintegral_or_large_coefficients_cannot_claim_tolerance_only_optimality(){
 for coefficient in ["0.5","0.1","1e-8","10000000000000000"]{
  let text=format!("NAME scaled\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1 r {coefficient}\nRHS\n rhs r -1e-16\nBOUNDS\n LI b x -2\n UI b x 0\nENDATA\n");let m=mps::parse(Cursor::new(text)).unwrap();
  for enabled in [false,true]{let r=solve(&m,&Limits{integer_row_rounding:enabled,..Limits::default()});assert!(r.status!="optimal"||r.objective==Some(1.),"{coefficient}: {} {:?}",r.status,r.objective);if r.optimal_original_conflict{assert!(r.dual_bound.is_none()&&r.gap.is_none());}}
 }
}

#[test]
fn a_pure_lp_also_withdraws_a_definitely_infeasible_optimal_point(){
 let text="NAME purelp\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1 r 1e-16\nRHS\n rhs r -1e-16\nBOUNDS\n LO b x -2\n UP b x 0\nENDATA\n";let m=mps::parse(Cursor::new(text)).unwrap();let r=solve(&m,&Limits::default());assert!(r.status!="optimal"||r.objective==Some(1.));if r.optimal_original_conflict{assert!(r.dual_bound.is_none()&&r.gap.is_none());}
}

#[test]
fn implied_integer_continuous_decimal_boundaries_withdraw_optimality() {
    for (a,b,rhs,c) in [
        ("1","-1","0.9999999999999999999999","1"),
        ("3","-4","0.7499999999999999999999","1"),
        ("1","-3","0.3333333333333333333332","1"),
        ("1","-2","0.049999999999999999999999","0.1"),
        ("1.0000000000000002220446049250313080847263336181640625","-2","0.5","1"),
    ] {
        for rounding in [false,true] {
            let text=format!("NAME implied\nROWS\n N obj\n E link\n L cap\nCOLUMNS\n x obj -1 link {a}\n y link {b} cap {c}\nRHS\n rhs link 0 cap {rhs}\nBOUNDS\n LI b x 0\n UI b x 2\n LO b y 0\n UP b y 2\nENDATA\n");
            let m=mps::parse(Cursor::new(text)).unwrap();
            let r=solve(&m,&Limits{seconds:2.,integer_row_rounding:rounding,..Limits::default()});
            assert_eq!(r.status,"numerical_error","{a} {b} {rhs}");
            assert!(r.optimal_original_conflict);
            assert_eq!(r.dual_bound,None);
            assert_eq!(r.gap,None);
        }
    }
}

#[test]
fn exact_input_continuous_third_requires_a_rational_witness() {
    let m=mps::parse(Cursor::new("NAME third\nROWS\n N obj\n E r\nCOLUMNS\n y obj 1 r 3\nRHS\n rhs r 1\nENDATA\n")).unwrap();
    let r=solve(&m,&Limits::default());
    assert_eq!(r.status,"optimal");
    assert!(r.rational_primal_proof.is_some());
    assert!((r.objective.unwrap()-1./3.).abs()<1e-12);
}
