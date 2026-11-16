import json,sys,os,resource,time
from pathlib import Path
import highspy,numpy as np
sys.path.insert(0,'scripts')
from validate_solution import original_lp
from verify_cover_cuts import validate_cut_proofs
from benchmark_common import sha256,atomic_json
os.sched_setaffinity(0,{40});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));model=Path('data/raw/benchmark/timtab1.mps.gz');records=[]
for variant in ['before','after']:
 result=Path('artifacts/mir-gate-paired-1/timtab1.mps.gz')/variant/'result.json';value=json.loads(result.read_text());audit=validate_cut_proofs(model,value);assert audit['valid'];cuts=[c['row']for c in value['cut_proofs']];lp=original_lp(model);lp.integrality_=[];h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('time_limit',60);h.passModel(lp);starts=[0];indices=[];vals=[]
 for c in cuts:
  for j,a in c['coefficients']:indices.append(j);vals.append(a)
  starts.append(len(indices))
 if cuts:h.addRows(len(cuts),np.full(len(cuts),-highspy.kHighsInf),np.array([c['upper']for c in cuts],dtype=float),len(indices),np.array(starts,dtype=np.int32),np.array(indices,dtype=np.int32),np.array(vals,dtype=float))
 start=time.monotonic();h.run();r=dict(variant=variant,status=str(h.getModelStatus()),objective=h.getObjectiveValue(),seconds=time.monotonic()-start,cut_count=len(cuts),proof_audit=audit,input_sha256=sha256(model),result_sha256=sha256(result),highs_version=h.version(),cpu=40,threads=1,memory_bytes=16*1024**3,purpose='comparison only, no native point/multiplier returned to Rust');records.append(r);print(json.dumps(r),flush=True)
atomic_json(Path('artifacts/mir-gate-timtab-lp-compare.json'),records)
