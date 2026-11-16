import json,os,sys,time
from pathlib import Path
import highspy
sys.path.insert(0,'scripts');from validate_solution import original_lp
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{40});model=Path('data/raw/benchmark/neos-3754480-nidda.mps.gz');results=[]
for path in sorted(Path('artifacts/numerical-certificate-v2-witnesses').glob('candidate*.json')):
 d=json.load(open(path));lp=original_lp(model);assert lp.col_names_==d['names'];lp.integrality_=[];lp.col_lower_=[-highspy.kHighsInf if v is None else v for v in d['lower']];lp.col_upper_=[highspy.kHighsInf if v is None else v for v in d['upper']]
 h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('time_limit',30);h.passModel(lp);start=time.monotonic();h.run();r=dict(node=d['node'],candidate=str(path),candidate_sha256=sha256(path),status=str(h.getModelStatus()),objective=h.getObjectiveValue(),seconds=time.monotonic()-start,version=h.version());results.append(r);print(r,flush=True)
atomic_json(Path('artifacts/numerical-certificate-node-native.json'),dict(purpose='comparison-only node LP diagnosis; no native solution used by Rust',input_sha256=sha256(model),results=results))
