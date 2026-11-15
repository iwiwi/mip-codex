#!/usr/bin/env python3
"""固定seedの符号付き競合graphを全列挙し、元MPSとclique証明を独立検査。"""
import copy,itertools,json,os,random,subprocess,sys
from decimal import Decimal
from pathlib import Path
from benchmark_common import require_solver_origin,sha256
from validate_solution import validate
from verify_cover_cuts import validate_cut_proofs
root=Path(__file__).resolve().parents[1];binary=root/'target/release/mip-codex'
info=require_solver_origin(root,binary);out=root/os.environ.get('MIP_CLIQUE_DIR','artifacts/clique-enumeration');out.mkdir(parents=True,exist_ok=True)
(out/'experiment.json').write_text(json.dumps(dict(build_info=info,binary_sha256=sha256(binary),seed=2026111507),indent=2)+'\n')
rng=random.Random(2026111507);records=[]
for case in range(int(sys.argv[1])if len(sys.argv)>1 else 200):
 n=rng.randint(3,9);negative=[rng.choice([False,True])for _ in range(n)];cost=[rng.randint(-4,4)for _ in range(n)]
 edges=[(a,b)for a in range(n)for b in range(a+1,n)if rng.random()<.65];mixed=case%2==1
 rows=[f' L r{k}'for k in range(len(edges))];columns=[]
 for j in range(n):
  columns.append(f' x{j} obj {cost[j]}')
  columns.extend(f' x{j} r{k} {-1 if negative[j]else 1}'for k,e in enumerate(edges)if j in e)
 if mixed:columns.extend(f' c{k} r{k} 1'for k in range(len(edges)))
 rhs=[f' rhs r{k} {1-sum(negative[j]for j in e)-(.25 if mixed else 0)}'for k,e in enumerate(edges)]
 bounds=[f' BV b x{j}'for j in range(n)]
 if mixed:
  for k in range(len(edges)):bounds.extend([f' LO b c{k} -0.25',f' UP b c{k} 0.75'])
 model=out/f'{case:04d}.mps';model.write_text('\n'.join(['NAME graph','ROWS',' N obj',*rows,'COLUMNS',*columns,'RHS',*rhs,'BOUNDS',*bounds,'ENDATA'])+'\n')
 expected=min(sum(c*x for c,x in zip(cost,point))for point in itertools.product([0,1],repeat=n)if all((1-point[a]if negative[a]else point[a])+(1-point[b]if negative[b]else point[b])<=1 for a,b in edges))
 command=[str(binary),'solve',str(model),'5'];env=dict(os.environ,MIP_CLIQUE_CUTS='1')
 with (out/f'{case:04d}.json').open('w')as stream,(out/f'{case:04d}.err').open('w')as err:p=subprocess.run(command,env=env,stdout=stream,stderr=err,timeout=15)
 result=json.loads((out/f'{case:04d}.json').read_text(),parse_float=Decimal)
 try:
  assert p.returncode==0 and result['status']=='optimal'and result['clique_cuts_enabled'],result
  assert abs(float(result['objective'])-expected)<=1e-7 and abs(float(result['dual_bound'])-expected)<=1e-7,(expected,result)
  check=validate(model,result);assert check['valid']is True,check
  proof=validate_cut_proofs(model,result);assert proof['valid']is True,proof
  corruptions=0
  for cut in result['cut_proofs']:
   if cut['proof'].get('clique')is None:continue
   for mode in ['missing','duplicate','rhs','sign']:
    bad=copy.deepcopy(cut)
    if mode=='missing':bad['proof']['clique'].pop()
    elif mode=='duplicate':bad['proof']['clique'][1]=copy.deepcopy(bad['proof']['clique'][0])
    elif mode=='rhs':bad['row']['upper']-=1
    else:bad['row']['coefficients'][0][1]*=-1
    assert validate_cut_proofs(model,dict(cut_proofs=[bad]))['valid']is False,(mode,bad)
    corruptions+=1
  record=dict(case=case,valid=True,expected=expected,cliques=result['cut_stats']['clique_added'],corruptions_rejected=corruptions)
 except Exception as error:record=dict(case=case,valid=False,error=str(error))
 records.append(record);print(json.dumps(record),flush=True)
summary=dict(cases=len(records),failures=sum(not x['valid']for x in records),cliques=sum(x.get('cliques',0)for x in records),corruptions_rejected=sum(x.get('corruptions_rejected',0)for x in records))
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary));raise SystemExit(bool(summary['failures']))
