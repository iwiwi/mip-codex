use mip_codex::{mps,heuristic::Repair};
use std::time::{Instant,Duration};
#[test]
fn repaired_points_preserve_feasibility_and_the_enumerated_objective_bound(){
 let mut rng=19u64;
 for case in 0..200 {
  let n=6+case%5;let mut next=||{rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;rng};
  let costs:Vec<_>=(0..n).map(|j|if j==0{1}else{2+(next()%30)as usize}).collect();
  let rows:Vec<Vec<_>>=(0..4).map(|r|{let mut row:Vec<_>=(0..n).filter(|_|next()%3==0).collect();if row.is_empty(){row.push(r%n);}row}).collect();
  let mut text=String::from("NAME oracle\nROWS\n N obj\n");for i in 0..rows.len(){text.push_str(&format!(" G r{i}\n"));}text.push_str("COLUMNS\n");
  for j in 0..n{text.push_str(&format!(" x{j} obj {}\n",costs[j]));for(i,row)in rows.iter().enumerate(){if row.contains(&j){text.push_str(&format!(" x{j} r{i} 1\n"));}}}
  text.push_str("RHS\n");for i in 0..rows.len(){text.push_str(&format!(" rhs r{i} 1\n"));}text.push_str("BOUNDS\n");for j in 0..n{text.push_str(&format!(" BV b x{j}\n"));}text.push_str("ENDATA\n");
  let m=mps::parse(std::io::Cursor::new(text)).unwrap();let repair=Repair::new(&m);let seed=vec![1.;n];let lower=vec![Some(0.);n];let upper=vec![Some(1.);n];
  let optimum=(0..1usize<<n).filter(|&mask|rows.iter().all(|r|r.iter().any(|&j|mask&(1<<j)!=0))).map(|mask|(0..n).filter(|&j|mask&(1<<j)!=0).map(|j|costs[j]).sum::<usize>()).min().unwrap() as f64;
  let proof=mip_codex::row_subgradient::Proof{multipliers:rows.iter().enumerate().map(|(i,_)|(i,format!("{:016x}",1f64.to_bits()))).collect(),normalized_bound:0.,iterations:0,seconds:0.};
  let mut repair=repair;let(x,_)=mip_codex::dual_guided_primal::improve(&m,&mut repair,&seed,&lower,&upper,&proof,Instant::now()+Duration::from_millis(10)).unwrap();assert!(m.check(&x,true,0.).is_ok());assert!(m.objective(&x)>=optimum);assert!(m.objective(&x)<=m.objective(&seed));
 }
}
