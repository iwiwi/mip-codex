from pathlib import Path
import os,sys,json,random,itertools,subprocess
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
root=Path.cwd();out=root/'artifacts/component-primal-synthetic';out.mkdir();rng=random.Random(2026111601);binary=root/'artifacts/component-primal-v1-diagnostic/bin/component-primal';records=[];os.sched_setaffinity(0,{1})
for case in range(20):
 d=out/f'{case:03}';d.mkdir();cost=[rng.randint(-4,4)for _ in range(10)];maximize=bool(case%2);seed=[rng.randrange(2)for _ in range(8)]+[0,1];rows=[]
 for block in range(2):
  for _ in range(3):
   a=[0]*10
   for j in range(block*4,block*4+4):a[j]=rng.randint(-3,3)
   if case%3==0:a[9]=rng.randint(-2,2)
   rhs=sum(x*y for x,y in zip(a,seed))+rng.randint(0,2);rows.append((a,rhs))
 text='NAME separated\n'+('OBJSENSE\n MAX\n'if maximize else'OBJSENSE\n MIN\n')+'ROWS\n N obj\n'+''.join(f' L r{i}\n'for i in range(len(rows)))+'COLUMNS\n'
 for j in range(10):
  text+=f' x{j} obj {cost[j]}\n'
  for i,(a,rhs)in enumerate(rows):
   if a[j]:text+=f' x{j} r{i} {a[j]}\n'
 text+='RHS\n'+''.join(f' rhs r{i} {rhs}\n'for i,(a,rhs)in enumerate(rows))+'BOUNDS\n'+''.join(f' BV b x{j}\n'for j in range(8))+' LI b x8 -2\n UI b x8 3\n LI b x9 1\n UI b x9 1\nENDATA\n';model=d/'model.mps';fixed=[];section=None
 for line in text.splitlines():
  fields=line.split()
  if not line.startswith(' '):section=fields[0];fixed.append(line);continue
  if section=='ROWS':line=f' {fields[0]:<2} {fields[1]}'
  elif section in ['COLUMNS','RHS']:line=f'    {fields[0]:<8}  {fields[1]:<8}  {fields[2]:>12}'
  elif section=='BOUNDS':line=f' {fields[0]:<2} {fields[1]:<8}  {fields[2]:<8}'+(f'  {fields[3]:>12}'if len(fields)>3 else'')
  fixed.append(line)
 model.write_text('\n'.join(fixed)+'\n');expected=[]
 for x in itertools.product(*([[0,1]]*8+[list(range(-2,4)),[1]])):
  if all(sum(a[j]*x[j]for j in range(10))<=rhs for a,rhs in rows):expected.append(sum(c*y for c,y in zip(cost,x)))
 assert expected;opt=(max if maximize else min)(expected)
 p=subprocess.run([str(binary),str(model),'5',str(d/'components')],capture_output=True,text=True,timeout=15);(d/'stdout.log').write_text(p.stdout);(d/'stderr.log').write_text(p.stderr);assert p.returncode==0;pnt=d/'components/result.json';v=json.load(open(pnt));assert v['status']=='feasible_component_diagnostic'and v['dual_bound']is None;assert abs(v['objective']-opt)<1e-7,(case,opt,v['objective']);audit=d/'audit';audit.mkdir();cmd=[sys.executable,'scripts/benchmark_validate.py',str(model),str(pnt),str(root/'third_party/miplib-checker/bin/solchecker'),str(audit)]
 with(audit/'log').open('w')as f:r=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,timeout=60)
 assert r.returncode==0;assert all(json.load(open(audit/(k+'-validation.json')))['valid']is True for k in ['strict','official']);records.append(dict(case=case,maximize=maximize,objective=opt,status=v['status'],model_sha256=sha256(model)));atomic_json(out/'records.json',records)
atomic_json(out/'exit.json',dict(status='complete',cases=len(records),strict_valid=len(records),enumeration_agreed=len(records)));print(json.load(open(out/'exit.json')))
