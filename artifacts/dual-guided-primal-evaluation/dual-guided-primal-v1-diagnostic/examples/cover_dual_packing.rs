//! 非負被覆行の双対乗数を座標ごとに増やす独立診断。既存ソルバーへ委譲しない。
use mip_codex::{mps,objective_bound::Certifier};
use std::time::Instant;
fn main(){
 let args:Vec<_>=std::env::args().collect();let m=mps::read(&args[1]).unwrap();let start=Instant::now();
 assert!(!m.maximize&&m.variables.iter().all(|v|v.lower==Some(0.)&&v.upper==Some(1.)&&v.objective>=0.));
 let rows:Vec<_>=m.rows.iter().enumerate().filter(|(_,r)|r.exact_integral&&r.lower.is_some_and(|b|b>0.)&&r.coefficients.iter().all(|&(_,a)|a>=0.)).map(|(i,_)|i).collect();
 let mut d=vec![0.;m.variables.len()];for &i in &rows{for &(j,a)in &m.rows[i].coefficients{d[j]+=a;}}
 let alpha=m.variables.iter().zip(&d).filter(|(_,d)|**d>0.).map(|(v,d)|v.objective/d).fold(f64::INFINITY,f64::min);
 let cert=Certifier::new(&m).unwrap();let lower:Vec<_>=m.variables.iter().map(|v|v.lower).collect();let upper:Vec<_>=m.variables.iter().map(|v|v.upper).collect();let mut records=Vec::new();
 for trial in 0..36{
  let init=if trial==0{alpha}else{0.};let mut residual:Vec<_>=m.variables.iter().zip(&d).map(|(v,d)|v.objective-init*d).collect();let mut y=vec![0.;m.rows.len()];for &i in &rows{y[i]=init;}
  let mut order=rows.clone();order.sort_by_key(|&i|match trial {0=>i as u64,1=>u64::MAX-i as u64,2=>m.rows[i].coefficients.len()as u64,_=>{let mut z=(i as u64).wrapping_add((trial as u64).wrapping_mul(0x9e3779b97f4a7c15));z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);z^(z>>31)}});
  for i in order{let r=&m.rows[i];let delta=r.coefficients.iter().filter(|&&(_,a)|a>0.).map(|&(j,a)|residual[j]/a).fold(f64::INFINITY,f64::min).max(0.);if !delta.is_finite(){continue;}y[i]+=delta;for &(j,a)in &r.coefficients{residual[j]-=delta*a;}}
  let bound=cert.bound(&lower,&upper,&y);records.push(serde_json::json!({"trial":trial,"bound":bound,"multipliers":y.iter().enumerate().filter(|(_,v)|**v!=0.).map(|(i,v)|serde_json::json!([i,format!("{:016x}",v.to_bits())])).collect::<Vec<_>>()}));
 }
 println!("{}",serde_json::json!({"seconds":start.elapsed().as_secs_f64(),"records":records}));
}
