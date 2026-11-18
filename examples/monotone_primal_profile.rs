//! 隔離した主解構成診断。部分問題の下界・不能・最適性は移さない。
use mip_codex::{
    heuristic::Repair,
    mps,
    solve::{solve, Limits},
};
#[path = "../src/exact.rs"]
mod exact;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 4);
    let model = mps::read(&args[1]).unwrap();
    let start = std::time::Instant::now();
    let mut options = vec![];
    for (j, v) in model.variables.iter().enumerate() {
        if v.integer {
            continue;
        }
        for d in [1., -1.] {
            if (d > 0. && v.upper.is_some()) || (d < 0. && v.lower.is_some()) {
                continue;
            }
            let mut count = 0;
            let mut valid = true;
            for row in &model.rows {
                let terms: Vec<_> = row
                    .coefficients
                    .iter()
                    .enumerate()
                    .filter(|(_, (k, _))| *k == j)
                    .collect();
                if terms.is_empty() {
                    continue;
                }
                if terms.len() != 1 {
                    valid = false;
                    break;
                }
                let (k, (_, a)) = terms[0];
                if !a.is_finite()
                    || *a == 0.
                    || !(row.exact_coefficients
                        || row.exact_coefficient_flags.get(k) == Some(&true))
                    || row.lower.is_some() && a * d <= 0.
                    || row.upper.is_some() && a * d >= 0.
                {
                    valid = false;
                    break;
                }
                count += 1;
            }
            if valid && count > 1 {
                options.push((count, j, d));
            }
        }
    }
    options.sort_by(|a, b| b.0.cmp(&a.0));
    let Some(&(dropped, j, d)) = options.first() else {
        println!("no monotone variable");
        return;
    };
    let mut subset = model.clone();
    subset.decimal_source = None;
    subset
        .rows
        .retain(|r| !r.coefficients.iter().any(|(k, _)| *k == j));
    for v in &mut subset.variables {
        v.objective = 0.;
        v.objective_enclosure = Some([0., 0.]);
    }
    subset.offset = 0.;
    subset.offset_enclosure = Some([0., 0.]);
    let base = 0f64
        .max(model.variables[j].lower.unwrap_or(f64::NEG_INFINITY))
        .min(model.variables[j].upper.unwrap_or(f64::INFINITY));
    let v = &mut subset.variables[j];
    v.lower = Some(base);
    v.upper = Some(base);
    v.lower_enclosure = Some([base, base]);
    v.upper_enclosure = Some([base, base]);
    let seconds: f64 = args[2].parse().unwrap();
    let limits = Limits {
        seconds,
        adaptive_resource_repair: false,
        resource_primal: false,
        terminal_neighborhood: false,
        ..Limits::default()
    };
    let r = solve(&subset, &limits);
    let Some(values) = r.values else {
        println!(
            "{}",
            serde_json::json!({"subset_status":r.status,"rows_dropped":dropped,"rows_remaining":subset.rows.len(),"columns":subset.variables.len(),"selected_variable":j,"direction":d,"point":false})
        );
        return;
    };
    let mut point: Vec<_> = model.variables.iter().map(|v| values[&v.name]).collect();
    let mut t = d * base;
    for row in &model.rows {
        let Some(&(_, a)) = row.coefficients.iter().find(|(k, _)| *k == j) else {
            continue;
        };
        let mut lows = vec![];
        let mut highs = vec![];
        for (k, &(col, c)) in row.coefficients.iter().enumerate() {
            if col == j {
                continue;
            }
            let ce = if row.exact_coefficients || row.exact_coefficient_flags.get(k) == Some(&true)
            {
                0.
            } else {
                (c.next_up() - c).abs().max((c - c.next_down()).abs())
            };
            let x = point[col];
            let xe = if x.fract() == 0. {
                0.
            } else {
                (x.next_up() - x).abs().max((x - x.next_down()).abs())
            };
            let products = [
                (c - ce).next_down() * (x - xe).next_down(),
                (c - ce).next_down() * (x + xe).next_up(),
                (c + ce).next_up() * (x - xe).next_down(),
                (c + ce).next_up() * (x + xe).next_up(),
            ];
            lows.push((
                products
                    .iter()
                    .copied()
                    .fold(f64::INFINITY, f64::min)
                    .next_down(),
                1.,
            ));
            highs.push((
                products
                    .iter()
                    .copied()
                    .fold(f64::NEG_INFINITY, f64::max)
                    .next_up(),
                1.,
            ));
        }
        let lo = exact::dot_bounds(lows.into_iter()).unwrap()[0];
        let hi = exact::dot_bounds(highs.into_iter()).unwrap()[1];
        if let Some(b) = row.lower {
            let b = row.lower_enclosure.map_or(b, |x| x[1]);
            t = t.max(((b - lo).next_up() / (a * d)).next_up());
        }
        if let Some(b) = row.upper {
            let b = row.upper_enclosure.map_or(b, |x| x[0]);
            t = t.max(((hi - b).next_up() / (-a * d)).next_up());
        }
    }
    point[j] = d * t;
    let accepted = Repair::new(&model).candidate_check(&point);
    let objective = model.objective(&point);
    let values: std::collections::BTreeMap<_, _> = model
        .variables
        .iter()
        .zip(point)
        .map(|(v, x)| (v.name.clone(), x))
        .collect();
    let result = serde_json::json!({"status":"primal_candidate","objective":objective,"dual_bound":null,"values":values,"subset_status":r.status,"rows_dropped":dropped,"rows_remaining":subset.rows.len(),"selected_variable":j,"direction":d,"internal_point_check":accepted,"component_seconds":start.elapsed().as_secs_f64(),"library_fingerprint":mip_codex::BUILD_SOURCE_FINGERPRINT,"purpose":"standalone feasibility experiment; no parent bound or infeasibility claim"});
    std::fs::write(&args[3], serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    println!(
        "{}",
        serde_json::json!({"objective":objective,"accepted":accepted,"rows_dropped":dropped,"rows_remaining":subset.rows.len(),"subset_status":r.status})
    );
}
