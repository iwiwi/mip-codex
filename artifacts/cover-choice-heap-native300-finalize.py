from pathlib import Path
import os,sys,json,subprocess,datetime
root=Path.cwd();os.sched_setaffinity(0,{46});os.environ['MIP_ABSOLUTE_CHECKER']='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute';sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from comparison_worker import read_glpk_values
from validate_solution import validate
out=root/'artifacts/cover-choice-heap-native300-controls';d=out/'glpk';log=(root/'artifacts/cover-choice-heap-native300-controls.log').read_text();assert 'TimeoutExpired'in log and'timed out after 419.'in log
processes=subprocess.check_output(['ps','-eo','args'],text=True).splitlines();assert not any(str(d)in s and('glpk-benchmark'in s or'comparison_worker.py'in s)for s in processes if not s.startswith(('/bin/bash','bash ','ps ')))
native=json.load(open(d/'native.json'));assert native['seconds']>420
late=dict(native,values=read_glpk_values(d),nodes=None,gap=None,measurement_status='late_after_wrapper_watchdog',included_in_300_second_comparison=False,requested_seconds=300,wrapper_watchdog_seconds=420,values_source='native direct API sidecar after natural late child completion')
atomic_json(d/'late-result.json',late)
model=root/'data/raw/exploratory/z26.mps.gz';v=validate(model,late);atomic_json(d/'late-validation.json',v);assert v['valid']
execution=dict(status='watchdog',requested_seconds=300,wrapper_watchdog_seconds=420,wrapper_exit_code_not_captured=True,original_traceback_preserved=True,child_alive_at_cleanup_inspection=False,late_native_seconds=native['seconds'],late_output_gmp_valid=True,late_output_excluded_from_time_matched_quality=True,reason='research subprocess.run watchdog killed wrapper; native child completed naturally after wrapper timeout; no matching child remained at inspection; retain all late artifacts without treating them as timed output')
atomic_json(d/'execution-watchdog-recovery.json',execution)
records=json.load(open(out/'records.json'));assert len(records)==2
records.append(dict(solver='glpk',status='watchdog',objective=None,dual_bound=None,seconds=None,wall_watchdog_seconds=420,version='5.0',gmp_valid=None,late_reference=dict(path='glpk/late-result.json',objective=late['objective'],dual_bound=late['dual_bound'],seconds=late['seconds'],gmp_valid=True)))
atomic_json(out/'records-completed.json',records);atomic_json(out/'exit.json',dict(status='complete_with_watchdog',outputs=3,timed_outputs=2,timed_points_gmp=2,watchdogs=1,late_points_gmp=1,all_cases_accounted_for=True))
print(json.dumps(dict(timed=records[:2],glpk_watchdog=420,late_objective=late['objective'],late_dual=late['dual_bound'],late_seconds=late['seconds'],late_gmp=True)),flush=True)
