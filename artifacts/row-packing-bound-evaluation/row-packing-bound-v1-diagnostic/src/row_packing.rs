//! 単位費用・単位係数の被覆行を非交差に選ぶ報告専用の整数下界。
use crate::model::Model;
use serde::{Deserialize, Serialize};
use std::time::Instant;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proof {
    pub rows: Vec<usize>,
    pub normalized_bound: i64,
    pub exchanges: usize,
    pub seconds: f64,
}
fn point(v: f64, e: Option<[f64; 2]>) -> bool {
    e == Some([v, v]) && v.is_finite()
}
pub fn compute(m: &Model, deadline: Instant) -> Option<Proof> {
    let start = Instant::now();
    if start >= deadline
        || m.variables.len() > 100_000
        || m.rows.len() > 100_000
        || m.rows.iter().map(|r| r.coefficients.len()).sum::<usize>() > 10_000_000
    {
        return None;
    }
    let sign = if m.maximize { -1. } else { 1. };
    if !point(m.offset, m.offset_enclosure)
        || m.offset.fract() != 0.
        || m.offset.abs() > ((1u64 << 52) - 100_000) as f64
        || !m.variables.iter().all(|v| {
            v.lower == Some(0.)
                && point(0., v.lower_enclosure)
                && sign * v.objective == 1.
                && point(v.objective, v.objective_enclosure)
        })
    {
        return None;
    }
    let rows: Vec<_> = m
        .rows
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            r.exact_integral
                && r.lower == Some(1.)
                && point(1., r.lower_enclosure)
                && !r.coefficients.is_empty()
                && r.coefficients.iter().all(|&(_, a)| a == 1.)
        })
        .map(|(i, _)| i)
        .collect();
    let mut selected = vec![false; m.rows.len()];
    let mut owner = vec![None; m.variables.len()];
    for &i in rows.iter().rev() {
        if Instant::now() >= deadline {
            break;
        }
        if m.rows[i]
            .coefficients
            .iter()
            .all(|&(j, _)| owner[j].is_none())
        {
            selected[i] = true;
            for &(j, _) in &m.rows[i].coefficients {
                owner[j] = Some(i);
            }
        }
    }
    let mut exchanges = 0;
    loop {
        if Instant::now() >= deadline {
            break;
        }
        let mut candidates = vec![Vec::new(); m.rows.len()];
        for &i in &rows {
            if Instant::now() >= deadline {
                break;
            }
            if selected[i] {
                continue;
            }
            let mut blocker = None;
            let mut several = false;
            for &(j, _) in &m.rows[i].coefficients {
                if let Some(k) = owner[j] {
                    if blocker.is_some_and(|v| v != k) {
                        several = true;
                        break;
                    }
                    blocker = Some(k);
                }
            }
            if !several {
                if let Some(k) = blocker {
                    candidates[k].push(i);
                } else {
                    selected[i] = true;
                    for &(j, _) in &m.rows[i].coefficients {
                        owner[j] = Some(i);
                    }
                }
            }
        }
        let mut change = None;
        let mut mark = vec![0usize; m.variables.len()];
        let mut stamp = 0;
        'find: for (k, list) in candidates.iter().enumerate() {
            if list.len() < 2 {
                continue;
            }
            for (a, &i) in list.iter().enumerate() {
                if Instant::now() >= deadline {
                    break 'find;
                }
                if m.rows[i]
                    .coefficients
                    .iter()
                    .any(|&(j, _)| owner[j].is_some_and(|v| v != k))
                {
                    continue;
                }
                stamp += 1;
                for &(j, _) in &m.rows[i].coefficients {
                    mark[j] = stamp;
                }
                for &h in &list[a + 1..] {
                    if Instant::now() >= deadline {
                        break 'find;
                    }
                    if m.rows[h]
                        .coefficients
                        .iter()
                        .all(|&(j, _)| mark[j] != stamp && !owner[j].is_some_and(|v| v != k))
                    {
                        change = Some((k, i, h));
                        break 'find;
                    }
                }
            }
        }
        let Some((k, i, h)) = change else {
            break;
        };
        selected[k] = false;
        for &(j, _) in &m.rows[k].coefficients {
            owner[j] = None;
        }
        for v in [i, h] {
            selected[v] = true;
            for &(j, _) in &m.rows[v].coefficients {
                if owner[j].is_some() {
                    return None;
                }
                owner[j] = Some(v);
            }
        }
        exchanges += 1;
    }
    let rows: Vec<_> = selected
        .iter()
        .enumerate()
        .filter(|(_, v)| **v)
        .map(|(i, _)| i)
        .collect();
    if rows.is_empty() {
        return None;
    }
    let normalized_bound = (sign * m.offset) as i64 + rows.len() as i64;
    Some(Proof {
        rows,
        normalized_bound,
        exchanges,
        seconds: start.elapsed().as_secs_f64(),
    })
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn packed_certificate_under_exhaustive_optimum(){
  let mut state=20261116u64;
  for case in 0..200{
   let n=2+case%7;let maximize=case%2==1;let sign=if maximize{-1.}else{1.};let mut rows=Vec::new();
   for i in 0..12{let mut coefficients=Vec::new();for j in 0..n{state=state.wrapping_mul(6364136223846793005).wrapping_add(1);if state>>62!=0{coefficients.push((j,1.));}}if coefficients.is_empty(){coefficients.push((0,1.));}rows.push(serde_json::json!({"name":format!("r{i}"),"lower":1.,"upper":null,"lower_enclosure":[1.,1.],"exact_integral":true,"coefficients":coefficients}));}
   let m:Model=serde_json::from_value(serde_json::json!({"name":"packing","maximize":maximize,"offset":3.,"offset_enclosure":[3.,3.],"variables":(0..n).map(|j|serde_json::json!({"name":format!("x{j}"),"lower":0.,"upper":1.,"integer":true,"lower_enclosure":[0.,0.],"upper_enclosure":[1.,1.],"objective":sign,"objective_enclosure":[sign,sign]})).collect::<Vec<_>>(),"rows":rows})).unwrap();
   let p=compute(&m,Instant::now()+std::time::Duration::from_secs(1)).unwrap();let opt=(0..1usize<<n).filter_map(|bits|{let x:Vec<_>=(0..n).map(|j|((bits>>j)&1)as f64).collect();m.check(&x,true,1e-8).is_ok().then(||sign*m.objective(&x))}).fold(f64::INFINITY,f64::min);assert!(p.normalized_bound as f64<=opt);let mut seen=vec![false;n];for i in p.rows{for &(j,_)in &m.rows[i].coefficients{assert!(!seen[j]);seen[j]=true;}}
   assert!(compute(&m,Instant::now()).is_none());
  }
 }
}
