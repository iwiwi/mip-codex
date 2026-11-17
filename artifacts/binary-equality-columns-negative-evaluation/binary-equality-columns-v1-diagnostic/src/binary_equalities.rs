//! Exact binary x=y rows identify columns; original points retain every variable.
use crate::model::Model;
use std::time::Instant;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Proof {
    pub groups: Vec<Vec<usize>>,
    pub rows: Vec<usize>,
}
#[derive(Clone)]
pub(crate) struct Aliases {
    pub representatives: Vec<usize>,
    pub redundant_rows: Vec<bool>,
    pub removed: usize,
}
fn find(parent: &mut [usize], mut j: usize) -> usize {
    while parent[j] != j { parent[j] = parent[parent[j]]; j = parent[j]; }
    j
}
fn binary(model: &Model, j: usize) -> bool {
    let v = &model.variables[j];
    v.integer && v.lower == Some(0.) && v.upper == Some(1.)
        && v.lower_enclosure == Some([0.; 2]) && v.upper_enclosure == Some([1.; 2])
}
fn edge(model: &Model, i: usize) -> Option<(usize, usize)> {
    let r=&model.rows[i];
    if r.lower != Some(0.) || r.upper != Some(0.) || r.lower_enclosure != Some([0.;2])
        || r.upper_enclosure != Some([0.;2]) || r.coefficients.len()!=2 {return None;}
    let [(j,a),(k,b)]=r.coefficients.as_slice() else {return None;};
    (j!=k && a.is_finite() && *a!=0. && a.fract()==0. && a.abs()<=(1u64<<52)as f64 && *a == -*b
        && r.coefficient_error(0,*a)==0. && r.coefficient_error(1,*b)==0.
        && binary(model,*j) && binary(model,*k)).then_some((*j,*k))
}
impl Aliases {
    pub fn proof(&self)->Proof {
        let mut groups=std::collections::BTreeMap::<usize,Vec<usize>>::new();
        for (j,&r) in self.representatives.iter().enumerate() {groups.entry(r).or_default().push(j);}
        Proof {groups:groups.into_values().filter(|g|g.len()>1).collect(),rows:self.redundant_rows.iter().enumerate().filter_map(|(i,&v)|v.then_some(i)).collect()}
    }
    pub fn build(model:&Model, deadline:Instant)->Option<Self> {
        let n=model.variables.len();
        if n>1_000_000 || model.rows.iter().map(|r|r.coefficients.len()).sum::<usize>()>10_000_000 || Instant::now()>=deadline {return None;}
        let mut parent:Vec<_>=(0..n).collect();
        for i in 0..model.rows.len() {
            if i%1024==0 && Instant::now()>=deadline {return None;}
            if let Some((j,k))=edge(model,i) {let a=find(&mut parent,j);let b=find(&mut parent,k);if a!=b {parent[a.max(b)]=a.min(b);}}
        }
        for j in 0..n {parent[j]=find(&mut parent,j);}
        let mut counts=vec![0usize;n];for &r in &parent {counts[r]+=1;}
        let mut safe=vec![true;n];let mut cost_abs=vec![0u128;n];
        for (j,v) in model.variables.iter().enumerate() {
            let r=parent[j];if counts[r]<2 {continue;}
            if !v.objective.is_finite() || v.objective.fract()!=0. || v.objective.abs()>(1u64<<52)as f64
                || v.objective_enclosure!=Some([v.objective;2]) {safe[r]=false;continue;}
            cost_abs[r]=cost_abs[r].saturating_add(v.objective.abs()as u128);
            if cost_abs[r]>(1u128<<52) {safe[r]=false;}
        }
        for (i,row) in model.rows.iter().enumerate() {
            if i%1024==0 && Instant::now()>=deadline {return None;}
            let mut magnitudes=std::collections::HashMap::<usize,u128>::new();
            for (k,&(j,a)) in row.coefficients.iter().enumerate() {
                let r=parent[j];if counts[r]<2 || !safe[r] {continue;}
                if !a.is_finite() || a.fract()!=0. || a.abs()>(1u64<<52)as f64 || row.coefficient_error(k,a)!=0. {safe[r]=false;continue;}
                let sum=magnitudes.entry(r).or_default();*sum=sum.saturating_add(a.abs()as u128);if *sum>(1u128<<52) {safe[r]=false;}
            }
        }
        for j in 0..n {if !safe[parent[j]] {parent[j]=j;}}
        let removed=parent.iter().enumerate().filter(|&(j,r)|j!=*r).count();if removed==0{return None;}
        let redundant_rows=(0..model.rows.len()).map(|i|edge(model,i).is_some_and(|(j,k)|parent[j]==parent[k])).collect();
        Some(Self {representatives:parent,redundant_rows,removed})
    }
    pub fn bounds(&self,lower:&[Option<f64>],upper:&[Option<f64>])->Option<(Vec<Option<f64>>,Vec<Option<f64>>)> {
        let mut l=lower.to_vec();let mut u=upper.to_vec();
        for (j,&r) in self.representatives.iter().enumerate() {
            if let Some(v)=lower[j] {l[r]=Some(l[r].map_or(v,|old|old.max(v)));}
            if let Some(v)=upper[j] {u[r]=Some(u[r].map_or(v,|old|old.min(v)));}
        }
        for (j,&r) in self.representatives.iter().enumerate() {
            if l[r].zip(u[r]).is_some_and(|(a,b)|a>b) {return None;}
            l[j]=l[r];u[j]=u[r];
        }
        Some((l,u))
    }
}
#[cfg(test)]
mod tests {
 use super::*;
 fn model(cost:&str,extra:&str)->Model {
  let text=format!("NAME alias\nROWS\n N c\n E e\n L r\nCOLUMNS\n x c {cost} e 1\n x r 1\n y c 2 e -1\n y r {extra}\nRHS\n rhs r 1\nBOUNDS\n BV b x\n BV b y\nENDATA\n");
  crate::mps::parse(std::io::Cursor::new(text)).unwrap()
 }
 #[test]fn exact_components_propagate_bounds_and_detect_conflict(){
  let m=model("1","1");let a=Aliases::build(&m,Instant::now()+std::time::Duration::from_secs(1)).unwrap();assert_eq!(a.removed,1);assert_eq!(a.representatives,vec![0,0]);assert!(a.redundant_rows[0]);
  let(l,u)=a.bounds(&[Some(0.),Some(1.)],&[Some(1.),Some(1.)]).unwrap();assert_eq!(l,vec![Some(1.);2]);assert_eq!(u,vec![Some(1.);2]);assert!(a.bounds(&[Some(1.),Some(0.)],&[Some(1.),Some(0.)]).is_none());
 }
 #[test]fn decimal_cost_coefficient_and_expired_budget_are_rejected(){
  let end=Instant::now()+std::time::Duration::from_secs(1);assert!(Aliases::build(&model("0.1","1"),end).is_none());assert!(Aliases::build(&model("1","0.1"),end).is_none());assert!(Aliases::build(&model("1","1"),Instant::now()).is_none());
  let mut m=model("1","1");m.rows[0].lower_enclosure=Some([-f64::from_bits(1),f64::from_bits(1)]);assert!(Aliases::build(&m,end).is_none());
 }
 #[test]fn excessive_integer_sum_does_not_round_a_merged_cost(){
  let mut m=model("4503599627370496","1");m.variables[1].objective=4503599627370496.;m.variables[1].objective_enclosure=Some([4503599627370496.;2]);assert!(Aliases::build(&m,Instant::now()+std::time::Duration::from_secs(1)).is_none());
 }
}
