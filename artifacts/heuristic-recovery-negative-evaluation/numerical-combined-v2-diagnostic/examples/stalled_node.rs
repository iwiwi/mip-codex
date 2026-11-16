use mip_codex::{
    mps,
    solve::{self, Engine, Limits},
};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut model = mps::read(&args[1]).unwrap();
    let node: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(&args[2]).unwrap()).unwrap();
    for (j, v) in model.variables.iter_mut().enumerate() {
        assert_eq!(node["names"][j].as_str().unwrap(), v.name);
        v.integer = false;
        for lower in [true, false] {
            let key = if lower { "lower" } else { "upper" };
            let b = node[key][j].as_f64();
            let old = if lower { v.lower } else { v.upper };
            if b != old {
                assert!(b.is_none_or(|x| x.fract() == 0.));
                if lower {
                    v.lower_enclosure = b.map(|x| [x, x]);
                } else {
                    v.upper_enclosure = b.map(|x| [x, x]);
                }
            }
            if lower {
                v.lower = b;
            } else {
                v.upper = b;
            }
        }
    }
    if std::env::var_os("NODE_SAFE_ROWS").is_some() {
        let mut safe = model.clone();
        safe.rows.retain(|r| r.exact_coefficients);
        for r in &mut safe.rows {
            r.lower = r.lower.and_then(|_| r.lower_enclosure.map(|b| b[0]));
            r.upper = r.upper.and_then(|_| r.upper_enclosure.map(|b| b[1]));
        }
        let mut lower: Vec<_> = model.variables.iter().map(|v| v.lower).collect();
        let mut upper: Vec<_> = model.variables.iter().map(|v| v.upper).collect();
        let mut prop = mip_codex::propagate::Propagator::new(&safe);
        let status = prop.propagate(&mut lower, &mut upper, None, None);
        eprintln!(
            "safe rows={} status={status:?} stats={:?}",
            safe.rows.len(),
            prop.stats
        );
        for (j, v) in model.variables.iter_mut().enumerate() {
            v.lower = lower[j];
            v.upper = upper[j];
            v.lower_enclosure = v.lower.map(|x| [x, x]);
            v.upper_enclosure = v.upper.map(|x| [x, x]);
        }
    }
    let limits = Limits {
        seconds: 30.,
        nodes: 1,
        propagation: std::env::var_os("NODE_PROPAGATION").is_some(),
        engine: if args[3] == "dense" {
            Engine::Dense
        } else {
            Engine::Revised
        },
        cover_cuts: false,
        aggregate_cuts: false,
        clique_cuts: false,
        mir_cuts: false,
        three_row_cuts: false,
        binary_graph: false,
        primal_repair: false,
        box_primal: false,
        lp_diving: false,
        feasibility_pump: false,
        primal_probe: false,
        rins: false,
        strong_branching: false,
        objective_rounding: false,
        ..Limits::default()
    };
    println!(
        "{}",
        serde_json::to_string(&solve::solve(&model, &limits)).unwrap()
    );
}
