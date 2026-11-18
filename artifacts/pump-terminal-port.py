from pathlib import Path
import subprocess
assert not any(x.strip() in ['cargo','rustc']for x in subprocess.check_output(['ps','-eo','comm='],text=True).splitlines())
changes=[]
def replace(path,a,b):
 p=Path(path);s=next((v for q,v in changes if q==p),p.read_text());assert s.count(a)==1,(path,a[:80]);s=s.replace(a,b);changes[:]=[(q,v)for q,v in changes if q!=p];changes.append((p,s))
replace('src/solve.rs','    pub pump_limited_rounding:bool,','    pub pump_limited_rounding:bool,\n    pub pump_terminal_repair:bool,')
replace('src/solve.rs','            pump_limited_rounding:true,','            pump_limited_rounding:true,\n            pump_terminal_repair:false,')
replace('src/solve.rs','pub struct PumpStats {','pub struct PumpStats {\n    pub terminal_repair_calls:usize,\n    pub terminal_repair_candidates:usize,\n    pub terminal_repair_moves:usize,\n    pub terminal_repair_work:usize,\n    pub terminal_repair_seconds:f64,')
a='''fn pump_limited_candidate(model:&Model,standard:&Standard,point:&[f64],checker:&heuristic::Repair<'_>)->Option<Vec<f64>> {
    if standard.terms.len()!=model.variables.len() || standard.shift.len()!=model.variables.len()
        || point.iter().any(|v|!v.is_finite())
        || standard.terms.iter().flatten().any(|&(k,_)|k>=point.len()) {return None;}
    let mut rounded:Vec<_>=standard.terms.iter().enumerate().map(|(j,t)|standard.shift[j]+t.iter().map(|&(k,a)|a*point[k]).sum::<f64>()).collect();
    for(j,v)in model.variables.iter().enumerate(){if v.integer{rounded[j]=rounded[j].round();}}
    if rounded.iter().all(|v|v.is_finite()) && checker.candidate_check(&rounded){Some(rounded)}else{None}
}'''
b='''fn pump_limited_map(model:&Model,standard:&Standard,point:&[f64])->Option<Vec<f64>> {
    if standard.terms.len()!=model.variables.len() || standard.shift.len()!=model.variables.len()
        || point.iter().any(|v|!v.is_finite())
        || standard.terms.iter().flatten().any(|&(k,_)|k>=point.len()) {return None;}
    let mapped:Vec<_>=standard.terms.iter().enumerate().map(|(j,t)|standard.shift[j]+t.iter().map(|&(k,a)|a*point[k]).sum::<f64>()).collect();
    mapped.iter().all(|v|v.is_finite()).then_some(mapped)
}
fn pump_limited_candidate(model:&Model,standard:&Standard,point:&[f64],checker:&heuristic::Repair<'_>)->Option<Vec<f64>> {
    let mut rounded=pump_limited_map(model,standard,point)?;
    for(j,v)in model.variables.iter().enumerate(){if v.integer{rounded[j]=rounded[j].round();}}
    if checker.candidate_check(&rounded){Some(rounded)}else{None}
}
// 主解だけの短い修復。補助LPの証明情報は一切使用しない。
fn pump_terminal_candidate(model:&Model,root:&Node,standard:&Standard,point:&[f64],global_deadline:Instant,checker:&mut heuristic::Repair<'_>,stats:&mut PumpStats)->Option<Vec<f64>> {
    let now=Instant::now();let remaining=global_deadline.saturating_duration_since(now);
    if remaining.is_zero(){return None;}
    let mapped=pump_limited_map(model,standard,point)?;
    let now=Instant::now();let window=global_deadline.saturating_duration_since(now).div_f64(100.).min(std::time::Duration::from_millis(100));
    if window.is_zero(){return None;}
    let deadline=now.checked_add(window)?.min(global_deadline);let before=checker.stats.clone();stats.terminal_repair_calls+=1;
    let candidate=checker.search_adaptive_incremental(&mapped,&root.lower,&root.upper,deadline,500_000,0x7c399ba3495e16d1);
    stats.terminal_repair_moves+=checker.stats.moves-before.moves;stats.terminal_repair_work+=checker.stats.work-before.work;stats.terminal_repair_seconds+=now.elapsed().as_secs_f64();
    if Instant::now()>=global_deadline{return None;}
    if candidate.is_some(){stats.terminal_repair_candidates+=1;}
    candidate
}'''
replace('src/solve.rs',a,b)
replace('src/solve.rs','    let checker = heuristic::Repair::new(model);\n    let mut current = point.to_vec();','    let mut checker = heuristic::Repair::new(model);\n    let mut current = point.to_vec();')
replace('src/solve.rs','''                            stats.limited_rejected+=1;''','''                            if limits.pump_terminal_repair {
                                if let Some(candidate)=pump_terminal_candidate(model,root,standard,&point,global_deadline,&mut checker,stats){
                                    stats.limited_candidates+=1;stats.candidates+=1;return Some(candidate);
                                }
                            }
                            stats.limited_rejected+=1;''')
a='''        if let Ok(v)=std::env::var("MIP_PUMP_LIMITED_ROUNDING"){limits.pump_limited_rounding=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_PUMP_LIMITED_ROUNDING must be 0 or 1".into())};}'''
replace('src/main.rs',a,a+'\n        if let Ok(v)=std::env::var("MIP_PUMP_TERMINAL_REPAIR"){limits.pump_terminal_repair=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_PUMP_TERMINAL_REPAIR must be 0 or 1".into())};}')
a='''    #[test]
    fn pump_projects_and_perturbs_without_changing_parent_lp() {'''
b='''    #[test]
    fn terminal_repair_recovers_rejected_rounding_without_changing_parent_lp() {
        let(m,node,p,_)=prepare(FRACTIONAL);let mut point=vec![0.;p.sparse.as_ref().unwrap().columns.len()];
        for j in 0..m.variables.len(){let &(k,a)=p.terms[j].iter().find(|&&(_,a)|a>0.).unwrap();point[k]=(0.7-p.shift[j])/a;}
        let mut checker=heuristic::Repair::new(&m);assert!(pump_limited_candidate(&m,&p,&point,&checker).is_none());let mut stats=PumpStats::default();
        let x=pump_terminal_candidate(&m,&node,&p,&point,Instant::now()+std::time::Duration::from_secs(2),&mut checker,&mut stats).unwrap();
        assert!(checker.candidate_check(&x));assert_eq!(stats.terminal_repair_calls,1);assert_eq!(stats.terminal_repair_candidates,1);assert!(stats.terminal_repair_moves>0);
        let parent=revised::minimize_warm(p.sparse.as_ref().unwrap(),1000,None,node.warm.as_deref());assert!(matches!(parent.outcome,lp::Outcome::Optimal{objective,..}if objective == -1.5));
    }
    #[test]
    fn terminal_repair_respects_node_bounds() {
        let(m,mut node,p,_)=prepare(FRACTIONAL);node.lower[0]=Some(1.);node.upper[1]=Some(0.);let point=vec![0.7;p.sparse.as_ref().unwrap().columns.len()];let mut checker=heuristic::Repair::new(&m);let mut stats=PumpStats::default();
        let x=pump_terminal_candidate(&m,&node,&p,&point,Instant::now()+std::time::Duration::from_secs(2),&mut checker,&mut stats).unwrap();assert_eq!(x,vec![1.,0.]);assert!(checker.candidate_check(&x));
    }
    #[test]
    fn terminal_repair_rejects_invalid_points_and_expired_deadlines() {
        let(m,node,p,_)=prepare(FRACTIONAL);let mut checker=heuristic::Repair::new(&m);let mut stats=PumpStats::default();let future=Instant::now()+std::time::Duration::from_secs(2);
        assert!(pump_terminal_candidate(&m,&node,&p,&[],future,&mut checker,&mut stats).is_none());assert!(pump_terminal_candidate(&m,&node,&p,&vec![f64::NAN;p.sparse.as_ref().unwrap().columns.len()],future,&mut checker,&mut stats).is_none());
        assert!(pump_terminal_candidate(&m,&node,&p,&vec![0.;p.sparse.as_ref().unwrap().columns.len()],Instant::now()-std::time::Duration::from_secs(1),&mut checker,&mut stats).is_none());assert_eq!(stats.terminal_repair_calls,0);
    }
'''+a
replace('src/solve.rs',a,b)
for p,s in changes:p.write_text(s)
print('ported',[(str(p),len(s))for p,s in changes])
