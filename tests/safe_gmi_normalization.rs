use mip_codex::{gomory::{derive,verify,Proof,Term},mps};
#[test]
fn power_of_two_normalization_preserves_the_exact_inequality() {
 let m=mps::parse(std::io::Cursor::new("NAME g\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1\n x r 3\n y obj 0\n y r -100000\nRHS\n rhs r 2\nBOUNDS\n LI b x 0\n UI b x 2\n LO b y 0\n UP b y 1\nENDATA\n")).unwrap();
 let p=Proof{normalized:false,mixed:true,terms:vec![Term{row:0,upper:true,weight:1}],pivot:0,upper_bounds:vec![]};let raw=derive(&m,&p).unwrap();assert_eq!(raw.coefficients,vec![(0,-1.),(1,100000.)]);
 let mut normalized=p.clone();normalized.normalized=true;let row=derive(&m,&normalized).unwrap();assert!(!row.exact_integral);assert!(row.exact_coefficients);assert!(verify(&m,&normalized,&row));assert!(!verify(&m,&normalized,&raw));
 for ((j,a),(k,b)) in raw.coefficients.iter().zip(&row.coefficients){assert_eq!(j,k);assert_eq!(*a,*b*65536.);}
 assert_eq!(raw.lower.unwrap(),row.lower.unwrap()*65536.);
}
