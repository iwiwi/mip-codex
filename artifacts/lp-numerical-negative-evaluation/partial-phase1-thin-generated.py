import json,os,random,subprocess,resource,sys,time
from pathlib import Path
import highspy
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
root=Path.cwd();out=root/'artifacts/partial-phase1-thin-generated';out.mkdir(exist_ok=False);os.sched_setaffinity(0,{2});rng=random.Random(2026111609);binary=root/'artifacts/partial-phase1-diagnostic/bin/mip-codex';records=[]
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for case in range(6):
 d=out/str(case);d.mkdir();n=6000;m=24;rows=[[rng.randrange(4)for _ in range(n)]for _ in range(m)];plant=[rng.random()<.1 for _ in range(n)];b=[sum(a*x for a,x in zip(row,plant))-rng.randrange(10)for row in rows];cost=[rng.randint(1,20)for _ in range(n)];lines=['NAME thin','ROWS',' N obj']+[f' G r{i}'for i in range(m)]+['COLUMNS']
 for j in range(n):
  entries=[('obj',cost[j])]+[(f'r{i}',rows[i][j])for i in range(m)if rows[i][j]]
  for k in range(0,len(entries),2):lines.append(' x'+str(j)+' '+ ' '.join(f'{a} {v}'for a,v in entries[k:k+2]))
 lines+=['RHS']+[f' rhs r{i} {v}'for i,v in enumerate(b)]+['BOUNDS']+[f' UP bd x{j} 1'for j in range(n)]+['ENDATA'];model=d/'model.mps';model.write_text('\n'.join(lines)+'\n');h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.readModel(str(model));h.run();expected=h.getObjectiveValue();assert h.getModelStatus()==highspy.HighsModelStatus.kOptimal
 for label in (['before','after']if case%2==0 else['after','before']):
  q=d/label;q.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')}
  for f in ['MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','BINARY_GRAPH','OBJECTIVE_ROUNDING','BOX_PRIMAL','PRIMAL_REPAIR','PRIMAL_PROBE','RINS','STRONG_BRANCHING','LP_DIVING','FEASIBILITY_PUMP','PROPAGATION']:env['MIP_'+f]='0'
  env.update(MIP_LP_PARTIAL_PRICING='0'if label=='before'else'1',OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');cmd=[str(binary),'solve',str(model),'20','1'];atomic_json(q/'experiment.json',dict(command=cmd,input_sha256=sha256(model),binary_sha256=sha256(binary),partial=env['MIP_LP_PARTIAL_PRICING'],cpu=2,memory_bytes=16*1024**3,highs_lp_objective=expected))
  with (q/'result.json').open('w')as output,(q/'stderr.log').open('w')as error:p=subprocess.run(cmd,env=env,stdout=output,stderr=error,timeout=80,preexec_fn=restrict)
  assert p.returncode==0;value=json.load(open(q/'result.json'));assert value['status']=='optimal'and abs(value['objective']-expected)<=1e-6*(1+abs(expected));record=dict(case=case,variant=label,seconds=value['seconds'],pivots=value['lp_pivots'],flips=value['lp_flips'],calls=value['lp_partial_pricing_calls'],columns=value['lp_partial_pricing_columns'],objective=value['objective'],highs=expected);records.append(record);print(json.dumps(record),flush=True);atomic_json(out/'records.json',records)
atomic_json(out/'exit.json',dict(status='complete',records=len(records)))
