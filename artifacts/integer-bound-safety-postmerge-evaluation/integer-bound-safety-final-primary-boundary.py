from pathlib import Path
import json,sys,subprocess,os
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
from verify_cover_cuts import validate_cut_proofs
from validate_solution import validate
root=Path.cwd();out=root/'artifacts/integer-bound-safety-final-primary-boundary';out.mkdir();binary=root/'target/release/mip-codex';old=root/'artifacts/cold-dual-boundary-pilot';expected={r['name']:r['exact_optimum']for r in json.loads((old/'records.json').read_text())};records=[]
atomic_json(out/'experiment.json',dict(binary_sha256=sha256(binary),build_info=json.loads(subprocess.check_output([str(binary),'build-info'])),cpu=42,seconds=2,box_primal=False,primal_repair=False,binary_graph=False))
for name,opt in expected.items():
 model=old/name/'model.mps';d=out/name;d.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_BOX_PRIMAL='0',MIP_PRIMAL_REPAIR='0',MIP_BINARY_GRAPH='0');p=subprocess.run(['taskset','-c','42',str(binary),'solve',str(model),'2'],env=env,capture_output=True,text=True,timeout=62);(d/'result.json').write_text(p.stdout);(d/'stderr.log').write_text(p.stderr);assert p.returncode==0;r=json.loads(p.stdout);check=validate_cut_proofs(model,r);atomic_json(d/'proof-validation.json',check);assert check['valid']is True,check;assert r['status']==('infeasible'if opt is None else'optimal'),(name,r['status']);assert r['objective']==opt,(name,r['objective'],opt)
 if r['values']is not None:
  v=validate(model,r);atomic_json(d/'validation.json',v);assert v['valid']is True,(name,v)
 records.append(dict(name=name,status=r['status'],objective=r['objective'],exact_optimum=opt))
atomic_json(out/'summary.json',dict(outputs=len(records),points=sum(r['objective']is not None for r in records),all_correct=True,records=records));atomic_json(out/'exit.json',dict(status='complete'));print(json.dumps(records))
