//! 独立成分ごとの自作探索から主解を結合する診断。最適性・不能は宣言しない。
use mip_codex::{
    heuristic,
    model::Model,
    mps,
    solve::{self, Limits, Result},
};
use std::{collections::HashMap, fs::File, time::Instant};
fn find(p: &mut [usize], j: usize) -> usize {
    if p[j] != j {
        p[j] = find(p, p[j]);
    }
    p[j]
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let model = mps::read(&args[1]).unwrap();
    let seconds: f64 = args[2].parse().unwrap();
    let out = std::path::Path::new(&args[3]);
    std::fs::create_dir_all(out).unwrap();
    let started = Instant::now();
    let deadline = started + std::time::Duration::from_secs_f64(seconds);
    let n = model.variables.len();
    let mut parent: Vec<_> = (0..n).collect();
    let mut sizes = vec![1usize; n];
    for row in &model.rows {
        if row.lower.is_none() && row.upper.is_none() {
            continue;
        }
        let mut first = None;
        for &(j, a) in &row.coefficients {
            if a == 0. {
                continue;
            }
            if let Some(k) = first {
                let mut u = find(&mut parent, j);
                let mut v = find(&mut parent, k);
                if u != v {
                    if sizes[u] < sizes[v] {
                        std::mem::swap(&mut u, &mut v);
                    }
                    parent[v] = u;
                    sizes[u] += sizes[v];
                }
            } else {
                first = Some(j);
            }
        }
    }
    let roots: Vec<_> = (0..n).map(|j| find(&mut parent, j)).collect();
    let mut groups = HashMap::<usize, Vec<usize>>::new();
    for (j, &r) in roots.iter().enumerate() {
        groups.entry(r).or_default().push(j);
    }
    let mut groups: Vec<_> = groups.into_iter().collect();
    groups.sort_by_key(|(r, g)| (std::cmp::Reverse(g.len()), *r));
    let mut row_groups = HashMap::<usize, Vec<usize>>::new();
    for (i, r) in model.rows.iter().enumerate() {
        if r.lower.is_none() && r.upper.is_none() {
            continue;
        }
        if let Some(&(j, _)) = r.coefficients.iter().find(|&&(_, a)| a != 0.) {
            row_groups.entry(roots[j]).or_default().push(i);
        }
    }
    let major = groups
        .iter()
        .filter(|(root, _)| row_groups.contains_key(root))
        .count();
    let mut remaining = major;
    let mut x = vec![0.; n];
    let mut successes = 0;
    let mut records = Vec::new();
    let mut failed = false;
    let mut inverse = vec![usize::MAX; n];
    for (index, (root, columns)) in groups.iter().enumerate() {
        if Instant::now() >= deadline {
            failed = true;
            break;
        }
        if !row_groups.contains_key(root) {
            for &j in columns {
                let v = &model.variables[j];
                let cost = if model.maximize {
                    -v.objective
                } else {
                    v.objective
                };
                let mut a = if cost >= 0. {
                    v.lower.unwrap_or(0.)
                } else {
                    v.upper.unwrap_or(0.)
                };
                if v.integer {
                    a = if cost >= 0. { a.ceil() } else { a.floor() };
                }
                if let Some(l) = v.lower {
                    a = a.max(if v.integer { l.ceil() } else { l });
                }
                if let Some(u) = v.upper {
                    a = a.min(if v.integer { u.floor() } else { u });
                }
                x[j] = a;
            }
            continue;
        }
        let row_indices = &row_groups[root];
        for (k, &j) in columns.iter().enumerate() {
            inverse[j] = k;
        }
        let sub = Model {
            name: format!("component-{index}"),
            maximize: model.maximize,
            offset: 0.,
            offset_enclosure: Some([0., 0.]),
            original_decimal: model.original_decimal,
            variables: columns
                .iter()
                .map(|&j| model.variables[j].clone())
                .collect(),
            rows: row_indices
                .iter()
                .map(|&i| {
                    let mut r = model.rows[i].clone();
                    let mut co = Vec::new();
                    let mut flags = Vec::new();
                    for (k, &(j, a)) in r.coefficients.iter().enumerate() {
                        if a != 0. {
                            assert_ne!(inverse[j], usize::MAX);
                            co.push((inverse[j], a));
                            flags.push(r.exact_coefficient_flags.get(k).copied().unwrap_or(false));
                        }
                    }
                    r.coefficients = co;
                    r.exact_coefficient_flags = flags;
                    r
                })
                .collect(),
        };
        let budget = deadline
            .saturating_duration_since(Instant::now())
            .as_secs_f64()
            / remaining as f64;
        remaining -= 1;
        let limits = Limits {
            seconds: budget,
            ..Limits::default()
        };
        let result = solve::solve(&sub, &limits);
        let valid = result
            .values
            .as_ref()
            .and_then(|values| {
                sub.variables
                    .iter()
                    .map(|v| values.get(&v.name).copied())
                    .collect::<Option<Vec<_>>>()
            })
            .filter(|v| heuristic::Repair::new(&sub).candidate_check(v));
        if let Some(values) = valid {
            for (k, &j) in columns.iter().enumerate() {
                x[j] = values[k];
            }
            successes += 1;
        } else {
            failed = true;
        }
        let d = out.join(format!("component-{index:05}"));
        std::fs::create_dir(&d).unwrap();
        serde_json::to_writer(File::create(d.join("model.json")).unwrap(), &sub).unwrap();
        serde_json::to_writer(File::create(d.join("result.json")).unwrap(), &result).unwrap();
        serde_json::to_writer(
            File::create(d.join("mapping.json")).unwrap(),
            &serde_json::json!({"columns":columns,"rows":row_indices}),
        )
        .unwrap();
        records.push(serde_json::json!({"index":index,"variables":columns.len(),"rows":row_indices.len(),"seconds":result.seconds,"status":result.status,"objective":result.objective,"has_point":result.values.is_some()}));
        serde_json::to_writer(File::create(out.join("progress.json")).unwrap(), &records).unwrap();
        for &j in columns {
            inverse[j] = usize::MAX;
        }
    }
    let valid = !failed && successes == major && heuristic::Repair::new(&model).candidate_check(&x);
    let result = Result {
        status: if valid {
            "feasible_component_diagnostic"
        } else {
            "unresolved_component_diagnostic"
        }
        .into(),
        objective: valid.then(|| model.objective(&x)),
        values: valid.then(|| {
            model
                .variables
                .iter()
                .zip(&x)
                .map(|(v, &a)| (v.name.clone(), a))
                .collect()
        }),
        seconds: started.elapsed().as_secs_f64(),
        ..Result::default()
    };
    serde_json::to_writer(File::create(out.join("result.json")).unwrap(), &result).unwrap();
    println!(
        "{}",
        serde_json::json!({"components":groups.len(),"constrained_components":major,"successful_components":successes,"valid":valid,"objective":result.objective,"seconds":result.seconds})
    );
}
