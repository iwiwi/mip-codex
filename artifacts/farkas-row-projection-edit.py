from pathlib import Path
p=Path('src/scaling.rs');s=p.read_text();at=s.index('fn point_valid(');s=s[:at]+'''// Project only forbidden signs; a candidate is accepted solely by the same
// directed-rounding contradiction test, never by a smallness threshold.
fn projected_row_certificate(p:&Problem,y:&[f64])->Option<Vec<f64>>{
 if y.len()!=p.rhs.len()||y.iter().any(|v|!v.is_finite()){return None;}
 let mut candidate=y.to_vec();let mut changed=false;
 for i in 0..candidate.len(){if !p.equality[i]&&!p.slack_upper[i].is_finite()&&candidate[i]>0.{candidate[i]=0.;changed=true;}}
 (changed&&contradiction(p,&candidate)).then_some(candidate)
}
''' +s[at:];needle='''                    let mut repaired = None;
                    for cap in [16, 256, 4096, 65536] {''';replacement='''                    let mut repaired = None;
                    if std::env::var("MIP_LP_FARKAS_ROW_PROJECT").is_ok_and(|v|v=="1") {
                        for sign in [1.,-1.] {
                            if deadline.is_some_and(|d|Instant::now()>=d){break;}
                            answer.stats.farkas_row_projection_trials+=1;
                            let oriented:Vec<_>=y.iter().map(|v|sign*v).collect();
                            if let Some(candidate)=projected_row_certificate(p,&oriented){repaired=Some(candidate);answer.stats.farkas_row_projections+=1;break;}
                        }
                    }
                    for cap in [16, 256, 4096, 65536] {
                        if repaired.is_some(){break;}''';assert needle in s;s=s.replace(needle,replacement,1);s+='''
#[cfg(test)]mod farkas_projection_tests{
 use super::*;
 fn example()->Problem{Problem{columns:vec![vec![(0,1.),(1,-1.),(2,1.)]],rhs:vec![0.,-1.,100.],costs:vec![0.],upper:vec![2.],equality:vec![false;3],slack_upper:vec![f64::INFINITY;3]}}
 #[test]fn forbidden_positive_slack_weight_is_projected_only_after_strict_recheck(){let p=example();let y=vec![-1.,-1.,2f64.powi(-80)];assert!(!contradiction(&p,&y));let projected=projected_row_certificate(&p,&y).unwrap();assert_eq!(projected,vec![-1.,-1.,0.]);assert!(contradiction(&p,&projected));assert_eq!(y[2],2f64.powi(-80));assert_eq!(p.rhs,vec![0.,-1.,100.]);let mut feasible=p.clone();feasible.rhs[1]=0.;assert!(projected_row_certificate(&feasible,&y).is_none());}
 #[test]fn finite_slacks_equalities_invalid_and_unchanged_weights_are_not_projected(){let mut p=example();let y=[-1.,-1.,2f64.powi(-80)];p.slack_upper[2]=2.;assert!(projected_row_certificate(&p,&y).is_none());p.slack_upper[2]=f64::INFINITY;p.equality[2]=true;assert!(projected_row_certificate(&p,&y).is_none());assert!(projected_row_certificate(&p,&[0.,0.,0.]).is_none());assert!(projected_row_certificate(&p,&[f64::NAN;3]).is_none());assert!(projected_row_certificate(&p,&[0.]).is_none());}
 #[test]fn exhaustive_feasible_integer_boxes_never_get_a_projected_contradiction(){for a in -2..=2{for b in -2..=2{for r in -2..=2{for t in -2..=2{if !(0..=2).any(|x|a*x<=r&&b*x<=t){continue;}let p=Problem{columns:vec![vec![(0,a as f64),(1,b as f64)]],rhs:vec![r as f64,t as f64],costs:vec![0.],upper:vec![2.],equality:vec![false;2],slack_upper:vec![f64::INFINITY;2]};for y0 in [-2.,-1.,0.,2f64.powi(-80),1.,2.]{for y1 in [-2.,-1.,0.,2f64.powi(-80),1.,2.]{assert!(projected_row_certificate(&p,&[y0,y1]).is_none());}}}}}}}
}
''';p.write_text(s)
p=Path('src/revised.rs');s=p.read_text().replace('pub struct Stats {','pub struct Stats {\n    pub farkas_row_projection_trials:usize,\n    pub farkas_row_projections:usize,',1);needle='    pub fn add(&mut self, other: &Self) {';assert needle in s;s=s.replace(needle,needle+'\n        self.farkas_row_projection_trials+=other.farkas_row_projection_trials;\n        self.farkas_row_projections+=other.farkas_row_projections;',1);p.write_text(s)
p=Path('src/solve.rs');s=p.read_text();needle='    pub lp_zero_phase_one_skips:Option<usize>,';assert needle in s;s=s.replace(needle,needle+'\n    pub lp_farkas_row_projection_trials:Option<usize>,\n    pub lp_farkas_row_projections:Option<usize>,',1);needle='        lp_zero_phase_one_skips:matches!(limits.engine,Engine::Revised).then_some(lp_stats.zero_phase_one_skips),';assert needle in s;s=s.replace(needle,needle+'\n        lp_farkas_row_projection_trials:matches!(limits.engine,Engine::Revised).then_some(lp_stats.farkas_row_projection_trials),\n        lp_farkas_row_projections:matches!(limits.engine,Engine::Revised).then_some(lp_stats.farkas_row_projections),',1);needle='    stats.add(&revised::Stats {';assert needle in s;s=s.replace(needle,needle+'\n        farkas_row_projection_trials:result.lp_farkas_row_projection_trials.unwrap_or(0),\n        farkas_row_projections:result.lp_farkas_row_projections.unwrap_or(0),',1);p.write_text(s)
