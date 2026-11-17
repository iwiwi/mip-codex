use mip_codex::{mps,heuristic::Repair,cover_greedy,solve::{solve,Limits}};
use std::time::{Instant,Duration};
fn model(n:usize,rows:&[Vec<usize>],costs:&[usize])->mip_codex::model::Model{let mut text="NAME cover\nROWS\n N obj\n".to_string();for i in 0..rows.len(){text+=&format!(" G r{i}\n");}text+="COLUMNS\n";for j in 0..n{text+=&format!(" x{j} obj {}\n",costs[j]);for(i,r)in rows.iter().enumerate(){if r.contains(&j){text+=&format!(" x{j} r{i} 1\n");}}}text+="RHS\n";for i in 0..rows.len(){text+=&format!(" rhs r{i} 1\n");}text+="BOUNDS\n";for j in 0..n{text+=&format!(" BV b x{j}\n");}text+="ENDATA\n";mps::parse(std::io::Cursor::new(text)).unwrap()}
#[test]fn generated_candidates_match_original_constraints_and_bruteforce_lower_limits(){let mut state=181u64;for case in 0..200{let n=3+case%7;let mut rows=Vec::new();for _ in 0..2+case%9{let mut r=Vec::new();for j in 0..n{state=state.wrapping_mul(6364136223846793005).wrapping_add(1);if state>>63!=0{r.push(j);}}if r.is_empty(){r.push(case%n);}rows.push(r);}let costs:Vec<_>=(0..n).map(|j|(j*3+case)%5).collect();let m=model(n,&rows,&costs);let repair=Repair::new(&m);let(x,s)=cover_greedy::candidate(&m,&repair,Instant::now()+Duration::from_secs(1)).unwrap();let x=x.unwrap();assert_eq!(s.candidates,1);assert!(m.check(&x,true,0.).is_ok());let value=m.objective(&x);let optimum=(0..1usize<<n).filter(|&b|rows.iter().all(|r|r.iter().any(|&j|b>>j&1==1))).map(|b|(0..n).filter(|&j|b>>j&1==1).map(|j|costs[j]).sum::<usize>()).min().unwrap()as f64;assert!(value>=optimum&&value<=costs.iter().sum::<usize>()as f64);}}
#[test]fn unsupported_shapes_and_expired_budget_do_not_produce_proofs(){let m=model(3,&[vec![0,1],vec![1,2]],&[1,2,1]);assert!(cover_greedy::candidate(&m,&Repair::new(&m),Instant::now()).is_none());let mut variants=Vec::new();let mut a=m.clone();a.maximize=true;variants.push(a);let mut a=m.clone();a.variables[0].objective=-1.;variants.push(a);let mut a=m.clone();a.rows[0].upper=Some(1.);variants.push(a);let mut a=m.clone();a.rows[0].coefficients[0].1=2.;variants.push(a);for a in variants{assert!(cover_greedy::candidate(&a,&Repair::new(&a),Instant::now()+Duration::from_secs(1)).is_none());}}
#[test]fn accepted_cover_remains_a_primal_at_the_node_limit(){let m=model(5,&[vec![0,1],vec![1,2],vec![2,3],vec![3,4],vec![4,0]],&[1;5]);let r=solve(&m,&Limits{seconds:2.,nodes:0,pure_cover_greedy:true,pure_cover_portfolio:false,pure_cover_exchange:false,binary_graph:false,parity_equalities:false,row_subgradient_report:false,dual_guided_primal:false,row_ray_report:false,row_packing_report:false,cover_cuts:false,clique_cuts:false,aggregate_cuts:false,mir_cuts:false,three_row_cuts:false,..Limits::default()});assert_eq!(r.status,"node_limit");assert_eq!(r.lp_solves,0);assert!(r.pure_cover_greedy.as_ref().is_some_and(|s|s.candidates==1));let v=r.values.unwrap();let x:Vec<_>=m.variables.iter().map(|c|v[&c.name]).collect();assert!(m.check(&x,true,0.).is_ok());}
#[test]fn empty_cover_and_nonfinite_costs_never_escape_as_feasible(){let m=model(3,&[vec![]],&[1,2,3]);let(x,s)=cover_greedy::candidate(&m,&Repair::new(&m),Instant::now()+Duration::from_secs(1)).unwrap();assert!(x.is_none());assert_eq!(s.candidates,0);let mut m=model(3,&[vec![0,1],vec![1,2]],&[0,0,0]);let(x,_)=cover_greedy::candidate(&m,&Repair::new(&m),Instant::now()+Duration::from_secs(1)).unwrap();let x=x.unwrap();assert!(m.check(&x,true,0.).is_ok());assert_eq!(m.objective(&x),0.);m.variables[0].objective=f64::NAN;assert!(cover_greedy::candidate(&m,&Repair::new(&m),Instant::now()+Duration::from_secs(1)).is_none());}
#[test]fn portfolio_preserves_its_baseline_and_original_feasibility(){for case in 0..50{let n=3+case%7;let rows:Vec<_>=(0..n).map(|i|vec![i,(i+1)%n]).collect();let costs:Vec<_>=(0..n).map(|j|(j*3+case)%5).collect();let m=model(n,&rows,&costs);let repair=Repair::new(&m);let baseline=cover_greedy::candidate(&m,&repair,Instant::now()+Duration::from_secs(2)).unwrap().0.unwrap();let(x,s)=cover_greedy::candidate_portfolio(&m,&repair,Instant::now()+Duration::from_secs(2),true).unwrap();let x=x.unwrap();assert!(m.check(&x,true,0.).is_ok());assert!(m.objective(&x)<=m.objective(&baseline));assert!(s.calls>1&&s.calls<=128);let optimum=(0..1usize<<n).filter(|&b|rows.iter().all(|r|r.iter().any(|&j|b>>j&1==1))).map(|b|(0..n).filter(|&j|b>>j&1==1).map(|j|costs[j]).sum::<usize>()).min().unwrap()as f64;assert!(m.objective(&x)>=optimum);}
 let m=model(5,&[vec![0,1],vec![1,2],vec![2,3],vec![3,4],vec![4,0]],&[1;5]);let r=solve(&m,&Limits{seconds:2.,nodes:0,pure_cover_greedy:true,pure_cover_portfolio:true,binary_graph:false,parity_equalities:false,row_subgradient_report:false,dual_guided_primal:false,row_ray_report:false,row_packing_report:false,cover_cuts:false,clique_cuts:false,aggregate_cuts:false,mir_cuts:false,three_row_cuts:false,..Limits::default()});assert_eq!(r.status,"node_limit");assert_eq!(r.lp_solves,0);assert!(r.values.is_some());}

#[test]
fn exchange_removes_two_columns_but_never_uncovers_their_shared_row() {
    let m=model(3,&[vec![0,2],vec![1,2]],&[1,1,1]);
    let (x,s)=cover_greedy::exchange(&m,&Repair::new(&m),&[1.,1.,0.],Instant::now()+Duration::from_secs(1),100000).unwrap();
    assert_eq!(x,vec![0.,0.,1.]); assert_eq!(s.improvements,1);
    let m=model(3,&[vec![0,2],vec![1,2],vec![0,1]],&[1,1,1]);
    let (x,_)=cover_greedy::exchange(&m,&Repair::new(&m),&[1.,1.,0.],Instant::now()+Duration::from_secs(1),100000).unwrap();
    assert_eq!(x,vec![1.,1.,0.]); assert!(m.check(&x,true,0.).is_ok());
}
#[test]
fn exchange_candidates_are_feasible_and_never_improve_below_enumerated_optima() {
    let mut state=1907u64;
    for case in 0..200 {
        let n=3+case%7; let mut rows=Vec::new();
        for _ in 0..2+case%9 { let mut row=Vec::new();for j in 0..n {state=state.wrapping_mul(6364136223846793005).wrapping_add(1);if state>>63!=0{row.push(j);}}if row.is_empty(){row.push(case%n);}rows.push(row); }
        let costs:Vec<_>=(0..n).map(|j|(j*3+case)%7).collect();let m=model(n,&rows,&costs);let repair=Repair::new(&m);
        let base=cover_greedy::candidate(&m,&repair,Instant::now()+Duration::from_secs(1)).unwrap().0.unwrap();
        let (x,_)=cover_greedy::exchange(&m,&repair,&base,Instant::now()+Duration::from_secs(1),100000).unwrap();
        assert!(m.check(&x,true,0.).is_ok());assert!(m.objective(&x)<=m.objective(&base));
        let optimum=(0..1usize<<n).filter(|&b|rows.iter().all(|r|r.iter().any(|&j|b>>j&1==1))).map(|b|(0..n).filter(|&j|b>>j&1==1).map(|j|costs[j]).sum::<usize>()).min().unwrap()as f64;
        assert!(m.objective(&x)>=optimum);
        assert!(cover_greedy::exchange(&m,&repair,&base,Instant::now(),100000).is_none());
    }
}

#[test]
fn exchange_stage_at_node_limit_returns_only_a_checked_primal() {
 let m=model(5,&[vec![0,1],vec![1,2],vec![2,3],vec![3,4],vec![4,0]],&[1;5]);
 let r=solve(&m,&Limits{seconds:2.,nodes:0,pure_cover_greedy:true,pure_cover_portfolio:true,pure_cover_exchange:true,binary_graph:false,parity_equalities:false,row_subgradient_report:false,dual_guided_primal:false,row_ray_report:false,row_packing_report:false,cover_cuts:false,clique_cuts:false,aggregate_cuts:false,mir_cuts:false,three_row_cuts:false,..Limits::default()});
 assert_eq!(r.status,"node_limit");assert_eq!(r.lp_solves,0);let values=r.values.unwrap();let point:Vec<_>=m.variables.iter().map(|v|values[&v.name]).collect();assert!(m.check(&point,true,0.).is_ok());
}
