import json,os,sys,time
from pathlib import Path
import highspy,numpy as np
sys.path.insert(0,'scripts')
from verify_cover_cuts import validate_cut_proofs
from validate_solution import original_lp
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{1});root=Path.cwd();model=root/'data/raw/benchmark/eil33-2.mps.gz';result=root/'artifacts/clique-cut-repeat60/0/after/eil33-2/result.json';r=json.loads(result.read_text());proof=validate_cut_proofs(model,r);assert proof['valid'];out=root/'artifacts/clique-cut-native-final-lp';out.mkdir(exist_ok=True);atomic_json(out/'proof.json',proof);records=[]
for label,cuts in [('original',[]),('with_verified_cuts',r['cut_proofs'])]:
 lp=original_lp(model);lp.integrality_=[];h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('time_limit',60);h.passModel(lp)
 starts=[0];indices=[];values=[]
 for cut in cuts:
  for j,a in cut['row']['coefficients']:indices.append(j);values.append(a)
  starts.append(len(indices))
 if cuts:h.addRows(len(cuts),np.full(len(cuts),-highspy.kHighsInf),np.array([c['row']['upper']for c in cuts],dtype=float),len(indices),np.array(starts,dtype=np.int32),np.array(indices,dtype=np.int32),np.array(values,dtype=float))
 begin=time.monotonic();h.run();record=dict(variant=label,highs_version=h.version(),status=str(h.getModelStatus()),objective=h.getObjectiveValue(),seconds=time.monotonic()-begin,cuts=len(cuts),purpose='comparison-only LP; no native point or multipliers returned to Rust',model_sha256=sha256(model),result_sha256=sha256(result));records.append(record);print(json.dumps(record),flush=True)
atomic_json(out/'results.json',records)
