import json,os,sys
from pathlib import Path
import highspy
sys.path.insert(0,'scripts');from benchmark_common import sha256,atomic_json
os.sched_setaffinity(0,{0});records=[];manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']}
for p in sorted(Path('artifacts/aggregate-three-combined-development-screen/cases').glob('*/after/result.json')):
 d=json.load(open(p));r=manifest[p.parent.parent.name];assert r['split']=='development'
 if d.get('objective')is not None or d.get('nodes',0)<1 or d.get('status')=='resource_limit':continue
 model=Path(r['path']);assert sha256(model)==r['sha256'];h=highspy.Highs();h.setOptionValue('output_flag',False);h.readModel(str(model));lp=h.getLp();domains=[];nint=0;lower=lp.col_lower_;upper=lp.col_upper_
 for j,t in enumerate(lp.integrality_):
  if t!=highspy.HighsVarType.kInteger:continue
  l,u=lower[j],upper[j];nint+=1
  if l>=0 and u<=1 or l==u:continue
  domains.append(j)
 if domains and nint<=1024:
  record=dict(name=r['name'],integer=nint,general=len(domains),rows=lp.num_row_,columns=lp.num_col_,nonzeros=len(lp.a_matrix_.value_),root_seconds=d.get('cut_initial_lp_seconds'),status=d['status'],nodes=d['nodes']);records.append(record);print(record,flush=True)
atomic_json(Path('artifacts/general-integer-pump-eligibility.json'),records)
