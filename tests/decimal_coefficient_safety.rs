use mip_codex::{mps,solve::{solve,Limits}};
fn input(coefficient:&str,maximize:bool,free:bool)->String{
 format!("NAME near\nOBJSENSE\n {}\nROWS\n N c\n L r\nCOLUMNS\n x c {} r {}\nRHS\n rhs r 1\nBOUNDS\n{}ENDATA\n",if maximize{"MAX"}else{"MIN"},if maximize{1}else{-1},coefficient,if free{" FR b x\n"}else{" LO b x 0\n UP b x 2\n"})
}
#[test]fn integer_looking_binary_values_do_not_erase_decimal_coefficient_uncertainty(){
 for token in ["0.99999999999999999999","1.00000000000000000001"]{let m=mps::parse(std::io::Cursor::new(input(token,false,false))).unwrap();assert_eq!(m.rows[0].coefficients[0].1,1.);assert!(!m.rows[0].exact_coefficients);assert!(!m.rows[0].exact_integral);assert_eq!(m.rows[0].exact_coefficient_flags,vec![false]);}
 let m=mps::parse(std::io::Cursor::new(input("1",false,false))).unwrap();assert!(m.rows[0].exact_integral&&m.rows[0].exact_coefficients);
}
#[test]fn conservative_original_bounds_survive_both_objective_directions(){
 for maximize in [false,true]{let m=mps::parse(std::io::Cursor::new(input("0.99999999999999999999",maximize,false))).unwrap();let r=solve(&m,&Limits::default());assert_eq!(r.status,"optimal");assert_eq!(r.objective,Some(if maximize{1.}else{-1.}));assert!(r.dual_bound.is_some_and(|b|if maximize{b>1.}else{b< -1.}));assert!(r.original_coefficient_bound_checks>0);assert!(r.original_coefficient_bound_successes>0);}
}
#[test]fn unbounded_boxes_do_not_accept_unverified_original_duals(){
 let m=mps::parse(std::io::Cursor::new(input("0.99999999999999999999",false,true))).unwrap();let r=solve(&m,&Limits::default());assert!(r.dual_bound.is_none_or(|b|b< -1.));if r.original_coefficient_bound_successes==0{assert_ne!(r.status,"optimal");}
}
#[test]fn uncertified_free_leaf_keeps_its_point_and_withdraws_optimality(){
 let text="NAME correlated\nROWS\n N c\n E r\nCOLUMNS\n x c -1.00000000000000000001 r 1.00000000000000000001\n y c 1 r -1\nRHS\n rhs r 0\nBOUNDS\n FR b x\n FR b y\nENDATA\n";
 let m=mps::parse(std::io::Cursor::new(text)).unwrap();
 let r=solve(&m,&Limits{zero_dominance:false,objective_box_report:false,..Limits::default()});
 assert_eq!(r.status,"numerical_error","{r:?}");assert_eq!(r.objective,Some(0.));assert!(r.values.is_some());
 assert!(r.original_coefficient_bound_checks>0);assert_eq!(r.original_coefficient_bound_successes,0);assert!(r.dual_bound.is_none_or(|b|b<=0.));
}
