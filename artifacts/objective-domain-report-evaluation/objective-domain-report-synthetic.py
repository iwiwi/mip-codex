import json,random,subprocess,os,sys,copy,math
from pathlib import Path
from fractions import Fraction
from importlib.util import spec_from_file_location,module_from_spec
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
root=Path.cwd();frozen=root/'artifacts/objective-domain-report-v1-diagnostic';binary=frozen/'bin/mip-codex';spec=spec_from_file_location('domain_checker',str(frozen/'scripts/verify_cover_cuts.py'));mod=module_from_spec(spec);spec.loader.exec_module(mod);out=root/'artifacts/objective-domain-report-synthetic';out.mkdir();rng=random.Random(2026111602);os.sched_setaffinity(0,{40});records=[];tampered=0
for case in range(200):
 d=out/f'{case:03}';d.mkdir();n=rng.randint(1,7);maximize=bool(case%2);offset=rng.randint(-5,5);cost=[rng.randint(-5,5)for _ in range(n)]
 if case%10==0:cost=[0]*n
 if case%11==0:cost[0]=0.125
 bounds=[(rng.randint(-3,0),rng.randint(1,5))for _ in range(n)]
 if case%3==0:
  j=rng.randrange(n);lo,up=bounds[j];bounds[j]=(None,up)if case%2 else(lo,None)
 text='NAME domain\nOBJSENSE\n'+(' MAX\n'if maximize else' MIN\n')+'ROWS\n N  obj\nCOLUMNS\n'
 for j,c in enumerate(cost):text+=f'    x{j:07}  {"obj":<8}  {c:>12}\n'
 text+=f'RHS\n    {"rhs":<8}  {"obj":<8}  {-offset:>12}\nBOUNDS\n'
 for j,(lo,up)in enumerate(bounds):
  text+=f' LI {"bounds":<8}  x{j:07}  {lo if lo is not None else 0:>12}\n'
  if lo is None:text+=f' MI {"bounds":<8}  x{j:07}\n'
  text+=(f' UP {"bounds":<8}  x{j:07}  {up:>12}\n'if up is not None else f' PL {"bounds":<8}  x{j:07}\n')
 model=d/'model.mps';model.write_text(text+'ENDATA\n');env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_OBJECTIVE_DOMAIN_REPORT='1',MIP_BINARY_GRAPH='0',MIP_THREE_ROW_CUTS='0',MIP_MIR_CUTS='0',MIP_CLIQUE_CUTS='0',MIP_AGGREGATE_CUTS='0',MIP_COVER_CUTS='0',MIP_ROUNDING_CUTS='0');p=subprocess.run([str(binary),'solve',str(model),'1','0'],env=env,capture_output=True,text=True,timeout=10);(d/'stdout.json').write_text(p.stdout);(d/'stderr.log').write_text(p.stderr);assert p.returncode==0;r=json.loads(p.stdout);assert r['status']=='node_limit' and r['lp_solves']==0
 sign=-1 if maximize else 1;expected=Fraction(sign*offset);supported=any(cost)and all(Fraction(c).denominator==1 for c in cost)
 for c,(lo,up)in zip(cost,bounds):
  c*=sign
  if not c:continue
  b=lo if c>0 else up
  if b is None:supported=False;break
  expected+=Fraction(c)*b
 expected=sign*expected if supported else None;actual=r.get('objective_domain_bound');assert (actual is None)==(expected is None),(case,actual,expected)
 if expected is not None:
  assert Fraction(str(actual))==expected and Fraction(str(r['dual_bound']))==expected
 check=mod.validate_cut_proofs(model,r);assert check['valid']is True,(case,check)
 if actual is not None:
  bad=[];v=copy.deepcopy(r);v['objective_domain_bound']+=1;bad.append(v);v=copy.deepcopy(r);v['objective_grid']=None;bad.append(v);v=copy.deepcopy(r);v['dual_bound']=actual-sign;bad.append(v);v=copy.deepcopy(r);v['objective_grid']['step']+=1;bad.append(v)
  for altered in bad:assert mod.validate_cut_proofs(model,altered)['valid']is False;tampered+=1
 records.append(dict(case=case,maximize=maximize,claimed=actual,independent_box_bound=str(expected)if expected is not None else None,validation=check,model_sha256=sha256(model)));atomic_json(out/'records.json',records)
atomic_json(out/'exit.json',dict(status='complete',cases=len(records),valid=len(records),tamper_rejected=tampered,with_report=sum(r['claimed']is not None for r in records)));print(json.load(open(out/'exit.json')))
