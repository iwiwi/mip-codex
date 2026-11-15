"""評価ランナー共通の原子的保存・プロセス同一性・資源ログ処理。"""
import hashlib
import json
import os
from pathlib import Path
import signal
import time


def atomic_json(path, value):
    path = Path(path)
    temporary = path.with_name(path.name+'.tmp-'+str(os.getpid()))
    with temporary.open('w') as stream:
        json.dump(value, stream, ensure_ascii=False, allow_nan=False, indent=2)
        stream.write('\n')
        stream.flush()
        os.fsync(stream.fileno())
    temporary.replace(path)


def sha256(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024*1024), b''):
            h.update(block)
    return h.hexdigest()


def process(pid):
    try:
        fields = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
        return dict(pid=int(pid), state=fields[0], pgid=int(fields[2]), start_ticks=int(fields[19]))
    except (FileNotFoundError, ProcessLookupError):
        return None


def same_process(record):
    current = process(record['pid'])
    return bool(current and current['state'] not in ('Z', 'X') and current['start_ticks'] == record['start_ticks'])


def group_alive(pgid):
    for path in Path('/proc').iterdir():
        if not path.name.isdigit():
            continue
        p = process(path.name)
        if p and p['pgid'] == pgid and p['state'] not in ('Z', 'X'):
            return True
    return False


def kill_group(pgid):
    try:
        os.killpg(pgid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def memory_events(config, job):
    path = Path('/sys/fs/cgroup/mip-codex-bench')/config['id']/job['id']/'memory.events'
    if not path.exists():
        return {}
    return {key: int(value) for key, value in (line.split() for line in path.read_text().splitlines())}


def resource_usage(path):
    result = {}
    if Path(path).exists():
        for line in Path(path).read_text().splitlines():
            line = line.strip()
            for prefix, key, typ in [('User time (seconds):', 'user_seconds', float),
                                     ('System time (seconds):', 'system_seconds', float),
                                     ('Maximum resident set size (kbytes):', 'max_rss_kib', int),
                                     ('Exit status:', 'exit_code', int)]:
                if line.startswith(prefix):
                    result[key] = typ(line[len(prefix):].strip())
            if line.startswith('Command terminated by signal '):
                result['signal'] = int(line.rsplit(' ', 1)[1])
    return result


def utc():
    return time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())


def cgroup_stats(config, job):
    root=Path('/sys/fs/cgroup/mip-codex-bench')/config['id']/job['id']
    result={'memory_events':memory_events(config,job)}
    for name in ('memory.peak','memory.current'):
        path=root/name
        if path.exists():result[name]=int(path.read_text())
    path=root/'cpu.stat'
    if path.exists():result['cpu.stat']={k:int(v) for k,v in (line.split() for line in path.read_text().splitlines())}
    return result
