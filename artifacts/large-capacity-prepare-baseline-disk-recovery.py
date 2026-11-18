from pathlib import Path
import os,sys,json,datetime,shutil,subprocess,collections
root=Path.cwd()
assert root==Path('/home/ubuntu/mip-codex')
os.sched_setaffinity(0,{43})
sys.path.insert(0,'scripts')
from benchmark_common import sha256,atomic_json,utc
original=root/'artifacts/runs/baseline-v2-20261115'
assert sha256(original/'config.json')=='39551a3b530d8c4e1ed4eb4069c361285e929a84a74d73966bb4c574b43c125c'
config=json.load(open(original/'config.json'))
start='2026-11-18T06:32:00Z'
end='2026-11-18T06:34:00Z'
affected=[]
windows=[]
for job in config['jobs']:
    case=original/'cases'/job['id']
    launch=json.load(open(case/'solver-launch.json'))
    summary=case/'summary.json'
    finished=json.load(open(summary)).get('finished_utc')if summary.exists()else None
    if launch['started_utc']<=end and (finished is None or finished>=start):
        affected.append(job)
        windows.append(dict(job_id=job['id'],solver=job['solver'],started_utc=launch['started_utc'],finished_utc=finished,selection='only execution/validation time overlap; no result metrics read for selection'))
assert len(affected)==1,dict(count=len(affected),solvers=dict(collections.Counter(j['solver']for j in affected)))
assert [j['solver']for j in affected]==['glpk']
for name,h in config['snapshot_sha256'].items():
    assert sha256(original/'snapshot'/name)==h
for name,h in config['library_sha256'].items():
    assert sha256(name)==h
out=root/'artifacts/runs/baseline-v2-disk-recovery-20261118'
out.mkdir()
snapshot=out/'snapshot'
for name,h in config['snapshot_sha256'].items():
    target=snapshot/name
    target.parent.mkdir(parents=True,exist_ok=True)
    shutil.copy2(original/'snapshot'/name,target)
    assert sha256(target)==h
config.update(id=out.name,created_utc=utc(),jobs=affected,disk_recovery=dict(original_directory=str(original),original_config_sha256=sha256(original/'config.json'),conservative_incident_window=[start,end],selection='all jobs overlapping incident based solely on execution/validation timestamps',original_results_retained=True,policy='keep all720 original cases and failures; rerun1 externally disk-contaminated case under identical frozen solver/resource settings; report originals and replacements explicitly; not selected by performance',windows=windows))
atomic_json(out/'config.json',config)
subprocess.run(['sudo','-n','/usr/bin/python3',str(snapshot/'scripts/cgroup_control.py'),'setup',str(out/'config.json')],check=True,capture_output=True,text=True)
for job in affected:
    group=Path('/sys/fs/cgroup/mip-codex-bench')/out.name/job['id']
    assert not(group/'cgroup.procs').read_text().strip()
receipt=dict(created_utc=utc(),directory=str(out),config_sha256=sha256(out/'config.json'),original_config_sha256=sha256(original/'config.json'),snapshot_files=len(config['snapshot_sha256']),libraries=len(config['library_sha256']),jobs=len(affected),solvers=dict(collections.Counter(j['solver']for j in affected)),selected_without_performance_metrics=True,original_results_retained=True,launched=False,run_gate='original720 terminal and all original cgroups empty, old coordinator terminal, global coordinator lock free',windows=windows)
atomic_json(root/'artifacts/large-capacity-baseline-disk-recovery-receipt.json',receipt)
print(json.dumps({k:v for k,v in receipt.items()if k!='windows'}),flush=True)
