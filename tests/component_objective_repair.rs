use mip_codex::{mps,heuristic::Repair,component_objective_repair};
use std::time::{Instant,Duration};
#[test]
fn combined_candidates_preserve_original_constraints_and_enumerated_optima(){
 let mut rng=719u64;let mut gains=0;
 for case in 0..100{
  let components=2+case%3;let width=2+case%3;let n=components*width+1;
  let mut text=String::from("NAME components\nROWS\n N obj\n");let mut rows=Vec::new();
  for group in 0..components{for _ in 0..2{let mut row=Vec::new();for j in 0..width{rng=rng.wrapping_mul(6364136223846793005).wrapping_add(1);if rng>>63!=0{row.push(group*width+j);}}if row.is_empty(){row.push(group*width);}rows.push(row);}}
  for i in 0..rows.len(){text.push_str(&format!(" G r{i}\n"));}text.push_str("COLUMNS\n");for j in 0..n{text.push_str(&format!(" x{j} obj 1\n"));for(i,row)in rows.iter().enumerate(){if row.contains(&j){text.push_str(&format!(" x{j} r{i} 1\n"));}}}text.push_str("RHS\n");for i in 0..rows.len(){text.push_str(&format!(" rhs r{i} 1\n"));}text.push_str("BOUNDS\n");for j in 0..n{text.push_str(&format!(" BV b x{j}\n"));}text.push_str("ENDATA\n");
  let m=mps::parse(std::io::Cursor::new(text)).unwrap();let seed=vec![1.;n];let repair=Repair::new(&m);let lo=vec![Some(0.);n];let hi=vec![Some(1.);n];let(x,s)=component_objective_repair::improve(&m,&repair,&seed,&lo,&hi,Instant::now()+Duration::from_millis(10)).unwrap();assert!(m.check(&x,true,0.).is_ok());assert!(m.objective(&x)<=m.objective(&seed));gains+=s.improvements;
  let optimum=(0..1usize<<n).filter(|&bits|rows.iter().all(|row|row.iter().any(|&j|bits&(1<<j)!=0))).map(|bits|bits.count_ones()as f64).fold(f64::INFINITY,f64::min);assert!(m.objective(&x)>=optimum);assert!(component_objective_repair::improve(&m,&repair,&seed,&lo,&hi,Instant::now()).is_none());
 }
 assert!(gains>0);
}
#[test]
fn component_points_do_not_certify_global_optimality_at_the_node_limit(){
 use mip_codex::solve::{solve,Limits};let mut text=String::from("NAME components\nROWS\n N obj\n E a\n E b\nCOLUMNS\n");for j in 0..4096{text.push_str(&format!(" x{j} obj 1\n"));if j<2{text.push_str(&format!(" x{j} a 1\n"));}if j==2||j==3{text.push_str(&format!(" x{j} b 1\n"));}}text.push_str("RHS\n rhs a 1 b 1\nBOUNDS\n");for j in 0..4096{text.push_str(&format!(" BV b x{j}\n"));}text.push_str("ENDATA\n");let m=mps::parse(std::io::Cursor::new(text)).unwrap();let r=solve(&m,&Limits{nodes:0,seconds:11.,binary_graph:false,cover_cuts:false,clique_cuts:false,aggregate_cuts:false,mir_cuts:false,three_row_cuts:false,component_objective_repair:true,row_subgradient_report:false,..Limits::default()});assert_eq!(r.status,"node_limit");assert_eq!(r.lp_solves,0);assert_eq!(r.objective,Some(2.));assert!(r.component_objective_repair.is_some());let v=r.values.unwrap();let x:Vec<_>=m.variables.iter().map(|c|v[&c.name]).collect();assert!(m.check(&x,true,0.).is_ok());
}
