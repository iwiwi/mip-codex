from pathlib import Path
import json,sys,subprocess,os,shutil,re
from fractions import Fraction as F
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
from verify_cover_cuts import validate_cut_proofs,exact_model
from validate_solution import validate
root=Path.cwd();out=root/'artifacts/integer-bound-safety-final-primary-corners';out.mkdir();binary=root/'target/release/mip-codex';records=[]
cases=[]
for p in sorted((root/'artifacts/integer-bound-safety-scaled-pilot').glob('*/model.mps')):cases.append((p.parent.name,p.read_text(),None))
cases.append(('pure-lp',(root/'artifacts/integer-bound-safety-continuous-pilot/model.mps').read_text(),None))
for name,lo,hi,cost,opt in [('lower','1.00000000000000001','3',1,2),('upper','-3','-1.00000000000000001',-1,2),('empty','0.1','0.9',1,'infeasible'),('positive','1e-16','2',1,1),('negative','-2','-1e-16',-1,1)]:
 text=f'NAME var\nROWS\n N obj\nCOLUMNS\n x obj {cost}\nRHS\n rhs obj 0\nBOUNDS\n LI b x {lo}\n UI b x {hi}\nENDATA\n';cases.append((name,text,opt))
cases.append(('fixed','NAME fixed\nROWS\n N obj\nCOLUMNS\n mark0 \'MARKER\' \'INTORG\'\n x obj 1\n mark1 \'MARKER\' \'INTEND\'\nRHS\n rhs obj 0\nBOUNDS\n FX b x 1.00000000000000001\nENDATA\n','infeasible'))
atomic_json(out/'experiment.json',dict(cpu=42,seconds=2,binary_sha256=sha256(binary),build_info=json.loads(subprocess.check_output([str(binary),'build-info'])),purpose='scaled integer and pure LP false optimal withdrawal plus lexical variable bounds, fixed integer domain'))
for name,text,opt in cases:
 d=out/name;d.mkdir();model=d/'model.mps';model.write_text(text);env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};p=subprocess.run(['taskset','-c','42',str(binary),'solve',str(model),'2'],env=env,capture_output=True,text=True,timeout=62);(d/'result.json').write_text(p.stdout);(d/'stderr.log').write_text(p.stderr);assert p.returncode==0,(name,p.stderr);r=json.loads(p.stdout);proof=validate_cut_proofs(model,r);atomic_json(d/'proof-validation.json',proof);assert proof['valid']is True,(name,proof)
 if opt is None:assert r['status']=='numerical_error'and r['optimal_original_conflict']and r['dual_bound']is None and r['gap']is None,(name,r['status'])
 elif opt=='infeasible':assert r['status']=='infeasible'and r['objective']is None,(name,r['status'])
 else:assert r['status']=='optimal'and r['objective']==opt,(name,r['status'],r['objective'])
 if r['values']is not None:
  names,_,_=exact_model(model);assert set(names)==set(r['values'])
  sol=d/'solution.sol';sol.write_text('=obj= '+str(r['objective'])+'\n'+''.join(n+' '+str(r['values'][n])+'\n'for n in names))
  checker=root/'third_party/miplib-checker/bin/solchecker-absolute';command=[str(checker),str(model),str(sol),'1e-6','1e-6'];q=subprocess.run(command,capture_output=True,text=True,timeout=300);log=q.stdout+q.stderr;(d/'absolute-checker.log').write_text(log);assert q.returncode==0 and 'Read MPS: 1'in log and 'Read SOL: 1'in log and re.search(r'Check SOL: Integrality 1 Constraints 1 Objective 1',log),(name,log)
  exact={label:F(re.search(r'^Exact '+label+r': (\S+)$',log,re.M)[1])for label in ['objective','bound violation','row violation','integer violation','objective violation']}
  assert all(exact[label]<=F(1,10**6)for label in ['bound violation','row violation','integer violation']);assert exact['objective violation']<=F(1,10**8)*(1+abs(exact['objective']))
  atomic_json(d/'validation.json',dict(valid=True,arithmetic='GMP rational direct absolute checker; optional HiGHS diagnostic reader not required for 1e16 coefficients',command=command,checker_sha256=sha256(checker),exact={k:str(v)for k,v in exact.items()}))
 records.append(dict(name=name,status=r['status'],objective=r['objective'],expected=opt,withdrawn=r['optimal_original_conflict']))
atomic_json(out/'summary.json',dict(outputs=len(records),all_correct=True,withdrawn=sum(r['withdrawn']for r in records),points=sum(r['objective']is not None for r in records),records=records));atomic_json(out/'exit.json',dict(status='complete'));print(json.dumps(records))
