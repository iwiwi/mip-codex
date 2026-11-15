import json,os,sys,time
from pathlib import Path
sys.path.insert(0,'scripts');from validate_solution import original_lp
from benchmark_common import atomic_json,sha256
import highspy
os.sched_setaffinity(0,{7});model=Path('data/raw/benchmark/germanrr.mps.gz');lp=original_lp(model);lp.integrality_=[]
h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('time_limit',60);h.setOptionValue('small_matrix_value',1e-12);h.passModel(lp);start=time.monotonic();h.run();r=dict(purpose='comparison LP only; no native weights/primal used in Rust',input_sha256=sha256(model),highs_version=h.version(),status=str(h.getModelStatus()),objective=h.getObjectiveValue(),seconds=time.monotonic()-start);atomic_json(Path('artifacts/root-refinement-native-lp.json'),r);print(json.dumps(r))
