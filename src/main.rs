use mip_codex::mps;
use std::io::{self, Write};
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || !["inspect", "dump", "solve"].contains(&args[1].as_str()) {
        return Err("usage: mip-codex inspect|dump|solve FILE.mps[.gz] [SECONDS] [NODES]".into());
    }
    let model = mps::read(&args[2])?;
    let mut out = io::BufWriter::new(io::stdout().lock());
    if args[1] == "solve" {
        let mut limits = mip_codex::solve::Limits::default();
        if let Ok(engine) = std::env::var("MIP_LP_ENGINE") {
            limits.engine = match engine.as_str() {
                "dense" => mip_codex::solve::Engine::Dense,
                "revised" => mip_codex::solve::Engine::Revised,
                _ => return Err("unknown MIP_LP_ENGINE".into()),
            };
        }
        if let Ok(warm) = std::env::var("MIP_LP_WARM_START") {
            limits.warm_start = match warm.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_LP_WARM_START must be 0 or 1".into()),
            };
        }
        if let Ok(propagation) = std::env::var("MIP_PROPAGATION") {
            limits.propagation = match propagation.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_PROPAGATION must be 0 or 1".into()),
            };
        }
        if let Ok(repair) = std::env::var("MIP_PRIMAL_REPAIR") {
            limits.primal_repair = match repair.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_PRIMAL_REPAIR must be 0 or 1".into()),
            };
        }
        if let Ok(diving) = std::env::var("MIP_LP_DIVING") {
            limits.lp_diving = match diving.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_LP_DIVING must be 0 or 1".into()),
            };
        }
        if let Ok(probe) = std::env::var("MIP_PRIMAL_PROBE") {
            limits.primal_probe = match probe.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_PRIMAL_PROBE must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_STRONG_BRANCHING") {
            limits.strong_branching = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_STRONG_BRANCHING must be 0 or 1".into()),
            };
        }
        if let Ok(cuts) = std::env::var("MIP_ROUNDING_CUTS") {
            limits.rounding_cuts = match cuts.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_ROUNDING_CUTS must be 0 or 1".into()),
            };
        }
        if let Ok(cuts) = std::env::var("MIP_COVER_CUTS") {
            limits.cover_cuts = match cuts.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_COVER_CUTS must be 0 or 1".into()),
            };
        }
        if let Ok(rule) = std::env::var("MIP_BRANCHING") {
            limits.branching = match rule.as_str() {
                "fractional" => mip_codex::branching::Rule::Fractional,
                "pseudocost" => mip_codex::branching::Rule::Pseudocost,
                _ => return Err("MIP_BRANCHING must be fractional or pseudocost".into()),
            };
        }
        if let Ok(order) = std::env::var("MIP_CHILD_ORDER") {
            limits.child_order = match order.as_str() {
                "down" => mip_codex::branching::Order::Down,
                "nearest" => mip_codex::branching::Order::Nearest,
                "estimate" => mip_codex::branching::Order::Estimate,
                _ => return Err("MIP_CHILD_ORDER must be down, nearest or estimate".into()),
            };
        }
        if let Ok(search) = std::env::var("MIP_SEARCH") {
            limits.search = match search.as_str() {
                "depth" => mip_codex::search::Strategy::DepthFirst,
                "best" => mip_codex::search::Strategy::BestBound,
                "hybrid" => mip_codex::search::Strategy::Hybrid,
                _ => return Err("MIP_SEARCH must be depth, best or hybrid".into()),
            };
        }
        if let Some(s) = args.get(3) {
            limits.seconds = s.parse().map_err(|_| "invalid seconds")?;
        }
        if !limits.seconds.is_finite() || limits.seconds < 0.0 || limits.seconds > 604800.0 {
            return Err("seconds out of range".into());
        }
        if let Some(s) = args.get(4) {
            limits.nodes = s.parse().map_err(|_| "invalid nodes")?;
        }
        let result = mip_codex::solve::solve(&model, &limits);
        serde_json::to_writer(&mut out, &result).map_err(|e| e.to_string())?;
    } else if args[1] == "dump" {
        serde_json::to_writer(&mut out, &model).map_err(|e| e.to_string())?;
    } else {
        serde_json::to_writer(&mut out,&serde_json::json!({"name":model.name,"variables":model.variables.len(),"rows":model.rows.len(),"nonzeros":model.rows.iter().map(|r|r.coefficients.len()).sum::<usize>(),"integers":model.variables.iter().filter(|v|v.integer).count(),"maximize":model.maximize,"offset":model.offset})).map_err(|e|e.to_string())?;
    }
    writeln!(out).map_err(|e| e.to_string())?;
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
