#!/usr/bin/env python3
"""一件の評価。再実行時は生存中の同じステージを監視し、再最適化しない。"""
import json
import fcntl
import math
import os
from pathlib import Path
import subprocess
import sys
import time
from benchmark_common import cgroup_stats, atomic_json, process, group_alive, kill_group, resource_usage, sha256, utc


def stage(directory, name, command, budget):
    launch_path, exit_path = directory/(name+'-launch.json'), directory/(name+'-exit.json')
    if exit_path.exists():
        return json.loads(exit_path.read_text())
    child = None
    if launch_path.exists():
        launch = json.loads(launch_path.read_text())
    else:
        started = time.monotonic()
        with (directory/(name+'-stdout.log')).open('w') as stdout, (directory/(name+'-stderr.log')).open('w') as stderr:
            child = subprocess.Popen(['/usr/bin/time', '-v', '-o', str(directory/(name+'-resource.log')), *command],
                                     stdout=stdout, stderr=stderr, start_new_session=True)
        identity = process(child.pid)
        launch = dict(command=command, pid=child.pid, pgid=child.pid, start_ticks=identity['start_ticks'],
                      started_utc=utc(), started_monotonic=started, deadline_monotonic=started+budget,
                      boot_id=Path('/proc/sys/kernel/random/boot_id').read_text().strip())
        atomic_json(launch_path, launch)
    timed_out = False
    boot_changed = launch['boot_id'] != Path('/proc/sys/kernel/random/boot_id').read_text().strip()
    while not boot_changed and group_alive(launch['pgid']):
        if time.monotonic() >= launch['deadline_monotonic']:
            timed_out = True
            kill_group(launch['pgid'])
        if child is not None:
            child.poll()
        time.sleep(0.1)
    code = child.wait() if child is not None else None
    usage = resource_usage(directory/(name+'-resource.log'))
    if code is None:
        code = usage.get('exit_code')
    result = dict(exit_code=code, watchdog=timed_out, recovered=child is None,
                  boot_changed=boot_changed, resource=usage,
                  wall_seconds=None if boot_changed else time.monotonic()-launch['started_monotonic'],
                  finished_utc=utc())
    atomic_json(exit_path, result)
    return result


def run(config_path, job_id):
    config_path = Path(config_path)
    root = config_path.parent
    config = json.loads(config_path.read_text())
    job = next(j for j in config['jobs'] if j['id'] == job_id)
    directory = root/'cases'/job_id
    directory.mkdir(parents=True, exist_ok=True)
    case_lock=(directory/'case.lock').open('a')
    fcntl.flock(case_lock,fcntl.LOCK_EX | fcntl.LOCK_NB)
    atomic_json(directory/'case-identity.json',process(os.getpid()))
    if (directory/'summary.json').exists():
        case_lock.close()
        return
    record = config['instances'][job['instance_index']]
    summary = dict(job_id=job_id, name=record['name'], split=record['split'], solver=job['solver'],
                   cpu=job['cpu'], input_sha256=record['sha256'], status='error')
    try:
        if sha256(record['absolute_path']) != record['sha256']:
            raise ValueError('input hash mismatch')
        solver = job['solver']
        os.environ.update(MIP_GLPK_DRIVER=str(root/'snapshot/bin/glpk-benchmark'),
                          MIP_MEMORY_MIB=str(config['memory_bytes']//1024**2))
        if solver.startswith('rust'):
            if '-' in solver:
                os.environ['MIP_LP_ENGINE'] = solver.split('-', 1)[1]
            command = [str(root/'snapshot/bin/mip-codex'), 'solve', record['absolute_path'], str(config['seconds'])]
        else:
            command = [config['python'], str(root/'snapshot/scripts/comparison_worker.py'), solver,
                       record['absolute_path'], str(config['seconds']), str(directory)]
        solver_exit = stage(directory, 'solver', command, config['seconds']+config['read_write_grace_seconds'])
        summary['execution'] = solver_exit
        summary['resources_after_solver']=cgroup_stats(config,job)
        if solver.startswith('rust') and (directory/'solver-stdout.log').exists():
            parsed = json.loads((directory/'solver-stdout.log').read_text())
            atomic_json(directory/'result.json', parsed)
        else:
            parsed = json.loads((directory/'result.json').read_text())
        summary.update({k: v for k, v in parsed.items() if k != 'values'})
        summary['solver_status'] = parsed['status']
        summary['native_primal_bound'] = parsed.get('objective')
        summary['native_gap'] = parsed.get('gap')
        summary['primal_bound'] = None
        summary['verified_optimal'] = False
        if parsed.get('cut_proofs') or parsed.get('objective_grid') is not None:
            cut_command=[config['python'],str(root/'snapshot/scripts/verify_cover_cuts.py'),
                         record['absolute_path'],str(directory/'result.json'),str(directory/'cut-validation.json')]
            summary['cut_validation_execution']=stage(directory,'cut-validation',cut_command,config['validation_seconds'])
            check_path=directory/'cut-validation.json'
            summary['cut_validation']=json.loads(check_path.read_text()) if check_path.exists() else dict(valid=None,error='cut validator did not finish')
            execution=summary['cut_validation_execution']
            if execution['exit_code'] != 0 or execution['watchdog'] or execution['boot_changed']:
                if summary['cut_validation'].get('valid') is not False:
                    summary['cut_validation']=dict(valid=None,error='cut validator process did not finish successfully')

        if parsed.get('values') is not None:
            validation_command = [config['python'], str(root/'snapshot/scripts/benchmark_validate.py'),
                                  record['absolute_path'], str(directory/'result.json'),
                                  str(root/'snapshot/bin/solchecker'), str(directory)]
            summary['validation_execution'] = stage(directory, 'validation', validation_command, config['validation_seconds'])
            for mode in ('strict', 'official'):
                path = directory/(mode+'-validation.json')
                summary[mode+'_validation'] = json.loads(path.read_text()) if path.exists() else dict(valid=None, error='validator did not finish')
            if summary['strict_validation']['valid'] is True:
                summary['primal_bound'] = summary['strict_validation']['objective']
                summary['verified_optimal'] = parsed['status'].lower() == 'optimal'
            else:
                summary['status'] = 'invalid_solution' if summary['strict_validation']['valid'] is False else 'unverified_solution'
        elif parsed['status'].lower() == 'optimal':
            summary['status'] = 'optimal_without_solution'
        primal, dual = summary['primal_bound'], parsed.get('dual_bound')
        summary['gap'] = abs(primal-dual)/max(1., abs(primal)) if primal is not None and dual is not None else None
        if summary['verified_optimal'] and (summary['gap'] is None or summary['gap']>1e-7):
            summary['status']='incomplete_optimality_claim'
            summary['verified_optimal']=False
        if solver_exit['exit_code'] != 0 or solver_exit['watchdog'] or solver_exit['boot_changed']:
            summary['status'] = 'watchdog' if solver_exit['watchdog'] else 'process_error'
            summary['verified_optimal'] = False
        if summary.get('cut_validation',{}).get('valid',True) is not True:
            summary['status']='invalid_cut_proof' if summary['cut_validation']['valid'] is False else 'unverified_cut_proof'
            summary['verified_optimal']=False
            summary['native_dual_bound']=summary.get('dual_bound')
            summary['dual_bound']=None;summary['gap']=None
        summary['result_sha256'] = sha256(directory/'result.json')
        if parsed.get('values') is not None:
            summary['solution_sha256'] = summary['result_sha256']
    except Exception as error:
        summary.update(status='error', error=f'{type(error).__name__}: {error}')
        summary['verified_optimal'] = False
    summary['cgroup_resources']=cgroup_stats(config,job)
    summary['finished_utc'] = utc()
    atomic_json(directory/'summary.json', summary)
    case_lock.close()


if __name__ == '__main__':
    run(sys.argv[1], sys.argv[2])
