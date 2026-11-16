//! 正の被覆行を一つの非負乗数で集約する下界診断。探索は行わない。
use mip_codex::{model::Model, objective_bound::Certifier};
use std::{fs::File,time::Instant};
fn main(){
 let args:Vec<_>=std::env::args().collect();
 let m:Model=if args[1].ends_with(".json"){serde_json::from_reader(File::open(&args[1]).unwrap()).unwrap()}else{mip_codex::mps::read(&args[1]).unwrap()};
 let start=Instant::now();
 let unit=m.variables.iter().all(|v|v.lower==Some(0.)&&v.upper==Some(1.)&&v.lower_enclosure==Some([0.,0.])&&v.upper_enclosure==Some([1.,1.]));
 if !unit{println!("{}",serde_json::json!({"supported":false,"reason":"unit_box"}));return;}
 let sign=if m.maximize{-1.}else{1.};
 let mut d=vec![0.;m.variables.len()];let mut b=0.;let mut rows=Vec::new();
 for(i,r)in m.rows.iter().enumerate(){
  if r.exact_integral&&r.lower.is_some_and(|v|v>0.)&&r.coefficients.iter().all(|&(_,v)|v>=0.){
   rows.push(i);b+=r.lower.unwrap();for &(j,a)in &r.coefficients{d[j]+=a;}
  }
 }
 let mut slope=b;let mut breaks=Vec::new();
 for(j,v)in m.variables.iter().enumerate(){let c=sign*v.objective;if c<=0.{slope-=d[j];}else if d[j]>0.{breaks.push((c/d[j],d[j]));}}
 breaks.sort_by(|a,b|a.0.total_cmp(&b.0));
 let mut alpha=0.;if slope>0.{for(t,w)in breaks{alpha=t;slope-=w;if slope<=0.{break;}}}
 let mut y=vec![0.;m.rows.len()];for &i in &rows{y[i]=alpha;}
 let lower:Vec<_>=m.variables.iter().map(|v|v.lower).collect();let upper:Vec<_>=m.variables.iter().map(|v|v.upper).collect();
 let bound=Certifier::new(&m).and_then(|c|c.bound(&lower,&upper,&y));
 println!("{}",serde_json::json!({"supported":true,"rows":rows,"multiplier":alpha,"multiplier_bits":format!("{:016x}",alpha.to_bits()),"normalized_bound":bound,"normalized_bound_bits":bound.map(|v|format!("{:016x}",v.to_bits())),"seconds":start.elapsed().as_secs_f64()}));
}
