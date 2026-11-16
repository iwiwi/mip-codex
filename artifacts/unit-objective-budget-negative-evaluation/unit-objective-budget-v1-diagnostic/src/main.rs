use mip_codex::mps;
use std::io::{self, Write};
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && args[1] == "build-info" {
        println!(
            "{}",
            serde_json::json!({"source_directory":env!("CARGO_MANIFEST_DIR"),"library_source_directory":mip_codex::BUILD_SOURCE_DIRECTORY,"source_fingerprint":env!("MIP_SOURCE_FINGERPRINT"),"library_source_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT,"package_version":env!("CARGO_PKG_VERSION")})
        );
        return Ok(());
    }

    if args.len() < 3
        || !["inspect", "dump", "objective-proof", "solve"].contains(&args[1].as_str())
    {
        return Err(
            "usage: mip-codex build-info OR inspect|dump|objective-proof|solve FILE.mps[.gz] [SECONDS] [NODES]"
                .into(),
        );
    }
    let model = mps::read(&args[2])?;
    let mut out = io::BufWriter::new(io::stdout().lock());
    if args[1] == "solve" {
        let mut limits = mip_codex::solve::Limits::default();
        if let Ok(value) = std::env::var("MIP_THREE_ROW_CUTS") {
            limits.three_row_cuts = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_THREE_ROW_CUTS must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_MIR_CUTS") {
            limits.mir_cuts = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_MIR_CUTS must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_CLIQUE_CUTS") {
            limits.clique_cuts = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_CLIQUE_CUTS must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_AGGREGATE_CUTS") {
            limits.aggregate_cuts = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_AGGREGATE_CUTS must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_FEASIBILITY_PUMP") {
            limits.feasibility_pump = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_FEASIBILITY_PUMP must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_AGGREGATE_INFEASIBILITY") {
            limits.aggregate_infeasibility = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_AGGREGATE_INFEASIBILITY must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_JOINT_INFEASIBILITY") {
            limits.joint_infeasibility = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_JOINT_INFEASIBILITY must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_DEFER_INFEASIBILITY") {
            limits.defer_infeasibility = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_DEFER_INFEASIBILITY must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_GRAPH_FRACTIONAL_COVER") {
            limits.graph_fractional_cover = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_GRAPH_FRACTIONAL_COVER must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_GRAPH_COVER_LOCAL") {
            limits.graph_cover_local = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_GRAPH_COVER_LOCAL must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_GRAPH_ADAPTIVE_LOCAL") {
            limits.graph_adaptive_local = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_GRAPH_ADAPTIVE_LOCAL must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_GRAPH_LOCAL_DIVERSIFY") {
            limits.graph_local_diversify = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_GRAPH_LOCAL_DIVERSIFY must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_GRAPH_ROOT_PREFIX") {
            limits.graph_root_prefix = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_GRAPH_ROOT_PREFIX must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_GRAPH_ROOT_COLORING") {
            limits.graph_root_coloring = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_GRAPH_ROOT_COLORING must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_GRAPH_LOCAL_SEARCH") {
            limits.graph_local_search = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_GRAPH_LOCAL_SEARCH must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_BINARY_GRAPH") {
            limits.binary_graph = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_BINARY_GRAPH must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_BOX_PRIMAL") {
            limits.box_primal = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_BOX_PRIMAL must be 0 or 1".into()),
            };
        }
        if let Ok(value) = std::env::var("MIP_LP_WORKSPACE_MIB") {
            limits.dense_bytes = value
                .parse::<usize>()
                .ok()
                .filter(|&mib| mib > 0)
                .and_then(|mib| mib.checked_mul(1024 * 1024))
                .ok_or("MIP_LP_WORKSPACE_MIB must be a positive, representable MiB value")?;
        }
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
        if let Ok(value) = std::env::var("MIP_FIXED_COLUMNS") {
            limits.fixed_columns = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_FIXED_COLUMNS must be 0 or 1".into()),
            };
        }
        if let Ok(v) = std::env::var("MIP_OBJECTIVE_DOMAIN_REPORT") {
            limits.objective_domain_report = match v.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_OBJECTIVE_DOMAIN_REPORT must be 0 or 1".into()),
            };
        }
        if let Ok(v) = std::env::var("MIP_ROW_PACKING_REPORT") {
            limits.row_packing_report=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_ROW_PACKING_REPORT must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROW_SUBGRADIENT_UNIFORM") {
            limits.row_subgradient_uniform=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_ROW_SUBGRADIENT_UNIFORM must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_DUAL_GUIDED_PRIMAL") {
            limits.dual_guided_primal=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_DUAL_GUIDED_PRIMAL must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROW_SUBGRADIENT_REPORT") {
            limits.row_subgradient_report=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_ROW_SUBGRADIENT_REPORT must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROW_RAY_REPORT") {
            limits.row_ray_report = match v.as_str() {"0"=>false,"1"=>true,_=>return Err("MIP_ROW_RAY_REPORT must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROOT_PRIMAL_DESCENT") {
            limits.root_primal_descent=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_ROOT_PRIMAL_DESCENT must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROOT_OBJECTIVE_BUDGET") {
            limits.root_objective_budget=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_ROOT_OBJECTIVE_BUDGET must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROOT_OBJECTIVE_REPAIR") {
            limits.root_objective_repair=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_ROOT_OBJECTIVE_REPAIR must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROOT_PRICED_BUDGET") {
            limits.root_priced_budget=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_ROOT_PRICED_BUDGET must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROOT_PRICED_CONTINUE") {
            limits.root_priced_continue=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_ROOT_PRICED_CONTINUE must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROOT_PRICED_OBJECTIVE") {
            limits.root_priced_objective=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_ROOT_PRICED_OBJECTIVE must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROOT_PRICE_PORTFOLIO") {
            limits.root_price_portfolio=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_ROOT_PRICE_PORTFOLIO must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROOT_PRICED_REPAIR") {
            limits.root_priced_repair=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_ROOT_PRICED_REPAIR must be 0 or 1".into())};
        }
        if let Ok(v) = std::env::var("MIP_ROOT_SEED_REPAIR") {
            limits.root_seed_repair = match v.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_ROOT_SEED_REPAIR must be 0 or 1".into()),
            };
        }
        if let Ok(v) = std::env::var("MIP_TERMINAL_NEIGHBORHOOD") {
            limits.terminal_neighborhood = match v.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_TERMINAL_NEIGHBORHOOD must be 0 or 1".into()),
            };
        }
        if let Ok(v) = std::env::var("MIP_PARITY_EQUALITIES") {
            limits.parity_equalities = match v.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_PARITY_EQUALITIES must be 0 or 1".into()),
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
        if let Ok(value) = std::env::var("MIP_OBJECTIVE_ROUNDING") {
            match value.as_str() {
                "0" => {
                    limits.objective_rounding = false;
                    limits.objective_rounding_all = false;
                }
                "1" => {
                    limits.objective_rounding = true;
                    limits.objective_rounding_all = false;
                }
                "all" => {
                    limits.objective_rounding = true;
                    limits.objective_rounding_all = true;
                }
                _ => return Err("MIP_OBJECTIVE_ROUNDING must be 0, 1, or all".into()),
            }
        }
        if let Ok(value) = std::env::var("MIP_RINS") {
            limits.rins = match value.as_str() {
                "0" => false,
                "1" => true,
                _ => return Err("MIP_RINS must be 0 or 1".into()),
            };
        }
        for name in ["MIP_STRONG_OBJECTIVE", "MIP_STRONG_INFEASIBLE_HINT"] {
            if let Ok(value) = std::env::var(name) {
                let enabled = match value.as_str() {
                    "0" => false,
                    "1" => true,
                    _ => return Err(format!("{name} must be 0 or 1").into()),
                };
                if name == "MIP_STRONG_OBJECTIVE" {
                    limits.strong_objective = enabled;
                } else {
                    limits.strong_infeasible_hint = enabled;
                }
            }
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
        if let Ok(value) = std::env::var("MIP_COVER_ROUNDS") {
            limits.cover_rounds = value.parse().map_err(|_| "invalid MIP_COVER_ROUNDS")?;
            if !(1..=16).contains(&limits.cover_rounds) {
                return Err("MIP_COVER_ROUNDS must be 1..16".into());
            }
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
    } else if args[1] == "objective-proof" {
        let proof = mip_codex::objective_bound::Grid::from_model(&model);
        serde_json::to_writer(&mut out, &serde_json::json!({"objective_grid":proof}))
            .map_err(|e| e.to_string())?;
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
