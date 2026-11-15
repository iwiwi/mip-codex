#!/usr/bin/env python3
"""固定snapshotとcgroupを使う再開可能なMILP比較。create/run/status。"""
import argparse
from collections import deque
import fcntl
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
from benchmark_common import cgroup_stats, atomic_json, memory_events, process, same_process, sha256, utc

REPO = Path(__file__).resolve().parents[1]


def populated(config, job):
    path = Path('/sys/fs/cgroup/mip-codex-bench')/config['id']/job['id']/'cgroup.events'
    return path.exists() and 'populated 1' in path.read_text()


def create(args):
    destination = Path(args.directory).resolve()
    if not re.fullmatch('[A-Za-z0-9_-]+', destination.name):
        raise ValueError('experiment directory basename must contain letters/numbers/_/- only')
    if args.seconds < 0 or args.seconds > 604800:
        raise ValueError('invalid time limit')
    cores = [int(value) for value in args.cores.split(',')]
    if not cores or len(set(cores)) != len(cores) or not set(cores) <= os.sched_getaffinity(0):
        raise ValueError('invalid CPU allocation')
    solvers = args.solvers.split(',')
    if len(set(solvers)) != len(solvers) or any(s not in ('rust', 'rust-dense', 'rust-revised', 'highs', 'scip', 'glpk') for s in solvers):
        raise ValueError('invalid solvers')
    source_paths = ['src', 'scripts', 'tools', 'third_party', 'Cargo.toml', 'Cargo.lock']
    dirty = bool(subprocess.check_output(['git', 'status', '--porcelain', '--', *source_paths], cwd=REPO))
    if dirty and not args.allow_dirty:
        raise ValueError('commit source changes before creating an evaluation snapshot')
    manifest_path = Path(args.manifest).resolve()
    manifest = json.loads(manifest_path.read_text())
    records = manifest['instances']
    if args.names:
        selected = set(args.names)
        if not selected <= {r['name'] for r in records}:
            raise ValueError('unknown selected instance')
        records = [r for r in records if r['name'] in selected]
    if not records or len({r['name'] for r in records}) != len(records):
        raise ValueError('empty or duplicate instances')
    instances = []
    for r in records:
        p = (REPO/r['path']).resolve()
        if sha256(p) != r['sha256']:
            raise ValueError('input hash mismatch '+r['name'])
        instances.append(dict(r, absolute_path=str(p)))
    destination.mkdir(parents=True, exist_ok=False)
    snapshot = destination/'snapshot'
    (snapshot/'scripts').mkdir(parents=True)
    (snapshot/'bin').mkdir()
    for path in (REPO/'scripts').glob('*.py'):
        shutil.copy2(path, snapshot/'scripts'/path.name)
    for path in (REPO/'src').glob('*.rs'):
        target = snapshot/'src'/path.name
        target.parent.mkdir(exist_ok=True)
        shutil.copy2(path, target)
    for path in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy2(REPO/path, snapshot/path)
    shutil.copytree(REPO/'tools/glpk-benchmark', snapshot/'tools/glpk-benchmark')
    shutil.copytree(REPO/'third_party/miplib-checker', snapshot/'third_party/miplib-checker', ignore=shutil.ignore_patterns('bin', 'obj', 'obj-absolute'))
    binaries = {'mip-codex': REPO/'target/release/mip-codex',
                'glpk-benchmark': REPO/'target/glpk-benchmark',
                'solchecker': REPO/'third_party/miplib-checker/bin/solchecker',
                'solchecker-absolute': REPO/'third_party/miplib-checker/bin/solchecker-absolute'}
    for name, path in binaries.items():
        shutil.copy2(path, snapshot/'bin'/name)
    python = str(REPO/'.venv/bin/python')
    libraries = {p.resolve() for p in (REPO/'.venv').glob('lib/python*/site-packages/**/*.so*') if p.is_file()}
    for binary in binaries.values():
        output = subprocess.check_output(['ldd', str(binary)], text=True)
        for path in re.findall(r'(/[^\s()]+)', output):
            if Path(path).is_file():
                libraries.add(Path(path).resolve())
    jobs = [dict(id=f'{i:03d}-{solver}', instance_index=i, solver=solver, cpu=cores[i % len(cores)])
            for i in range(len(instances)) for solver in solvers]
    config = dict(id=destination.name, created_utc=utc(), git_sha=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip(),
                  source_dirty=dirty, python=python, repository=str(REPO), uid=os.getuid(), gid=os.getgid(),
                  seconds=args.seconds, memory_bytes=16*1024**3, cores=cores, threads=1,
                  read_write_grace_seconds=60, validation_seconds=600,
                  manifest_sha256=sha256(manifest_path), instances=instances, jobs=jobs,
                  snapshot_sha256={str(p.relative_to(snapshot)): sha256(p) for p in sorted(snapshot.rglob('*')) if p.is_file()},
                  library_sha256={str(p): sha256(p) for p in sorted(libraries)},
                  memory_policy='per-case cgroup memory.max (aggregate) plus per-process RLIMIT_AS; swap disabled',
                  validation_policy='primary strict absolute 1e-6; official GMP relative 1e-6 supplementary')
    atomic_json(destination/'config.json', config)
    (destination/'python-packages.txt').write_bytes(subprocess.check_output([python, '-m', 'pip', 'freeze']))
    (destination/'machine.txt').write_bytes(subprocess.check_output(['uname', '-a'])+subprocess.check_output(['lscpu']))
    subprocess.run(['sudo', '-n', '/usr/bin/python3', str(snapshot/'scripts/cgroup_control.py'), 'setup', str(destination/'config.json')], check=True)
    print(destination/'config.json')


def verify_snapshot(root, config):
    for name, expected in config['snapshot_sha256'].items():
        if sha256(root/'snapshot'/name) != expected:
            raise ValueError('snapshot changed '+name)
    for name, expected in config['library_sha256'].items():
        if sha256(name) != expected:
            raise ValueError('installed dependency changed '+name)


def identity_alive(directory, name):
    path = directory/name
    return path.exists() and same_process(json.loads(path.read_text()))


def launch(root, config, job):
    directory = root/'cases'/job['id']
    directory.mkdir(parents=True, exist_ok=True)
    events_before = memory_events(config, job)
    command = ['sudo', '-n', '/usr/bin/python3', str(root/'snapshot/scripts/cgroup_control.py'), 'exec',
               config['id'], job['id'], str(job['cpu']), str(config['uid']), str(config['gid']), str(config['memory_bytes']),
               config['python'], str(root/'snapshot/scripts/benchmark_case.py'), str(root/'config.json'), job['id']]
    with (directory/'supervisor.log').open('a') as log:
        child = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    record = dict(process(child.pid), command=command, launched_utc=utc(), events_before=events_before)
    previous = directory/'supervisor-launch.json'
    if previous.exists():
        with (directory/'supervisor-history.jsonl').open('a') as stream:
            stream.write(json.dumps(json.loads(previous.read_text()))+'\n')
    atomic_json(previous, record)
    return child


def run(args):
    root = Path(args.directory).resolve()
    config = json.loads((root/'config.json').read_text())
    global_lock = Path(config['repository'])/'artifacts/runs/coordinator.lock'
    global_lock.parent.mkdir(parents=True, exist_ok=True)
    with global_lock.open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        verify_snapshot(root, config)
        atomic_json(root/'coordinator-identity.json', dict(process(os.getpid()), started_utc=utc()))
        queues = {core: deque(j for j in config['jobs'] if j['cpu'] == core) for core in config['cores']}
        active, children, recoveries = {}, {}, {}
        while True:
            completed = 0
            for core in config['cores']:
                queue = queues[core]
                while queue:
                    job = queue[0]
                    directory = root/'cases'/job['id']
                    summary = directory/'summary.json'
                    if summary.exists() and not populated(config, job):
                        completed += 1
                        queue.popleft()
                        active.pop(core, None)
                        child = children.pop(core, None)
                        if child is not None:
                            child.wait()
                        continue
                    active[core] = job
                    if core in children:
                        children[core].poll()
                    alive = identity_alive(directory, 'case-identity.json') or identity_alive(directory, 'supervisor-launch.json')
                    if not alive:
                        launch_file = directory/'supervisor-launch.json'
                        if launch_file.exists():
                            old = json.loads(launch_file.read_text())
                            events = memory_events(config, job)
                            if events.get('oom_kill', 0) > old['events_before'].get('oom_kill', 0) and not populated(config, job):
                                failed=dict(job_id=job['id'], name=config['instances'][job['instance_index']]['name'],
                                            solver=job['solver'],status='oom',verified_optimal=False,
                                            memory_events_before=old['events_before'],cgroup_resources=cgroup_stats(config,job),finished_utc=utc())
                                native={}
                                result_path=directory/'result.json'
                                solver_exit=directory/'solver-exit.json'
                                if result_path.exists():
                                    native=json.loads(result_path.read_text())
                                    failed.update(solver_status=native.get('status'),native_primal_bound=native.get('objective'),dual_bound=native.get('dual_bound'))
                                if solver_exit.exists() and json.loads(solver_exit.read_text()).get('exit_code')==0:
                                    failed['status']='validation_oom'
                                    strict=directory/'strict-validation.json'
                                    if strict.exists():
                                        check=json.loads(strict.read_text());failed['strict_validation']=check
                                        if check.get('valid') is True:
                                            failed['primal_bound']=check['objective']
                                            failed['verified_optimal']=native.get('status','').lower()=='optimal'
                                atomic_json(summary,failed)
                                continue
                        count = recoveries.get(job['id'], 0)
                        if count >= 3:
                            if populated(config, job):
                                # A live child is evidence of ongoing work, never a reason to restart optimization.
                                break
                            atomic_json(summary, dict(job_id=job['id'], name=config['instances'][job['instance_index']]['name'],
                                                      solver=job['solver'], status='supervisor_crash', verified_optimal=False, finished_utc=utc()))
                            continue
                        children[core] = launch(root, config, job)
                        recoveries[job['id']] = count+1
                    break
            # Rebuild summary from terminal per-case files; no successful result is overwritten.
            terminal = []
            for job in config['jobs']:
                path = root/'cases'/job['id']/'summary.json'
                if path.exists():
                    terminal.append(json.loads(path.read_text()))
            atomic_json(root/'progress.json', dict(updated_utc=utc(), total=len(config['jobs']), completed=len(terminal),
                                                   active={str(c): j['id'] for c, j in active.items()},
                                                   optimal_strict_verified=sum(r.get('verified_optimal', False) for r in terminal),
                                                   errors=sum(r.get('status') in ('error', 'oom', 'validation_oom', 'supervisor_crash', 'invalid_solution', 'unverified_solution', 'watchdog', 'process_error', 'incomplete_optimality_claim', 'optimal_without_solution') for r in terminal)))
            with (root/'results.jsonl.tmp').open('w') as stream:
                for result in terminal:
                    stream.write(json.dumps(result, ensure_ascii=False, allow_nan=False)+'\n')
            (root/'results.jsonl.tmp').replace(root/'results.jsonl')
            if all(not queue for queue in queues.values()):
                atomic_json(root/'exit.json', dict(status='complete', finished_utc=utc(), cases=len(terminal)))
                print('complete', len(terminal), flush=True)
                return
            time.sleep(0.5)


def status(args):
    root = Path(args.directory).resolve()
    state = json.loads((root/'progress.json').read_text()) if (root/'progress.json').exists() else {}
    identity_path = root/'coordinator-identity.json'
    identity = json.loads(identity_path.read_text()) if identity_path.exists() else None
    state['coordinator_alive'] = bool(identity and same_process(identity))
    print(json.dumps(state, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='action', required=True)
    c = commands.add_parser('create')
    c.add_argument('directory')
    c.add_argument('--seconds', type=int, default=3600)
    c.add_argument('--cores', default='8,9,10,11,12,13,14,15')
    c.add_argument('--solvers', default='highs,scip,glpk')
    c.add_argument('--manifest', default=str(REPO/'data/manifest.json'))
    c.add_argument('--names', nargs='*')
    c.add_argument('--allow-dirty', action='store_true')
    for name in ('run', 'status'):
        p = commands.add_parser(name)
        p.add_argument('directory')
    args = parser.parse_args()
    {'create': create, 'run': run, 'status': status}[args.action](args)
