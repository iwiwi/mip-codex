#!/usr/bin/env python3
"""Record actual installed comparison binaries and complete effective options."""
import datetime,hashlib,importlib,json,os,pathlib,platform,subprocess,sys
import highspy,pyscipopt
ROOT=pathlib.Path(__file__).resolve().parents[1];docs=ROOT/'docs';h=highspy.Highs();h.setOptionValue('output_flag',False)
for k,v in [('threads',1),('parallel','off'),('random_seed',0),('mip_rel_gap',1e-6),('time_limit',3600.)]:h.setOptionValue(k,v)
h.writeOptions(str(docs/'highs-options.txt'))
s=pyscipopt.Model();s.hideOutput()
for k,v in [('parallel/maxnthreads',1),('lp/threads',1),('randomization/randomseedshift',0),('limits/gap',1e-6),('limits/time',3600.)]:s.setParam(k,v)
s.writeParams(str(docs/'scip-options.set'),comments=True,onlychanged=False)
def digest(p):return dict(path=str(p),sha256=hashlib.sha256(p.read_bytes()).hexdigest(),bytes=p.stat().st_size)
site=pathlib.Path(highspy.__file__).parent.parent
libraries=list((site/'highspy').glob('*.so'))+list((site/'pyscipopt').glob('*.so'))+list((site/'pyscipopt.libs').glob('*.so*'))+[ROOT/'tools/glpk_driver',pathlib.Path('/usr/bin/glpsol')]+list(pathlib.Path('/usr/lib/aarch64-linux-gnu').glob('libglpk.so.40.3.1'))
report=dict(recorded_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),versions=dict(highs=h.version(),scip=f'{s.getMajorVersion()}.{s.getMinorVersion()}.{s.getTechVersion()}',pyscipopt=pyscipopt.__version__,glpk=subprocess.check_output(['glpsol','--version'],text=True).splitlines()[0],rust=subprocess.check_output(['rustc','--version'],text=True).strip()),packages=subprocess.check_output([sys.executable,'-m','pip','freeze'],text=True).splitlines(),libraries=[digest(p) for p in libraries],platform=platform.platform(),allowed_cpus=sorted(os.sched_getaffinity(0)),lscpu=subprocess.check_output(['lscpu'],text=True),glpk_command=['tools/glpk_driver','INPUT','3600','OUTPUT.json'],glpk_settings=dict(presolve='GLP_ON',tm_lim_ms=3600000,mip_gap=1e-6,other='glp_init_iocp defaults',callback='record best active node bound and node count; no search decisions'),glpk_build='cc -O2 -Wall -Wextra scripts/glpk_driver.c -lglpk -lm -o tools/glpk_driver',build_command='cargo build --release; Cargo.lock pins dependencies; faer default features disabled, std+sparse-linalg enabled')
(docs/'environment.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report['versions']))
