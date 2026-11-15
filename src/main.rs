mod lp;
mod mip;
mod model;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 || !["inspect", "solve"].contains(&args[1].as_str()) {
        eprintln!("usage: mip-codex inspect|solve model.mps[.gz] [seconds]");
        std::process::exit(2);
    }
    let m = match model::read_mps(std::path::Path::new(&args[2])) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    if args[1] == "inspect" {
        println!(
            "{}",
            serde_json::json!({"name":m.name,"variables":m.variables.len(),"rows":m.rows.len(),"nonzeros":m.rows.iter().map(|r|r.terms.len()).sum::<usize>(),"integers":m.variables.iter().filter(|v|v.integer).count()})
        );
    } else {
        let seconds = args
            .get(3)
            .map_or(Ok(60.), |s| s.parse::<f64>())
            .unwrap_or(-1.);
        if !seconds.is_finite() || seconds < 0. || seconds > 604800. {
            eprintln!("invalid time limit");
            std::process::exit(2);
        }
        let mut result = serde_json::to_value(mip::solve(&m, seconds)).unwrap();
        result["variable_names"] = serde_json::json!(
            m.variables
                .iter()
                .map(|v| v.name.as_str())
                .collect::<Vec<_>>()
        );
        println!("{result}");
    }
}
