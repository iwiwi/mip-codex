use mip_codex::mps;
fn main(){
 let p=std::env::args().nth(1).unwrap();let m=mps::read(&p).unwrap();let n=m.variables.len();let mut parent:Vec<usize>=(0..n).collect();
 fn find(p:&mut[usize],mut j:usize)->usize{while p[j]!=j{p[j]=p[p[j]];j=p[j];}j}
 let mut eligible=0;let mut joined=0;
 for r in &m.rows {
  if r.coefficients.len()!=2||r.lower!=Some(0.)||r.upper!=Some(0.)||r.lower_enclosure!=Some([0.;2])||r.upper_enclosure!=Some([0.;2])||!r.exact_coefficients {continue;}
  let [(j,a),(k,b)]=r.coefficients.as_slice() else{unreachable!()};
  if *a==0.||!a.is_finite()||a.fract()!=0.||*a != -*b {continue;}
  if ![*j,*k].into_iter().all(|i|{let v=&m.variables[i];v.integer&&v.lower==Some(0.)&&v.upper==Some(1.)&&v.lower_enclosure==Some([0.;2])&&v.upper_enclosure==Some([1.;2])}){continue;}
  eligible+=1;let x=find(&mut parent,*j);let y=find(&mut parent,*k);if x!=y{parent[x]=y;joined+=1;}
 }
 let mut sizes=vec![0usize;n];for j in 0..n{let r=find(&mut parent,j);sizes[r]+=1;}
 println!("{}",serde_json::json!({"variables":n,"rows":m.rows.len(),"eligible":eligible,"redundant_columns":joined,"aliased_variables":sizes.iter().filter(|&&x|x>1).sum::<usize>(),"largest_component":sizes.iter().max()}));
}
