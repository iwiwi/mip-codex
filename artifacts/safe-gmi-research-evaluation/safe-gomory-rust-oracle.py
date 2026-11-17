from pathlib import Path
import json,subprocess,shutil,hashlib,os,math
root=Path.cwd();repo=Path('/home/ubuntu/mip-dual-restoration');out=root/'artifacts/safe-gomory-rust-oracle';out.mkdir();os.sched_setaffinity(0,{42});binary=repo/'target/release/examples/safe_gomory_oracle';shutil.copy2(binary,out/'helper');shutil.copy2(repo/'examples/safe_gomory_oracle.rs',out/'helper.rs');info=json.loads(subprocess.check_output([str(out/'helper'),'build-info']));cases=[json.loads(x)for x in(root/'artifacts/safe-gomory-algebra-oracle/cases.jsonl').read_text().splitlines()];inputs=[]
for v in cases:
 n=len(v['lower']);m=len(v['b']);lines=['NAME gomory','ROWS',' N obj']+[' L r'+str(i)for i in range(m)]+['COLUMNS'," MARK0 'MARKER' 'INTORG'"]
 for j in range(n):
  lines.append(f' x{j} obj 0')
  for i,row in enumerate(v['a']):
   if row[j]:lines.append(f' x{j} r{i} {row[j]}')
 lines += [" MARK1 'MARKER' 'INTEND'",'RHS']+[f' rhs r{i} {b}'for i,b in enumerate(v['b'])]+['BOUNDS']
 for j,(lo,up)in enumerate(zip(v['lower'],v['upper'])):lines.extend([f' LI b x{j} {lo}',f' UI b x{j} {up}'])
 lines.append('ENDATA');inputs.append(dict(model='\n'.join(lines)+'\n',proof=dict(terms=[dict(row=i,upper=True,weight=w)for i,w in enumerate(v['weights'])if w],pivot=v['pivot'],upper_bounds=[j for j,d in enumerate(v['directions'])if d==-1])))
(out/'input.json').write_text(json.dumps(inputs)+'\n')
with(out/'input.json').open()as inp,(out/'rust.jsonl').open('w')as stream,(out/'stderr.log').open('w')as err:p=subprocess.run([str(out/'helper')],stdin=inp,stdout=stream,stderr=err,timeout=60)
assert p.returncode==0;rows=[json.loads(x)for x in(out/'rust.jsonl').read_text().splitlines()];assert len(rows)==len(cases);derived=0
for v,row in zip(cases,rows):
 if v['rhs']==0 and all(c==0 for c in v['coefficients']):assert row is None;continue
 if (sum(w*b for w,b in zip(v['weights'],v['b']))-sum(sum(w*a[j]for w,a in zip(v['weights'],v['a']))*(lo if d==1 else up)for j,(lo,up,d)in enumerate(zip(v['lower'],v['upper'],v['directions']))))%v['denominator']==0:assert row is None;continue
 if not any(v['coefficients']):assert row is None;continue
 assert row is not None;common=math.gcd(v['rhs'],*v['coefficients']);expected=[[j,float(c//common)]for j,c in enumerate(v['coefficients'])if c];assert row['coefficients']==expected,(v['case'],row,expected);assert row['lower']==v['rhs']//common and row['upper']is None;derived+=1
summary=dict(cases=len(cases),derived=derived,failures=0,build_info=info,binary_sha256=hashlib.sha256((out/'helper').read_bytes()).hexdigest(),input_sha256=hashlib.sha256((out/'input.json').read_bytes()).hexdigest());(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
