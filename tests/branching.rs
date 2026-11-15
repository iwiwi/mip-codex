use mip_codex::{
    branching::{Costs, Edge, Order, Rule},
    mps,
    solve::{Limits, solve},
};
fn model() -> mip_codex::model::Model {
    mps::parse(std::io::Cursor::new(
        "NAME choose\nROWS\n N c\nCOLUMNS\n x c 1\n y c 1\nBOUNDS\n BV b x\n BV b y\nENDATA\n",
    ))
    .unwrap()
}
#[test]
fn observed_costs_change_ranking_without_changing_feasible_region() {
    let m = model();
    let mut c = Costs::new(2);
    assert_eq!(c.select(&m, &[0.4, 0.5], Rule::Pseudocost), Some(1));
    for up in [false, true] {
        for _ in 0..8 {
            c.observe(
                Edge {
                    variable: 0,
                    up,
                    distance: 0.5,
                    parent_objective: 0.,
                },
                50.,
            );
            c.observe(
                Edge {
                    variable: 1,
                    up,
                    distance: 0.5,
                    parent_objective: 0.,
                },
                0.5,
            );
        }
    }
    assert_eq!(c.select(&m, &[0.4, 0.5], Rule::Pseudocost), Some(0));
    assert_eq!(c.select(&m, &[0.4, 0.5], Rule::Fractional), Some(1));
    assert_eq!(c.updates, 32);
    assert_eq!(c.selections, 1);
}
#[test]
fn zero_costs_fall_back_and_direction_uses_distance() {
    let m = model();
    let mut c = Costs::new(2);
    c.observe(
        Edge {
            variable: 0,
            up: false,
            distance: 0.5,
            parent_objective: 2.,
        },
        2.,
    );
    assert_eq!(c.select(&m, &[0.25, 0.75], Rule::Pseudocost), Some(1));
    assert!(!c.prefer_up(0, 0.75, Order::Down));
    assert!(c.prefer_up(0, 0.75, Order::Nearest));
    assert!(c.prefer_up(0, 0.75, Order::Estimate));
}
#[test]
fn all_rules_orders_and_objective_senses_preserve_optimum() {
    for sense in [" MIN", " MAX"] {
        let text = format!(
            "NAME mixed\nOBJSENSE\n{sense}\nROWS\n N c\n L r\nCOLUMNS\n x c -3 r 2\n y c -2 r 2\nRHS\n rhs c -7 r 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n"
        );
        let m = mps::parse(std::io::Cursor::new(text)).unwrap();
        for branching in [Rule::Fractional, Rule::Pseudocost] {
            for child_order in [Order::Down, Order::Nearest, Order::Estimate] {
                let r = solve(
                    &m,
                    &Limits {
                        box_primal: false,
                        binary_graph: false,
                        objective_rounding: false,
                        rins: false,
                        strong_branching: false,
                        cover_cuts: false,
                        branching,
                        child_order,
                        primal_repair: false,
                        primal_probe: false,
                        lp_diving: false,
                        ..Limits::default()
                    },
                );
                assert_eq!(r.status, "optimal");
                assert_eq!(r.objective, Some(if sense == " MIN" { 4. } else { 7. }));
                assert_eq!(r.dual_bound, r.objective);
            }
        }
    }
}
