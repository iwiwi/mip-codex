//! Experimental exact pure-integer Gomory derivation; not connected to search.
//! Mathematical references and derivation: docs/SAFE_GOMORY_HYPOTHESIS.md.
use crate::model::{Model, Row};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Term { pub row: usize, pub upper: bool, pub weight: i64 }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proof { #[serde(default)] pub normalized: bool, #[serde(default)] pub mixed: bool, pub terms: Vec<Term>, pub pivot: usize, pub upper_bounds: Vec<usize> }
fn integer(v: f64) -> Option<i128> {
    (v.is_finite() && v.abs() <= (1u64 << 52) as f64 && v.fract() == 0.).then_some(v as i128)
}
fn gcd(mut a: i128, mut b: i128) -> i128 { while b != 0 { let r=a%b; a=b; b=r; } a }
pub fn derive(model: &Model, proof: &Proof) -> Option<Row> {
    let n=model.variables.len();
    if n==0 || n>100_000 || proof.pivot>=n || proof.terms.is_empty() || proof.terms.len()>1024
        || (!proof.mixed && !model.variables.iter().all(|v|v.integer))
        || !model.variables[proof.pivot].integer
        || proof.upper_bounds.windows(2).any(|w|w[0]>=w[1])
        || proof.upper_bounds.last().is_some_and(|&j|j>=n)
        || proof.terms.windows(2).any(|w|(w[0].row,w[0].upper)>=(w[1].row,w[1].upper)) { return None; }
    let mut anchors=Vec::with_capacity(n);let mut directions=Vec::with_capacity(n);
    for (j,v) in model.variables.iter().enumerate() {
        let upper=proof.upper_bounds.binary_search(&j).is_ok();
        let (b,e)=if upper {(v.upper?,v.upper_enclosure?)}else{(v.lower?,v.lower_enclosure?)};
        if e!=[b;2] {return None;}
        anchors.push(integer(b)?);directions.push(if upper {-1i128}else{1});
    }
    let mut rows=Vec::new();let mut a=vec![0i128;n];let mut b=0i128;let mut work=0usize;
    for t in &proof.terms {
        if t.weight==0 {return None;}
        let row=model.rows.get(t.row)?;
        work=work.checked_add(row.coefficients.len())?;if work>1_000_000 || !row.exact_integral || !row.exact_coefficients {return None;}
        let (rhs,enclosure)=if t.upper {(row.upper?,row.upper_enclosure?)}else{(row.lower?,row.lower_enclosure?)};
        if enclosure!=[rhs;2] {return None;}
        let sign=if t.upper {1i128}else{-1};let rhs=integer(rhs)?.checked_mul(sign)?;
        let mut coefficients=Vec::new();let mut previous=None;
        for &(j,v) in &row.coefficients {
            if j>=n || previous.is_some_and(|k|k>=j) {return None;}previous=Some(j);
            let v=integer(v)?.checked_mul(sign)?;
            a[j]=a[j].checked_add(v.checked_mul(t.weight as i128)?)?;coefficients.push((j,v));
        }
        b=b.checked_add(rhs.checked_mul(t.weight as i128)?)?;let slack_integer=coefficients.iter().all(|&(j,a)|a==0||model.variables[j].integer);rows.push((coefficients,rhs,t.weight as i128,slack_integer));
    }
    let orientation=if a[proof.pivot]<0 {-1i128}else{1};
    for v in &mut a {*v=v.checked_mul(orientation)?;}b=b.checked_mul(orientation)?;
    let modulus=a[proof.pivot];if modulus<=0 {return None;}
    let mut shifted=b;
    for (&v,&anchor) in a.iter().zip(&anchors) {shifted=shifted.checked_sub(v.checked_mul(anchor)?)?;}
    let fractional=shifted.rem_euclid(modulus);if fractional==0 {return None;}
    let h=modulus.checked_sub(fractional)?;
    let gmi=|coefficient:i128,integer:bool|->Option<i128> {
        if integer {let rem=coefficient.rem_euclid(modulus);if rem<=fractional {rem.checked_mul(h)}else{fractional.checked_mul(modulus.checked_sub(rem)?)}}
        else if coefficient>=0 {coefficient.checked_mul(h)}else{fractional.checked_mul(coefficient.checked_neg()?)}
    };
    let mut c=Vec::with_capacity(n);let mut rhs=if proof.mixed {fractional.checked_mul(h)?}else{fractional};
    for (j,((&v,&direction),&anchor)) in a.iter().zip(&directions).zip(&anchors).enumerate() {
        let value=v.checked_mul(direction)?;
        let rem=if proof.mixed {gmi(value,model.variables[j].integer)?}else{value.rem_euclid(modulus)};
        let coefficient=rem.checked_mul(direction)?;rhs=rhs.checked_add(coefficient.checked_mul(anchor)?)?;c.push(coefficient);
    }
    for (row,b,weight,slack_integer) in rows {
        let value=weight.checked_mul(orientation)?;
        let rem=if proof.mixed {gmi(value,slack_integer)?}else{value.rem_euclid(modulus)};
        rhs=rhs.checked_sub(rem.checked_mul(b)?)?;
        for (j,v) in row {c[j]=c[j].checked_sub(rem.checked_mul(v)?)?;}
    }
    let mut common=rhs.checked_abs()?;
    for &v in &c {common=gcd(common,v.checked_abs()?);}
    if common>1 {rhs/=common;for v in &mut c {*v/=common;}}
    let exact_limit=1i128<<52;
    if rhs.checked_abs()?>exact_limit || c.iter().any(|v|v.checked_abs().is_none_or(|v|v>exact_limit)) {return None;}
    let coefficients:Vec<_>=c.into_iter().enumerate().filter(|&(_,v)|v!=0).map(|(j,v)|(j,v as f64)).collect();
    if coefficients.is_empty() {return None;}
    let largest=coefficients.iter().map(|&(_,v)|v.abs()).fold((rhs as f64).abs(),f64::max);
    let scale=if proof.normalized && largest>16384. {crate::scaling::power(largest)}else{1.};
    let coefficients:Vec<_>=coefficients.into_iter().map(|(j,v)|(j,v/scale)).collect();
    let rhs=rhs as f64/scale;
    let integral=rhs.fract()==0. && coefficients.iter().all(|&(_,v)|v.fract()==0.);
    Some(Row{name:String::new(),coefficients,lower:Some(rhs),upper:None,lower_enclosure:Some([rhs;2]),upper_enclosure:None,exact_integral:integral,exact_coefficients:true,exact_coefficient_flags:vec![]})
}
pub fn verify(model: &Model, proof: &Proof, row: &Row) -> bool {
    derive(model,proof).is_some_and(|r|r.coefficients==row.coefficients && r.lower==row.lower && row.upper.is_none()
        && r.lower_enclosure==row.lower_enclosure && row.upper_enclosure.is_none() && row.exact_integral==r.exact_integral
        && row.exact_coefficients && row.exact_coefficient_flags.is_empty())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn example()->Model {
        crate::mps::parse(std::io::Cursor::new("NAME g\nROWS\n N obj\n L r\nCOLUMNS\n x obj 0\n x r 2\n y obj 0\n y r 2\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n")).unwrap()
    }
    fn proof()->Proof {Proof{normalized:false,mixed:false,terms:vec![Term{row:0,upper:true,weight:1}],pivot:0,upper_bounds:vec![]}}
    #[test]
    fn fractional_knapsack_cut_and_tampering() {
        let m=example();let p=proof();let cut=derive(&m,&p).unwrap();
        assert_eq!(cut.coefficients,vec![(0,-1.),(1,-1.)]);assert_eq!(cut.lower,Some(-1.));assert!(verify(&m,&p,&cut));
        for x in 0..=1 {for y in 0..=1 {if 2*x+2*y<=3 {assert!(-x-y>=-1);}}}
        let mut bad=cut.clone();bad.lower=Some(0.);bad.lower_enclosure=Some([0.;2]);assert!(!verify(&m,&p,&bad));
    }
    #[test]
    fn complements_and_negative_multipliers_remain_valid() {
        let m=example();let mut p=proof();p.upper_bounds=vec![0,1];p.terms[0].weight=-3;
        let cut=derive(&m,&p).unwrap();assert!(verify(&m,&p,&cut));
        for x in 0..=1 {for y in 0..=1 {if 2*x+2*y<=3 {let point=[x as f64,y as f64];assert!(cut.coefficients.iter().map(|&(j,a)|a*point[j]).sum::<f64>()>=cut.lower.unwrap());}}}
    }
    #[test]
    fn invalid_or_inexact_proofs_are_rejected() {
        let m=example();let mut p=proof();p.terms.push(p.terms[0].clone());assert!(derive(&m,&p).is_none());
        let mut p=proof();p.pivot=2;assert!(derive(&m,&p).is_none());
        let mut m=example();m.variables[0].integer=false;assert!(derive(&m,&proof()).is_none());
        let mut m=example();m.rows[0].upper_enclosure=Some([3.,3.00001]);assert!(derive(&m,&proof()).is_none());
        let mut m=example();m.rows[0].coefficients[0].1=f64::INFINITY;assert!(derive(&m,&proof()).is_none());
    }
}
#[cfg(test)]
mod mixed_tests {
    use super::*;
    fn example(negative:bool)->Model {
        let text=format!("NAME g\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1\n x r 2\n y obj 0\n y r {}\nRHS\n rhs r 3\nBOUNDS\n LI b x 0\n UI b x 2\n LO b y 0\n UP b y 2\nENDATA\n",if negative{-1}else{1});
        crate::mps::parse(std::io::Cursor::new(text)).unwrap()
    }
    #[test]
    fn mixed_cut_handles_both_signs_of_continuous_coefficients() {
        for negative in [false,true] {
            let m=example(negative);let p=Proof{normalized:false,mixed:true,terms:vec![Term{row:0,upper:true,weight:1}],pivot:0,upper_bounds:vec![]};let cut=derive(&m,&p).unwrap();
            if !negative {assert_eq!(cut.coefficients,vec![(0,-1.)]);assert_eq!(cut.lower,Some(-1.));}
            for x in 0..=2 {for t in 0..=8 {let point=[x as f64,t as f64/4.];if m.check(&point,false,0.).is_ok(){assert!(cut.coefficients.iter().map(|&(j,a)|a*point[j]).sum::<f64>()>=cut.lower.unwrap());}}}
            assert!(verify(&m,&p,&cut));
        }
    }
    #[test]
    fn a_continuous_pivot_cannot_be_used_as_the_integer_split() {
        let m=example(false);let p=Proof{normalized:false,mixed:true,terms:vec![Term{row:0,upper:true,weight:1}],pivot:1,upper_bounds:vec![]};assert!(derive(&m,&p).is_none());
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundProof { pub normalized_bound: f64, pub multipliers: Vec<(usize,String)> }
