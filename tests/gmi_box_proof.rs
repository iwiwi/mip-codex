use mip_codex::{mps,objective_bound::Certifier};
#[test]
fn inferred_box_bounds_hold_for_every_enumerated_feasible_grid_point() {
 let model=mps::parse(std::io::Cursor::new("NAME box\nROWS\n N obj\n L r0\n L r1\n L r2\nCOLUMNS\n x obj 0\n x r0 2\n x r1 1\n x r2 -1\n y obj 1\n y r0 1\n z obj 0\n z r1 -1\nRHS\n rhs r0 5\n rhs r1 4\n rhs r2 0\nBOUNDS\n LO b x 0\n LO b y 0\n MI b z\n UP b z 0\nENDATA\n")).unwrap();
 let certifier=Certifier::new(&model).unwrap();let original_lower:Vec<_>=model.variables.iter().map(|v|v.lower).collect();let original_upper:Vec<_>=model.variables.iter().map(|v|v.upper).collect();let weights=[0.,0.,-1.];assert!(certifier.bound(&original_lower,&original_upper,&weights).is_none());
 let(lower,upper,steps)=certifier.infer_box_proof(&model);assert!(steps.len()>=3);assert!(upper[0].unwrap()>=2.5&&upper[0].unwrap()<2.500001);assert!(upper[1].unwrap()>=5.&&upper[1].unwrap()<5.000001);assert!(lower[2].unwrap()<=-4.&&lower[2].unwrap()>-4.000001);
 let bound=certifier.bound(&lower,&upper,&weights).unwrap();assert!(bound<=-2.5&&bound>-2.500001);
 for x in 0..=10 {for y in 0..=20 {for z in -16..=0 {let point=[x as f64/4.,y as f64/4.,z as f64/4.];if model.check(&point,false,0.).is_ok(){for j in 0..3 {assert!(lower[j].is_none_or(|l|point[j]>=l));assert!(upper[j].is_none_or(|u|point[j]<=u));}}}}}
}
#[test]
fn a_missing_bound_is_not_invented_without_a_supporting_row() {
 let model=mps::parse(std::io::Cursor::new("NAME unbounded\nROWS\n N obj\n L r\nCOLUMNS\n x obj 0\n x r -1\n y obj 1\nRHS\n rhs r 0\nBOUNDS\n LO b x 0\n LO b y 0\nENDATA\n")).unwrap();let c=Certifier::new(&model).unwrap();let(l,u,steps)=c.infer_box_proof(&model);assert!(steps.is_empty());assert!(u[0].is_none());assert!(c.bound(&l,&u,&[-1.]).is_none());
}
