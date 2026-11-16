import json,os,resource,sys,time
from pathlib import Path
import highspy
sys.path.insert(0,'scripts')
from validate_solution import original_lp
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{3});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));root=Path.cwd();out=root/'artifacts/static-pricing-native-lp';out.mkdir(exist_ok=True);records=[]
for name in sys.argv[1:]:
 d=root/'artifacts/pricing-guarded-root-survey'/name;meta=json.loads((d/'experiment.json').read_text());r=json.loads((d/'summary.json').read_text());assert meta['split']=='development'and r['point_dump'];model=Path(meta['command'][2]);assert sha256(model)==meta['input_sha256'];lp=original_lp(model);lp.integrality_=[];h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('time_limit',60);h.passModel(lp);start=time.monotonic();h.run();record=dict(name=name,highs_version=h.version(),status=str(h.getModelStatus()),highs_objective=h.getObjectiveValue(),rust_point_objective=r['lp_objective'],rust_dual_bound=r['dual_bound'],seconds=time.monotonic()-start,cpu=3,threads=1,memory_bytes=16*1024**3,input_sha256=sha256(model),point_sha256=sha256(d/'root-lp.json'),purpose='comparison-only continuous LP; no native point or multipliers returned to Rust')
 if h.getModelStatus()==highspy.HighsModelStatus.kOptimal:
  tol=1e-8*(1+abs(h.getObjectiveValue()));record['point_objective_matches']=abs(r['lp_objective']-h.getObjectiveValue())<=tol;record['dual_not_above_optimum']=r['dual_bound']<=h.getObjectiveValue()+tol;assert record['point_objective_matches']and record['dual_not_above_optimum']
 records.append(record);atomic_json(out/(name+'.json'),record);print(json.dumps(record),flush=True)
atomic_json(out/'results.json',records)
