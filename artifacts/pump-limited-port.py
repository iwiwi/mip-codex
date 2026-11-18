from pathlib import Path
import subprocess
root=Path.cwd();assert root==Path('/home/ubuntu/mip-pump-limited-rounding')
for n in ['cargo','rustc']:assert subprocess.run(['pgrep','-x',n],capture_output=True).returncode==1
def edit(s,a,b):assert s.count(a)==1,a[:90];return s.replace(a,b)
p=root/'src/solve.rs';s=p.read_text();s=edit(s,'    pub feasibility_pump: bool,','    pub feasibility_pump: bool,\n    pub pump_limited_rounding:bool,');s=edit(s,'            feasibility_pump: true,','            feasibility_pump: true,\n            pump_limited_rounding:false,')
s=edit(s,'pub struct PumpStats {\n    pub calls: usize,','pub struct PumpStats {\n    pub limited_points:usize,\n    pub limited_candidates:usize,\n    pub limited_rejected:usize,\n    pub limited_deadline_skips:usize,\n    pub calls: usize,')
header='''fn binary_pump(
    model: &Model,
    root: &Node,
    standard: &Standard,
    point: &[f64],
    limits: &Limits,
    deadline: Instant,
    stats: &mut PumpStats,
    lp_stats: &mut revised::Stats,
) -> Option<Vec<f64>> {'''
helper='''// 局所LPの打切り点は可行点の種としてだけ扱い、境界や最適性は引き継がない。
fn pump_limited_candidate(model:&Model,standard:&Standard,point:&[f64],checker:&heuristic::Repair<'_>)->Option<Vec<f64>> {
    if standard.terms.len()!=model.variables.len() || standard.shift.len()!=model.variables.len()
        || point.iter().any(|v|!v.is_finite())
        || standard.terms.iter().flatten().any(|&(k,_)|k>=point.len()) {return None;}
    let mut rounded:Vec<_>=standard.terms.iter().enumerate().map(|(j,t)|standard.shift[j]+t.iter().map(|&(k,a)|a*point[k]).sum::<f64>()).collect();
    for(j,v)in model.variables.iter().enumerate(){if v.integer{rounded[j]=rounded[j].round();}}
    if rounded.iter().all(|v|v.is_finite()) && checker.candidate_check(&rounded){Some(rounded)}else{None}
}
'''
new=helper+header+'''
    binary_pump_with_global_deadline(model,root,standard,point,limits,deadline,deadline,stats,lp_stats)
}
fn binary_pump_with_global_deadline(
    model: &Model, root:&Node, standard:&Standard, point:&[f64], limits:&Limits,
    deadline:Instant, global_deadline:Instant, stats:&mut PumpStats, lp_stats:&mut revised::Stats,
)->Option<Vec<f64>> {'''
s=edit(s,header,new)
a='''            lp::Outcome::TimeLimit | lp::Outcome::IterationLimit => {
                stats.limited += 1;
                return None;
            }'''
b='''            lp::Outcome::TimeLimit | lp::Outcome::IterationLimit => {
                stats.limited += 1;
                if limits.pump_limited_rounding {
                    if Instant::now()<global_deadline {
                        if let Some(point)=answer.limited_point {
                            stats.limited_points+=1;
                            if let Some(candidate)=pump_limited_candidate(model,standard,&point,&checker){
                                stats.limited_candidates+=1;stats.candidates+=1;return Some(candidate);
                            }
                            stats.limited_rejected+=1;
                        }
                    } else {stats.limited_deadline_skips+=1;}
                }
                return None;
            }'''
# Restrict the replacement to this function, because other heuristics have their own limit arms.
a_start=s.index('fn binary_pump_with_global_deadline(');a_end=s.index('/// 主解探索の局所LP',a_start);piece=s[a_start:a_end];piece=edit(piece,a,b);s=s[:a_start]+piece+s[a_end:]
s=edit(s,'if let Some(x) = binary_pump(\n                        model,','if let Some(x) = binary_pump_with_global_deadline(\n                        model,');s=edit(s,'                        pump_deadline,\n                        &mut pump,','                        pump_deadline,\n                        deadline,\n                        &mut pump,')
anchor='''    #[test]
    fn pump_projects_and_perturbs_without_changing_parent_lp() {'''
tests='''    #[test]
    fn limited_projection_can_yield_a_primal_without_certifying_the_auxiliary_lp() {
        let(m,node,p,point)=prepare(FRACTIONAL);
        for enabled in [false,true]{
            let limits=Limits{pivots:1,pump_limited_rounding:enabled,..Limits::default()};
            let mut stats=PumpStats::default();let mut lp_stats=revised::Stats::default();
            let candidate=binary_pump(&m,&node,&p,&point,&limits,Instant::now()+std::time::Duration::from_secs(2),&mut stats,&mut lp_stats);
            if enabled{let x=candidate.expect("one pivot reaches an integer point before the limit");assert!(m.check(&x,true,1e-7).is_ok());assert_eq!(m.objective(&x),1.);assert_eq!(stats.limited_candidates,1);}else{assert!(candidate.is_none());}
            assert_eq!(stats.limited,1);
            let parent=revised::minimize_warm(p.sparse.as_ref().unwrap(),1000,None,node.warm.as_deref());
            assert!(matches!(parent.outcome,lp::Outcome::Optimal{objective,..}if objective == -1.5));
        }
    }
    #[test]
    fn limited_rounding_rejects_nonfinite_and_infeasible_points() {
        let(m,_,p,_)=prepare(FRACTIONAL);let checker=heuristic::Repair::new(&m);let n=p.sparse.as_ref().unwrap().columns.len();
        assert!(pump_limited_candidate(&m,&p,&[],&checker).is_none());
        assert!(pump_limited_candidate(&m,&p,&vec![f64::NAN;n],&checker).is_none());
        let x=vec![0.;n];let good=pump_limited_candidate(&m,&p,&x,&checker).unwrap();assert!(m.check(&good,true,1e-7).is_ok());
        assert!(pump_limited_candidate(&m,&p,&vec![1.;n],&checker).is_none());
    }
'''+anchor
s=edit(s,anchor,tests)
p2=root/'src/main.rs';t=p2.read_text();a='        if let Ok(value) = std::env::var("MIP_FEASIBILITY_PUMP") {';assert t.count(a)==1;i=t.index(a);t=t[:i]+'''        if let Ok(v)=std::env::var("MIP_PUMP_LIMITED_ROUNDING"){limits.pump_limited_rounding=match v.as_str(){"0"=>false,"1"=>true,_=>return Err("MIP_PUMP_LIMITED_ROUNDING must be 0 or 1".into())};}
'''+t[i:]
p.write_text(s);p2.write_text(t);print('pump recovery flag and original-model checks added; LP/bound mathematics unchanged')
