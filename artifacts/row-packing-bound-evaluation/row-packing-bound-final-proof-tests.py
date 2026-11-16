from pathlib import Path
import json,sys,subprocess,copy
sys.path.insert(0,'/home/ubuntu/mip-dual-restoration/scripts');from verify_cover_cuts import validate_cut_proofs
source=Path('artifacts/cover-row-packing-synthetic');out=Path('artifacts/row-packing-bound-final-proof-tests');out.mkdir();count=bad=0
for p in sorted(source.iterdir()):
 if not p.is_dir():continue
 m=json.load(open(p/'model.json'));maximize=int(p.name)%2==1;m['maximize']=maximize
 if maximize:
  for v in m['variables']:v['objective']=-1;v['objective_enclosure']=[-1,-1]
 q=out/p.name;q.mkdir();(q/'model.json').write_text(json.dumps(m));proof=json.loads(subprocess.check_output(['target-dual-restoration/release/examples/row_packing_bound',str(q/'model.json')],text=True));assert proof is not None;(q/'proof.json').write_text(json.dumps(proof))
 lines=['NAME pack','OBJSENSE',' MAX' if maximize else ' MIN','ROWS',' N obj']+[f" G {r['name']}" for r in m['rows']]+['COLUMNS']
 for j,v in enumerate(m['variables']):
  lines+=[f" x{j} obj {v['objective']}"]
  for r in m['rows']:
   for k,c in r['coefficients']:
    if k==j:lines+=[f" x{j} {r['name']} {c}"]
 lines+=['RHS',f" rhs obj {-m['offset']}"]+[f" rhs {r['name']} {r['lower']}" for r in m['rows']]+['BOUNDS']+[f' BV b x{j}' for j in range(len(m['variables']))]+['ENDATA'];path=q/'model.mps';path.write_text('\n'.join(lines)+'\n');result=dict(row_packing_proof=proof);v=validate_cut_proofs(path,result);assert v['valid']is True,v;count+=1
 cases=[]
 b=copy.deepcopy(result);b['row_packing_proof']['normalized_bound']+=1;cases.append(b)
 b=copy.deepcopy(result);b['row_packing_proof']['rows']=[len(m['rows'])];cases.append(b)
 b=copy.deepcopy(result);b['row_packing_proof']['rows']=[0,0];cases.append(b)
 for b in cases:
  v=validate_cut_proofs(path,b);assert v['valid']is False,v;bad+=1
(out/'summary.json').write_text(json.dumps(dict(valid=count,tamper_rejected=bad),indent=2));print(count,bad)
