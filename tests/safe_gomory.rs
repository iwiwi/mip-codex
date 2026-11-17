use mip_codex::{mps,solve::{solve,Limits}};
#[test]
fn exact_gomory_cut_strengthens_the_fractional_root_and_keeps_its_proof() {
 let model=mps::parse(std::io::Cursor::new("NAME g\nROWS\n N obj\n L r\nCOLUMNS\n x obj -1\n x r 2\n y obj -1\n y r 2\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n")).unwrap();
 let limits=Limits{seconds:2.,nodes:1,gomory_cuts:true,cover_cuts:false,mir_cuts:false,aggregate_cuts:false,clique_cuts:false,three_row_cuts:false,rounding_cuts:false,binary_graph:false,parity_equalities:false,primal_repair:false,lp_diving:false,primal_probe:false,rins:false,feasibility_pump:false,strong_branching:false,row_ray_report:false,row_packing_report:false,row_subgradient_report:false,dual_guided_primal:false,pure_cover_greedy:false,..Limits::default()};
 let result=solve(&model,&limits);assert!(result.cut_stats.gomory_added>0,"{result:?}");assert!(result.cut_proofs.iter().any(|c|c.proof.gomory.is_some()));assert_eq!(result.objective,Some(-1.));assert_eq!(result.status,"optimal");
 for c in &result.cut_proofs {assert!(mip_codex::cuts::verify(&model,c));}
 let values=result.values.unwrap();let point:Vec<_>=model.variables.iter().map(|v|values[&v.name]).collect();assert!(model.check(&point,true,0.).is_ok());
}
#[test]
fn reporting_a_gmi_bound_preserves_the_fractional_search_root() {
 for maximize in [false,true] {
  let sense=if maximize {"OBJSENSE\n MAX\n"}else{""};let cost=if maximize{1}else{-1};
  let text=format!("NAME report\n{sense}ROWS\n N obj\n L r\nCOLUMNS\n x obj {cost}\n x r 2\n y obj {cost}\n y r 2\nRHS\n rhs r 3\nBOUNDS\n BV b x\n BV b y\nENDATA\n");
  let model=mps::parse(std::io::Cursor::new(text)).unwrap();
  let limits=Limits{seconds:2.,nodes:1,gomory_cuts:true,gomory_report_only:true,cover_cuts:false,mir_cuts:false,aggregate_cuts:false,clique_cuts:false,three_row_cuts:false,rounding_cuts:false,binary_graph:false,parity_equalities:false,primal_repair:false,lp_diving:false,primal_probe:false,rins:false,feasibility_pump:false,strong_branching:false,objective_rounding:false,objective_domain_report:false,row_ray_report:false,row_packing_report:false,row_subgradient_report:false,dual_guided_primal:false,pure_cover_greedy:false,..Limits::default()};
  let mut baseline=limits.clone();baseline.gomory_cuts=false;let before=solve(&model,&baseline);
  let result=solve(&model,&limits);assert_eq!(result.status,"node_limit");assert_eq!(result.nodes,1);assert_eq!(result.status,before.status);assert_eq!(result.objective,before.objective);
  let proof=result.gomory_bound_proof.unwrap();assert!(proof.normalized_bound<=-1. && proof.normalized_bound>=-1.000001);let bound=result.dual_bound.unwrap();assert!((bound-if maximize{1.}else{-1.}).abs()<1e-6);
  for c in &result.cut_proofs{assert!(mip_codex::cuts::verify(&model,c));}
 }
}
