import json,sys,copy
from pathlib import Path
sys.path.insert(0,'/home/ubuntu/mip-dual-restoration/scripts')
from verify_cover_cuts import validate_cut_proofs
root=Path('artifacts/row-ray-bound-synthetic');count=bad=0
for p in sorted(root.iterdir()):
 if not p.is_dir():continue
 m=json.load(open(p/'model.json'));a=json.load(open(p/'result.json'));lines=['NAME ray','OBJSENSE',' MAX' if m['maximize'] else ' MIN','ROWS',' N obj']+[f" G {r['name']}" for r in m['rows']]+['COLUMNS']
 for j,v in enumerate(m['variables']):
  lines+=[f" x{j} obj {v['objective']}"]
  for r in m['rows']:
   for k,c in r['coefficients']:
    if k==j:lines+=[f" x{j} {r['name']} {c}"]
 lines+=['RHS',f" rhs obj {-m['offset']}"]+[f" rhs {r['name']} {r['lower']}" for r in m['rows']]+['BOUNDS']+[f' BV b x{j}' for j in range(len(m['variables']))]+['ENDATA'];path=p/'model.mps';path.write_text('\n'.join(lines)+'\n')
 proof={k:a[k] for k in ['rows','multiplier_bits','normalized_bound','seconds']};result=dict(row_ray_proof=proof);v=validate_cut_proofs(path,result);assert v['valid'] is True,v;count+=1
 cases=[]
 b=copy.deepcopy(result);b['row_ray_proof']['normalized_bound']=1e12;cases.append(b)
 b=copy.deepcopy(result);b['row_ray_proof']['rows']=[len(m['rows'])];cases.append(b)
 b=copy.deepcopy(result);b['row_ray_proof']['multiplier_bits']='bff0000000000000';cases.append(b)
 b=copy.deepcopy(result);b['row_ray_proof']['rows']=[0,0];cases.append(b)
 b=copy.deepcopy(result);b['row_ray_proof']['multiplier_bits']='7ff0000000000000';cases.append(b)
 for b in cases:
  v=validate_cut_proofs(path,b);assert v['valid'] is False,v;bad+=1
(root/'proof-test-summary.json').write_text(json.dumps(dict(valid=count,tamper_rejected=bad),indent=2)+'\n');print(count,bad)
