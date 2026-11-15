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
