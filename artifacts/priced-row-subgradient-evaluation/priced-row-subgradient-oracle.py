from pathlib import Path
import sys,json,random,subprocess,copy,shutil
from fractions import Fraction as F
from importlib.util import spec_from_file_location,module_from_spec
sys.path.insert(0,'scripts');from benchmark_common import require_solver_origin,sha256
r=Path.cwd();repo=Path('/home/ubuntu/mip-priced-repair');b=repo/'target/release/examples/row_subgradient_oracle';info=require_solver_origin(repo,b);o=r/'artifacts/priced-row-subgradient-oracle';o.mkdir();(o/'bin').mkdir();shutil.copy2(b,o/'bin/helper');shutil.copy2(repo/'examples/row_subgradient_oracle.rs',o/'source.rs');(o/'experiment.json').write_text(json.dumps(dict(build_info=info,binary_sha256=sha256(b),source_reference='artifacts/priced-row-subgradient-v1-diagnostic',cpu=37,cases=200),indent=2)+'\n');spec=spec_from_file_location('frozen_checker',r/'artifacts/priced-row-subgradient-v1-diagnostic/scripts/verify_cover_cuts.py');check=module_from_spec(spec);spec.loader.exec_module(check);rng=random.Random(4201);records=[]
for case in range(200):
 n=6+case%5;cost=[1]+[2+rng.randrange(31)for _ in range(n-1)];offset=F(case%9-4,4);rows=[]
 for i in range(4):
  row=[j for j in range(n)if rng.randrange(3)==0]
  if not row:row=[rng.randrange(n)]
  rows.append(row)
 d=o/str(case);d.mkdir();m=d/'model.mps';s='NAME oracle\nROWS\n N obj\n'+''.join(f' G r{i}\n'for i in range(4))+' L cap\nCOLUMNS\n'
 for j in range(n):
  s+=f' x{j} obj {cost[j]/4:.2f}\n'+''.join(f' x{j} r{i} 1\n'for i,row in enumerate(rows)if j in row)+f' x{j} cap 1\n'
 s+='RHS\n'+''.join(f' rhs r{i} 1\n'for i in range(4))+f' rhs cap {n-1}\n rhs obj {float(-offset):.2f}\nBOUNDS\n'+''.join(f' BV b x{j}\n'for j in range(n))+'ENDATA\n';m.write_text(s)
 optimum=offset+min(F(sum(cost[j]for j in range(n)if mask>>j&1),4)for mask in range(1<<n)if mask.bit_count()<=n-1 and all(any(mask>>j&1 for j in row)for row in rows));cmd=['taskset','-c','37','timeout','--signal=KILL','10',str(o/'bin/helper'),str(m)]
 with(d/'result.json').open('w')as out,(d/'stderr.log').open('w')as err:p=subprocess.run(cmd,stdout=out,stderr=err)
 assert p.returncode==0;v=json.load(open(d/'result.json'));audit=check.validate_cut_proofs(m,v);assert audit['valid']is True,audit;assert F(str(v['dual_bound']))<=optimum;tampered=0
 if v.get('row_subgradient_proof')is not None:
  for kind in ['bound','negative','duplicate','box']:
   bad=copy.deepcopy(v);proof=bad['row_subgradient_proof'];path=m
   if kind=='bound':proof['normalized_bound']=float(optimum+1);bad['dual_bound']=float(optimum+1)
   elif kind=='negative':proof['multipliers'][0][1]='bff0000000000000'
   elif kind=='duplicate':proof['multipliers'].append(proof['multipliers'][0])
   else:path=d/'changed-box.mps';path.write_text(s.replace(' BV b x0\n',' UI b x0 2\n'))
   assert check.validate_cut_proofs(path,bad)['valid']is False,(kind,case);tampered+=1
 record=dict(case=case,input_sha256=sha256(m),result_sha256=sha256(d/'result.json'),exact_optimum=str(optimum),bound=v['dual_bound'],new_proof=v.get('row_subgradient_proof')is not None,validation=audit,tampered_rejected=tampered);(d/'validation.json').write_text(json.dumps(record,indent=2)+'\n');records.append(record);(o/'summary.json').write_text(json.dumps(dict(cases=len(records),valid=len(records),new_proofs=sum(x['new_proof']for x in records),tampered_rejected=sum(x['tampered_rejected']for x in records)),indent=2)+'\n')
(o/'records.json').write_text(json.dumps(records,indent=2)+'\n');(o/'exit.json').write_text(json.dumps(dict(status='complete'))+'\n');print(json.load(open(o/'summary.json')))
