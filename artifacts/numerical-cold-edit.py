from pathlib import Path
p=Path('src/revised.rs');s=p.read_text();s=s.replace('pub struct Stats {','pub struct Stats {\n    pub cold_recovery_attempts:usize,\n    pub cold_recovery_candidates:usize,',1);s=s.replace('impl Stats {','impl Stats {',1);needle='    pub fn add(&mut self, other: &Self) {';assert needle in s;s=s.replace(needle,needle+'\n        self.cold_recovery_attempts+=other.cold_recovery_attempts;\n        self.cold_recovery_candidates+=other.cold_recovery_candidates;',1)
old='''    if retain {
        warm_impl(p, limit, deadline, warm)
    } else {
        cold(p, limit, deadline, false)
    }
}'''
new='''    let first=if retain {warm_impl(p,limit,deadline,warm)}else{cold(p,limit,deadline,false)};
    recover_cold(p,first,limit,deadline,retain,std::env::var("MIP_LP_NUMERICAL_RECOVERY").is_ok_and(|v|v=="1"))
}
fn recover_cold(p:&Problem,first:Answer,limit:usize,deadline:Option<Instant>,retain:bool,enabled:bool)->Answer{
    if !enabled||!matches!(first.outcome,Outcome::NumericalError)||deadline.is_some_and(|d|Instant::now()>=d){return first;}
    let spent=first.stats.pivots.saturating_add(first.stats.flips);let remaining=limit.saturating_sub(spent);if remaining==0{return first;}
    // Different initial basis only. The enclosing scaling solve still validates
    // primal/dual, contradiction, and unbounded witnesses in original LP units.
    let mut retry=cold_initialized(p,remaining,deadline,retain,false,false,false);
    retry.stats.cold_recovery_attempts+=1;
    retry.stats.cold_recovery_candidates+=usize::from(matches!(retry.outcome,Outcome::Optimal{..}|Outcome::Infeasible|Outcome::Unbounded));
    retry.stats.add(&first.stats);retry
}'''
assert old in s;s=s.replace(old,new,1);s+='''
#[cfg(test)]mod cold_recovery_tests{
 use super::*;
 fn problem()->Problem{Problem{columns:vec![vec![(0,1.)]],rhs:vec![1.],costs:vec![-1.],upper:vec![2.],equality:vec![false],slack_upper:vec![f64::INFINITY]}}
 fn failed()->Answer{Answer{outcome:Outcome::NumericalError,dual:None,warm:None,stats:Stats{pivots:3,warm_attempts:5,..Stats::default()},witness:None,witness_point:None,limited_point:None}}
 #[test]fn alternative_basis_recovers_a_candidate_without_changing_the_problem(){let p=problem();let a=recover_cold(&p,failed(),100,None,true,true);assert!(matches!(&a.outcome,Outcome::Optimal{objective,x}if *objective== -1.&&x==&vec![1.]));assert_eq!(a.stats.cold_recovery_attempts,1);assert_eq!(a.stats.cold_recovery_candidates,1);assert_eq!(a.stats.warm_attempts,5);assert!(a.stats.pivots>=3);assert_eq!(p.rhs,vec![1.]);assert_eq!(p.costs,vec![-1.]);assert_eq!(p.upper,vec![2.]);assert!(crate::scaling::primal_dual(&p,match &a.outcome{Outcome::Optimal{x,..}=>x,_=>unreachable!()},a.witness.as_deref().unwrap()).is_some());}
 #[test]fn disabled_expired_spent_or_non_numerical_answers_are_not_retried(){let p=problem();for(enabled,limit,deadline)in[(false,100,None),(true,100,Some(Instant::now())),(true,3,None)]{let a=recover_cold(&p,failed(),limit,deadline,true,enabled);assert!(matches!(a.outcome,Outcome::NumericalError));assert_eq!(a.stats.cold_recovery_attempts,0);assert_eq!(a.stats.pivots,3);}for outcome in [Outcome::TimeLimit,Outcome::IterationLimit,Outcome::Infeasible,Outcome::Unbounded,Outcome::Optimal{objective:-1.,x:vec![1.]}]{let mut a=failed();a.outcome=outcome;let a=recover_cold(&p,a,100,None,true,true);assert_eq!(a.stats.cold_recovery_attempts,0);}}
 #[test]fn limited_retry_never_promotes_a_partial_point_to_a_bound(){let p=problem();let a=recover_cold(&p,failed(),4,None,true,true);assert_eq!(a.stats.cold_recovery_attempts,1);if matches!(a.outcome,Outcome::IterationLimit|Outcome::TimeLimit){assert!(a.dual.is_none());assert!(a.warm.is_none());}assert!(a.stats.pivots+a.stats.flips<=4);}
}
''';p.write_text(s)
p=Path('src/solve.rs');s=p.read_text();needle='    pub lp_zero_phase_one_skips:Option<usize>,';assert needle in s;s=s.replace(needle,needle+'\n    pub lp_cold_recovery_attempts:Option<usize>,\n    pub lp_cold_recovery_candidates:Option<usize>,',1);needle='        lp_zero_phase_one_skips:matches!(limits.engine,Engine::Revised).then_some(lp_stats.zero_phase_one_skips),';assert needle in s;s=s.replace(needle,needle+'\n        lp_cold_recovery_attempts:matches!(limits.engine,Engine::Revised).then_some(lp_stats.cold_recovery_attempts),\n        lp_cold_recovery_candidates:matches!(limits.engine,Engine::Revised).then_some(lp_stats.cold_recovery_candidates),',1);needle='    stats.add(&revised::Stats {';assert needle in s;s=s.replace(needle,needle+'\n        cold_recovery_attempts:result.lp_cold_recovery_attempts.unwrap_or(0),\n        cold_recovery_candidates:result.lp_cold_recovery_candidates.unwrap_or(0),',1);p.write_text(s)
