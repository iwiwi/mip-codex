from pathlib import Path
import os,subprocess,signal,time,json,shutil
root=Path.cwd();os.sched_setaffinity(0,set(range(56,64)))
assert shutil.disk_usage('/').free>2*1024**3
records=[]
for label,args,extra in [('tests',['cargo','test'],dict(CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',CARGO_BUILD_JOBS='2')),('build',['cargo','build','--release'],{})]:
    destination=root/f'artifacts/large-primal-budget-default-{label}.log'
    env=dict(os.environ,CARGO_INCREMENTAL='0',**extra)
    with destination.open('w')as log:
        child=subprocess.Popen(args,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
        low_disk=False
        while child.poll()is None:
            if shutil.disk_usage('/').free<750*1024**2:
                low_disk=True;os.killpg(child.pid,signal.SIGTERM);child.wait();break
            time.sleep(.5)
    record=dict(label=label,exit_code=child.returncode,low_disk_stop=low_disk,free_bytes=shutil.disk_usage('/').free)
    records.append(record)
    Path('artifacts/large-primal-budget-default-build-execution.json').write_text(json.dumps(records,indent=2))
    assert child.returncode==0 and not low_disk,record
