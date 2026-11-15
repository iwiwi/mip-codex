import copy,json,os,subprocess,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from validate_solution import validate
from verify_cover_cuts import validate_cut_proofs
from benchmark_common import require_solver_origin,atomic_json
root=Path.cwd();binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary);d=root/'artifacts/aggregate-cuts-toy';model=d/'model.mps';records=[]
os.sched_setaffinity(0,{2})
for enabled in [0,1]:
 with(d/f'{enabled}.json').open('w')as out,(d/f'{enabled}.err').open('w')as err:p=subprocess.run([str(binary),'solve',str(model),'10','1'],stdout=out,stderr=err,env=dict(os.environ,MIP_AGGREGATE_CUTS=str(enabled)),timeout=30)
 r=json.loads((d/f'{enabled}.json').read_text());assert p.returncode==0;point=validate(model,r);proof=validate_cut_proofs(model,r);assert point['valid']and proof['valid']
 rejected=0
 if enabled:
  assert r['status']=='optimal'and r['objective']==r['dual_bound']==-1
  assert r['cut_stats']['aggregate_added']>0
  ci=next(i for i,c in enumerate(r['cut_proofs'])if c['proof'].get('aggregation'))
  for mode in range(7):
   bad=copy.deepcopy(r);c=bad['cut_proofs'][ci]
   if mode==0:c['proof']['aggregation'][1]['weight']=0
   elif mode==1:c['proof']['aggregation'][1]['source_name']+='!'
   elif mode==2:c['proof']['aggregation'][1]['source_row']=c['proof']['aggregation'][0]['source_row']
   elif mode==3:c['proof']['aggregation'][0]['upper_side']=not c['proof']['upper_side']
   elif mode==4:c['proof']['divisor']=1
   elif mode==5:c['row']['upper']-=1
   else:c['proof']['aggregation'][1]['weight']=True
   assert validate_cut_proofs(model,bad)['valid']is False,mode;rejected+=1
 records.append(dict(enabled=enabled,status=r['status'],objective=r['objective'],dual_bound=r['dual_bound'],point=point,proof=proof,rejected=rejected))
atomic_json(d/'checks.json',dict(build_info=info,records=records));print(json.dumps(records))
