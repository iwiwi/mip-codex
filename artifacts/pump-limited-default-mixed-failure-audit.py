from pathlib import Path
import os,sys,json,ast
root=Path.cwd();os.sched_setaffinity(0,{41});os.environ['MIP_ABSOLUTE_CHECKER']='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute';sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from validate_solution import validate
from verify_cover_cuts import validate_cut_proofs
out=root/'artifacts/pump-limited-default-mixed-failure-audit';out.mkdir();old={r['case']:r for r in json.load(open('artifacts/pump-limited-mixed-failure-audit/records.json'))};records=[]
for line in Path('artifacts/pump-limited-default-mixed-oracle.log').read_text().splitlines():
 try:r=json.loads(line)
 except ValueError:continue
 if r.get('status')!='fail':continue
 case=r['case'];point=ast.literal_eval(r['error']);model=root/f'artifacts/pump-limited-default-mixed-oracle/failure-{case}.mps';assert sha256(model)==old[case]['input_sha256'];assert point['status']=='numerical_error' and point['pump']['calls']==0 and point['optimal_original_conflict'];d=out/str(case);d.mkdir();atomic_json(d/'result.json',point);v=validate(model,point);atomic_json(d/'validation.json',v);assert v['valid'];p=validate_cut_proofs(model,point);atomic_json(d/'proof.json',p);assert p['valid'];records.append(dict(case=case,input_sha256=sha256(model),result_sha256=sha256(d/'result.json'),gmp_valid=True,proof_valid=True,status=point['status'],objective=point['objective'],pump_calls=0,exact_reference_reused_same_input_sha256=True,exact_optimum=old[case]['exact_optimum']))
assert sorted(r['case'] for r in records)==[8,40,87,122,169];atomic_json(out/'records.json',records);atomic_json(out/'summary.json',dict(models=5,gmp_valid=5,proof_valid=5,numerical_error=5,pump_calls=0,exact_fraction_reference_reused_same_input=5,false_optimality_claims=0));print('5 same canonical numerical errors;5 GMP/proofs valid')
