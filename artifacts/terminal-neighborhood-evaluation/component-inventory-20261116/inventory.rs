use mip_codex::mps;
use std::collections::HashMap;
fn find(p: &mut [usize], j: usize) -> usize { if p[j] != j {p[j] = find(p,p[j]);} p[j] }
fn main() {
 let path=std::env::args().nth(1).unwrap();let m=mps::read(path).unwrap();
 let fixed:Vec<_>=m.variables.iter().map(|v|v.lower.is_some()&&v.lower==v.upper).collect();
 let mut p:Vec<_>=(0..m.variables.len()).collect();let mut size=vec![1usize;p.len()];
 for r in &m.rows {if r.lower.is_none()&&r.upper.is_none(){continue;}let mut first=None;for &(j,a)in &r.coefficients {if a==0.||fixed[j]{continue;}if let Some(k)=first {let mut u=find(&mut p,j);let mut v=find(&mut p,k);if u!=v {if size[u]<size[v]{std::mem::swap(&mut u,&mut v);}p[v]=u;size[u]+=size[v];}}else{first=Some(j);}}}
 let mut groups=HashMap::<usize,[usize;4]>::new();for(j,v)in m.variables.iter().enumerate(){if fixed[j]{continue;}let root=find(&mut p,j);let g=groups.entry(root).or_default();g[0]+=1;g[1]+=usize::from(v.integer);g[2]+=usize::from(v.objective!=0.);}
 for r in &m.rows {if let Some(&(j,_))=r.coefficients.iter().find(|&&(j,a)|a!=0.&&!fixed[j]) {let root=find(&mut p,j);if let Some(g)=groups.get_mut(&root){g[3]+=1;}}}
 let mut components:Vec<_>=groups.values().copied().collect();components.sort_by_key(|g|std::cmp::Reverse(g[0]));
 println!("{}",serde_json::json!({"variables":m.variables.len(),"fixed":fixed.iter().filter(|&&v|v).count(),"components":components.len(),"integer_components":components.iter().filter(|g|g[1]>0).count(),"largest":components.iter().take(10).collect::<Vec<_>>(),"columns":"variables,integer,nonzero_cost,rows"}));
}
