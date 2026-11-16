import sys,json,os,time,resource
from pathlib import Path
import highspy,numpy as np
sys.path.insert(0,'scripts')
from validate_solution import original_lp
from verify_cover_cuts import validate_cut_proofs
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{39});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));records=[]
for item in json.loads(Path('artifacts/mir-smoke-comparison.json').read_text())['records']:
 if item['mir_added']==0:continue
 name=item['name'];model=Path('data/raw/benchmark')/name
 for variant,key in [('before','before_result'),('after','after_result')]:
  result=Path(item[key]);r=json.loads(result.read_text());audit=validate_cut_proofs(model,r);assert audit['valid'];cuts=[c['row']for c in r['cut_proofs']];lp=original_lp(model);lp.integrality_=[];h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('time_limit',60);h.passModel(lp);starts=[0];indices=[];values=[]
  for c in cuts:
   for j,a in c['coefficients']:indices.append(j);values.append(a)
   starts.append(len(indices))
  if cuts:h.addRows(len(cuts),np.full(len(cuts),-highspy.kHighsInf),np.array([c['upper']for c in cuts],dtype=float),len(indices),np.array(starts,dtype=np.int32),np.array(indices,dtype=np.int32),np.array(values,dtype=float))
  t=time.monotonic();h.run();record=dict(name=name,variant=variant,status=str(h.getModelStatus()),objective=h.getObjectiveValue(),seconds=time.monotonic()-t,cuts=len(cuts),proof_audit=audit,result_sha256=sha256(result),input_sha256=sha256(model),highs_version=h.version(),cpu=39,threads=1,memory_bytes=16*1024**3,purpose='comparison only; no native point or multipliers returned to Rust');records.append(record);print(json.dumps(record),flush=True);atomic_json(Path('artifacts/mir-actual-native.json'),records)
