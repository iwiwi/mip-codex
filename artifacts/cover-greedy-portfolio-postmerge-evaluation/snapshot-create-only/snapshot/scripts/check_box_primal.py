#!/usr/bin/env python3
"""LP/BBを0節点に制限し、箱からの主解だけを全列挙とGMPで検査する。"""
import itertools,json,os,random,subprocess,sys
from decimal import Decimal
from pathlib import Path
from benchmark_common import require_solver_origin,sha256
from validate_solution import validate
root=Path(__file__).resolve().parents[1];binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary)
out=root/os.environ.get('MIP_BOX_DIR','artifacts/box-enumeration');out.mkdir(parents=True,exist_ok=True)
(out/'experiment.json').write_text(json.dumps(dict(build_info=info,binary_sha256=sha256(binary),seed=2026111508),indent=2)+'\n')
rng=random.Random(2026111508);records=[]
for case in range(int(sys.argv[1])if len(sys.argv)>1 else 200):
 n=rng.randint(3,9);cover=case%2==1;maximize=case%3==0;cost=[rng.randint(-5,5)for _ in range(n)];offset=rng.randint(-7,7)
 edges=[(a,b)for a in range(n)for b in range(a+1,n)if rng.random()<.5]
 columns=[]
 for j in range(n):
  columns.append(f' x{j} obj {cost[j]}');columns.extend(f' x{j} r{k} 1'for k,e in enumerate(edges)if j in e)
 model=out/f'{case:04d}.mps';model.write_text('\n'.join(['NAME box','OBJSENSE',' MAX'if maximize else' MIN','ROWS',' N obj',*[f' {"G"if cover else"L"} r{k}'for k in range(len(edges))],'COLUMNS',*columns,'RHS',f' rhs obj {-offset}',*[f' rhs r{k} 1'for k in range(len(edges))],'BOUNDS',*[f' BV b x{j}'for j in range(n)],'ENDATA'])+'\n')
 objectives=[offset+sum(c*x for c,x in zip(cost,point))for point in itertools.product([0,1],repeat=n)if all((point[a]+point[b]>=1)if cover else(point[a]+point[b]<=1)for a,b in edges)]
 optimum=(max if maximize else min)(objectives);baseline=offset+(sum(cost)if cover else 0)
 env=dict(os.environ,MIP_BINARY_GRAPH='0',MIP_BOX_PRIMAL='1',MIP_COVER_CUTS='0',MIP_ROUNDING_CUTS='0',MIP_STRONG_BRANCHING='0',MIP_PRIMAL_REPAIR='0',MIP_LP_DIVING='0',MIP_PRIMAL_PROBE='0',MIP_RINS='0')
 with (out/f'{case:04d}.json').open('w')as stream,(out/f'{case:04d}.err').open('w')as err:p=subprocess.run([str(binary),'solve',str(model),'5','0'],env=env,stdout=stream,stderr=err,timeout=15)
 r=json.loads((out/f'{case:04d}.json').read_text(),parse_float=Decimal)
 try:
  assert p.returncode==0 and r['status']=='node_limit' and r['nodes']==0 and r['lp_solves']==0 and r['box_primal_enabled'],r
  check=validate(model,r);assert check['valid']is True,check
  actual=float(r['objective'])
  if maximize:assert baseline<=actual<=optimum,(baseline,actual,optimum)
  else:assert optimum<=actual<=baseline,(optimum,actual,baseline)
  record=dict(case=case,valid=True,objective=actual,optimum=optimum,baseline=baseline,improved=actual!=baseline,optimal_point=actual==optimum,validation=check)
 except Exception as error:record=dict(case=case,valid=False,error=str(error))
 records.append(record);print(json.dumps(record),flush=True)
summary=dict(cases=len(records),failures=sum(not r['valid']for r in records),improved=sum(r.get('improved',False)for r in records),optimal_points=sum(r.get('optimal_point',False)for r in records))
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary));raise SystemExit(bool(summary['failures']))
