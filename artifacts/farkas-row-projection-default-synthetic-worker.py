from pathlib import Path
import sys,os,resource,json,subprocess,time,signal
root=Path.cwd();label,profile,flag,cpu,count=sys.argv[1:6];cpu=int(cpu);count=int(count);os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);sys.path.insert(0,'scripts');from benchmark_common import sha256,atomic_json
out=root/'artifacts/farkas-row-projection-default-synthetic'/label;out.mkdir();binary=root/'target/release/mip-codex';expected=json.load(open('artifacts/farkas-row-projection-default-origin.json'))['binary_sha256'];assert sha256(binary)==expected;env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_LP_FARKAS_ROW_PROJECT=flag,MIP_SYNTHETIC_PROFILE=profile,MIP_SYNTHETIC_DIR=str(out),MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute',OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');
if flag=='default':env.pop('MIP_LP_FARKAS_ROW_PROJECT',None)
cmd=[str(root/'.venv/bin/python'),'artifacts/farkas-row-projection-default-check-synthetic.py',str(count)];atomic_json(out/'experiment.json',dict(command=cmd,cpu=cpu,count=count,profile=profile,flag=flag,binary_sha256=expected,driver_sha256=sha256(root/'artifacts/farkas-row-projection-default-check-synthetic.py'),memory_bytes=16*1024**3,watchdog_seconds=1800));start=time.monotonic();watchdog=False
with(out/'stdout.log').open('w')as so,(out/'stderr.log').open('w')as se:
 p=subprocess.Popen(cmd,stdout=so,stderr=se,env=env,start_new_session=True)
 try:p.wait(timeout=1800)
 except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
atomic_json(out/'execution.json',dict(exit_code=p.returncode,watchdog=watchdog,wall_seconds=time.monotonic()-start));rows=[]
for line in(out/'stdout.log').read_text().splitlines():
 try:rows.append(json.loads(line))
 except json.JSONDecodeError:pass
final=next((r for r in reversed(rows)if 'cases'in r and 'failures'in r),None);failures=[]
for r in rows:
 if r.get('status')=='fail':
  path=out/f'{r["case"]:04d}'/'result.json';v=json.load(open(path))if path.exists()else{};failures.append(dict(case=r['case'],solver_status=v.get('status'),objective=v.get('objective'),dual_bound=v.get('dual_bound'),lp_projections=v.get('lp_farkas_row_projections')))
values=[json.load(open(p))for p in out.glob('*/result.json')];summary=dict(status='complete'if final is not None and not watchdog else'failed',exit_code=p.returncode,oracle_summary=final,failures=failures,outputs=len(values),projection_trials=sum(v.get('lp_farkas_row_projection_trials')or 0 for v in values),projections=sum(v.get('lp_farkas_row_projections')or 0 for v in values),binary_sha256_unchanged=sha256(binary)==expected);atomic_json(out/'summary.json',summary);print(json.dumps(summary),flush=True)
