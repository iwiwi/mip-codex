use mip_codex::mps;
use std::time::{Instant,Duration};
#[test]
fn reported_bounds_do_not_exceed_small_enumerated_optima(){
 let mut improved=0;
 let mut rng=19u64;
 for case in 0..200 {
  let n=6+case%5;let mut next=||{rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;rng};
  let costs:Vec<_>=(0..n).map(|j|if j==0{1}else{2+(next()%30)as usize}).collect();
  let rows:Vec<Vec<_>>=(0..4).map(|r|{let mut row:Vec<_>=(0..n).filter(|_|next()%3==0).collect();if row.is_empty(){row.push(r%n);}row}).collect();
  let mut text=String::from("NAME oracle\nROWS\n N obj\n");for i in 0..rows.len(){text.push_str(&format!(" G r{i}\n"));}text.push_str("COLUMNS\n");
  for j in 0..n{text.push_str(&format!(" x{j} obj {}\n",costs[j]));for(i,row)in rows.iter().enumerate(){if row.contains(&j){text.push_str(&format!(" x{j} r{i} 1\n"));}}}
  text.push_str("RHS\n");for i in 0..rows.len(){text.push_str(&format!(" rhs r{i} 1\n"));}text.push_str("BOUNDS\n");for j in 0..n{text.push_str(&format!(" BV b x{j}\n"));}text.push_str("ENDATA\n");
  let m=mps::parse(std::io::Cursor::new(text)).unwrap();let seed=vec![1.;n];
  let optimum=(0..1usize<<n).filter(|&mask|rows.iter().all(|r|r.iter().any(|&j|mask&(1<<j)!=0))).map(|mask|(0..n).filter(|&j|mask&(1<<j)!=0).map(|j|costs[j]).sum::<usize>()).min().unwrap() as f64;
  let initial=mip_codex::row_ray::compute(&m,Instant::now()+Duration::from_millis(100)).unwrap();
  if let Some(proof)=mip_codex::row_subgradient::compute(&m,m.objective(&seed),Some(&initial),Instant::now()+Duration::from_millis(20)){
   assert!(proof.normalized_bound<=optimum);assert!(proof.normalized_bound>initial.normalized_bound);assert!(proof.multipliers.windows(2).all(|w|w[0].0<w[1].0));improved+=1;
  }
 }
 assert!(improved>0);
}

#[test]
fn reporting_a_stronger_bound_does_not_solve_the_global_node_limited_search(){
 use mip_codex::solve::{solve,Limits};
 let mut text=String::from("NAME report\nROWS\n N obj\n G a\n G b\n L cap\nCOLUMNS\n");
 for j in 0..4096{text.push_str(&format!(" x{j} obj {}\n",match j{0=>1,1=>2,2=>100,3=>200,_=>0}));if j<2{text.push_str(&format!(" x{j} a 1\n"));}if j==2||j==3{text.push_str(&format!(" x{j} b 1\n"));}text.push_str(&format!(" x{j} cap 1\n"));}
 text.push_str("RHS\n rhs a 1 b 1\n rhs cap 2\nBOUNDS\n");for j in 0..4096{text.push_str(&format!(" BV b x{j}\n"));}text.push_str("ENDATA\n");let m=mps::parse(std::io::Cursor::new(text)).unwrap();let limits=Limits{nodes:0,seconds:2.,binary_graph:false,cover_cuts:false,clique_cuts:false,aggregate_cuts:false,mir_cuts:false,three_row_cuts:false,root_priced_objective:false,dual_guided_primal:true,row_subgradient_uniform:true,post_primal_subgradient:true,row_subgradient_report:false,..Limits::default()};let before=solve(&m,&limits);let after=solve(&m,&Limits{row_subgradient_report:true,..limits});assert_eq!(before.objective,Some(101.));assert_eq!(after.objective,before.objective);assert_eq!(after.status,"node_limit");assert_eq!(after.lp_solves,0);assert!(after.dual_guided_primal.is_some());let p=after.row_subgradient_proof.unwrap();assert!(p.normalized_bound>before.dual_bound.unwrap());assert!(p.normalized_bound<=101.);
}

#[test]
fn uniform_cost_flag_preserves_the_old_api_and_can_strengthen_the_ray(){
 let m=mps::parse(std::io::Cursor::new("NAME uniform\nROWS\n N obj\n G a\n G b\n G c\n G d\nCOLUMNS\n x0 obj 1 a 1\n x0 c 1\n x1 obj 1 a 1\n x1 b 1\n x2 obj 1 b 1\n x2 c 1\n x3 obj 1 d 1\n x4 obj 1 d 1\nRHS\n rhs a 1 b 1\n rhs c 1 d 1\nBOUNDS\n BV b x0\n BV b x1\n BV b x2\n BV b x3\n BV b x4\nENDATA\n")).unwrap();let initial=mip_codex::row_ray::compute(&m,Instant::now()+Duration::from_secs(1)).unwrap();assert!(mip_codex::row_subgradient::compute(&m,5.,Some(&initial),Instant::now()+Duration::from_millis(20)).is_none());let p=mip_codex::row_subgradient::compute_with_uniform(&m,5.,Some(&initial),Instant::now()+Duration::from_millis(20),true).unwrap();assert!(p.normalized_bound>initial.normalized_bound);assert!(p.normalized_bound<=2.5);
}
