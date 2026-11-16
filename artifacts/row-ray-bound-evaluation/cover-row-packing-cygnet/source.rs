//! 単位費用・単位係数の被覆行を互いに重ならず選び、1対2交換する下界診断。
use mip_codex::{mps,objective_bound::Certifier};
use std::time::{Instant,Duration};
fn main(){let args:Vec<_>=std::env::args().collect();let m=mps::read(&args[1]).unwrap();let start=Instant::now();let end=start+Duration::from_secs(5);
assert!(!m.maximize&&m.variables.iter().all(|v|v.lower==Some(0.)&&v.objective==1.));
let rows:Vec<_>=m.rows.iter().enumerate().filter(|(_,r)|r.exact_integral&&r.lower==Some(1.)&&r.coefficients.iter().all(|&(_,a)|a==1.)).map(|(i,_)|i).collect();let mut selected=vec![false;m.rows.len()];let mut owner=vec![None;m.variables.len()];
for &i in rows.iter().rev(){if m.rows[i].coefficients.iter().all(|&(j,_)|owner[j].is_none()){selected[i]=true;for &(j,_)in &m.rows[i].coefficients{owner[j]=Some(i);}}}
let initial=selected.iter().filter(|&&v|v).count();let mut exchanges=0;
loop{if Instant::now()>=end{break;}let mut candidates=vec![Vec::new();m.rows.len()];
for &i in &rows{if selected[i]{continue;}let mut blocker=None;let mut several=false;for &(j,_)in &m.rows[i].coefficients{if let Some(k)=owner[j]{if blocker.is_some_and(|v|v!=k){several=true;break;}blocker=Some(k);}}if !several{if let Some(k)=blocker{candidates[k].push(i);}else{selected[i]=true;for &(j,_)in &m.rows[i].coefficients{owner[j]=Some(i);}}}}
let mut change=None;let mut mark=vec![0usize;m.variables.len()];let mut stamp=0;
'find:for(k,list)in candidates.iter().enumerate(){if list.len()<2{continue;}for(a,&i)in list.iter().enumerate(){if Instant::now()>=end{break 'find;}stamp+=1;for &(j,_)in &m.rows[i].coefficients{mark[j]=stamp;}for &h in &list[a+1..]{if m.rows[h].coefficients.iter().all(|&(j,_)|mark[j]!=stamp){change=Some((k,i,h));break 'find;}}}}
let Some((k,i,h))=change else{break;};selected[k]=false;for &(j,_)in &m.rows[k].coefficients{owner[j]=None;}for v in [i,h]{selected[v]=true;for &(j,_)in &m.rows[v].coefficients{assert!(owner[j].is_none());owner[j]=Some(v);}}exchanges+=1;
}
let y:Vec<_>=selected.iter().map(|&v|if v{1.}else{0.}).collect();let lower:Vec<_>=m.variables.iter().map(|v|v.lower).collect();let upper:Vec<_>=m.variables.iter().map(|v|v.upper).collect();let bound=Certifier::new(&m).and_then(|c|c.bound(&lower,&upper,&y));println!("{}",serde_json::json!({"initial":initial,"exchanges":exchanges,"bound":bound,"selected_rows":selected.iter().enumerate().filter(|(_,v)|**v).map(|(i,_)|i).collect::<Vec<_>>(),"seconds":start.elapsed().as_secs_f64()}));}
