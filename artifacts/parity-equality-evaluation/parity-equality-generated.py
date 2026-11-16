import itertools,json,os,random,subprocess,sys,copy
from pathlib import Path
from decimal import Decimal
from fractions import Fraction as F
sys.path.insert(0,'/home/ubuntu/mip-dual-restoration/scripts');from verify_parity import validate_parity_proof
from validate_solution import validate
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{0});rng=random.Random(2026111612);out=Path('artifacts/parity-equality-generated');out.mkdir(exist_ok=False);binary=Path('artifacts/parity-equality-final-diagnostic/bin/mip-codex').resolve();records=[];tampered=0
for case in range(200):
 n=rng.randint(1,6);m=rng.randint(1,8);a=[[rng.randint(-3,3)for _ in range(n)]for _ in range(m)];planted=[rng.randrange(2)for _ in range(n)];rhs=[sum(c*x for c,x in zip(row,planted))for row in a]
 if case%4==0:rhs[0]+=1
 costs=[rng.randint(-20,20)for _ in range(n+m)];maximize=case%2==1;offset=rng.randint(-10,10);d=out/f'{case:03}';d.mkdir();model=d/'model.mps';text=f'NAME parity{case}\nOBJSENSE\n '+('MAX'if maximize else'MIN')+'\nROWS\n N obj\n'+''.join(f' E r{i}\n'for i in range(m))+'COLUMNS\n'
 for j in range(n):text+=f' x{j} obj {costs[j]/10}\n'+''.join(f' x{j} r{i} {a[i][j]}\n'for i in range(m))
 for i in range(m):text+=f' z{i} obj {costs[n+i]/10} r{i} -2\n'
 text+='RHS\n'+f' b obj {-offset/10}\n'+''.join(f' b r{i} {v}\n'for i,v in enumerate(rhs))+'BOUNDS\n'+''.join(f' BV bd x{j}\n'for j in range(n))+''.join(f' LI bd z{i} -20\n UI bd z{i} 20\n'for i in range(m))+'ENDATA\n';model.write_text(text);solutions=[]
 for bits in itertools.product([0,1],repeat=n):
  residual=[b-sum(c*x for c,x in zip(row,bits))for row,b in zip(a,rhs)]
  if any(r%2 or not -20<=-r//2<=20 for r in residual):continue
  p=[*bits,*[-r//2 for r in residual]];objective=F(offset,10)+sum((F(c,10)*x for c,x in zip(costs,p)),F(0));solutions.append((objective,p))
 with (d/'result.json').open('w')as f,(d/'stderr.log').open('w')as e:proc=subprocess.run([str(binary),'solve',str(model.resolve()),'5'],stdout=f,stderr=e,timeout=15)
 assert proc.returncode==0;v=json.load(open(d/'result.json'),parse_float=Decimal);expected=(max if maximize else min)((x[0]for x in solutions),default=None)
 if expected is None:assert v['status']=='infeasible',v
 else:
  assert v['status']=='optimal'and abs(F(v['objective'])-expected)<=F(1,10**8)*(1+abs(expected)),(case,expected,v)
  validation=validate(model,v);assert validation['valid']is True;atomic_json(d/'point-audit.json',validation)
 proof=validate_parity_proof(model,v);assert proof['valid'];atomic_json(d/'proof-audit.json',proof)
 if proof.get('applicable'):
  assert len(solutions)==1
  for field in ['rank','binary_variables','auxiliary_variables']:
   bad=copy.deepcopy(v)
   if field=='rank':bad['parity_proof'][field]+=1
   else:bad['parity_proof'][field]=bad['parity_proof'][field]+[-1]
   assert validate_parity_proof(model,bad)['valid']is False;tampered+=1
 records.append(dict(case=case,status=v['status'],parity=proof.get('applicable'),solutions=len(solutions),input_sha256=sha256(model),output_sha256=sha256(d/'result.json')))
 atomic_json(out/'progress.json',dict(completed=len(records),parity=sum(r['parity']for r in records),tampered_rejected=tampered))
atomic_json(out/'records.json',records);atomic_json(out/'summary.json',dict(cases=len(records),parity=sum(r['parity']for r in records),tampered_rejected=tampered,valid=True));print(json.load(open(out/'summary.json')))
