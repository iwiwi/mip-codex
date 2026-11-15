#!/usr/bin/env python3
"""独立全列挙、元MPSのGMP、根彩色のFraction検査でgraph経路を照合。"""
import copy,itertools,json,os,random,subprocess,sys
from decimal import Decimal
from pathlib import Path
from benchmark_common import require_solver_origin,sha256
from validate_solution import validate
from verify_cover_cuts import validate_cut_proofs
root=Path(__file__).resolve().parents[1];binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary)
out=root/os.environ.get('MIP_GRAPH_DIR','artifacts/binary-graph-enumeration');out.mkdir(parents=True,exist_ok=True)
(out/'experiment.json').write_text(json.dumps(dict(build_info=info,binary_sha256=sha256(binary),seed=2026111510,offset_profile="dyadic_eighths"),indent=2)+'\n')
rng=random.Random(2026111510);records=[]
for case in range(int(sys.argv[1])if len(sys.argv)>1 else 200):
 n=rng.randint(3,10);maximize=case%2==1;weight=rng.randint(1,4);offset=rng.randint(-56,56)/8
 constraints=[];edges=set()
 for a in range(n):
  for b in range(a+1,n):
   if rng.random()<.5:
    ca=rng.randint(1,5);cb=rng.randint(1,5);constraints.append(([(a,ca),(b,cb)],max(ca,cb)));edges.add((a,b))
 if case%5==0:
  group=sorted(rng.sample(range(n),min(n,4)));constraints.append(([(j,1)for j in group],1));edges.update(itertools.combinations(group,2))
 columns=[]
 for j in range(n):
  columns.append(f' x{j} obj {weight if maximize else-weight}')
  columns.extend(f' x{j} r{k} {-a if case%3==1 else a}'for k,(terms,_)in enumerate(constraints)for v,a in terms if v==j)
 model=out/f'{case:04d}.mps';model.write_text('\n'.join(['NAME graph','OBJSENSE',' MAX'if maximize else' MIN','ROWS',' N obj',*[f' {"G"if case%3==1 else"L"} r{k}'for k in range(len(constraints))],'COLUMNS',*columns,'RHS',f' rhs obj {-offset}',*[f' rhs r{k} {-rhs if case%3==1 else rhs}'for k,(_,rhs)in enumerate(constraints)],'BOUNDS',*[f' BV b x{j}'for j in range(n)],'ENDATA'])+'\n')
 optimum=max(sum(point)for point in itertools.product([0,1],repeat=n)if all(point[a]+point[b]<=1 for a,b in edges))
 for limit in [0,1,2,3,7,100000]:
  env=dict(os.environ,MIP_BINARY_GRAPH='1');dest=out/f'{case:04d}-{limit}.json'
  with dest.open('w')as stream,(out/f'{case:04d}-{limit}.err').open('w')as err:p=subprocess.run([str(binary),'solve',str(model),'5',str(limit)],env=env,stdout=stream,stderr=err,timeout=15)
  r=json.loads(dest.read_text(),parse_float=Decimal)
  try:
   assert p.returncode==0 and r['binary_graph']is not None and r['lp_engine']=='BinaryGraph',r
   s=r['binary_graph'];assert s['best']<=optimum<=s['upper'],(optimum,s)
   if s['complete']:assert s['best']==optimum and r['status']=='optimal'
   if limit==100000:assert s['complete']
   primary=validate(model,r);assert primary['valid']is True,primary
   proof=validate_cut_proofs(model,r);assert proof['valid']is True,proof
   rejected=0
   for mode in ['bound','permutation','edge_count']+(['prefix_permutation','prefix_color','prefix_remaining']if s.get('root_prefix_colorings')else[]):
    bad=copy.deepcopy(r)
    if mode=='bound':bad['dual_bound']+=-1 if maximize else 1
    elif mode=='permutation':bad['binary_graph']['order'][1]=bad['binary_graph']['order'][0]
    elif mode=='edge_count':bad['binary_graph']['conflict_edges']+=1
    elif mode=='prefix_permutation':bad['binary_graph']['root_search_order'][1]=bad['binary_graph']['root_search_order'][0]
    elif mode=='prefix_color':bad['binary_graph']['root_prefix_colorings'][0][0]=0
    else:bad['binary_graph']['remaining_upper']=0
    assert validate_cut_proofs(model,bad)['valid']is False,mode
    rejected+=1
   record=dict(case=case,limit=limit,valid=True,optimum=optimum,best=s['best'],upper=s['upper'],complete=s['complete'],primary=primary,proof=proof,corruptions_rejected=rejected)
  except Exception as error:record=dict(case=case,limit=limit,valid=False,error=str(error))
  records.append(record);print(json.dumps(record),flush=True)
summary=dict(models=int(sys.argv[1])if len(sys.argv)>1 else 200,runs=len(records),failures=sum(not r['valid']for r in records),corruptions_rejected=sum(r.get('corruptions_rejected',0)for r in records))
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary));raise SystemExit(bool(summary['failures']))
