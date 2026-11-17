//! 原問題の零費用singletonを制約を緩める整数端点へ固定する証明。
use crate::model::Model;
use serde::{Deserialize,Serialize};
use std::time::Instant;
#[derive(Clone,Debug,Serialize,Deserialize)]pub struct Fix{pub variable:usize,pub source_row:usize,pub upper_endpoint:bool,pub value_bits:String}
impl Fix{pub fn value(&self)->Option<f64>{let v=f64::from_bits(u64::from_str_radix(&self.value_bits,16).ok()?);v.is_finite().then_some(v)}}
#[derive(Clone,Debug,Serialize,Deserialize)]pub struct Proof{pub fixes:Vec<Fix>}
fn integer(v:f64)->Option<i128>{(v.is_finite()&&v.fract()==0.&&v.abs()<=(1u64<<52)as f64).then_some(v as i128)}
fn exact_integer(v:Option<f64>,b:Option<[f64;2]>)->Option<i128>{let v=v?;(b==Some([v;2])).then_some(())?;integer(v)}
pub fn derive(model:&Model,deadline:Instant)->Option<Proof>{
 if Instant::now()>=deadline||model.variables.len()>1_000_000||!model.variables.iter().any(|v|v.objective==0.&&v.objective_enclosure==Some([0.;2])){return None;}
 let mut seen=vec![None;model.variables.len()];let mut multiple=vec![false;model.variables.len()];
 for(i,row)in model.rows.iter().enumerate(){
  if i%1024==0&&Instant::now()>=deadline{return None;}
  if !row.exact_coefficients{return None;}
  for&(j,a)in &row.coefficients{if a==0.{continue;}if seen[j].is_some(){multiple[j]=true;}else{seen[j]=Some((i,a));}}
 }
 let original_fixed:Vec<_>=model.variables.iter().map(|v|match(v.lower,v.upper){(Some(l),Some(u))=>{let(l,u)=if v.integer{(l.ceil(),u.floor())}else{(l,u)};(l.is_finite()&&l==u).then_some(l)},_=>None}).collect();
 let mut fixes=Vec::new();let mut shifts=vec![Some(0i128);model.rows.len()];
 for(j,v)in model.variables.iter().enumerate(){
  if j%1024==0&&Instant::now()>=deadline{return None;}
  if original_fixed[j].is_some()||multiple[j]||v.objective!=0.||v.objective_enclosure!=Some([0.;2]){continue;}
  let Some((i,a))=seen[j]else{continue;};let row=&model.rows[i];if !row.exact_integral{continue;}
  let upper=match(row.lower.is_some(),row.upper.is_some()){(false,true)=>a<0.,(true,false)=>a>0.,_=>continue};
  let (b,enclosure)=if upper{(v.upper,v.upper_enclosure)}else{(v.lower,v.lower_enclosure)};
  let Some(b)=b else{continue;};if !b.is_finite()||enclosure!=Some([b;2]){continue;}
  let value=if v.integer{if upper{b.floor()}else{b.ceil()}}else{b};let Some(value_i)=integer(value)else{continue;};
  if v.lower.is_some_and(|l|!l.is_finite()||value<l)||v.upper.is_some_and(|u|!u.is_finite()||value>u)||v.lower.is_some()&&v.lower_enclosure.is_none_or(|b|!b[0].is_finite()||!b[1].is_finite()||b[0]>b[1]||value<b[1])||v.upper.is_some()&&v.upper_enclosure.is_none_or(|b|!b[0].is_finite()||!b[1].is_finite()||b[0]>b[1]||value>b[0]){continue;}
  let Some(a_i)=integer(a)else{continue;};let bound=if row.upper.is_some(){exact_integer(row.upper,row.upper_enclosure)}else{exact_integer(row.lower,row.lower_enclosure)};let Some(bound)=bound else{continue;};
  shifts[i]=shifts[i].and_then(|s|s.checked_add(a_i.checked_mul(value_i)?.abs())).filter(|s|bound.abs().checked_add(*s).is_some_and(|v|v<=1i128<<52));
  fixes.push(Fix{variable:j,source_row:i,upper_endpoint:upper,value_bits:format!("{:016x}",value.to_bits())});
 }
 let affected:std::collections::BTreeSet<_>=fixes.iter().map(|f|f.source_row).collect();
 for i in affected {if Instant::now()>=deadline{return None;}let row=&model.rows[i];let bound=if row.upper.is_some(){exact_integer(row.upper,row.upper_enclosure)}else{exact_integer(row.lower,row.lower_enclosure)}?;for&(j,a)in &row.coefficients{if let Some(v)=original_fixed[j]{shifts[i]=shifts[i].and_then(|s|s.checked_add(integer(a)?.checked_mul(integer(v)?)?.abs())).filter(|s|bound.abs().checked_add(*s).is_some_and(|n|n<=1i128<<52));}}}
 fixes.retain(|f|shifts[f.source_row].is_some());
 (!fixes.is_empty()&&Instant::now()<deadline).then_some(Proof{fixes})
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn one_sided_original_zero_cost_columns_choose_the_relaxing_endpoint(){
  for sense in ["L","G"]{for a in [-2,2]{let text=format!("NAME dom\nROWS\n N c\n {sense} r\nCOLUMNS\n x c 1 r 1\n z c 0 r {a}\nRHS\n rhs r 2\nBOUNDS\n BV b x\n LI b z 0\n UI b z 3\nENDATA\n");let m=crate::mps::parse(std::io::Cursor::new(text)).unwrap();let p=derive(&m,Instant::now()+std::time::Duration::from_secs(1)).unwrap();assert_eq!(p.fixes.len(),1);let f=&p.fixes[0];assert_eq!(f.variable,1);assert_eq!(f.value(),Some(if(sense=="L")== (a<0){3.}else{0.}));}}
 }
 #[test]fn original_fixed_columns_are_included_in_exact_shift_work_guard(){let text="NAME large\nROWS\n N c\n G r\nCOLUMNS\n x c 1 r 2\n z c 0 r 1\nRHS\n rhs r 0\nBOUNDS\n FX b x 4503599627370496\n BV b z\nENDATA\n";let m=crate::mps::parse(std::io::Cursor::new(text)).unwrap();assert!(derive(&m,Instant::now()+std::time::Duration::from_secs(1)).is_none());}
 #[test]fn equality_multiple_incidence_unknown_cost_and_expired_budget_are_not_fixed(){
  let text="NAME no\nROWS\n N c\n L r\n L s\nCOLUMNS\n x c 0 r 1\n x s 1\n z c 1 r 1\nRHS\n rhs r 2 s 2\nBOUNDS\n BV b x\n BV b z\nENDATA\n";let m=crate::mps::parse(std::io::Cursor::new(text)).unwrap();assert!(derive(&m,Instant::now()+std::time::Duration::from_secs(1)).is_none());assert!(derive(&m,Instant::now()).is_none());
 }
}

#[cfg(test)]mod mip_tests{
 use super::*;
 #[test]fn original_mip_optimum_and_point_are_preserved_for_both_senses_with_offset(){
  for maximize in [false,true]{let text=format!("NAME dommip\nOBJSENSE\n {}\nROWS\n N c\n L r\nCOLUMNS\n x c {} r 1\n z c 0 r -2\nRHS\n rhs r 0 c -100\nBOUNDS\n BV b x\n LI b z 0\n UI b z 3\nENDATA\n",if maximize{"MAX"}else{"MIN"},if maximize{1}else{-1});let m=crate::mps::parse(std::io::Cursor::new(text)).unwrap();let before=crate::solve::solve(&m,&crate::solve::Limits{seconds:1.,zero_dominance:false,..crate::solve::Limits::default()});let after=crate::solve::solve(&m,&crate::solve::Limits{seconds:1.,zero_dominance:true,..crate::solve::Limits::default()});assert_eq!(before.status,"optimal");assert_eq!(after.status,before.status);assert_eq!(after.objective,before.objective);assert_eq!(after.objective,Some(if maximize{101.}else{99.}));assert_eq!(after.zero_dominance_proof.as_ref().unwrap().fixes.len(),1);let x:Vec<_>=m.variables.iter().map(|v|after.values.as_ref().unwrap()[&v.name]).collect();assert!(m.check(&x,true,0.).is_ok());assert_eq!(m.variables[1].upper,Some(3.));}
 }
 #[test]fn integer_endpoint_is_rounded_in_the_relaxing_direction(){let m=crate::mps::parse(std::io::Cursor::new("NAME fracbound\nROWS\n N c\n L r\nCOLUMNS\n x c 1 r 1\n z c 0 r 1\nRHS\n rhs r 3\nBOUNDS\n BV b x\n LI b z 0.5\n UI b z 2.5\nENDATA\n")).unwrap();let p=derive(&m,Instant::now()+std::time::Duration::from_secs(1)).unwrap();assert_eq!(p.fixes.len(),1);assert_eq!(p.fixes[0].value(),Some(1.));}
}
