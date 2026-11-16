use mip_codex::{mps,heuristic::Repair,priced_objective_repair::improve_priced};
use std::time::{Instant,Duration};
#[test]
fn small_cover_points_stay_feasible_and_above_the_enumerated_optimum(){
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
  let(x,_)=improve_priced(&m,&repair,&seed,&lower,&upper,Instant::now()+Duration::from_millis(10)).unwrap();assert!(m.check(&x,true,0.).is_ok());assert!(m.objective(&x)>=optimum);assert!(m.objective(&x)<=m.objective(&seed));
 }
}

#[test]
fn successful_series_can_continue_and_unsuccessful_series_stops_at_sixteen(){
 use mip_codex::priced_objective_repair::improve_priced_continued;
 let m=mps::parse(std::io::Cursor::new("NAME continued\nROWS\n N obj\n G a\n G b\nCOLUMNS\n x obj 100 a 1\n x b 1\n y obj 1 a 1\n z obj 1 b 1\nRHS\n rhs a 1 b 1\nBOUNDS\n BV b x\n BV b y\n BV b z\nENDATA\n")).unwrap();let repair=Repair::new(&m);let lo=vec![Some(0.);3];let hi=vec![Some(1.);3];
 for continued in [false,true]{let(x,s)=improve_priced_continued(&m,&repair,&[1.,1.,1.],&lo,&hi,Instant::now()+Duration::from_secs(1),continued).unwrap();assert_eq!(m.objective(&x),2.);assert!(m.check(&x,true,0.).is_ok());assert!(s.improvements>0);assert_eq!(s.calls,if continued{64}else{16});}
 let(x,s)=improve_priced_continued(&m,&repair,&[0.,1.,1.],&lo,&hi,Instant::now()+Duration::from_secs(1),true).unwrap();assert_eq!(m.objective(&x),2.);assert_eq!(s.improvements,0);assert_eq!(s.calls,16);
}
