from pathlib import Path
from importlib.util import spec_from_file_location,module_from_spec
from decimal import Decimal
import json,subprocess,shutil,sys,os,struct,hashlib
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();out=root/'artifacts/safe-gmi-box-chain-oracle';out.mkdir();os.sched_setaffinity(0,{40});binary=Path('/home/ubuntu/mip-dual-restoration/target/release/examples/gmi_bound_oracle');shutil.copy2(binary,out/'helper');shutil.copy2('/home/ubuntu/mip-dual-restoration/examples/gmi_bound_oracle.rs',out/'helper.rs');base=json.load(open('artifacts/safe-gmi-fraction-oracle/input.json'))[:200];algebra=[json.loads(x)for x in Path('artifacts/safe-gomory-algebra-oracle/cases.jsonl').read_text().splitlines()][:200];inputs=[]
for v,a in zip(base,algebra):
 n=len(a['upper']);text=v['model'];text=text.replace('COLUMNS\n',''.join(f' L box{j}\n'for j in range(n))+'COLUMNS\n');text=''.join(text)
 for j in range(n):text=text.replace(f' x{j} obj 0\n',f' x{j} obj 0\n x{j} box{j} 1\n')
 text=text.replace('BOUNDS\n',''.join(f' rhs box{j} {u}\n'for j,u in enumerate(a['upper']))+'BOUNDS\n');text='\n'.join(line for line in text.splitlines()if not line.startswith((' UI ',' UP ')))+'\n';inputs.append(dict(model=text,weights=[-abs(w)for w in a['weights']]+[0]*n))
atomic_json(out/'input.json',inputs)
with(out/'input.json').open()as inp,(out/'rust.jsonl').open('w')as stream,(out/'stderr.log').open('w')as err:p=subprocess.run([str(out/'helper')],stdin=inp,stdout=stream,stderr=err,timeout=60)
assert p.returncode==0;rows=[json.loads(x)for x in(out/'rust.jsonl').read_text().splitlines()];assert len(rows)==200;spec=spec_from_file_location('checker',root/'artifacts/safe-gmi-box-diagnostic/scripts/verify_cover_cuts.py');checker=module_from_spec(spec);spec.loader.exec_module(checker);count=0;steps=0
for case,(v,r)in enumerate(zip(inputs,rows)):
 d=out/str(case);d.mkdir();model=d/'model.mps';model.write_text(v['model']);assert r['gomory_bound_proof']is not None;r['dual_bound']=r['gomory_bound_proof']['normalized_bound'];normal=checker.validate_cut_proofs(model,r);decimal=checker.validate_cut_proofs(model,json.loads(json.dumps(r),parse_float=Decimal));assert normal['valid']is True and decimal['valid']is True,(case,normal,decimal);assert r['gomory_bound_proof']['bounds'];steps+=len(r['gomory_bound_proof']['bounds']);bad=json.loads(json.dumps(r));bad['gomory_bound_proof']['bounds'][0]['value_bits']=struct.pack('>d',-1e9).hex();wrong=checker.validate_cut_proofs(model,bad);assert wrong['valid']is False,(case,wrong);atomic_json(d/'result.json',r);atomic_json(d/'validation.json',dict(valid=normal,decimal=decimal,tampered=wrong));count+=1
atomic_json(out/'summary.json',dict(models=200,valid=count,decimal_valid=count,box_steps=steps,tampered_rejected=count,failures=0,binary_sha256=sha256(out/'helper'),build_info=json.loads(subprocess.check_output([str(out/'helper'),'build-info']))));print(json.load(open(out/'summary.json')))
