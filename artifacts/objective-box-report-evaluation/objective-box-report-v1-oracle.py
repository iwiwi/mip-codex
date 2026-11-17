from pathlib import Path
from fractions import Fraction as F
import os,sys,json,subprocess,random,struct,copy,math
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from importlib.util import spec_from_file_location,module_from_spec
root=Path.cwd();out=root/'artifacts/objective-box-report-v1-oracle';out.mkdir();os.sched_setaffinity(0,{37});frozen=root/'artifacts/objective-box-report-v1-diagnostic';binary=frozen/'bin/mip-codex';s=spec_from_file_location('box_checker',frozen/'scripts/verify_cover_cuts.py');c=module_from_spec(s);s.loader.exec_module(c);rng=random.Random(2026111705);records=[];rejected=0
atomic_json(out/'experiment.json',dict(cpu=37,seed=2026111705,binary_sha256=sha256(binary),build_info=json.loads(subprocess.check_output([str(binary),'build-info'])),purpose='100 independent decimal boxes, both objective senses; exact Fraction objective and certificate tampering'))
for i in range(100):
 costs=[F(rng.randint(-99,99),10)for _ in range(5)];lower=[rng.randint(-9,0)for _ in costs];upper=[rng.randint(1,9)for _ in costs];offset=F(rng.randint(-99,99),10)
 for maximize in [False,True]:
  d=out/f'{i}-{int(maximize)}';d.mkdir();p=d/'model.mps';text='NAME box\nOBJSENSE\n '+('MAX'if maximize else'MIN')+'\nROWS\n N obj\n L harmless\nCOLUMNS\n'+''.join(f' x{j} obj {float(a)} harmless 0\n'for j,a in enumerate(costs))+f'RHS\n rhs obj {float(-offset)} harmless 1\nBOUNDS\n'+''.join(f' LO b x{j} {l}\n UP b x{j} {u}\n'for j,(l,u)in enumerate(zip(lower,upper)))+'ENDATA\n';p.write_text(text)
  env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_OBJECTIVE_BOX_REPORT='1',MIP_OBJECTIVE_DOMAIN_REPORT='0');run=subprocess.run([str(binary),'solve',str(p),'2'],env=env,capture_output=True,text=True,timeout=62);(d/'stderr.log').write_text(run.stderr);(d/'result.json').write_text(run.stdout);assert run.returncode==0;v=json.loads(run.stdout);proof=c.validate_cut_proofs(p,v);atomic_json(d/'proof-validation.json',proof);assert proof['valid']is True and proof['objective_box']['valid'];sign=-1 if maximize else 1;optimum=offset+sum(a*(u if (a>0)==maximize else l)for a,l,u in zip(costs,lower,upper));assert v['status']=='optimal'and abs(F(str(v['objective']))-optimum)<=F(1,1000000)
  for bits in [struct.pack('>d',float(sign*optimum)+1).hex(),'7ff0000000000000','fff0000000000000','7ff8000000000000','bad']:
   bad={'objective_box_proof':{'normalized_bound_bits':bits}};assert c.validate_cut_proofs(p,bad)['valid']is False;(d/f'tamper-{rejected%5}.json').write_text(json.dumps(bad));rejected+=1
  records.append(dict(case=i,maximize=maximize,exact_optimum=str(optimum),valid=True));atomic_json(out/'progress.json',dict(outputs=len(records),valid=len(records),tamper_rejected=rejected))
atomic_json(out/'records.json',records);atomic_json(out/'exit.json',dict(status='complete',outputs=len(records),valid=len(records),tamper_rejected=rejected))
