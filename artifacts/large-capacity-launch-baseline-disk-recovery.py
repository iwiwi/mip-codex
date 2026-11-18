from pathlib import Path
import os,sys,json,subprocess,time
root=Path.cwd()
assert root==Path('/home/ubuntu/mip-codex')
os.sched_setaffinity(0,{43})
sys.path.insert(0,'scripts')
from benchmark_common import sha256,atomic_json,same_process,utc
original=root/'artifacts/runs/baseline-v2-20261115'
receipt=json.load(open('artifacts/large-capacity-baseline-disk-recovery-receipt.json'))
recovery=Path(receipt['directory'])
assert sha256(original/'config.json')==receipt['original_config_sha256']
assert sha256(recovery/'config.json')==receipt['config_sha256']
config=json.load(open(original/'config.json'))
deadline=time.monotonic()+21600
while True:
    assert time.monotonic()<deadline,'original baseline not terminal after6hours'
    terminal=original/'exit.json'
    ready=terminal.exists()and json.load(open(terminal)).get('cases')==720
    ready=ready and len(list((original/'cases').glob('*/summary.json')))==720
    identity=original/'coordinator-identity.json'
    ready=ready and not same_process(json.load(open(identity)))
    if ready:
        for job in config['jobs']:
            group=Path('/sys/fs/cgroup/mip-codex-bench')/config['id']/job['id']
            if (group/'cgroup.procs').read_text().strip():
                ready=False
                break
    if ready:
        break
    time.sleep(2)
assert not(recovery/'coordinator-identity.json').exists()
atomic_json(root/'artifacts/large-capacity-baseline-disk-recovery-launch.json',dict(launched_utc=utc(),original720_terminal=True,original_all_cgroups_empty=True,original_coordinator_terminal=True,config_sha256=sha256(recovery/'config.json'),original_results_sha256=sha256(original/'results.jsonl'),policy='one timestamp-overlap GLPK rerun; keep original results and report all failures; no performance-based selection'))
cmd=[str(root/'.venv/bin/python'),str(recovery/'snapshot/scripts/benchmark.py'),'run',str(recovery)]
with(recovery/'coordinator.log').open('w')as log:
    run=subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT)
atomic_json(recovery/'coordinator-exit-code.json',dict(exit_code=run.returncode,finished_utc=utc()))
run.check_returncode()
print('recovery terminal; original720 retained; recovery cases1',flush=True)
