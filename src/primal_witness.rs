//! 元十進モデルに対する十分条件の有理実行可能点証拠。最適化は行わない。
use num_bigint::{BigInt, Sign};
use std::{cmp::Ordering,collections::BTreeMap};
use serde::Serialize;
const MAX_BITS:u64=8192;
#[derive(Clone,Debug,PartialEq,Eq)]
struct Rat { n:BigInt,d:BigInt }
impl Rat {
    fn new(mut n:BigInt,mut d:BigInt)->Option<Self>{
        if d==BigInt::from(0) || n.bits()>MAX_BITS || d.bits()>MAX_BITS{return None;}
        if d.sign()==Sign::Minus {n=-n;d=-d;}
        let mut a=if n.sign()==Sign::Minus{-&n}else{n.clone()};let mut b=d.clone();
        while b!=BigInt::from(0){let r=&a%&b;a=b;b=r;}
        Some(Self{n:n/&a,d:d/a})
    }
    fn integer(n:i64)->Self{Self{n:BigInt::from(n),d:BigInt::from(1)}}
    fn decimal(s:&str)->Option<Self>{
        let(n,e)=crate::mps::decimal_parts(s)?;
        if e>=0 {Self::new(n*BigInt::from(10u8).pow(e as u32),BigInt::from(1))}
        else{Self::new(n,BigInt::from(10u8).pow((-e)as u32))}
    }
    fn binary(v:f64)->Option<Self>{
        if !v.is_finite(){return None;}
        if v==0.{return Some(Self::integer(0));}
        let bits=v.to_bits();let exp=((bits>>52)&2047)as i32;let mut n=BigInt::from((bits&((1u64<<52)-1))|if exp==0{0}else{1u64<<52});
        if bits>>63!=0{n=-n;}
        let e=if exp==0{-1074}else{exp-1075};
        if e>=0{Self::new(n<<(e as usize),BigInt::from(1))}else{Self::new(n,BigInt::from(1)<<((-e)as usize))}
    }
    fn add(&self,b:&Self)->Option<Self>{Self::new(&self.n*&b.d+&b.n*&self.d,&self.d*&b.d)}
    fn sub(&self,b:&Self)->Option<Self>{Self::new(&self.n*&b.d-&b.n*&self.d,&self.d*&b.d)}
    fn mul(&self,b:&Self)->Option<Self>{Self::new(&self.n*&b.n,&self.d*&b.d)}
    fn div(&self,b:&Self)->Option<Self>{Self::new(&self.n*&b.d,&self.d*&b.n)}
    fn cmp(&self,b:&Self)->Ordering{(&self.n*&b.d).cmp(&(&b.n*&self.d))}
    fn abs(&self)->Self{Self{n:if self.n.sign()==Sign::Minus{-&self.n}else{self.n.clone()},d:self.d.clone()}}
    fn zero(&self)->bool{self.n==BigInt::from(0)}
}
/// 非二進十進数のみを保持する。cloneしてモデルを変更した場合は指紋不一致で使わない。
#[derive(Clone,Debug,Default)]
pub struct Source {
    signature:u64,
    sealed_rows:usize,
    sealed_variables:usize,
    pub(crate) coefficients:BTreeMap<(usize,usize),String>,
    pub(crate) row_bounds:BTreeMap<(usize,usize),String>,
    pub(crate) variable_bounds:BTreeMap<(usize,usize),String>,
    pub(crate) objective:BTreeMap<usize,String>,
    pub(crate) offset:Option<String>,
}
impl Source {
    pub(crate) fn seal(&mut self,m:&crate::model::Model){self.signature=signature(m);self.sealed_rows=m.rows.len();self.sealed_variables=m.variables.len();}
    pub(crate) fn matches(&self,m:&crate::model::Model)->bool{self.signature==signature(m)}
    pub(crate) fn contains_original_row(&self,index:usize)->bool{index<self.sealed_rows}
    /// Original rows and variables must match exactly; appended rows are permitted.
    /// Only used for conservative LP row enclosures, not primal proof reconstruction.
    pub(crate) fn matches_original_prefix(&self,m:&crate::model::Model)->bool{
        m.variables.len()==self.sealed_variables && m.rows.len()>=self.sealed_rows
            && self.signature==signature_prefix(m,self.sealed_rows)
    }
}
fn signature(m:&crate::model::Model)->u64{signature_prefix(m,m.rows.len())}
fn signature_prefix(m:&crate::model::Model,rows:usize)->u64{
    let mut h=0xcbf29ce484222325u64;
    let mut feed=|bytes:&[u8]|{for &b in bytes {h=(h^u64::from(b)).wrapping_mul(0x100000001b3);}h=(h^255).wrapping_mul(0x100000001b3);};
    feed(m.name.as_bytes());feed(&[u8::from(m.maximize),u8::from(m.original_decimal)]);feed(&m.offset.to_bits().to_le_bytes());
    for v in &m.variables{
        feed(v.name.as_bytes());feed(&[u8::from(v.integer)]);feed(&v.objective.to_bits().to_le_bytes());
        for b in [v.lower,v.upper]{feed(&[u8::from(b.is_some())]);if let Some(b)=b{feed(&b.to_bits().to_le_bytes());}}
    }
    feed(&(m.variables.len()as u64).to_le_bytes());
    for r in m.rows.iter().take(rows){
        feed(r.name.as_bytes());for b in [r.lower,r.upper]{feed(&[u8::from(b.is_some())]);if let Some(b)=b{feed(&b.to_bits().to_le_bytes());}}
        feed(&(r.coefficients.len()as u64).to_le_bytes());for &(j,a)in &r.coefficients{feed(&(j as u64).to_le_bytes());feed(&a.to_bits().to_le_bytes());}
    }
    feed(&(rows as u64).to_le_bytes());h
}
#[derive(Clone,Debug,Serialize)]
pub struct Coordinate { pub variable:usize,pub numerator:String,pub denominator:String }
#[derive(Clone,Debug,Serialize)]
pub struct Proof {
    pub values:Vec<Coordinate>,
    #[serde(skip)]
    objective:Rat,
}
impl Proof {
    pub(crate) fn respects_bound(&self,m:&crate::model::Model,bound:Option<f64>)->bool{
        let Some(bound)=bound else{return true;};
        let Some(declared)=crate::model::solution_number(bound).and_then(|n|Rat::decimal(&n.to_string()))else{return false;};
        if m.maximize{declared.cmp(&self.objective)!=Ordering::Less}else{declared.cmp(&self.objective)!=Ordering::Greater}
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn binary_and_decimal_arithmetic_are_exact(){
        let third=Rat::integer(1).div(&Rat::integer(3)).unwrap();assert_eq!(third.mul(&Rat::integer(3)).unwrap(),Rat::integer(1));
        assert_ne!(Rat::binary(0.1).unwrap(),Rat::decimal("0.1").unwrap());
        assert_eq!(Rat::binary(0.5).unwrap(),Rat::decimal(".5").unwrap());
        assert_eq!(Rat::decimal("-12.5d-1").unwrap(),Rat::integer(-5).div(&Rat::integer(4)).unwrap());
        assert_eq!(Rat::binary(f64::from_bits(1)).unwrap().d,BigInt::from(1)<<1074usize);
        assert_eq!(Rat::decimal("0").unwrap(),Rat::integer(0));assert!(Rat::integer(1).div(&Rat::integer(0)).is_none());
    }
}

struct ExactRow { a:Vec<(usize,Rat)>,bounds:[Option<Rat>;2] }
struct ExactModel { bounds:Vec<[Option<Rat>;2]>,rows:Vec<ExactRow>,cost:Vec<Rat>,offset:Rat }
fn number(value:f64,token:Option<&String>)->Option<Rat>{match token{Some(t)=>Rat::decimal(t),None=>Rat::binary(value)}}
impl ExactModel {
    fn read(m:&crate::model::Model,s:&Source,deadline:std::time::Instant)->Option<Self>{
        if m.variables.len()>4096 || m.rows.iter().map(|r|r.coefficients.len()).sum::<usize>()>32768{return None;}
        let mut bounds=Vec::new();let mut cost=Vec::new();let mut rows=Vec::new();
        for (j,v)in m.variables.iter().enumerate(){
            if std::time::Instant::now()>=deadline{return None;}
            let mut pair=[None,None];
            for(side,b)in [v.lower,v.upper].into_iter().enumerate(){if let Some(b)=b{pair[side]=Some(number(b,s.variable_bounds.get(&(j,side)))?);}}
            bounds.push(pair);cost.push(number(v.objective,s.objective.get(&j))?);
        }
        for(i,r)in m.rows.iter().enumerate(){
            if std::time::Instant::now()>=deadline{return None;}
            let mut pair=[None,None];for(side,b)in [r.lower,r.upper].into_iter().enumerate(){if let Some(b)=b{pair[side]=Some(number(b,s.row_bounds.get(&(i,side)))?);}}
            let mut a=Vec::new();for &(j,v)in &r.coefficients{a.push((j,number(v,s.coefficients.get(&(i,j)))?));}
            rows.push(ExactRow{a,bounds:pair});
        }
        Some(Self{bounds,rows,cost,offset:number(m.offset,s.offset.as_ref())?})
    }
    fn feasible(&self,x:&[Rat],m:&crate::model::Model,deadline:std::time::Instant)->Option<bool>{
        for((v,b),r)in m.variables.iter().zip(&self.bounds).zip(x){
            if v.integer && r.d!=BigInt::from(1){return Some(false);}
            if b[0].as_ref().is_some_and(|b|r.cmp(b)==Ordering::Less)||b[1].as_ref().is_some_and(|b|r.cmp(b)==Ordering::Greater){return Some(false);}
        }
        for row in &self.rows {
            if std::time::Instant::now()>=deadline{return None;}
            let activity=activity(&row.a,x)?;
            if row.bounds[0].as_ref().is_some_and(|b|activity.cmp(b)==Ordering::Less)||row.bounds[1].as_ref().is_some_and(|b|activity.cmp(b)==Ordering::Greater){return Some(false);}
        }
        Some(true)
    }
}
fn activity(a:&[(usize,Rat)],x:&[Rat])->Option<Rat>{let mut sum=Rat::integer(0);for(j,a)in a{sum=sum.add(&a.mul(&x[*j])?)?;}Some(sum)}
fn finish(e:&ExactModel,m:&crate::model::Model,raw:&[f64],x:&[Rat],reported:f64,deadline:std::time::Instant)->Option<Proof>{
    if !e.feasible(x,m,deadline)?{return None;}
    let tolerance=Rat::decimal("1e-7")?;let mut values=Vec::new();let mut obj=e.offset.clone();
    for(j,(&value,r))in raw.iter().zip(x).enumerate(){
        if std::time::Instant::now()>=deadline{return None;}
        let nominal=Rat::binary(value)?;
        let rendered=Rat::decimal(&crate::model::solution_number(value)?.to_string())?;
        if r.sub(&rendered)?.abs().cmp(&tolerance)==Ordering::Greater{return None;}
        if *r!=nominal{values.push(Coordinate{variable:j,numerator:r.n.to_string(),denominator:r.d.to_string()});}
        obj=obj.add(&e.cost[j].mul(r)?)?;
    }
    let report=Rat::binary(reported)?;
    // JSON十進表現との差も独立checkerのrel1e-8より十分小さくする。
    let tol=Rat::decimal("1e-9")?.mul(&Rat::integer(1).add(&report.abs())?)?;
    if obj.sub(&report)?.abs().cmp(&tol)==Ordering::Greater{return None;}
    Some(Proof{values,objective:obj})
}
/// 最適化せず、数値点の近傍に元モデルを厳密に満たす有理点を構成する。
/// 不成立・不明・時間/サイズ上限は全てNoneで保守的な撤回を維持する。
pub(crate) fn certify(m:&crate::model::Model,raw:&[f64],reported:f64)->Option<Proof>{
    if !m.original_decimal || raw.len()!=m.variables.len(){return None;}
    let deadline=std::time::Instant::now()+std::time::Duration::from_millis(50);
    if m.variables.len()>4096 || m.rows.iter().map(|r|r.coefficients.len()).sum::<usize>()>32768{return None;}
    let source=m.decimal_source.as_ref()?;if source.signature!=signature(m){return None;}
    let e=ExactModel::read(m,source,deadline)?;let mut x=Vec::new();let mut fixed=Vec::new();
    for(j,(v,&value))in m.variables.iter().zip(raw).enumerate(){
        let mut point=Rat::binary(if v.integer{value.round()}else{value})?;let mut is_fixed=v.integer;
        if !v.integer {
            for(side,b)in [v.lower,v.upper].into_iter().enumerate(){
                if b.is_some_and(|b|(value-b).abs()<=1e-7){point=e.bounds[j][side].as_ref()?.clone();is_fixed=true;break;}
            }
        }
        x.push(point);fixed.push(is_fixed);
    }
    if e.feasible(&x,m,deadline)?{return finish(&e,m,raw,&x,reported,deadline);}
    let free:Vec<_>=fixed.iter().enumerate().filter_map(|(j,&b)|(!b).then_some(j)).collect();
    if free.is_empty() || free.len()>64{return None;}
    let mut position=vec![None;raw.len()];for(k,&j)in free.iter().enumerate(){position[j]=Some(k);}
    let mut equations=Vec::new();
    // 等式を優先し、その後に数値的に活性な不等式を候補式に使う。
    for equality in [true,false]{
        for(i,row)in e.rows.iter().enumerate(){
            if std::time::Instant::now()>=deadline{return None;}
            let is_equality=matches!((&row.bounds[0],&row.bounds[1]),(Some(a),Some(b))if a==b);
            if is_equality!=equality{continue;}
            let bound=if equality{row.bounds[0].as_ref()?}else{
                let nominal=crate::exact::dot(m.rows[i].coefficients.iter().map(|&(j,a)|(a,raw[j])))?;
                let side=if m.rows[i].lower.is_some_and(|b|(nominal-b).abs()<=1e-7){0}else if m.rows[i].upper.is_some_and(|b|(nominal-b).abs()<=1e-7){1}else{continue;};
                row.bounds[side].as_ref()?
            };
            let mut a=vec![Rat::integer(0);free.len()+1];let mut rhs=bound.clone();
            for(j,c)in &row.a{
                if let Some(k)=position[*j]{a[k]=c.clone();}else{rhs=rhs.sub(&c.mul(&x[*j])?)?;}
            }
            a[free.len()]=rhs;equations.push(a);if equations.len()>=256{break;}
        }
        if equations.len()>=256{break;}
    }
    eliminate(&mut equations,&free,&mut x,deadline)?;
    finish(&e,m,raw,&x,reported,deadline)
}
fn eliminate(a:&mut[Vec<Rat>],free:&[usize],x:&mut[Rat],deadline:std::time::Instant)->Option<()>{
    let n=free.len();let mut rank=0;let mut pivots=Vec::new();let mut work=0usize;
    for col in 0..n{
        if std::time::Instant::now()>=deadline{return None;}
        let Some(pivot)=(rank..a.len()).find(|&r|!a[r][col].zero())else{continue;};a.swap(rank,pivot);
        let divisor=a[rank][col].clone();for j in col..=n{a[rank][j]=a[rank][j].div(&divisor)?;}
        for r in rank+1..a.len(){
            if std::time::Instant::now()>=deadline{return None;}
            let factor=a[r][col].clone();if factor.zero(){continue;}
            for j in col..=n{a[r][j]=a[r][j].sub(&factor.mul(&a[rank][j])?)?;work+=1;if work>100000{return None;}}
        }
        pivots.push(col);rank+=1;if rank==a.len(){break;}
    }
    for row in &a[rank..]{if row[..n].iter().all(Rat::zero)&&!row[n].zero(){return None;}}
    for r in (0..rank).rev(){
        let col=pivots[r];let mut value=a[r][n].clone();for j in col+1..n{value=value.sub(&a[r][j].mul(&x[free[j]])?)?;}
        x[free[col]]=value;
    }
    Some(())
}

#[cfg(test)]mod witness_tests {
    use super::*;
    fn parse(s:&str)->crate::model::Model{crate::mps::parse(std::io::Cursor::new(s)).unwrap()}
    #[test]fn thirds_and_decimal_fixed_points_have_exact_nearby_witnesses(){
        let m=parse("NAME third\nROWS\n N obj\n E r\nCOLUMNS\n y obj 1 r 3\nRHS\n rhs r 1\nENDATA\n");
        let before=serde_json::to_string(&m).unwrap();let proof=certify(&m,&[1./3.],1./3.).unwrap();assert_eq!(proof.values.len(),1);assert_eq!(proof.values[0].numerator,"1");assert_eq!(proof.values[0].denominator,"3");
        assert!(proof.respects_bound(&m,Some(1./3.)));
        assert!(!proof.respects_bound(&m,Some((1./3f64).next_up())));
        let mut max=m.clone();max.maximize=true;
        assert!(!proof.respects_bound(&max,Some(1./3.)));
        assert!(proof.respects_bound(&max,Some((1./3f64).next_up())));
        assert_eq!(serde_json::to_string(&m).unwrap(),before);
        assert!(certify(&m,&[1./3.+1e-4],1./3.).is_none());assert!(certify(&m,&[1./3.],0.4).is_none());
        let mut modified=m.clone();modified.rows[0].upper=Some(2.);assert!(certify(&modified,&[1./3.],1./3.).is_none());assert!(certify(&m,&[1./3.],1./3.).is_some());
        let m=parse("NAME fixed\nROWS\n N obj\nCOLUMNS\n y obj 1\nRHS\n rhs obj 0\nBOUNDS\n FX b y 0.3\nENDATA\n");let proof=certify(&m,&[0.3],0.3).unwrap();assert_eq!(proof.values[0].numerator,"3");assert_eq!(proof.values[0].denominator,"10");
    }
    #[test]fn implicit_integer_lattice_false_points_never_get_a_witness(){
        for(a,b,cap,c,y)in [("1","-1","0.9999999999999999999999","1",1.),("3","-4","0.7499999999999999999999","1",0.75),("1","-3","0.3333333333333333333332","1",1./3.),("1","-2","0.049999999999999999999999","0.1",0.5),("1.0000000000000002220446049250313080847263336181640625","-2","0.5","1",0.5)]{
            let m=parse(&format!("NAME bad\nROWS\n N obj\n E link\n L cap\nCOLUMNS\n x obj -1 link {a}\n y link {b} cap {c}\nRHS\n rhs link 0 cap {cap}\nBOUNDS\n LI b x 0\n UI b x 2\n LO b y 0\n UP b y 2\nENDATA\n"));assert!(certify(&m,&[1.,y],-1.).is_none(),"{a} {b} {cap}");assert!(certify(&m,&[0.,0.],0.).is_some());
        }
    }
    #[test]fn original_objective_uncertainty_cannot_claim_a_contradictory_dual(){
        let m=parse("NAME cost\nROWS\n N obj\n E row\nCOLUMNS\n y obj 1.00000000000000000001 row 2\nRHS\n rhs row -6 obj -1.00000000000000000001\nBOUNDS\n LO b y -10\n UP b y 10\nENDATA\n");
        assert!(m.original_objective_uncertain());let p=certify(&m,&[-3.],-2.).unwrap();assert!(p.values.is_empty());
        assert!(!p.respects_bound(&m,Some(-2.)));assert!(p.respects_bound(&m,Some((-2f64).next_down())));
        let r=crate::solve::solve(&m,&crate::solve::Limits::default());assert_eq!(r.status,"numerical_error");assert!(r.dual_bound.is_none()&&r.gap.is_none());
    }
    #[test]fn source_preserves_duplicate_sums_ranges_and_bound_overwrites(){
        let m=parse("NAME metadata\nROWS\n N obj\n E r\nCOLUMNS\n x obj -1.000000000000000000001 r 1\n x obj 1\nRHS\n rhs r 1.000000000000000000001\nRANGES\n rng r -1\nBOUNDS\n FX b x 0.3\n BV b x\nENDATA\n");let s=m.decimal_source.as_ref().unwrap();assert!(s.variable_bounds.is_empty());assert_eq!(Rat::decimal(s.objective.get(&0).unwrap()).unwrap(),Rat::decimal("-1e-21").unwrap());assert_eq!(Rat::decimal(s.row_bounds.get(&(0,0)).unwrap()).unwrap(),Rat::decimal("1e-21").unwrap());assert!(certify(&m,&[1.],-1e-21).is_some());assert!(certify(&m,&[0.],0.).is_none());
    }
}
