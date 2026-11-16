#!/usr/bin/env python3
"""専用マシンの比較ジョブ用cgroup。sudoでsetup/execのみを実行する。"""
import json
import os
from pathlib import Path
import re
import resource
import sys

BASE = Path('/sys/fs/cgroup/mip-codex-bench')

def component(value):
    if not re.fullmatch(r'[A-Za-z0-9_-]+', value):
        raise ValueError('invalid cgroup component')
    return value

def write(path, name, value):
    (path/name).write_text(str(value)+'\n')

if sys.argv[1] == 'setup':
    config = json.load(open(sys.argv[2]))
    parent = BASE/component(config['id'])
    BASE.mkdir(exist_ok=True)
    write(BASE, 'cgroup.subtree_control', '+cpu +memory +pids')
    parent.mkdir(exist_ok=True)
    write(parent, 'cgroup.subtree_control', '+cpu +memory +pids')
    for job in config['jobs']:
        group = parent/component(job['id'])
        group.mkdir(exist_ok=True)
        write(group, 'memory.max', config['memory_bytes'])
        write(group, 'memory.swap.max', 0)
        write(group, 'memory.oom.group', 1)
        write(group, 'cpu.max', '100000 100000')
        write(group, 'pids.max', 512)
elif sys.argv[1] == 'exec':
    experiment, job, cpu, uid, gid, memory = sys.argv[2:8]
    group = BASE/component(experiment)/component(job)
    write(group, 'cgroup.procs', os.getpid())
    os.sched_setaffinity(0, {int(cpu)})
    resource.setrlimit(resource.RLIMIT_AS, (int(memory), int(memory)))
    os.setgroups([])
    os.setgid(int(gid))
    os.setuid(int(uid))
    os.environ.update(OMP_NUM_THREADS='1', OPENBLAS_NUM_THREADS='1', MKL_NUM_THREADS='1',
                      NUMEXPR_NUM_THREADS='1', PYTHONHASHSEED='0')
    os.execv(sys.argv[8], sys.argv[8:])
else:
    raise ValueError('usage: cgroup_control.py setup CONFIG | exec EXP JOB CPU UID GID MEMORY CMD...')
